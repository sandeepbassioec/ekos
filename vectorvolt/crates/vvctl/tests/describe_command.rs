//! Integration tests for Layer 179: vvctl hal/describe command (VV-CTL-179).

use vvctl::commands::hal::{HalArgs, run};

fn minimal_args() -> HalArgs {
    HalArgs {
        rows: 1000,
        iters: 1,
        no_bench: true,
        pin: false,
    }
}

#[test]
fn hal_no_bench_ok() {
    assert!(run(minimal_args()).is_ok());
}

#[test]
fn hal_small_rows_ok() {
    let args = HalArgs {
        rows: 100,
        iters: 1,
        no_bench: true,
        pin: false,
    };
    assert!(run(args).is_ok());
}

#[test]
fn hal_1_row_ok() {
    let args = HalArgs {
        rows: 1,
        iters: 1,
        no_bench: true,
        pin: false,
    };
    assert!(run(args).is_ok());
}

#[test]
fn hal_large_rows_no_bench_ok() {
    let args = HalArgs {
        rows: 1_000_000,
        iters: 1,
        no_bench: true,
        pin: false,
    };
    assert!(run(args).is_ok());
}

#[test]
fn hal_no_pin_ok() {
    let args = HalArgs {
        rows: 100,
        iters: 1,
        no_bench: true,
        pin: false,
    };
    assert!(run(args).is_ok());
}

#[test]
fn hal_iters_1_ok() {
    let args = HalArgs {
        rows: 500,
        iters: 1,
        no_bench: true,
        pin: false,
    };
    assert!(run(args).is_ok());
}

#[test]
fn hal_repeated_calls_ok() {
    let args = HalArgs {
        rows: 100,
        iters: 1,
        no_bench: true,
        pin: false,
    };
    assert!(run(args).is_ok());
    let args2 = HalArgs {
        rows: 200,
        iters: 1,
        no_bench: true,
        pin: false,
    };
    assert!(run(args2).is_ok());
}

#[test]
fn hal_describe_nonempty() {
    let d = vectorvolt_hal::describe();
    assert!(!d.is_empty());
}

#[test]
fn hal_no_bench_fast() {
    let args = HalArgs {
        rows: 100,
        iters: 1,
        no_bench: true,
        pin: false,
    };
    let start = std::time::Instant::now();
    run(args).unwrap();
    assert!(start.elapsed().as_secs() < 5, "no_bench should be fast");
}

#[test]
fn hal_tiny_rows_no_bench_ok() {
    let args = HalArgs {
        rows: 10,
        iters: 1,
        no_bench: true,
        pin: false,
    };
    assert!(run(args).is_ok());
}

#[test]
fn hal_1000_rows_ok() {
    let args = HalArgs {
        rows: 1000,
        iters: 2,
        no_bench: true,
        pin: false,
    };
    assert!(run(args).is_ok());
}

#[test]
fn hal_5000_rows_ok() {
    let args = HalArgs {
        rows: 5000,
        iters: 1,
        no_bench: true,
        pin: false,
    };
    assert!(run(args).is_ok());
}

#[test]
fn hal_args_default_no_bench() {
    let args = minimal_args();
    assert!(args.no_bench);
}
