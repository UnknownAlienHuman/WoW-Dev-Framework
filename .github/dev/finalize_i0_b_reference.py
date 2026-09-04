from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CRATE = ROOT / "crates" / "wow-reference"


def rewrite_array(source: str, key: str, transform) -> str:
    match = re.search(rf"(?m)^(?P<indent>\s*){re.escape(key)}\s*=\s*\[", source)
    if match is None:
        return source
    opening = source.find("[", match.start())
    depth = 0
    closing = None
    in_string = False
    escaped = False
    for index in range(opening, len(source)):
        character = source[index]
        if in_string:
            if escaped:
                escaped = False
            elif character == "\\":
                escaped = True
            elif character == '"':
                in_string = False
            continue
        if character == '"':
            in_string = True
        elif character == "[":
            depth += 1
        elif character == "]":
            depth -= 1
            if depth == 0:
                closing = index
                break
    if closing is None:
        raise SystemExit(f"unterminated {key} array")
    values = re.findall(r'"([^"\\]*(?:\\.[^"\\]*)*)"', source[opening + 1 : closing])
    values = transform(values)
    indent = match.group("indent")
    replacement = "[\n" + "".join(f'{indent}    "{value}",\n' for value in values) + f"{indent}]"
    return source[:opening] + replacement + source[closing + 1 :]


def ensure_dependency(source: str, name: str, specification: str) -> str:
    header = re.search(r"(?m)^\[dependencies\]\s*$", source)
    if header is None:
        return source.rstrip() + f"\n\n[dependencies]\n{name} = {specification}\n"
    section_end = re.search(r"(?m)^\[[^]]+\]\s*$", source[header.end() :])
    end = len(source) if section_end is None else header.end() + section_end.start()
    section = source[header.end() : end]
    if re.search(rf"(?m)^\s*{re.escape(name)}\s*=", section):
        return source
    return source[:end].rstrip() + f"\n{name} = {specification}\n\n" + source[end:].lstrip("\n")


def replace_test_function(source: str, function_name: str, replacement: str) -> str:
    marker_index = source.find(f"fn {function_name}()")
    if marker_index < 0:
        return source
    test_index = source.rfind("#[test]", 0, marker_index)
    brace_index = source.find("{", marker_index)
    if test_index < 0 or brace_index < 0:
        raise SystemExit(f"malformed test function {function_name}")
    depth = 0
    for index in range(brace_index, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[:test_index] + replacement.rstrip() + source[index + 1 :]
    raise SystemExit(f"unterminated test function {function_name}")


if not (CRATE / "src" / "deterministic_view.rs").is_file():
    raise SystemExit("deterministic_view.rs was not published")
if not (CRATE / "tests" / "deterministic_view.rs").is_file():
    raise SystemExit("deterministic_view integration tests were not published")

lib_path = CRATE / "src" / "lib.rs"
exports = '''// I0-B deterministic reference-view boundary.
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
        source = source.rstrip() + "\n\n" + exports
    elif "ReferenceViewResult" not in source:
        source = source.rstrip() + "\n\n" + exports[exports.index("pub use") :]
else:
    source = "//! Deterministic reference-data boundary for the WoW development framework.\n\n" + exports
lib_path.write_text(source.rstrip() + "\n", encoding="utf-8")

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
root_manifest = rewrite_array(
    root_manifest,
    "exclude",
    lambda values: [value for value in values if value != "crates/wow-reference"],
)

def add_reference(values: list[str]) -> list[str]:
    if "crates/*" not in values and "crates/wow-reference" not in values:
        values.append("crates/wow-reference")
    return sorted(dict.fromkeys(values))

before = root_manifest
root_manifest = rewrite_array(root_manifest, "members", add_reference)
if root_manifest == before and not re.search(r"(?m)^members\s*=", root_manifest):
    raise SystemExit("workspace members array was not found")
root_manifest = rewrite_array(root_manifest, "default-members", add_reference)
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
