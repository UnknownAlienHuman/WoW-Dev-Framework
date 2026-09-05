from __future__ import annotations

import hashlib
import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BUILD_PATH = ROOT / "scripts" / "build-blizzard-api-reference.py"
VERIFY_PATH = ROOT / "scripts" / "verify-blizzard-api-reference.py"


def load(name: str, path: Path):
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


BUILD = load("wow_api_reference", ROOT / "scripts" / "wow_api_reference.py")
VERIFY = BUILD


def build_draft(manifest, source, allow_partial=False):
    value, raw = BUILD.load_json(manifest)
    return BUILD.build_reference_draft(source=source, manifest=value, manifest_bytes=raw,
                                       allow_partial=allow_partial)


def verify(path):
    value, _ = BUILD.load_json(path)
    BUILD.verify_reference_draft(value, require_complete=True)
    return {"coverage": value["coverage"]["status"]}
DOC_PATH = "Interface/AddOns/Blizzard_APIDocumentationGenerated/ExampleDocumentation.lua"
DOCUMENT = r'''
local Example = {
    Name = "Example",
    Type = "System",
    Namespace = "C_Example",
    Environment = "All",
    Functions = {
        {
            Name = "Lookup",
            Type = "Function",
            HasRestrictions = true,
            SecretArguments = "AllowedWhenUntainted",
            Arguments = {
                { Name = "id", Type = "number", Nilable = false, NeverSecret = true },
            },
            Returns = {
                { Name = "result", Type = "ExampleInfo", Nilable = true },
            },
        },
    },
    Events = {
        {
            Name = "ExampleChanged",
            Type = "Event",
            Event = "EXAMPLE_CHANGED",
            Payload = {
                { Name = "id", Type = "number", Nilable = false },
            },
        },
    },
    Tables = {
        {
            Name = "ExampleInfo",
            Type = "Structure",
            Fields = {
                { Name = "name", Type = "cstring", Nilable = false },
                { Name = "kind", Type = "ExampleKind", Nilable = false, Default = Enum.ExampleKind.One },
            },
        },
    },
};
APIDocumentation:AddDocumentationTable(Example);
'''


class Fixture:
    def __init__(self, root: Path, document: str = DOCUMENT) -> None:
        self.root = root
        subprocess.run(["git", "init", "-q", str(root)], check=True)
        subprocess.run(["git", "-C", str(root), "config", "user.name", "Test"], check=True)
        subprocess.run(["git", "-C", str(root), "config", "user.email", "test@example.invalid"], check=True)
        target = root / DOC_PATH
        target.parent.mkdir(parents=True)
        target.write_text(document, encoding="utf-8", newline="\n")
        (root / "version.txt").write_text("99.0.0.12345\n", encoding="utf-8")
        subprocess.run(["git", "-C", str(root), "add", "."], check=True)
        subprocess.run(["git", "-C", str(root), "commit", "-qm", "fixture"], check=True)
        self.revision = subprocess.check_output(["git", "-C", str(root), "rev-parse", "HEAD"], text=True).strip()
        payload = subprocess.check_output(["git", "-C", str(root), "show", f"{self.revision}:{DOC_PATH}"])
        self.sha256 = hashlib.sha256(payload).hexdigest()

    def manifest(self, path: Path) -> Path:
        value = {
            "schema_version": 1,
            "source": {
                "source_id": "fixture",
                "selector": "test",
                "resolved_commit": self.revision,
                "version": "99.0.0.12345",
            },
            "files": [{
                "path": DOC_PATH,
                "semantic_class": "generated-api-documentation",
                "sha256": f"sha256:{self.sha256}",
            }],
        }
        value["manifest_digest"] = BUILD.sha256_id(BUILD.canonical_json_bytes(value))
        path.write_text(json.dumps(value), encoding="utf-8")
        return path


class BlizzardApiReferenceTests(unittest.TestCase):
    def test_complete_deterministic_normalization(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = Fixture(root / "source")
            manifest = fixture.manifest(root / "manifest.json")
            first = build_draft(manifest, fixture.root)
            second = build_draft(manifest, fixture.root)
            self.assertEqual(first, second)
            self.assertEqual(first["coverage"]["status"], "complete")
            self.assertTrue(first["coverage"]["negative_authority"])
            function = first["systems"][0]["functions"][0]
            self.assertEqual(function["qualified_name"], "C_Example.Lookup")
            self.assertEqual(function["restrictions"]["has_restrictions"], True)
            default = first["systems"][0]["tables"][0]["fields"][1]["default"]
            self.assertEqual(default, {"$lua_symbol": "Enum.ExampleKind.One"})

    def test_unnamed_table_group_retains_binding_not_namespace(self) -> None:
        document = '''local FixtureConstants = {Tables = {{Name = "FixtureKind", Type = "Enumeration",
            Fields = {{Name = "First", Type = "FixtureKind", EnumValue = -1}}}}}
            APIDocumentation:AddDocumentationTable(FixtureConstants)'''
        parsed = BUILD.parse_generated_document(document)
        group = BUILD.normalize_document(parsed, path=DOC_PATH, sha256="sha256:" + "0" * 64, git_object="a" * 40)
        self.assertEqual(group["name"], "FixtureConstants")
        self.assertIsNone(group["namespace"])
        self.assertIsNone(group["type"])
        self.assertEqual(group["attributes"]["name_origin"], "declaration_binding")
        self.assertEqual(group["tables"][0]["qualified_name"], "FixtureKind")
        self.assertEqual(group["tables"][0]["fields"][0]["enum_value"], -1)
        with self.assertRaises(BUILD.ReferenceBuildError):
            BUILD.normalize_document(BUILD.parse_generated_document(
                'local Invalid = {Functions={}} APIDocumentation:AddDocumentationTable(Invalid)'),
                path=DOC_PATH, sha256="sha256:" + "0" * 64, git_object="a" * 40)

    def test_exact_commit_ignores_dirty_worktree(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = Fixture(root / "source")
            manifest = fixture.manifest(root / "manifest.json")
            (fixture.root / DOC_PATH).write_text("not valid Lua", encoding="utf-8")
            draft = build_draft(manifest, fixture.root)
            self.assertEqual(draft["coverage"]["parsed_files"], 1)

    def test_manifest_byte_mismatch_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = Fixture(root / "source")
            manifest = fixture.manifest(root / "manifest.json")
            value = json.loads(manifest.read_text(encoding="utf-8"))
            value["files"][0]["sha256"] = "sha256:" + "0" * 64
            manifest.write_text(json.dumps(value), encoding="utf-8")
            with self.assertRaises(BUILD.ReferenceBuildError):
                build_draft(manifest, fixture.root)

    def test_partial_coverage_disables_negative_authority(self) -> None:
        broken = DOCUMENT.replace('Name = "Lookup"', "Name = function() end", 1)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = Fixture(root / "source", broken)
            manifest = fixture.manifest(root / "manifest.json")
            with self.assertRaises(BUILD.ReferenceBuildError):
                build_draft(manifest, fixture.root)
            draft = build_draft(manifest, fixture.root, True)
            self.assertEqual(draft["coverage"]["status"], "partial")
            self.assertFalse(draft["coverage"]["negative_authority"])
            self.assertEqual(draft["coverage"]["failed_files"], 1)

    def test_cli_output_verifies_and_tampering_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = Fixture(root / "source")
            manifest = fixture.manifest(root / "manifest.json")
            output = root / "draft.json"
            built = subprocess.run([
                sys.executable, str(BUILD_PATH), "--manifest", str(manifest),
                "--source", str(fixture.root), "--output", str(output), "--json",
            ], capture_output=True, text=True, check=False)
            self.assertEqual(built.returncode, 0, built.stderr)
            summary = verify(output)
            self.assertEqual(summary["coverage"], "complete")
            draft = json.loads(output.read_text(encoding="utf-8"))
            draft["source"]["version"] = "tampered"
            output.write_text(json.dumps(draft), encoding="utf-8")
            with self.assertRaises(BUILD.ReferenceBuildError):
                verify(output)


if __name__ == "__main__":
    unittest.main()
