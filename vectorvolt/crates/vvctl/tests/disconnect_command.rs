//! vvctl disconnect (VV-CTL-200) has no wire-protocol counterpart, so it must
//! fail loudly instead of reporting a fake success.

use vvctl::commands::disconnect::{DisconnectArgs, run};

#[test]
fn disconnect_is_not_implemented_and_errors() {
    let err = run(DisconnectArgs {
        host: "127.0.0.1".into(),
        port: 7700,
    })
    .unwrap_err();
    assert!(err.to_string().contains("not implemented"), "{err}");
}
