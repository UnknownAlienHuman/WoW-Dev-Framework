//! Synthetic declarations pass through the production source loader and emitter.
use super::Result;
use std::{collections::BTreeMap, fs, path::Path, sync::atomic::AtomicBool};
use wow_reference::native::{ingest_document, source_digest};

const SYSTEM: &str = r#"APIDocumentation:AddDocumentationTable({Name="Probe",Type="System",Namespace="C_Probe",Functions={
{Name="Read",Arguments={{Name="id",Type="number"}},Returns={{Name="count",Type="number"},{Name="label",Type="string"}}},
{Name="Record",Returns={{Name="record",Type="ProbeRecord"}}},
{Name="Object",Returns={{Name="object",Type="ProbeObjectAPI"}}},
{Name="Numbers",Arguments={{Name="values",Type="table",InnerType="number"}}},
{Name="Mode",Arguments={{Name="mode",Type="ProbeMode"}}},
{Name="Optional",Arguments={{Name="value",Type="number",Nilable=true}}}
},Tables={
{Name="ProbeRecord",Type="Structure",Fields={{Name="label",Type="string"}}},
{Name="ProbeMode",Type="Enumeration",Fields={{Name="Active",EnumValue=1},{Name="Inactive",EnumValue=0}}}
}})"#;
const OBJECT: &str = r#"APIDocumentation:AddDocumentationTable({Name="ProbeObjectAPI",Type="ScriptObject",Functions={
{Name="SetLabel",Arguments={{Name="label",Type="string"}}},
{Name="Label",Returns={{Name="label",Type="string"}}}
}})"#;
const POSITIVE: &str = r#"local count, label = C_Probe.Read(42)
print(count + 1, label:upper())
local record = C_Probe.Record()
print(record.label:upper())
local object = C_Probe.Object()
object:SetLabel("test")
print(object:Label():upper())
C_Probe.Numbers({1, 2, 3})
C_Probe.Mode(Enum.ProbeMode.Active)
C_Probe.Optional()
C_Probe.Optional(nil)
C_Probe.Optional(1)
"#;
/// Line is zero-based, as reported by the consumers' LSP diagnostics.
pub const NEGATIVES: &[(&str, &str, u64)] = &[
    ("argument.lua", "C_Probe.Read(\"wrong\")\n", 0),
    (
        "return.lua",
        "---@type string\nlocal wrong = C_Probe.Read(42)\nprint(wrong)\n",
        1,
    ),
    (
        "multiple.lua",
        "local _, label = C_Probe.Read(42)\n---@type number\nlocal wrong = label\nprint(wrong)\n",
        2,
    ),
    (
        "structure.lua",
        "local record = C_Probe.Record()\n---@type number\nlocal wrong = record.label\nprint(wrong)\n",
        2,
    ),
    (
        "method.lua",
        "local object = C_Probe.Object()\nobject:SetLabel(42)\n",
        1,
    ),
    ("array.lua", "C_Probe.Numbers({\"wrong\"})\n", 0),
    ("enum.lua", "C_Probe.Mode(\"wrong\")\n", 0),
    (
        "missing.lua",
        "local object = C_Probe.Object()\nobject:Missing()\n",
        1,
    ),
    ("global.lua", "NonexistentProbeGlobal()\n", 0),
];
pub fn prepare(root: &Path) -> Result<BTreeMap<String, String>> {
    fs::create_dir(root)?;
    let cancelled = AtomicBool::new(false);
    let documents = [("system.lua", SYSTEM), ("object.lua", OBJECT)]
        .iter()
        .map(|(path, source)| {
            ingest_document(
                &"1".repeat(40),
                path,
                source,
                &source_digest(source.as_bytes()),
                &cancelled,
            )
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let library = wow_annotations::native::project(&documents, "Mainline", &cancelled)?;
    if !library.issues.is_empty() {
        return Err("probe source projection is incomplete".into());
    }
    for file in &library.files {
        fs::write(root.join(&file.path), &file.text)?;
    }
    fs::write(root.join("positive.lua"), POSITIVE)?;
    // Ketho namespaces are open tables; this is an observed limitation, not a positive API assertion.
    fs::write(root.join("open-namespace.lua"), "C_Probe.Missing()\n")?;
    for (name, text, _) in NEGATIVES {
        fs::write(root.join(name), text)?;
    }
    // Only test-owned JSON config. No user settings, globals, suppression or Lua config.
    fs::write(
        root.join("emmy.json"),
        r#"{"runtime":{"version":"Lua5.1"},"diagnostics":{"enable":true}}"#,
    )?;
    fs::write(
        root.join("luals.json"),
        r#"{"runtime.version":"Lua 5.1","diagnostics.enable":true,"workspace.checkThirdParty":false}"#,
    )?;
    snapshot(root)
}
pub fn snapshot(root: &Path) -> Result<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let meta = fs::symlink_metadata(entry.path())?;
        if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > 1024 * 1024 {
            return Err("unexpected probe input entry".into());
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "non-UTF-8 fixture name")?;
        result.insert(name, source_digest(&fs::read(entry.path())?));
    }
    Ok(result)
}
