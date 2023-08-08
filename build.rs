use std::io::Result;
use prost_build::compile_protos;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=src/main.rs");
    compile_protos(&["src/gtfs-realtime.proto"], &["src/"])?;
    Ok(())
}
