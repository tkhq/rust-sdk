use std::io::Result;
use prost_build;

fn main() -> Result<()> {
    prost_build::compile_protos(&["proto/services/coordinator/public/v1/public_api.proto"], &["proto"])?;
    Ok(())
}
