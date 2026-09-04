from __future__ import annotations

import importlib.util
import json
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from io import StringIO
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "sync-wow-dev-skill.py"
SPEC = importlib.util.spec_from_file_location("sync_wow_dev_skill", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class SkillSyncTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.canonical = self.root / MODULE.CANONICAL_PATH
        self.canonical.parent.mkdir(parents=True)
        self.content = b"# wow-dev\n\nCanonical World of Warcraft workflow.\n"
        self.canonical.write_bytes(self.content)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def test_missing_targets_report_drift(self) -> None:
        canonical = MODULE.read_canonical(self.root)
        status = MODULE.inspect(self.root, canonical)
        self.assertEqual({entry["status"] for entry in status}, {"missing"})
        self.assertEqual(MODULE.main(["--root", os.fspath(self.root), "--check"]), MODULE.EXIT_DRIFT)

    def test_write_creates_exact_copies(self) -> None:
        return_code = MODULE.main(["--root", os.fspath(self.root), "--write"])
        self.assertEqual(return_code, 0)
        for relative in MODULE.TARGET_PATHS:
            self.assertEqual((self.root / relative).read_bytes(), self.content)

    def test_stale_copy_is_replaced(self) -> None:
        MODULE.synchronize(
            self.root,
            self.content,
            MODULE.inspect(self.root, self.content),
        )
        target = self.root / MODULE.TARGET_PATHS[0]
        target.write_text("stale\n", encoding="utf-8")
        self.assertEqual(MODULE.main(["--root", os.fspath(self.root), "--check"]), MODULE.EXIT_DRIFT)
        self.assertEqual(MODULE.main(["--root", os.fspath(self.root), "--write"]), 0)
        self.assertEqual(target.read_bytes(), self.content)

    def test_check_never_mutates_targets(self) -> None:
        target = self.root / MODULE.TARGET_PATHS[0]
        target.parent.mkdir(parents=True)
        target.write_bytes(b"stale\n")
        before = target.read_bytes()
        MODULE.main(["--root", os.fspath(self.root), "--check"])
        self.assertEqual(target.read_bytes(), before)

    def test_json_output_contains_no_local_root(self) -> None:
        MODULE.main(["--root", os.fspath(self.root), "--write"])
        output = StringIO()
        with redirect_stdout(output):
            return_code = MODULE.main(
                ["--root", os.fspath(self.root), "--check", "--json"]
            )
        self.assertEqual(return_code, 0)
        parsed = json.loads(output.getvalue())
        self.assertTrue(parsed["current"])
        self.assertNotIn(os.fspath(self.root), output.getvalue())

    def test_temporary_files_are_removed_after_write(self) -> None:
        MODULE.main(["--root", os.fspath(self.root), "--write"])
        for relative in MODULE.TARGET_PATHS:
            leftovers = list((self.root / relative).parent.glob(".SKILL.md.*.tmp"))
            self.assertEqual(leftovers, [])

    def test_non_utf8_canonical_skill_is_rejected(self) -> None:
        self.canonical.write_bytes(b"\xff\xfe")
        with self.assertRaises(MODULE.SkillSyncError):
            MODULE.read_canonical(self.root)

    def test_parent_escape_is_rejected(self) -> None:
        with self.assertRaises(MODULE.SkillSyncError):
            MODULE.validate_relative(Path("../outside"))


if __name__ == "__main__":
    unittest.main()
