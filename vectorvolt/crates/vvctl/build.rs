//! Windows (MSVC) gives the main thread a 1 MiB stack, which the large `main`
//! frame of a debug build overflows (even `vvctl --version` crashed). Reserve
//! 16 MiB for the binaries; no effect on other targets.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        println!("cargo:rustc-link-arg-bins=/STACK:16777216");
    }
}
