from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "build-blizzard-source-manifest.py"
SPEC = importlib.util.spec_from_file_location("build_blizzard_source_manifest", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


class SnapshotManifestTests(unittest.TestCase):
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
        self.write("Interface/AddOns/Test/Test.xml", b"<Ui/>\n")
        self.write(
            "Interface/AddOns/Blizzard_APIDocumentationGenerated/TestDocumentation.lua",
            b"local api = {}\n",
        )
        self.write("Interface/Icons/ignored.blp", b"asset")
        self.git("add", ".")
        self.git("commit", "-m", "initial")
        self.initial_commit = self.git("rev-parse", "HEAD").strip()

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

    def build(self, revision: str = "HEAD") -> dict[str, object]:
        return MODULE.build_manifest(
            self.repository,
            revision,
            source_id="test-source",
            selector="live",
            extensions=frozenset(MODULE.DEFAULT_EXTENSIONS),
            version_path="version.txt",
            limits=MODULE.Limits(
                max_files=100,
                max_file_bytes=1024 * 1024,
                max_total_bytes=16 * 1024 * 1024,
            ),
        )

    def test_manifest_is_deterministic_and_sorted(self) -> None:
        first = self.build()
        second = self.build()
        self.assertEqual(first, second)
        files = first["files"]
        assert isinstance(files, list)
        paths = [record["path"] for record in files]
        self.assertEqual(paths, sorted(paths, key=lambda value: value.encode("utf-8")))
        self.assertNotIn("Interface/Icons/ignored.blp", paths)
        self.assertEqual(first["source"]["revision"], self.initial_commit)
        self.assertEqual(first["source"]["version"], "12.1.0.70000")

    def test_manifest_digest_covers_every_other_field(self) -> None:
        manifest = self.build()
        supplied = manifest.pop("manifest_sha256")
        expected = hashlib.sha256(MODULE.canonical_bytes(manifest)).hexdigest()
        self.assertEqual(supplied, expected)

    def test_dirty_working_tree_does_not_change_exact_commit(self) -> None:
        before = self.build(self.initial_commit)
        self.write("Interface/AddOns/Test/Test.lua", b"uncommitted change\n")
        after = self.build(self.initial_commit)
        self.assertEqual(before, after)

    def test_new_commit_creates_new_manifest_identity(self) -> None:
        before = self.build(self.initial_commit)
        self.write("Interface/AddOns/Test/Test.lua", b"local value = 2\n")
        self.git("add", ".")
        self.git("commit", "-m", "update")
        after = self.build("HEAD")
        self.assertNotEqual(before["manifest_sha256"], after["manifest_sha256"])
        self.assertNotEqual(before["source"]["revision"], after["source"]["revision"])

    def test_generated_api_is_classified_separately(self) -> None:
        manifest = self.build()
        by_path = {record["path"]: record for record in manifest["files"]}
        path = "Interface/AddOns/Blizzard_APIDocumentationGenerated/TestDocumentation.lua"
        self.assertEqual(by_path[path]["kind"], "generated_api")

    def test_blob_identity_matches_git(self) -> None:
        manifest = self.build()
        by_path = {record["path"]: record for record in manifest["files"]}
        path = "Interface/AddOns/Test/Test.lua"
        expected = self.git("rev-parse", f"{self.initial_commit}:{path}").strip()
        self.assertEqual(by_path[path]["git_blob_sha1"], expected)

    def test_missing_version_file_is_rejected(self) -> None:
        self.git("rm", "version.txt")
        self.git("commit", "-m", "remove version")
        with self.assertRaisesRegex(MODULE.ManifestError, "version file is absent"):
            self.build("HEAD")

    def test_archive_path_validation_rejects_escape_and_backslash(self) -> None:
        for candidate in ("../escape", "/absolute", "one\\two", "one/./two"):
            with self.subTest(candidate=candidate):
                with self.assertRaises(MODULE.ManifestError):
                    MODULE.validate_archive_path(candidate)

    def test_cli_writes_canonical_json_atomically(self) -> None:
        destination = self.root / "out" / "manifest.json"
        return_code = MODULE.main(
            [
                "--source",
                os.fspath(self.repository),
                "--revision",
                self.initial_commit,
                "--selector",
                "live",
                "--source-id",
                "test-source",
                "--output",
                os.fspath(destination),
            ]
        )
        self.assertEqual(return_code, 0)
        parsed = json.loads(destination.read_text(encoding="utf-8"))
        self.assertEqual(parsed["source"]["revision"], self.initial_commit)
        self.assertFalse(any(destination.parent.glob(".manifest.json.*.tmp")))


if __name__ == "__main__":
    unittest.main()
