fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path().unwrap();
    // SAFETY: build scripts are single-threaded; no concurrent env access.
    unsafe { std::env::set_var("PROTOC", protoc) };

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["proto/ekos/v1/kernel.proto"], &["proto"])?;
    Ok(())
}
