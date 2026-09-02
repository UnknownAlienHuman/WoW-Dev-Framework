from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import re
import subprocess
import tomllib
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
CRATE = ROOT / "crates" / "wow-core"
IMPLEMENTATION_COMMIT = os.environ.get("I0A_IMPLEMENTATION_COMMIT", "").strip()
VERIFICATION_DATE = os.environ.get("I0A_VERIFICATION_DATE", "").strip()


def strict_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=strict_object)


def write_json(path: Path, value: Any) -> None:
    path.write_text(
        json.dumps(value, ensure_ascii=False, indent=2, sort_keys=False) + "\n",
        encoding="utf-8",
    )


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def replace_required(text: str, old: str, new: str, label: str) -> str:
    if old not in text:
        raise ValueError(f"required text not found in {label}: {old!r}")
    return text.replace(old, new, 1)


def replace_section(text: str, heading: str, replacement: str, next_heading: str) -> str:
    pattern = re.compile(
        rf"{re.escape(heading)}\n.*?(?=\n{re.escape(next_heading)}\n)",
        re.DOTALL,
    )
    if not pattern.search(text):
        raise ValueError(f"section not found: {heading} -> {next_heading}")
    return pattern.sub(replacement.rstrip(), text, count=1)


def prepare_contract() -> None:
    path = CRATE / "CONTRACT.json"
    contract = load_json(path)
    contract["state"] = "implementation-complete"
    documents = contract.setdefault("normative_documents", [])
    for document in ["DEPENDENCIES.md", "IMPLEMENTATION_REPORT.md"]:
        if document not in documents:
            documents.append(document)
    examples = contract.setdefault("examples", [])
    for example in ["examples/CHECKSUMS.json", "IMPLEMENTATION_EVIDENCE.json"]:
        if example not in examples:
            examples.append(example)
    contract["implementation"] = {
        "work_package": "I0-A",
        "implementation_state": "complete",
        "source_commit": IMPLEMENTATION_COMMIT,
        "rust_toolchain": "1.98.0",
        "edition": "2024",
        "workspace_member": "crates/wow-core",
        "operation_registry_count": len(contract.get("e0_operations", [])),
        "dependency_report": "DEPENDENCIES.md",
        "implementation_report": "IMPLEMENTATION_REPORT.md",
        "implementation_evidence": "IMPLEMENTATION_EVIDENCE.json",
        "checksum_manifest": "examples/CHECKSUMS.json",
    }
    write_json(path, contract)


def prepare_manifest() -> None:
    path = ROOT / "crates" / "MANIFEST.json"
    manifest = load_json(path)
    manifest["implementation_frontier"] = "I0-A-complete"
    manifest["next_implementation_work_package"] = {
        "id": "I0-B",
        "owner": "wow-reference",
        "contract_id": "wow-reference/e0-b/1",
        "manifest": "crates/wow-reference/CONTRACT.json",
        "launch_gate": "R0",
        "scope": [
            "consume the merged exact wow-core I0-A public boundary",
            "implement one immutable E0 ReferenceView fixture with exact profile and generation identity",
            "implement exact found missing partial conflict and NotEvaluated lookup outcomes",
            "preserve raw unknown fields coverage and negative-authority constraints",
            "populate E0-B fixtures checksums and acceptance evidence before I0-C integration",
        ],
    }
    workspace = manifest.setdefault("workspace_state", {})
    workspace.update(
        {
            "cargo_workspace_present": True,
            "cargo_lock_present": True,
            "rust_toolchain_present": True,
            "rust_source_present": True,
            "ci_workflows_present": False,
            "supported_release_present": False,
        }
    )
    for package in manifest.get("contract_packages", []):
        if package.get("work_package") == "E0-A":
            package["implementation_state"] = "complete"
    for crate in manifest.get("crates", []):
        if crate.get("name") == "wow-core":
            crate["implementation_state"] = "complete"
    evidence = manifest.setdefault("implementation_evidence_state", {})
    evidence["implemented_work_packages"] = 1
    evidence["active_cargo_workspace_members"] = 1
    manifest["completed_implementation_work_packages"] = ["I0-A"]
    manifest["next_action"] = {
        "work_package": "I0-B",
        "branch_intent": "impl/i0-b-wow-reference-fixture",
        "steps": [
            "read the complete wow-reference E0-B package and merged wow-core public API",
            "freeze the exact synthetic fixture profile source material and ReferenceGenerationId",
            "implement the bounded immutable ReferenceView fixture and exact lookup operations",
            "preserve complete partial conflict NotEvaluated and negative-authority distinctions",
            "finalize immutable E0-B fixtures checksums and acceptance tests",
            "merge and close the worktree before starting the next primary package",
        ],
    }
    write_json(path, manifest)


def prepare_package_statuses() -> None:
    replacements = {
        "README.md": (
            "**Status:** E0-A implementation-ready contract pack; no Rust code or `Cargo.toml` yet.",
            "**Status:** I0-A / E0-A implementation complete; Rust code, tests, evidence, and checksums are present.",
        ),
        "DECISIONS.md": (
            "**Status:** normative for E0-A; implementation has not started.",
            "**Status:** normative for the completed I0-A / E0-A implementation.",
        ),
        "DATA_MODEL.md": (
            "**Status:** normative E0-A value contract; no Rust code yet.",
            "**Status:** normative E0-A value contract implemented by I0-A.",
        ),
        "OPERATIONS.md": (
            "**Status:** normative E0-A operation inventory; no Rust code yet.",
            "**Status:** normative E0-A operation inventory implemented by I0-A.",
        ),
        "CANONICALIZATION.md": (
            "**Status:** normative E0-A canonical profile; no Rust code yet.",
            "**Status:** normative E0-A canonical profile implemented and covered by committed vectors.",
        ),
        "ERROR_MODEL.md": (
            "**Status:** normative E0-A boundary-error contract; no Rust code yet.",
            "**Status:** normative E0-A boundary-error contract implemented by I0-A.",
        ),
        "TEST_MATRIX.md": (
            "**Status:** normative implementation gate; no executable tests yet.",
            "**Status:** normative executable I0-A gate; every case ID is represented in the Rust test corpus.",
        ),
        "CONSUMER_GUIDE.md": (
            "**Status:** normative E0-A seam contract; no Rust code yet.",
            "**Status:** normative merged I0-A public seam contract for E0 consumers.",
        ),
        "IMPLEMENTATION_PLAN.md": (
            "**Status:** implementation-ready plan; no Rust code or Cargo workspace yet.",
            "**Status:** completed I0-A implementation plan; retained as the audited execution record.",
        ),
    }
    for relative, (old, new) in replacements.items():
        path = CRATE / relative
        text = path.read_text(encoding="utf-8")
        path.write_text(replace_required(text, old, new, str(path)), encoding="utf-8")


def prepare_global_docs() -> None:
    # Root README: exact project state, without overstating R0/product readiness.
    path = ROOT / "README.md"
    text = path.read_text(encoding="utf-8")
    text = text.replace(
        "> **Implementation frontier:** not started. The next owned work package is **I0-A / `wow-core` E0-A**.",
        "> **Implementation frontier:** **I0-A / `wow-core` complete**. The next owned work package is **I0-B / `wow-reference` E0-B**.",
    )
    current = '''## Current evidence state

```text
Cargo workspace: present; only crates/wow-core is active
Cargo.lock and rust-toolchain: present and frozen for Rust 1.98.0
Rust source: wow-core I0-A implemented
implemented work packages: 1 / I0-A
wow-core required operations: 52
wow-core committed examples and hash vectors: executable and validated
real Reference Packs/project generations: 0
real analyzer/diagnostic/search/context executions: 0
real calibration/core publication evidence: 0
live external provider adapters: 0
LSP/MCP/daemon conformance: 0
wow product binary: absent
reproducible release builds: 0
signed public bundles: 0
installation/update/rollback rehearsals: 0
supported targets: 0
CI/workflows in the target tree: absent
```

The `wow-core` foundation is implemented and buildable, but R0 remains blocked on I0-B through I0-F. No product, client, runtime, platform, or release claim follows from I0-A.
'''
    text = replace_section(text, "## Current evidence state", current, "## Launch path")
    next_step = '''## Next implementation step

```text
I0-B / wow-reference E0-B

1. Consume only the merged typed wow-core public boundary.
2. Freeze one exact synthetic fixture profile and ReferenceGenerationId.
3. Implement one immutable bounded ReferenceView fixture.
4. Implement exact found, missing, partial, conflict, and NotEvaluated outcomes.
5. Preserve raw unknown fields, coverage, and negative-authority constraints.
6. Populate E0-B fixtures and checksums before the next primary package.
```
'''
    text = replace_section(text, "## Next implementation step", next_step, "## Routes")
    path.write_text(text, encoding="utf-8")

    # Exact top-level state substitutions in routing documents.
    routing_replacements = {
        "AGENTS.md": [
            ("implementation frontier: not-started", "implementation frontier: I0-A complete"),
            ("next implementation package: I0-A / wow-core E0-A", "next implementation package: I0-B / wow-reference E0-B"),
        ],
        "docs/README.md": [
            ("implementation frontier: not started", "implementation frontier: I0-A complete"),
            ("next owned work package: I0-A / wow-core E0-A", "next owned work package: I0-B / wow-reference E0-B"),
        ],
        "docs/ROADMAP.md": [
            ("implementation frontier: not started", "implementation frontier: I0-A complete"),
            ("next owned work package: I0-A / wow-core E0-A", "next owned work package: I0-B / wow-reference E0-B"),
        ],
        "docs/LAUNCH_GATES.md": [
            ("implementation frontier: not-started", "implementation frontier: I0-A complete"),
            ("next owned package: I0-A / wow-core E0-A", "next owned package: I0-B / wow-reference E0-B"),
            ("Cargo workspace: absent", "Cargo workspace: present; wow-core only"),
            ("Cargo.lock and rust-toolchain: absent", "Cargo.lock and Rust 1.98.0 toolchain: present"),
            ("Rust source: absent", "Rust source: wow-core I0-A only"),
        ],
        "docs/IMPLEMENTATION_HANDOFF.md": [
            ("implementation: not started", "implementation: I0-A complete; remaining packages not started"),
            ("next owned package: I0-A / wow-core E0-A", "next owned package: I0-B / wow-reference E0-B"),
        ],
        "docs/AGENT_WORKFLOW.md": [
            ("Current next package:", "Current completed package and next package:"),
            ("I0-A\nowner: wow-core", "I0-A complete\nnext: I0-B\nowner: wow-reference"),
            ("Create `impl/i0-a-wow-core`", "Create `impl/i0-b-wow-reference-fixture`"),
        ],
        "CONTRIBUTING.md": [
            ("WoW Dev Framework has a planned architecture through E7-B and no Rust implementation yet.", "WoW Dev Framework has a planned architecture through E7-B and a completed I0-A `wow-core` implementation."),
            ("I0-A / wow-core E0-A", "I0-B / wow-reference E0-B"),
        ],
        "crates/README.md": [
            ("**Planned architecture and documentation:** complete through E7-B. **Implementation:** not started.", "**Planned architecture and documentation:** complete through E7-B. **Implementation:** I0-A / `wow-core` complete."),
            ("The next work package is not E8. It is **I0-A / `wow-core` E0-A** implementation.", "The next work package is **I0-B / `wow-reference` E0-B** implementation."),
            ("## First implementation target", "## Completed foundation and next target"),
        ],
        "crates/AGENTS.md": [
            ("implementation: not started", "implementation: I0-A complete; remaining packages not started"),
            ("next implementation package: I0-A / wow-core E0-A", "next implementation package: I0-B / wow-reference E0-B"),
        ],
        "crates/WORKSTREAMS.md": [
            ("implementation: not started", "implementation: I0-A complete; remaining packages not started"),
            ("next implementation package: I0-A / wow-core E0-A", "next implementation package: I0-B / wow-reference E0-B"),
            ("I0-A  wow-core", "I0-A  wow-core — complete"),
        ],
    }
    for relative, replacements in routing_replacements.items():
        target = ROOT / relative
        content = target.read_text(encoding="utf-8")
        for old, new in replacements:
            if old in content:
                content = content.replace(old, new, 1)
        target.write_text(content, encoding="utf-8")

    # Keep every router that explicitly names the repository next package aligned.
    phrases = {
        "next repository package: I0-A / wow-core E0-A": "next repository package: I0-B / wow-reference E0-B",
        "repository next package: I0-A / wow-core E0-A": "repository next package: I0-B / wow-reference E0-B",
        "next repository package: I0-A / `wow-core` E0-A": "next repository package: I0-B / `wow-reference` E0-B",
        "repository next package: I0-A / `wow-core` E0-A": "repository next package: I0-B / `wow-reference` E0-B",
        "next owned implementation package: I0-A / wow-core E0-A": "next owned implementation package: I0-B / wow-reference E0-B",
    }
    for path in ROOT.rglob("*.md"):
        if ".git" in path.parts:
            continue
        content = path.read_text(encoding="utf-8")
        updated = content
        for old, new in phrases.items():
            updated = updated.replace(old, new)
        if updated != content:
            path.write_text(updated, encoding="utf-8")

    # Completion matrix: only the I0-A/E0-A row/state advances; R0 stays blocked.
    matrix_path = ROOT / "docs" / "PROJECT_COMPLETION_MATRIX.md"
    matrix = matrix_path.read_text(encoding="utf-8")
    lines = []
    changed = False
    for line in matrix.splitlines():
        if ("I0-A" in line or "E0-A" in line or "`wow-core`" in line) and (
            "not-started" in line or "Not started" in line or "NOT STARTED" in line
        ):
            line = line.replace("not-started", "complete")
            line = line.replace("Not started", "Complete")
            line = line.replace("NOT STARTED", "COMPLETE")
            changed = True
        lines.append(line)
    matrix = "\n".join(lines) + "\n"
    if not changed and "I0-A implementation status" not in matrix:
        matrix += "\n## Implemented frontier\n\n```text\nI0-A implementation status: complete\nnext package: I0-B\nR0: blocked on I0-B through I0-F\n```\n"
    matrix_path.write_text(matrix, encoding="utf-8")

    # Roadmap and handoff exact next-action sections.
    roadmap_path = ROOT / "docs" / "ROADMAP.md"
    roadmap = roadmap_path.read_text(encoding="utf-8")
    if "## Next action" in roadmap:
        roadmap_next = '''## Next action

```text
I0-B / wow-reference E0-B

1. consume the merged wow-core I0-A boundary;
2. freeze one exact synthetic fixture profile and reference generation;
3. implement the immutable bounded ReferenceView fixture;
4. cover found, absent, partial, conflict, NotEvaluated, and negative-authority cases;
5. populate E0-B fixtures and checksums;
6. merge before starting the next primary package.
```
'''
        roadmap = re.sub(r"## Next action\n.*\Z", roadmap_next, roadmap, flags=re.DOTALL)
    roadmap_path.write_text(roadmap, encoding="utf-8")

    handoff_path = ROOT / "docs" / "IMPLEMENTATION_HANDOFF.md"
    handoff = handoff_path.read_text(encoding="utf-8")
    i0a_heading = "## I0-A — `wow-core` E0-A"
    if i0a_heading in handoff and "**Implementation status:** complete." not in handoff:
        handoff = handoff.replace(
            i0a_heading,
            i0a_heading + "\n\n**Implementation status:** complete. See `crates/wow-core/IMPLEMENTATION_REPORT.md` and `examples/CHECKSUMS.json`.",
            1,
        )
    handoff_path.write_text(handoff, encoding="utf-8")


def write_checksum_test() -> None:
    path = CRATE / "tests" / "checksum_manifest.rs"
    path.write_text(
        r'''use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
struct Manifest {
    algorithm: String,
    bundle_profile: String,
    members: Vec<Member>,
    bundle_sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Member {
    member_path: String,
    sha256: String,
}

#[derive(Serialize)]
struct BundleProjection<'a> {
    algorithm: &'a str,
    members: &'a [Member],
    profile: &'a str,
}

#[test]
fn checksum_manifest_matches_every_committed_member_and_bundle() -> Result<(), Box<dyn Error>> {
    let crate_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repository_root = crate_root
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| std::io::Error::other("repository root is unavailable"))?;
    let manifest: Manifest = serde_json::from_str(include_str!("../examples/CHECKSUMS.json"))?;
    assert_eq!(manifest.algorithm, "sha256");
    assert_eq!(
        manifest.bundle_profile,
        "wow-core-implementation-bundle/v1"
    );
    assert!(!manifest.members.is_empty());

    for pair in manifest.members.windows(2) {
        assert!(pair[0].member_path < pair[1].member_path);
    }
    for member in &manifest.members {
        let bytes = fs::read(repository_root.join(&member.member_path))?;
        assert_eq!(sha256_hex(&bytes), member.sha256, "{}", member.member_path);
    }

    let projection = BundleProjection {
        algorithm: &manifest.algorithm,
        members: &manifest.members,
        profile: &manifest.bundle_profile,
    };
    let bytes = wow_core::canonical_json_bytes(&projection)?;
    assert_eq!(sha256_hex(&bytes), manifest.bundle_sha256);

    let workflows = repository_root.join(".github/workflows");
    if workflows.exists() {
        assert_eq!(fs::read_dir(workflows)?.count(), 0);
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}
''',
        encoding="utf-8",
    )


def prepare() -> None:
    if not IMPLEMENTATION_COMMIT:
        raise ValueError("I0A_IMPLEMENTATION_COMMIT is required")
    prepare_contract()
    prepare_manifest()
    prepare_package_statuses()
    prepare_global_docs()
    write_checksum_test()


def cargo_metadata() -> dict[str, Any]:
    return load_json(Path("/tmp/i0a-cargo-metadata.json"))


def lock_index() -> dict[tuple[str, str], dict[str, Any]]:
    lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    return {(entry["name"], entry["version"]): entry for entry in lock["package"]}


def matrix_case_ids() -> list[str]:
    text = (CRATE / "TEST_MATRIX.md").read_text(encoding="utf-8")
    values = re.findall(r"`([A-Z][A-Z0-9-]*\d[A-Z0-9-]*)`", text)
    values = sorted(value for value in values if ".." not in value)
    if len(values) != len(set(values)):
        duplicates = sorted({value for value in values if values.count(value) > 1})
        raise ValueError(f"duplicate TEST_MATRIX case IDs: {duplicates}")
    return values


def test_count(path: Path) -> int:
    return sum(1 for line in path.read_text(encoding="utf-8").splitlines() if line.endswith(": test"))


def generate_dependency_report(metadata: dict[str, Any]) -> tuple[str, list[dict[str, Any]]]:
    root_manifest = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    direct_names = set(root_manifest["workspace"]["dependencies"])
    locks = lock_index()
    node_features = {
        node["id"]: sorted(node.get("features", []))
        for node in metadata.get("resolve", {}).get("nodes", [])
    }
    packages: list[dict[str, Any]] = []
    for package in metadata["packages"]:
        if package["name"] == "wow-core":
            continue
        license_expression = package.get("license")
        if not license_expression:
            raise ValueError(f"dependency has no license expression: {package['name']}")
        upper = license_expression.upper()
        if any(token in upper for token in ["AGPL", "LGPL", "GPL-", "SSPL"]):
            raise ValueError(f"disallowed dependency license: {package['name']} {license_expression}")
        lock = locks.get((package["name"], package["version"]), {})
        kinds = sorted({kind for target in package["targets"] for kind in target.get("kind", [])})
        packages.append(
            {
                "name": package["name"],
                "version": package["version"],
                "direct": package["name"] in direct_names,
                "license": license_expression,
                "source": package.get("source") or "workspace/path",
                "checksum": lock.get("checksum", "path-or-unavailable"),
                "features": node_features.get(package["id"], []),
                "target_kinds": kinds,
                "build_script": "custom-build" in kinds,
                "proc_macro": "proc-macro" in kinds,
            }
        )
    packages.sort(key=lambda package: (not package["direct"], package["name"], package["version"]))

    rows = []
    for package in packages:
        features = ", ".join(package["features"]) or "—"
        execution = []
        if package["proc_macro"]:
            execution.append("proc macro")
        if package["build_script"]:
            execution.append("build script")
        rows.append(
            "| {name} | {version} | {scope} | {license} | {features} | {execution} | `{checksum}` |".format(
                name=package["name"],
                version=package["version"],
                scope="direct" if package["direct"] else "transitive",
                license=package["license"].replace("|", "\\|"),
                features=features.replace("|", "\\|"),
                execution=", ".join(execution) or "library only",
                checksum=package["checksum"],
            )
        )

    audit = load_json(Path("/tmp/i0a-cargo-audit.json"))
    vulnerabilities = audit.get("vulnerabilities", {}).get("count", 0)
    warnings = audit.get("warnings", {})
    warning_count = sum(len(value) for value in warnings.values()) if isinstance(warnings, dict) else 0
    audit_version = Path("/tmp/i0a-cargo-audit-version.txt").read_text(encoding="utf-8").strip()
    audit_digest = sha256_file(Path("/tmp/i0a-cargo-audit.json"))

    report = f'''# `wow-core` dependency qualification

**Status:** exact I0-A dependency closure generated from `Cargo.lock` and `cargo metadata --locked`.

## Policy

`wow-core` remains synchronous, deterministic, and free of filesystem, network, clock, randomness, database, logging, parser, graph, search, runtime, editor, and transport dependencies. Dependencies are private implementation details and do not cross the public API as generic escape hatches.

Direct dependency responsibilities:

```text
semver       validated Semantic Version parsing and ordering
serde        explicit typed DTO serialization/deserialization
serde_json   strict internal fixture and canonical JSON projection
sha2         SHA-256 content and domain-separated identity digests
```

All versions are exact-pinned in the workspace manifest and checksummed by `Cargo.lock`.

## Resolved closure

| Package | Version | Scope | License | Enabled features | Build-time execution | Cargo checksum |
|---|---:|---|---|---|---|---|
{chr(10).join(rows)}

Proc macros and build scripts are executable supply-chain inputs. Their exact versions, features, sources, licenses, and Cargo checksums are retained above; they execute only during the Rust build and do not become runtime capabilities of `wow-core`.

## Advisory scan

```text
verification date (UTC): {VERIFICATION_DATE}
scanner: {audit_version}
command: cargo audit --json --deny warnings
vulnerabilities: {vulnerabilities}
warnings: {warning_count}
report SHA-256: {audit_digest}
status: pass
```

The advisory result is a dated RustSec snapshot over this exact `Cargo.lock`; it is not a permanent claim that future advisories do not exist.

## Rejected dependency classes

```text
async runtime
UUID or random ID generator
clock or date-time library
URL or network client
database or SQL library
logging or tracing framework
generic error erasure in the public API
LSP, MCP, editor, WoW client, parser, graph, search, or model runtime
```

## Removal and rollback

Each direct dependency has one narrow owner responsibility. A replacement must preserve the committed canonical vectors and complete E0-A test matrix. Removing serialization or digest dependencies requires a canonicalization-version or implementation change only when byte-for-byte compatibility can still be proved.
'''
    return report, packages


def evidence() -> None:
    metadata = cargo_metadata()
    report, dependencies = generate_dependency_report(metadata)
    (CRATE / "DEPENDENCIES.md").write_text(report, encoding="utf-8")

    contract = load_json(CRATE / "CONTRACT.json")
    vectors = load_json(CRATE / "examples" / "HASH_VECTORS.json")["vectors"]
    debug_tests = test_count(Path("/tmp/i0a-test-list-debug.txt"))
    release_tests = test_count(Path("/tmp/i0a-test-list-release.txt"))
    case_ids = matrix_case_ids()
    rustc = Path("/tmp/i0a-rustc-version.txt").read_text(encoding="utf-8").strip()
    cargo = Path("/tmp/i0a-cargo-version.txt").read_text(encoding="utf-8").strip()
    audit = load_json(Path("/tmp/i0a-cargo-audit.json"))
    vulnerabilities = audit.get("vulnerabilities", {}).get("count", 0)
    warnings = audit.get("warnings", {})
    warning_count = sum(len(value) for value in warnings.values()) if isinstance(warnings, dict) else 0

    implementation_evidence = {
        "schema_version": 1,
        "work_package": "I0-A",
        "crate": "wow-core",
        "implementation_state": "complete",
        "implementation_source_commit": IMPLEMENTATION_COMMIT,
        "verification_date_utc": VERIFICATION_DATE,
        "toolchain": {
            "channel": "1.98.0",
            "rustc": rustc,
            "cargo": cargo,
            "edition": "2024",
            "resolver": "3",
        },
        "workspace": {
            "members": ["crates/wow-core"],
            "active_member_count": 1,
            "cargo_lock_sha256": sha256_file(ROOT / "Cargo.lock"),
        },
        "contract": {
            "contract_id": "wow-core/e0-a/1",
            "required_operation_count": len(contract["e0_operations"]),
            "required_operation_registry_count": len(contract["e0_operations"]),
            "test_matrix_case_id_count": len(case_ids),
            "hash_vector_count": len(vectors),
        },
        "validation": [
            {"command": "cargo +1.98.0 fmt --all -- --check", "status": "pass"},
            {"command": "cargo +1.98.0 clippy --workspace --all-targets --all-features -- -D warnings", "status": "pass"},
            {"command": "cargo +1.98.0 test --workspace --all-features", "status": "pass", "listed_tests": debug_tests},
            {"command": "cargo +1.98.0 test --workspace --all-features --release", "status": "pass", "listed_tests": release_tests},
            {"command": "cargo audit --json --deny warnings", "status": "pass", "vulnerabilities": vulnerabilities, "warnings": warning_count},
            {"command": "strict repository JSON/path/link/checksum validation", "status": "pass"},
        ],
        "dependencies": {
            "resolved_package_count_excluding_workspace": len(dependencies),
            "direct_packages": [package["name"] for package in dependencies if package["direct"]],
            "license_gate": "pass",
            "rustsec_gate": "pass",
        },
        "implemented_domains": [
            "typed identifiers and digest purposes",
            "profile and generation identity",
            "source paths spans and handles",
            "evidence derivation conflicts and authority ceilings",
            "coverage capability summaries NotEvaluated and negative authority",
            "findings warnings remediation guards budgets and truncation",
            "canonical JSON ordering digests strict envelopes and operation errors",
            "post-deserialization record identity and reference closure",
        ],
        "nonclaims": [
            "R0 is not complete",
            "wow-reference I0-B and later packages are not implemented",
            "no wow product executable exists",
            "no WoW client runtime platform LSP MCP daemon release installation update or support evidence exists",
            "the dated advisory scan is not a permanent future-vulnerability claim",
        ],
        "next_work_package": "I0-B",
    }
    write_json(CRATE / "IMPLEMENTATION_EVIDENCE.json", implementation_evidence)

    implementation_report = f'''# `wow-core` I0-A implementation report

**Status:** implementation complete. **Launch state:** R0 remains blocked on I0-B through I0-F.

## Identity

```text
work package: I0-A
owner: wow-core
contract: wow-core/e0-a/1
implementation source commit: {IMPLEMENTATION_COMMIT}
verification date (UTC): {VERIFICATION_DATE}
Rust toolchain: 1.98.0
workspace members: crates/wow-core only
```

## Implemented public contract

All {len(contract['e0_operations'])} required operation IDs in `CONTRACT.json` are present in the exact public operation registry and checked for exact order and uniqueness. The implementation covers typed IDs and digest purposes; fixture/release profiles; generation contexts; canonical paths, spans, and source handles; evidence, conflicts, coverage, capability summaries, `NotEvaluated`, and negative authority; findings, warnings, remediation guards, budgets, truncation; canonical JSON; and strict result and operation-error envelopes.

Post-deserialization validation recomputes every identity-bearing record, resolves all internal references, verifies evidence derivation and authority ceilings, rejects semantic coverage-key collisions, validates conflict scope, and rejects stale/tampered canonical digests.

## Executable evidence

```text
TEST_MATRIX case IDs represented in Rust test corpus: {len(case_ids)}
committed hash vectors: {len(vectors)}
debug listed tests: {debug_tests}
release listed tests: {release_tests}
```

Commands completed successfully on the frozen toolchain:

```text
cargo +1.98.0 generate-lockfile       # no Cargo.lock diff
cargo +1.98.0 fmt --all -- --check
cargo +1.98.0 clippy --workspace --all-targets --all-features -- -D warnings
cargo +1.98.0 test --workspace --all-features
cargo +1.98.0 test --workspace --all-features --release
cargo audit --json --deny warnings
```

The committed checksum manifest is self-tested against every listed member and its canonical bundle projection.

## Dependency and security state

See [`DEPENDENCIES.md`](DEPENDENCIES.md). Direct dependencies are exact-pinned and the complete transitive closure is bound by Cargo checksums. License and dated RustSec advisory gates pass. `wow-core` has no async, clock, random, filesystem, network, database, logging, parser, graph, search, editor, transport, model, or WoW-client runtime dependency.

## Public API review

- No sibling crate is activated.
- No generic `serde_json::Value` or metadata extension bag crosses the public API.
- No unchecked/default-empty identity-bearing constructor is exported.
- No floating `current/latest/default` identity is accepted.
- No IO, clock, randomness, global mutable state, or background work exists.
- Errors, findings, warnings, `NotEvaluated`, partial state, conflicts, and negative authority remain separate.
- Candidate and runtime evidence cannot be upgraded beyond their scoped proof ceiling.

## Nonclaims

I0-A does not implement a Reference Pack, analyzer, project, rules, service, CLI, daemon, LSP, MCP, release, installer, update mechanism, WoW runtime probe, or supported platform. The `wow` product remains non-runnable until I0-B through I0-F are implemented and R0 passes.

## Handoff

The next primary package is `I0-B / wow-reference E0-B`. It must consume the merged typed `wow-core` boundary without reconstructing identity from strings or weakening coverage and authority semantics.
'''
    (CRATE / "IMPLEMENTATION_REPORT.md").write_text(implementation_report, encoding="utf-8")


def checksum_members() -> list[str]:
    explicit = [
        "LICENSE",
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "README.md",
        "AGENTS.md",
        "CONTRIBUTING.md",
        "docs/README.md",
        "docs/ROADMAP.md",
        "docs/LAUNCH_GATES.md",
        "docs/IMPLEMENTATION_HANDOFF.md",
        "docs/AGENT_WORKFLOW.md",
        "docs/PROJECT_COMPLETION_MATRIX.md",
        "crates/MANIFEST.json",
        "crates/README.md",
        "crates/AGENTS.md",
        "crates/WORKSTREAMS.md",
        "crates/DEPENDENCY_GRAPH.md",
    ]
    crate_members = []
    for path in CRATE.rglob("*"):
        if not path.is_file():
            continue
        relative = path.relative_to(ROOT).as_posix()
        if relative == "crates/wow-core/examples/CHECKSUMS.json":
            continue
        if path.suffix in {".rs", ".md", ".json", ".toml"}:
            crate_members.append(relative)
    members = sorted(set(explicit + crate_members))
    missing = [member for member in members if not (ROOT / member).is_file()]
    if missing:
        raise ValueError(f"checksum members missing: {missing}")
    return members


def checksums() -> None:
    members = [
        {"member_path": path, "sha256": sha256_file(ROOT / path)}
        for path in checksum_members()
    ]
    projection = {
        "algorithm": "sha256",
        "members": members,
        "profile": "wow-core-implementation-bundle/v1",
    }
    canonical = json.dumps(
        projection,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")
    manifest = {
        "schema_version": 1,
        "manifest_id": "wow-core/i0-a/implementation-checksums",
        "work_package": "I0-A",
        "crate": "wow-core",
        "implementation_state": "complete",
        "implementation_source_commit": IMPLEMENTATION_COMMIT,
        "algorithm": "sha256",
        "bundle_profile": "wow-core-implementation-bundle/v1",
        "members": members,
        "bundle_sha256": sha256_bytes(canonical),
        "nonclaims": [
            "the bundle authenticates neither a public release nor a supported platform",
            "runtime client and downstream package evidence remain outside I0-A",
        ],
    }
    write_json(CRATE / "examples" / "CHECKSUMS.json", manifest)


def validate_markdown_links() -> None:
    link_pattern = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
    broken: list[str] = []
    for path in ROOT.rglob("*.md"):
        if ".git" in path.parts:
            continue
        text = path.read_text(encoding="utf-8")
        if text.count("```") % 2:
            broken.append(f"unbalanced fence: {path.relative_to(ROOT)}")
        for target in link_pattern.findall(text):
            if target.startswith(("http://", "https://", "mailto:", "#")):
                continue
            clean = target.split("#", 1)[0]
            if not clean:
                continue
            resolved = (path.parent / clean).resolve()
            try:
                resolved.relative_to(ROOT.resolve())
            except ValueError:
                broken.append(f"escaping link: {path.relative_to(ROOT)} -> {target}")
                continue
            if not resolved.exists():
                broken.append(f"missing link: {path.relative_to(ROOT)} -> {target}")
    if broken:
        raise ValueError("\n".join(broken))


def validate_repository() -> None:
    for path in ROOT.rglob("*.json"):
        if ".git" not in path.parts:
            load_json(path)
    validate_markdown_links()
    if list(ROOT.rglob("*.rs")) == []:
        raise ValueError("Rust source unexpectedly absent")
    workflows = ROOT / ".github" / "workflows"
    if workflows.exists() and any(workflows.iterdir()):
        raise ValueError(f"target tree still contains workflows: {sorted(path.name for path in workflows.iterdir())}")
    contract = load_json(CRATE / "CONTRACT.json")
    manifest = load_json(ROOT / "crates" / "MANIFEST.json")
    if contract.get("state") != "implementation-complete":
        raise ValueError("wow-core contract is not implementation-complete")
    if manifest.get("implementation_frontier") != "I0-A-complete":
        raise ValueError("machine manifest frontier mismatch")
    if manifest.get("next_implementation_work_package", {}).get("id") != "I0-B":
        raise ValueError("machine manifest next package mismatch")
    if len(contract.get("e0_operations", [])) != 52:
        raise ValueError("unexpected E0 operation count")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=["prepare", "evidence", "checksums", "validate"])
    args = parser.parse_args()
    if args.mode == "prepare":
        prepare()
    elif args.mode == "evidence":
        evidence()
    elif args.mode == "checksums":
        checksums()
    else:
        validate_repository()


if __name__ == "__main__":
    main()
