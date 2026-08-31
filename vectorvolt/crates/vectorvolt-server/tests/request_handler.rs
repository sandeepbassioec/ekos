use vectorvolt_server::{handle, Request, Response};

#[test]
fn ping_returns_pong() {
    let resp = handle(Request::Ping);
    assert!(matches!(resp, Response::Pong));
}

#[test]
fn status_returns_status_ok() {
    let resp = handle(Request::Status);
    assert!(matches!(resp, Response::StatusOk(_)));
}

#[test]
fn status_ok_value_nonempty() {
    let resp = handle(Request::Status);
    if let Response::StatusOk(s) = resp {
        assert!(!s.is_empty());
    } else {
        panic!("expected StatusOk");
    }
}

#[test]
fn status_ok_contains_status_ok() {
    let resp = handle(Request::Status);
    if let Response::StatusOk(s) = resp {
        assert!(s.contains("ok"));
    } else {
        panic!("expected StatusOk");
    }
}

#[test]
fn ping_variant_not_status() {
    let resp = handle(Request::Ping);
    assert!(!matches!(resp, Response::StatusOk(_)));
}

#[test]
fn status_variant_not_pong() {
    let resp = handle(Request::Status);
    assert!(!matches!(resp, Response::Pong));
}

#[test]
fn multiple_pings() {
    for _ in 0..5 {
        assert!(matches!(handle(Request::Ping), Response::Pong));
    }
}

#[test]
fn multiple_status() {
    for _ in 0..5 {
        assert!(matches!(handle(Request::Status), Response::StatusOk(_)));
    }
}

#[test]
fn ping_response_is_pong_variant() {
    let r = handle(Request::Ping);
    assert!(matches!(r, Response::Pong));
}

#[test]
fn handle_status_message_nonempty() {
    if let Response::StatusOk(s) = handle(Request::Status) {
        assert!(!s.is_empty());
    } else {
        panic!("expected StatusOk");
    }
}

#[test]
fn request_enum_has_ping() {
    let _r = Request::Ping;
}

#[test]
fn request_enum_has_status() {
    let _r = Request::Status;
}

#[test]
fn response_enum_has_pong() {
    let _r = Response::Pong;
}
