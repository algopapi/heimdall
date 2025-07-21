use std::{env, error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let descriptor_path = std::path::PathBuf::from(out_dir).join("stream_descriptor.bin");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(descriptor_path)
        .compile(&["proto/stream.proto"], &["proto"])?;

    Ok(())
}
