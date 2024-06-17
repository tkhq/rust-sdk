use prost_build;
use std::io::Result;
use std::path::PathBuf;

// Generate the rust definitions from protobuf inputs.
// To understand this, see the usage documentation in pbjson_build

fn main() -> Result<()> {
    let descriptor_path =
        PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("proto_descriptor.bin");

    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        .compile_well_known_types()
        .extern_path(".google.protobuf", "::pbjson_types")
        .compile_protos(
            &["proto/services/coordinator/public/v1/public_api.proto"],
            &["proto"],
        )?;

    let descriptor_set = std::fs::read(descriptor_path)?;
    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_set)?
        .emit_fields()
        .build(&[
            ".external.activity.v1",
            ".external.data.v1",
            ".external.options.v1",
            ".external.webauthn.v1",
            ".google.api",
            ".google.rpc",
            ".grpc.gateway.protoc_gen_openapiv2.options",
            ".immutable.activity.v1",
            ".immutable.common.v1",
            ".immutable.data.v1",
            ".immutable.webauthn.v1",
            ".services.coordinator.public.v1",
        ])?;

    Ok(())
}
