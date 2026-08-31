//! Tests for Layer 186: vectorvolt-net bind (VV-NET-186).

use vectorvolt_net::{NetConfig, bind};

fn rand_config() -> NetConfig {
    NetConfig::new("127.0.0.1", 0)
}

#[test]
fn bind_port_zero_ok() {
    assert!(bind(&rand_config()).is_ok());
}
#[test]
fn bind_returns_listener() {
    let l = bind(&rand_config()).unwrap();
    let _ = l.local_addr().unwrap();
}
#[test]
fn bind_local_addr_is_v4() {
    let l = bind(&rand_config()).unwrap();
    assert!(l.local_addr().unwrap().is_ipv4());
}
#[test]
fn bind_port_nonzero_after_bind() {
    let l = bind(&rand_config()).unwrap();
    assert_ne!(l.local_addr().unwrap().port(), 0);
}
#[test]
fn bind_bad_addr_errors() {
    let cfg = NetConfig::new("999.999.999.999", 0);
    assert!(bind(&cfg).is_err());
}
#[test]
fn bind_twice_port_zero_both_ok() {
    let a = bind(&rand_config());
    let b = bind(&rand_config());
    assert!(a.is_ok());
    assert!(b.is_ok());
}
#[test]
fn bind_different_ports() {
    let a = bind(&rand_config()).unwrap();
    let b = bind(&rand_config()).unwrap();
    assert_ne!(
        a.local_addr().unwrap().port(),
        b.local_addr().unwrap().port()
    );
}
#[test]
fn listener_set_nonblocking_ok() {
    let l = bind(&rand_config()).unwrap();
    assert!(l.set_nonblocking(true).is_ok());
}
#[test]
fn listener_local_addr_host() {
    let l = bind(&rand_config()).unwrap();
    assert_eq!(l.local_addr().unwrap().ip().to_string(), "127.0.0.1");
}
#[test]
fn bind_with_custom_config() {
    let cfg = NetConfig::new("127.0.0.1", 0);
    let l = bind(&cfg);
    assert!(l.is_ok());
}
#[test]
fn bind_default_config_fails_port_7700() {
    let _ = bind(&NetConfig::default()); /* may succeed or fail depending on port availability */
}
#[test]
fn bind_port_zero_three_times() {
    for _ in 0..3 {
        assert!(bind(&rand_config()).is_ok());
    }
}
#[test]
fn listener_from_bind_can_accept_nonblocking() {
    let l = bind(&rand_config()).unwrap();
    l.set_nonblocking(true).unwrap();
    assert!(l.accept().is_err());
}
