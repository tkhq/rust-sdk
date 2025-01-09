use prost_build;
use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(
        &["proto/services/coordinator/public/v1/public_api.proto"],
        &["proto"],
    )?;
    Ok(())
}
