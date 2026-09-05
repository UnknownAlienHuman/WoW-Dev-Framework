//! Native CLI compatibility checks for retained v1 wire importers. Fixtures are
//! synthetic and do not claim current Blizzard inventory or runtime evidence.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use wow_reference::wire_json::canonical_json_bytes;
const API: &str = include_str!("fixtures/api-v1.json");
const TOPOLOGY: &str = include_str!("fixtures/topology-v1.json");
const API_BIN: &str = env!("CARGO_BIN_EXE_wow-reference-api");
const TOP_BIN: &str = env!("CARGO_BIN_EXE_wow-reference-topology");
const SOURCE_BIN: &str = env!("CARGO_BIN_EXE_wow-reference-source");
type Result<T> = std::result::Result<T, Box<dyn Error>>;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Result<Self> {
        let root = std::env::temp_dir().join(format!(
            "wdf-native-wire-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root)?;
        fs::write(root.join("api.json"), API)?;
        fs::write(root.join("topology.json"), TOPOLOGY)?;
        Ok(Self(root))
    }
    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn call(bin: &str, args: &[&std::ffi::OsStr], code: i32) -> Result<Value> {
    let output = Command::new(bin).args(args).output()?;
    assert_eq!(
        output.status.code(),
        Some(code),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    if code == 0 {
        Ok(serde_json::from_slice(&output.stdout)?)
    } else {
        Ok(Value::Null)
    }
}
fn reseal(path: &Path, mut value: Value, key: &str) -> Result<()> {
    value.as_object_mut().ok_or("invalid fixture")?.remove(key);
    let bytes = canonical_json_bytes(&value)?;
    value[key] = json!(format!(
        "sha256:{}",
        Sha256::digest(&bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    ));
    fs::write(path, serde_json::to_vec(&value)?)?;
    Ok(())
}
#[test]
fn native_wire_cli_lookup_and_idempotent_no_clobber_publication() -> Result<()> {
    let fixture = Fixture::new()?;
    let api = fixture.path("api.json");
    let top = fixture.path("topology.json");
    let bundle = fixture.path("bundle.json");
    let summary = call(API_BIN, &["verify".as_ref(), api.as_os_str()], 0)?;
    assert_eq!(summary["facts"], 1);
    assert_eq!(
        call(
            API_BIN,
            &[
                "lookup".as_ref(),
                api.as_os_str(),
                "function".as_ref(),
                "C_Fixture.Lookup".as_ref()
            ],
            0
        )?["status"],
        "found"
    );
    assert_eq!(
        call(
            API_BIN,
            &[
                "lookup".as_ref(),
                api.as_os_str(),
                "function".as_ref(),
                "C_Fixture.Missing".as_ref()
            ],
            0
        )?["status"],
        "absent_authoritative"
    );
    call(TOP_BIN, &["verify".as_ref(), top.as_os_str()], 0)?;
    let args = [
        "materialize".as_ref(),
        api.as_os_str(),
        top.as_os_str(),
        bundle.as_os_str(),
    ];
    call(SOURCE_BIN, &args, 0)?;
    let first = fs::read(&bundle)?;
    call(SOURCE_BIN, &args, 0)?;
    assert_eq!(first, fs::read(&bundle)?);
    fs::write(&bundle, b"unrelated output")?;
    call(SOURCE_BIN, &args, 2)?;
    assert_eq!(fs::read(&bundle)?, b"unrelated output");
    Ok(())
}
#[test]
fn native_wire_cli_rejects_tampering_and_cross_generation_bundle() -> Result<()> {
    let fixture = Fixture::new()?;
    let api = fixture.path("api.json");
    let top = fixture.path("topology.json");
    let mut value: Value = serde_json::from_str(API)?;
    value["source"]["version"] = json!("tampered");
    fs::write(&api, serde_json::to_vec(&value)?)?;
    call(API_BIN, &["verify".as_ref(), api.as_os_str()], 2)?;
    value["source"]["revision"] = json!("5".repeat(40));
    reseal(&api, value, "draft_sha256")?;
    call(API_BIN, &["verify".as_ref(), api.as_os_str()], 0)?;
    call(
        SOURCE_BIN,
        &["verify".as_ref(), api.as_os_str(), top.as_os_str()],
        2,
    )?;
    Ok(())
}
#[test]
fn native_wire_cli_keeps_partial_topology_non_authoritative() -> Result<()> {
    let fixture = Fixture::new()?;
    let top = fixture.path("topology.json");
    let mut value: Value = serde_json::from_str(TOPOLOGY)?;
    value["coverage"]["status"] = json!("partial");
    value["coverage"]["negative_authority"] = json!(false);
    value["coverage"]["unresolved_references"] = json!(1);
    value["descriptors"][0]["entries"][0]["resolution"] = json!("missing");
    value["edges"][0]["resolution"] = json!("missing");
    value["issues"] = json!([{"code":"missing_target","source_path":"Interface/AddOns/Fixture/Fixture.toc","line":2,"declared":"Logic.lua","target":"Interface/AddOns/Fixture/Logic.lua","message":"missing"}]);
    reseal(&top, value.clone(), "topology_sha256")?;
    let summary = call(TOP_BIN, &["verify".as_ref(), top.as_os_str()], 0)?;
    assert_eq!(summary["negative_authority"], false);
    assert_eq!(
        call(
            TOP_BIN,
            &[
                "document".as_ref(),
                top.as_os_str(),
                "Interface/AddOns/Missing.xml".as_ref()
            ],
            0
        )?["status"],
        "not_authoritative"
    );
    value["issues"] = json!([]);
    value["coverage"]["status"] = json!("complete");
    value["coverage"]["negative_authority"] = json!(true);
    value["coverage"]["unresolved_references"] = json!(0);
    reseal(&top, value, "topology_sha256")?;
    call(TOP_BIN, &["verify".as_ref(), top.as_os_str()], 2)?;
    Ok(())
}
