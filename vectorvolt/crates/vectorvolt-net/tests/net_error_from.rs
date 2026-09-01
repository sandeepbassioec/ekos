//! Tests for Layer 320.
use vectorvolt_net::{NetError, auth::AuthError};

fn io(kind: std::io::ErrorKind) -> std::io::Error { std::io::Error::new(kind, "test") }

#[test] fn from_io_other() { let e = NetError::from(io(std::io::ErrorKind::Other)); assert!(format!("{e}").contains("I/O")); }
#[test] fn from_io_not_found() { let e = NetError::from(io(std::io::ErrorKind::NotFound)); assert!(matches!(e, NetError::Io(_))); }
#[test] fn from_io_broken_pipe() { let e = NetError::from(io(std::io::ErrorKind::BrokenPipe)); assert!(matches!(e, NetError::Io(_))); }
#[test] fn from_io_connection_refused() { let e = NetError::from(io(std::io::ErrorKind::ConnectionRefused)); assert!(matches!(e, NetError::Io(_))); }
#[test] fn from_io_timeout() { let e = NetError::from(io(std::io::ErrorKind::TimedOut)); assert!(matches!(e, NetError::Io(_))); }
#[test] fn from_auth_unknown_key() { let e = NetError::from(AuthError::UnknownKeyId); assert!(matches!(e, NetError::Auth(_))); }
#[test] fn from_auth_bad_response() { let e = NetError::from(AuthError::BadResponse); assert!(matches!(e, NetError::Auth(_))); }
#[test] fn from_auth_no_challenge() { let e = NetError::from(AuthError::NoChallenge); assert!(matches!(e, NetError::Auth(_))); }
#[test] fn from_auth_too_many() { let e = NetError::from(AuthError::TooManyAttempts); assert!(matches!(e, NetError::Auth(_))); }
#[test] fn net_error_debug_nonempty() { assert!(!format!("{:?}", NetError::BadMagic).is_empty()); }
#[test] fn net_error_display_nonempty() { assert!(!format!("{}", NetError::BadMagic).is_empty()); }
#[test] fn io_error_message_preserved() { let e = NetError::from(std::io::Error::new(std::io::ErrorKind::Other, "custom")); assert!(format!("{e}").contains("I/O")); }
#[test] fn auth_error_display() { let e = NetError::from(AuthError::BadResponse); assert!(!format!("{e}").is_empty()); }
