use std::{env, error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("stream_descriptor.bin"))
        .compile(&["proto/stream.proto"], &["proto"])?;

    tonic_build::configure().compile(&["proto/event.proto"], &["proto"])?;

    Ok(())
}
