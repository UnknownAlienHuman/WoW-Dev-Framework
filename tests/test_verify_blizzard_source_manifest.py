from __future__ import annotations

import copy
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load_script(name: str, filename: str):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / filename)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


BUILDER = load_script("build_blizzard_source_manifest", "build-blizzard-source-manifest.py")
VERIFIER = load_script("verify_blizzard_source_manifest", "verify-blizzard-source-manifest.py")


class ManifestVerificationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.repository = self.root / "source"
        self.repository.mkdir()
        self.git("init", "--initial-branch=live")
        self.git("config", "user.name", "Test Author")
        self.git("config", "user.email", "test@example.invalid")
        self.write("version.txt", b"12.1.0.70000\n")
        self.write("Interface/AddOns/Test/Test.toc", b"## Interface: 120100\nTest.lua\n")
        self.write("Interface/AddOns/Test/Test.lua", b"local value = 1\n")
        self.write(
            "Interface/AddOns/Blizzard_APIDocumentationGenerated/TestDocumentation.lua",
            b"local api = {}\n",
        )
        self.git("add", ".")
        self.git("commit", "-m", "initial")
        self.first_commit = self.git("rev-parse", "HEAD").strip()
        self.manifest = self.build(self.first_commit)
        self.manifest_path = self.root / "manifest.json"
        self.write_manifest(self.manifest)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def git(self, *arguments: str) -> str:
        completed = subprocess.run(
            ["git", "-C", os.fspath(self.repository), *arguments],
            check=True,
            capture_output=True,
            text=True,
        )
        return completed.stdout

    def write(self, relative: str, content: bytes) -> None:
        path = self.repository / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)

    def build(self, revision: str) -> dict[str, object]:
        return BUILDER.build_manifest(
            self.repository,
            revision,
            source_id="test-source",
            selector="live",
            extensions=frozenset(BUILDER.DEFAULT_EXTENSIONS),
            version_path="version.txt",
            limits=BUILDER.Limits(
                max_files=100,
                max_file_bytes=1024 * 1024,
                max_total_bytes=16 * 1024 * 1024,
            ),
        )

    def write_manifest(self, value: dict[str, object]) -> None:
        self.manifest_path.write_text(
            json.dumps(value, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

    def test_valid_manifest_passes_shape_and_byte_verification(self) -> None:
        VERIFIER.validate_manifest(self.manifest)
        VERIFIER.rebuild_and_compare(self.manifest, self.repository)
        return_code = VERIFIER.main(
            [os.fspath(self.manifest_path), "--source", os.fspath(self.repository)]
        )
        self.assertEqual(return_code, 0)

    def test_content_tampering_is_rejected(self) -> None:
        tampered = copy.deepcopy(self.manifest)
        tampered["files"][0]["bytes"] += 1
        with self.assertRaisesRegex(VERIFIER.VerificationError, "included_bytes"):
            VERIFIER.validate_manifest(tampered)

    def test_digest_tampering_is_rejected(self) -> None:
        tampered = copy.deepcopy(self.manifest)
        tampered["manifest_sha256"] = "0" * 64
        with self.assertRaisesRegex(VERIFIER.VerificationError, "manifest_sha256"):
            VERIFIER.validate_manifest(tampered)

    def test_unknown_fields_are_rejected(self) -> None:
        tampered = copy.deepcopy(self.manifest)
        tampered["source"]["remote_url"] = "not-allowed"
        with self.assertRaisesRegex(VERIFIER.VerificationError, "source keys differ"):
            VERIFIER.validate_manifest(tampered)

    def test_duplicate_json_keys_are_rejected(self) -> None:
        self.manifest_path.write_text('{"schema_version":1,"schema_version":1}\n')
        with self.assertRaisesRegex(VERIFIER.VerificationError, "duplicate JSON key"):
            VERIFIER.read_json(self.manifest_path)

    def test_stale_current_ref_returns_distinct_status(self) -> None:
        self.write("Interface/AddOns/Test/Test.lua", b"local value = 2\n")
        self.git("add", ".")
        self.git("commit", "-m", "second")
        second_commit = self.git("rev-parse", "HEAD").strip()
        is_current, current_revision = VERIFIER.current_status(
            self.manifest, self.repository, "HEAD"
        )
        self.assertFalse(is_current)
        self.assertEqual(current_revision, second_commit)
        return_code = VERIFIER.main(
            [
                os.fspath(self.manifest_path),
                "--source",
                os.fspath(self.repository),
                "--current-ref",
                "HEAD",
                "--json",
            ]
        )
        self.assertEqual(return_code, VERIFIER.EXIT_STALE)

    def test_current_ref_returns_success(self) -> None:
        return_code = VERIFIER.main(
            [
                os.fspath(self.manifest_path),
                "--source",
                os.fspath(self.repository),
                "--current-ref",
                self.first_commit,
            ]
        )
        self.assertEqual(return_code, 0)

    def test_working_tree_changes_do_not_break_exact_verification(self) -> None:
        self.write("Interface/AddOns/Test/Test.lua", b"dirty but uncommitted\n")
        VERIFIER.rebuild_and_compare(self.manifest, self.repository)


if __name__ == "__main__":
    unittest.main()
