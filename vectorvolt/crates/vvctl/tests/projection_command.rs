//! vvctl projection has no server API behind it: every subcommand must fail
//! with "not implemented" rather than print placeholder output.

use vvctl::commands::projection_cmd::{ProjectionArgs, ProjectionCommand, run};

#[test]
fn every_projection_subcommand_reports_not_implemented() {
    for command in [
        ProjectionCommand::List,
        ProjectionCommand::Status { name: "m".into() },
        ProjectionCommand::Replay { name: "m".into() },
        ProjectionCommand::InvalidateCache { name: "m".into() },
    ] {
        let err = run(ProjectionArgs { command }).unwrap_err();
        assert!(err.to_string().contains("not implemented"), "{err}");
    }
}
