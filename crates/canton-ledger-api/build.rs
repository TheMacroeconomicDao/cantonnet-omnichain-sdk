//! Compile Ledger API proto files when present in proto/ (see proto/README.md).

use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = Path::new("proto");
    let v2_dir = proto_dir.join("com/daml/ledger/api/v2");

    if !v2_dir.join("command_service.proto").exists() {
        println!("cargo:warning=Ledger API v2 proto not in proto/. See proto/README.md.");
        println!("cargo:rerun-if-changed=proto/");
        return Ok(());
    }

    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &[
                v2_dir.join("command_service.proto"),
                v2_dir.join("command_submission_service.proto"),
                v2_dir.join("command_completion_service.proto"),
                v2_dir.join("update_service.proto"),
                v2_dir.join("state_service.proto"),
                v2_dir.join("admin/party_management_service.proto"),
                v2_dir.join("package_service.proto"),
                v2_dir.join("version_service.proto"),
            ],
            &[proto_dir],
        )?;

    println!("cargo:rustc-cfg=proto_compiled");
    println!("cargo:rerun-if-changed=proto/");
    Ok(())
}
