//! Tests for Layer 318.
use vectorvolt_net::{NetConfig, NetClient, ConnectionPool, TlsConfig, Frame, encode_frame, decode_frame};

#[test] fn netconfig_default() { let c = NetConfig::default(); assert_eq!(c.port, 7700); }
#[test] fn netconfig_socket_addr() { assert_eq!(NetConfig::new("1.2.3.4", 9).socket_addr(), "1.2.3.4:9"); }
#[test] fn netclient_stub_addr() { let c = NetClient::connect("a").unwrap(); assert_eq!(c.addr, "a"); }
#[test] fn netclient_ping_ok() { assert!(NetClient::connect("b").unwrap().ping().is_ok()); }
#[test] fn pool_size() { assert_eq!(ConnectionPool::new(5).size(), 5); }
#[test] fn tls_default_disabled() { assert!(!TlsConfig::default().enabled); }
#[test] fn frame_encode_decode() { let b = encode_frame(&[1,2,3]); let f = decode_frame(&b).unwrap(); assert_eq!(f.payload, vec![1,2,3]); }
#[test] fn frame_len_field() { let f = decode_frame(&encode_frame(&[9,8])).unwrap(); assert_eq!(f.len, 2); }
#[test] fn netconfig_clone() { let c = NetConfig::default(); assert_eq!(c, c.clone()); }
#[test] fn pool_clone() { let p = ConnectionPool::new(3); assert_eq!(p.max, p.clone().max); }
#[test] fn frame_clone() { let f = Frame { len: 0, payload: vec![] }; assert_eq!(f, f.clone()); }
#[test] fn tls_clone() { let t = TlsConfig { enabled: true }; assert!(t.clone().enabled); }
#[test] fn netconfig_ne_different_port() { assert_ne!(NetConfig::new("x", 1), NetConfig::new("x", 2)); }
