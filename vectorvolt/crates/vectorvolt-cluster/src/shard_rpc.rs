//! Node-to-node shard data-plane RPC over the control HTTP server (gap F7).
//!
//! * Server side: [`serve_shard_rpc`] maps `POST /shardnode/<op>` onto any
//!   [`ShardNode`] (in `vvctl serve`, the engine-backed one).
//! * Client side: [`RemoteShardNode`] implements [`ShardNode`] by calling that
//!   endpoint on a peer, so the migration coordinator drives local and remote
//!   nodes through one trait.
//!
//! Requests carry the shared secret in `x-vv-secret` when one is configured.

use crate::migration::{Change, DataError, ShardLoad, ShardNode};
use crate::node::NodeId;
use async_trait::async_trait;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::time::Duration;

/// One HTTP/1.1 call (`Connection: close`). Returns `(status, body)`.
pub async fn http_call(
    addr: &str,
    method: &str,
    path: &str,
    body: &str,
    secret: Option<&str>,
    timeout: Duration,
) -> Result<(u16, String), String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let fut = async {
        let mut s = tokio::net::TcpStream::connect(addr)
            .await
            .map_err(|e| format!("connect {addr}: {e}"))?;
        let sec = secret
            .map(|v| format!("x-vv-secret: {v}\r\n"))
            .unwrap_or_default();
        let req = format!(
            "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n{sec}Content-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        );
        s.write_all(req.as_bytes())
            .await
            .map_err(|e| format!("write {addr}: {e}"))?;
        let mut raw = Vec::new();
        s.read_to_end(&mut raw)
            .await
            .map_err(|e| format!("read {addr}: {e}"))?;
        let raw = String::from_utf8_lossy(&raw).to_string();
        let (head, resp) = raw.split_once("\r\n\r\n").unwrap_or((&raw, ""));
        let code = head
            .split_whitespace()
            .nth(1)
            .and_then(|c| c.parse::<u16>().ok())
            .ok_or_else(|| format!("bad response from {addr}"))?;
        Ok::<_, String>((code, resp.to_string()))
    };
    tokio::time::timeout(timeout, fut)
        .await
        .map_err(|_| format!("timeout calling {addr}{path}"))?
}

#[derive(Serialize, Deserialize)]
enum WireErr {
    NotOwner,
    Frozen,
    Down,
    NotHosted(u64),
    Other(String),
}

impl From<&DataError> for WireErr {
    fn from(e: &DataError) -> Self {
        match e {
            DataError::NotOwner => WireErr::NotOwner,
            DataError::Frozen => WireErr::Frozen,
            DataError::Down => WireErr::Down,
            DataError::NotHosted(i) => WireErr::NotHosted(*i),
            DataError::Other(s) => WireErr::Other(s.clone()),
        }
    }
}

impl From<WireErr> for DataError {
    fn from(e: WireErr) -> Self {
        match e {
            WireErr::NotOwner => DataError::NotOwner,
            WireErr::Frozen => DataError::Frozen,
            WireErr::Down => DataError::Down,
            WireErr::NotHosted(i) => DataError::NotHosted(i),
            WireErr::Other(s) => DataError::Other(s),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Shard {
    shard_id: u64,
}
#[derive(Serialize, Deserialize)]
struct Host {
    shard_id: u64,
    lo: u64,
    hi: u64,
    serving: bool,
}
#[derive(Serialize, Deserialize)]
struct Put {
    key: u64,
    value: Vec<u8>,
}
#[derive(Serialize, Deserialize)]
struct Key {
    key: u64,
}
#[derive(Serialize, Deserialize)]
struct Scan {
    shard_id: u64,
    after: Option<u64>,
    limit: usize,
}
#[derive(Serialize, Deserialize)]
struct Since {
    shard_id: u64,
    seq: u64,
    limit: usize,
}
#[derive(Serialize, Deserialize)]
struct Import {
    shard_id: u64,
    entries: Vec<(u64, Vec<u8>)>,
}
#[derive(Serialize, Deserialize)]
struct Apply {
    shard_id: u64,
    changes: Vec<Change>,
}
#[derive(Serialize, Deserialize)]
struct Split {
    shard_id: u64,
    at: u64,
    new_id: u64,
}

fn parse<T: DeserializeOwned>(body: &str) -> Result<T, (u16, String)> {
    serde_json::from_str(body).map_err(|e| (400, format!("bad request: {e}")))
}

fn reply<T: Serialize>(r: Result<T, DataError>) -> (u16, String) {
    match r {
        Ok(v) => (200, serde_json::to_string(&v).unwrap_or_default()),
        Err(e) => (
            409,
            serde_json::to_string(&WireErr::from(&e)).unwrap_or_default(),
        ),
    }
}

/// Serve one `/shardnode/<op>` call against `node`.
pub async fn serve_shard_rpc(node: &dyn ShardNode, op: &str, body: &str) -> (u16, String) {
    macro_rules! p {
        ($t:ty) => {
            match parse::<$t>(body) {
                Ok(v) => v,
                Err(e) => return e,
            }
        };
    }
    match op {
        "host" => {
            let r = p!(Host);
            reply(node.host_shard(r.shard_id, r.lo, r.hi, r.serving).await)
        }
        "put" => {
            let r = p!(Put);
            reply(node.put(r.key, r.value).await)
        }
        "delete" => {
            let r = p!(Key);
            reply(node.delete(r.key).await)
        }
        "get" => {
            let r = p!(Key);
            reply(node.get(r.key).await)
        }
        "begin_capture" => {
            let r = p!(Shard);
            reply(node.begin_capture(r.shard_id).await)
        }
        "scan" => {
            let r = p!(Scan);
            reply(node.scan(r.shard_id, r.after, r.limit).await)
        }
        "changes" => {
            let r = p!(Since);
            reply(node.changes_since(r.shard_id, r.seq, r.limit).await)
        }
        "import" => {
            let r = p!(Import);
            reply(node.apply_import(r.shard_id, r.entries).await)
        }
        "apply" => {
            let r = p!(Apply);
            reply(node.apply_changes(r.shard_id, r.changes).await)
        }
        "freeze" => reply(node.freeze(p!(Shard).shard_id).await),
        "rollback" => reply(node.rollback_source(p!(Shard).shard_id).await),
        "activate" => reply(node.activate(p!(Shard).shard_id).await),
        "release" => reply(node.release(p!(Shard).shard_id).await),
        "discard" => reply(node.discard(p!(Shard).shard_id).await),
        "split" => {
            let r = p!(Split);
            reply(node.split_local(r.shard_id, r.at, r.new_id).await)
        }
        "median" => reply(node.median_key(p!(Shard).shard_id).await),
        "loads" => reply(node.loads().await),
        _ => (404, format!("unknown shardnode op {op}")),
    }
}

/// A peer's [`ShardNode`] reached over its control HTTP endpoint.
pub struct RemoteShardNode {
    id: NodeId,
    addr: String,
    secret: Option<String>,
    timeout: Duration,
}

impl RemoteShardNode {
    pub fn new(id: NodeId, addr: impl Into<String>, secret: Option<String>) -> Self {
        RemoteShardNode {
            id,
            addr: addr.into(),
            secret,
            timeout: Duration::from_secs(60),
        }
    }

    async fn call<Q: Serialize, R: DeserializeOwned>(
        &self,
        op: &str,
        req: &Q,
    ) -> Result<R, DataError> {
        let body = serde_json::to_string(req).map_err(|e| DataError::Other(e.to_string()))?;
        let (code, resp) = http_call(
            &self.addr,
            "POST",
            &format!("/shardnode/{op}"),
            &body,
            self.secret.as_deref(),
            self.timeout,
        )
        .await
        // Unreachable peer: surface as Down so the migrator rolls back.
        .map_err(|_| DataError::Down)?;
        match code {
            200 => serde_json::from_str(&resp).map_err(|e| DataError::Other(e.to_string())),
            409 => Err(serde_json::from_str::<WireErr>(&resp)
                .map(DataError::from)
                .unwrap_or_else(|_| DataError::Other(resp))),
            c => Err(DataError::Other(format!(
                "peer {} returned {c}: {resp}",
                self.addr
            ))),
        }
    }
}

#[async_trait]
impl ShardNode for RemoteShardNode {
    fn id(&self) -> NodeId {
        self.id.clone()
    }
    async fn host_shard(
        &self,
        shard_id: u64,
        lo: u64,
        hi: u64,
        serving: bool,
    ) -> Result<(), DataError> {
        self.call(
            "host",
            &Host {
                shard_id,
                lo,
                hi,
                serving,
            },
        )
        .await
    }
    async fn put(&self, key: u64, value: Vec<u8>) -> Result<(), DataError> {
        self.call("put", &Put { key, value }).await
    }
    async fn delete(&self, key: u64) -> Result<(), DataError> {
        self.call("delete", &Key { key }).await
    }
    async fn get(&self, key: u64) -> Result<Option<Vec<u8>>, DataError> {
        self.call("get", &Key { key }).await
    }
    async fn begin_capture(&self, shard_id: u64) -> Result<u64, DataError> {
        self.call("begin_capture", &Shard { shard_id }).await
    }
    async fn scan(
        &self,
        shard_id: u64,
        after: Option<u64>,
        limit: usize,
    ) -> Result<Vec<(u64, Vec<u8>)>, DataError> {
        self.call(
            "scan",
            &Scan {
                shard_id,
                after,
                limit,
            },
        )
        .await
    }
    async fn changes_since(
        &self,
        shard_id: u64,
        seq: u64,
        limit: usize,
    ) -> Result<Vec<Change>, DataError> {
        self.call(
            "changes",
            &Since {
                shard_id,
                seq,
                limit,
            },
        )
        .await
    }
    async fn apply_import(
        &self,
        shard_id: u64,
        entries: Vec<(u64, Vec<u8>)>,
    ) -> Result<(), DataError> {
        self.call("import", &Import { shard_id, entries }).await
    }
    async fn apply_changes(&self, shard_id: u64, changes: Vec<Change>) -> Result<(), DataError> {
        self.call("apply", &Apply { shard_id, changes }).await
    }
    async fn freeze(&self, shard_id: u64) -> Result<(), DataError> {
        self.call("freeze", &Shard { shard_id }).await
    }
    async fn rollback_source(&self, shard_id: u64) -> Result<(), DataError> {
        self.call("rollback", &Shard { shard_id }).await
    }
    async fn activate(&self, shard_id: u64) -> Result<(), DataError> {
        self.call("activate", &Shard { shard_id }).await
    }
    async fn release(&self, shard_id: u64) -> Result<(), DataError> {
        self.call("release", &Shard { shard_id }).await
    }
    async fn discard(&self, shard_id: u64) -> Result<(), DataError> {
        self.call("discard", &Shard { shard_id }).await
    }
    async fn split_local(&self, shard_id: u64, at: u64, new_id: u64) -> Result<(), DataError> {
        self.call(
            "split",
            &Split {
                shard_id,
                at,
                new_id,
            },
        )
        .await
    }
    async fn median_key(&self, shard_id: u64) -> Result<Option<u64>, DataError> {
        self.call("median", &Shard { shard_id }).await
    }
    async fn loads(&self) -> Result<Vec<ShardLoad>, DataError> {
        self.call("loads", &serde_json::json!({})).await
    }
}
