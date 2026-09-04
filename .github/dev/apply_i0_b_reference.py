from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CRATE = ROOT / "crates" / "wow-reference"


def ensure_workspace_entry(source: str, key: str, entry: str) -> str:
    pattern = re.compile(rf"(?ms)^(?P<prefix>{re.escape(key)}\s*=\s*\[)(?P<body>.*?)(?P<suffix>^\])")
    match = pattern.search(source)
    if match is None:
        if key == "members":
            raise SystemExit("workspace members array was not found")
        return source
    body = match.group("body")
    if entry in body or "crates/*" in body:
        return source
    insertion = f'    "{entry}",\n'
    return source[: match.start("body")] + body + insertion + source[match.end("body") :]


def ensure_dependency(source: str, name: str, specification: str) -> str:
    if re.search(rf"(?m)^\s*{re.escape(name)}\s*=", source):
        return source
    header = "[dependencies]"
    if header not in source:
        return source.rstrip() + f"\n\n{header}\n{name} = {specification}\n"
    start = source.index(header) + len(header)
    return source[:start] + f"\n{name} = {specification}" + source[start:]


def replace_test_function(source: str, function_name: str, replacement: str) -> str:
    marker = f"fn {function_name}()"
    marker_index = source.find(marker)
    if marker_index < 0:
        return source
    test_index = source.rfind("#[test]", 0, marker_index)
    if test_index < 0:
        raise SystemExit(f"test attribute for {function_name} was not found")
    brace_index = source.find("{", marker_index)
    if brace_index < 0:
        raise SystemExit(f"opening brace for {function_name} was not found")
    depth = 0
    end_index = -1
    for index in range(brace_index, len(source)):
        character = source[index]
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
            if depth == 0:
                end_index = index + 1
                break
    if end_index < 0:
        raise SystemExit(f"closing brace for {function_name} was not found")
    return source[:test_index] + replacement.rstrip() + source[end_index:]


CRATE.mkdir(parents=True, exist_ok=True)
(CRATE / "src").mkdir(parents=True, exist_ok=True)
(CRATE / "tests").mkdir(parents=True, exist_ok=True)

lib_path = CRATE / "src" / "lib.rs"
exports = '''
// I0-B deterministic reference-view boundary.
mod deterministic_view;

pub use deterministic_view::{
    CoverageStatus, LookupResult, LookupUnknownReason, REFERENCE_VIEW_SCHEMA, ReferenceConflict,
    ReferencePartition, ReferenceRecord, ReferenceRecordKind, ReferenceView, ReferenceViewError,
    ReferenceViewResult, RestrictionFacet, RestrictionState,
};
'''
if lib_path.exists():
    source = lib_path.read_text(encoding="utf-8")
    if "mod deterministic_view;" not in source:
        source = source.rstrip() + "\n\n" + exports.lstrip()
    elif "ReferenceViewResult" not in source:
        source = source.rstrip() + "\n\n" + exports.split("pub use", 1)[1].join(["pub use", ""])
    lib_path.write_text(source, encoding="utf-8")
else:
    lib_path.write_text(
        "//! Deterministic reference-data boundary for the WoW development framework.\n\n"
        + exports.lstrip(),
        encoding="utf-8",
    )

manifest_path = CRATE / "Cargo.toml"
if manifest_path.exists():
    manifest = manifest_path.read_text(encoding="utf-8")
else:
    manifest = '''[package]
name = "wow-reference"
version = "0.1.0"
edition = "2024"
rust-version = "1.98"
license = "MIT OR Apache-2.0"
publish = false

[dependencies]
'''
for name, specification in (
    ("serde", '{ version = "=1.0.228", features = ["derive"] }'),
    ("serde_json", '"=1.0.150"'),
    ("sha2", '"=0.11.0"'),
    ("wow-core", '{ path = "../wow-core" }'),
):
    manifest = ensure_dependency(manifest, name, specification)
manifest_path.write_text(manifest.rstrip() + "\n", encoding="utf-8")

root_manifest_path = ROOT / "Cargo.toml"
root_manifest = root_manifest_path.read_text(encoding="utf-8")
root_manifest = re.sub(
    r'(?m)^\s*"?crates/wow-reference"?\s*,?\s*$\n?',
    "",
    root_manifest,
) if "exclude" in root_manifest and "crates/wow-reference" in root_manifest else root_manifest
root_manifest = ensure_workspace_entry(root_manifest, "members", "crates/wow-reference")
root_manifest = ensure_workspace_entry(root_manifest, "default-members", "crates/wow-reference")
root_manifest_path.write_text(root_manifest, encoding="utf-8")

policy_path = ROOT / "crates" / "wow-core" / "tests" / "repository_policy.rs"
if policy_path.exists():
    policy = policy_path.read_text(encoding="utf-8")
    policy = replace_test_function(
        policy,
        "workspace_activates_only_wow_core",
        '''#[test]
fn workspace_activates_foundation_crates() {
    assert!(ROOT_MANIFEST.contains("\\\"crates/wow-core\\\""));
    assert!(ROOT_MANIFEST.contains("\\\"crates/wow-reference\\\""));
}''',
    )
    policy_path.write_text(policy, encoding="utf-8")
