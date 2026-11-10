//! F7/F8 wired into the real system: two `vvctl serve` processes on 127.0.0.1
//! form a shard-map Raft group; `vvctl shard split|list|move` act on them over
//! the control port, rows live in the real `TableEngine`, a 2-shard saga whose
//! second step fails is compensated in the real tables, and commits carry
//! versionstamps into the CDC log.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn free_port() -> u16 {
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    l.local_addr().unwrap().port()
}

struct Node {
    child: Child,
    id: String,
    ctl: SocketAddr,
    dir: tempfile::TempDir,
}

impl Drop for Node {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn http(addr: SocketAddr, method: &str, path: &str, body: &str) -> (u16, String) {
    let Ok(mut s) = TcpStream::connect_timeout(&addr, Duration::from_secs(5)) else {
        return (0, "connect failed".into());
    };
    s.set_read_timeout(Some(Duration::from_secs(60))).unwrap();
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: x\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    s.write_all(req.as_bytes()).unwrap();
    let mut raw = String::new();
    s.read_to_string(&mut raw).unwrap();
    let (head, resp) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
    let code = head
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .unwrap_or(0);
    (code, resp.to_string())
}

fn wait(limit: Duration, mut f: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + limit;
    while Instant::now() < end {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    f()
}

struct Spec {
    id: String,
    bind: u16,
    status: u16,
    dir: tempfile::TempDir,
}

fn spec() -> Spec {
    Spec {
        id: uuid::Uuid::new_v4().to_string(),
        bind: free_port(),
        status: free_port(),
        dir: tempfile::tempdir().unwrap(),
    }
}

fn write_peers(dir: &Path, other: &Spec) {
    std::fs::write(
        dir.join("shard_cluster.json"),
        serde_json::json!({ "peers": [{
            "id": other.id,
            "raft": format!("127.0.0.1:{}", other.bind),
            "http": format!("127.0.0.1:{}", other.status),
        }]})
        .to_string(),
    )
    .unwrap();
}

fn spawn(s: Spec) -> Node {
    let log = std::fs::File::create(s.dir.path().join("serve.log")).unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_vvctl"))
        .arg("serve")
        .arg(s.dir.path())
        // `cluster_bind` is a positional argument of `serve`.
        .arg(format!("127.0.0.1:{}", s.bind))
        .args(["--columns", "pk", "v"])
        .args(["--addr", "127.0.0.1:0", "--no-grpc", "--metrics-port", "0"])
        .args(["--cluster-node-id", &s.id])
        .args(["--cluster-status-port", &s.status.to_string()])
        .stdout(Stdio::null())
        .stderr(log)
        .spawn()
        .expect("spawn vvctl serve");
    Node {
        child,
        id: s.id,
        ctl: format!("127.0.0.1:{}", s.status).parse().unwrap(),
        dir: s.dir,
    }
}

fn vvctl(node: &Node, args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_vvctl"))
        .args(["shard", "--node", &node.ctl.to_string()])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "vvctl shard {args:?} failed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn table(node: &Node) -> serde_json::Value {
    let (c, b) = http(node.ctl, "GET", "/shards", "");
    assert_eq!(c, 200, "{b}");
    serde_json::from_str(&b).unwrap()
}

fn shards(node: &Node) -> Vec<(u64, u64, u64, String)> {
    let t = table(node);
    let mut v: Vec<_> = t["shards"]
        .as_object()
        .map(|m| {
            m.values()
                .map(|s| {
                    (
                        s["shard_id"].as_u64().unwrap(),
                        s["lo"].as_u64().unwrap(),
                        s["hi"].as_u64().unwrap(),
                        s["owner"].as_str().unwrap().to_string(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

fn row(k: u64, v: u64) -> Vec<u8> {
    [k.to_le_bytes(), v.to_le_bytes()].concat()
}

fn put(node: &Node, k: u64, v: u64) {
    let body = serde_json::json!({ "key": k, "value": row(k, v) }).to_string();
    let (c, b) = http(node.ctl, "POST", "/shardnode/put", &body);
    assert_eq!(c, 200, "put {k}: {b}");
}

/// `Some(second column)` if the node serves the key, `None` if absent,
/// `Err(code)` when the node does not own it.
fn get(node: &Node, k: u64) -> Result<Option<u64>, u16> {
    let (c, b) = http(
        node.ctl,
        "POST",
        "/shardnode/get",
        &serde_json::json!({ "key": k }).to_string(),
    );
    if c != 200 {
        return Err(c);
    }
    let v: Option<Vec<u8>> = serde_json::from_str(&b).unwrap();
    Ok(v.map(|b| u64::from_le_bytes(b[8..16].try_into().unwrap())))
}

fn saga_state(node: &Node, id: &str) -> serde_json::Value {
    let mut last = serde_json::Value::Null;
    wait(Duration::from_secs(30), || {
        let (c, b) = http(node.ctl, "GET", &format!("/sagas/{id}"), "");
        if c == 200 {
            last = serde_json::from_str(&b).unwrap();
            last["state"] != "running"
        } else {
            false
        }
    });
    last
}

#[test]
fn shards_split_move_saga_and_versionstamps_on_real_nodes() {
    let (sa, sb) = (spec(), spec());
    write_peers(sa.dir.path(), &sb);
    write_peers(sb.dir.path(), &sa);
    let (a, b) = (spawn(sa), spawn(sb));

    // Raft elects a leader, which creates the initial shard on itself; both
    // nodes learn the map.
    assert!(
        wait(Duration::from_secs(40), || {
            let both = |n: &Node| {
                let (c, body) = http(n.ctl, "GET", "/shards", "");
                c == 200 && body.contains("\"shard_id\"")
            };
            both(&a) && both(&b)
        }),
        "shard map never initialised.\nA log:\n{}",
        std::fs::read_to_string(a.dir.path().join("serve.log")).unwrap_or_default()
    );
    let first = shards(&a);
    assert_eq!(first.len(), 1);
    let (owner, other) = if first[0].3 == a.id {
        (&a, &b)
    } else {
        (&b, &a)
    };

    // Rows land in the real engine of the owner.
    for k in 0..200u64 {
        put(owner, k, k * 10);
    }
    assert_eq!(get(other, 5), Err(409), "non-owner must refuse");

    // Versionstamps reach the CDC log.
    assert!(wait(Duration::from_secs(5), || {
        std::fs::read(owner.dir.path().join("cdc.log"))
            .map(|b| String::from_utf8_lossy(&b).contains("\"versionstamp\""))
            .unwrap_or(false)
    }));
    let stamp = |n: &Node| {
        let (c, b) = http(n.ctl, "GET", "/versionstamp", "");
        assert_eq!(c, 200, "{b}");
        serde_json::from_str::<serde_json::Value>(&b).unwrap()["versionstamp"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let (s1, s2) = (stamp(owner), stamp(owner));
    assert!(s2 > s1, "stamps must increase: {s1} !< {s2}");
    assert_eq!(s1.len(), 24);

    // vvctl shard split (through the follower, forwarded to the leader), list.
    vvctl(&a, &["split", "1", "--at", "100"]);
    // Followers apply the committed map asynchronously.
    assert!(wait(Duration::from_secs(10), || shards(&a) == shards(&b)
        && shards(&b).len() == 2));
    let listing = vvctl(&b, &["list"]);
    assert!(listing.contains("100"), "{listing}");
    let after_split = shards(&a);
    assert_eq!(after_split.len(), 2);
    assert_eq!(after_split[0].2, 100);
    assert_eq!(after_split[1].1, 100);
    assert!(after_split.iter().all(|s| s.3 == owner.id));

    // vvctl shard move: shard 2 (keys >= 100) to the other node, live.
    let moved = vvctl(owner, &["move", "2", &other.id]);
    assert!(
        moved.to_lowercase().contains("moved") || moved.contains("copied") || !moved.is_empty()
    );
    assert!(wait(Duration::from_secs(10), || shards(&a) == shards(&b)
        && shards(&b)[1].3 == other.id));
    let after_move = shards(&b);
    assert_eq!(
        after_move[0].3, owner.id,
        "{after_move:?} owner={} other={}",
        owner.id, other.id
    );
    assert_eq!(after_move[1].3, other.id, "{after_move:?}");
    for k in [100u64, 150, 199] {
        assert_eq!(get(other, k), Ok(Some(k * 10)), "key {k} on new owner");
        assert_eq!(get(owner, k), Err(409), "key {k} released at source");
    }
    for k in [0u64, 50, 99] {
        assert_eq!(get(owner, k), Ok(Some(k * 10)));
    }
    // Writes to the moved range now go to the new owner only.
    put(other, 120, 1200);
    let (c, _) = http(
        owner.ctl,
        "POST",
        "/shardnode/put",
        &serde_json::json!({ "key": 121, "value": row(121, 1) }).to_string(),
    );
    assert_eq!(c, 409);

    // Saga across both shards: step 2 fails, step 1 is compensated for real.
    let saga = serde_json::json!({
        "saga_id": "fail-1",
        "steps": [
            { "id": "reserve", "shard": 1,
              "forward": { "ops": [ { "op": "put", "key": 7, "values": [7, 777] } ] },
              "compensate": { "ops": [ { "op": "delete", "key": 7 } ] } },
            { "id": "charge", "shard": 2, "max_attempts": 2,
              "forward": { "ops": [ { "op": "expect", "key": 5000 },
                                    { "op": "put", "key": 5001, "values": [5001, 1] } ] },
              "compensate": { "ops": [] } }
        ]
    });
    let (c, b2) = http(owner.ctl, "POST", "/sagas", &saga.to_string());
    assert_eq!(c, 202, "{b2}");
    let st = saga_state(owner, "fail-1");
    assert_eq!(st["state"], "aborted", "{st}");
    assert_eq!(get(owner, 7), Ok(None), "step 1 must be compensated");
    assert_eq!(get(other, 5001), Ok(None), "step 2 must not have applied");

    // A saga that succeeds applies both steps on their owners.
    let ok = serde_json::json!({
        "saga_id": "ok-1",
        "steps": [
            { "id": "a", "shard": 1,
              "forward": { "ops": [ { "op": "put", "key": 8, "values": [8, 88] } ] },
              "compensate": { "ops": [ { "op": "delete", "key": 8 } ] } },
            { "id": "b", "shard": 2,
              "forward": { "ops": [ { "op": "put", "key": 108, "values": [108, 1088] } ] },
              "compensate": { "ops": [ { "op": "delete", "key": 108 } ] } }
        ]
    });
    let (c, b3) = http(other.ctl, "POST", "/sagas", &ok.to_string());
    assert_eq!(c, 202, "{b3}");
    assert_eq!(saga_state(other, "ok-1")["state"], "committed");
    assert_eq!(get(owner, 8), Ok(Some(88)));
    assert_eq!(get(other, 108), Ok(Some(1088)));
    assert!(http(owner.ctl, "GET", "/sagas/nope", "").0 == 404);
}

#[test]
fn unfinished_saga_in_the_log_resumes_at_startup() {
    use vectorvolt_cluster::saga::{Record, SagaDef, SagaLog, StepDef};
    let s = spec();
    let steps = [("one", 11u64), ("two", 12u64)]
        .iter()
        .map(|(id, k)| {
            let fwd =
                serde_json::json!({ "ops": [ { "op": "put", "key": k, "values": [k, k * 2] } ] });
            StepDef::new(id, 1, &serde_json::to_vec(&fwd).unwrap(), br#"{"ops":[]}"#)
        })
        .collect();
    // A saga whose coordinator died right after logging `Begin`.
    std::fs::create_dir_all(s.dir.path().join("sagas")).unwrap();
    let mut log = SagaLog::open(&s.dir.path().join("sagas").join("resume-1.log")).unwrap();
    log.append(Record::Begin(SagaDef {
        saga_id: "resume-1".into(),
        steps,
    }))
    .unwrap();
    drop(log);

    let n = spawn(s);
    let st = saga_state(&n, "resume-1");
    assert_eq!(st["state"], "committed", "{st}");
    assert_eq!(get(&n, 11), Ok(Some(22)));
    assert_eq!(get(&n, 12), Ok(Some(24)));
}
