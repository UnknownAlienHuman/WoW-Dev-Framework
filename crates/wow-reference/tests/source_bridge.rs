use std::error::Error;
use std::io;
use std::path::Path;
use std::process::Command;

#[test]
fn python_and_rust_share_the_executable_wire_contract() -> Result<(), Box<dyn Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let python = std::env::var_os("PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let output = Command::new(python)
        .arg(root.join("tests/source_bridge_roundtrip.py"))
        .env("WDF_API_BIN", env!("CARGO_BIN_EXE_wow-reference-api"))
        .env(
            "WDF_TOPOLOGY_BIN",
            env!("CARGO_BIN_EXE_wow-reference-topology"),
        )
        .env("WDF_SOURCE_BIN", env!("CARGO_BIN_EXE_wow-reference-source"))
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "source bridge failed: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
        .into());
    }
    Ok(())
}
