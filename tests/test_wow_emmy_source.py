from __future__ import annotations

import importlib.util
import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parents[1] / "scripts" / "wow_emmy_source.py"
SPEC = importlib.util.spec_from_file_location("wow_emmy_source", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def git(root: Path, *arguments: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(root), *arguments],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return result.stdout.strip()


def write_tree(root: Path, *, marker: str = "v1") -> None:
    files = {
        "Cargo.toml": """
[workspace]
resolver = "2"
members = ["crates/emmylua_code_analysis"]

[workspace.package]
version = "0.21.0"
edition = "2024"
rust-version = "1.85"
license = "MIT"
""".lstrip(),
        "crates/emmylua_code_analysis/Cargo.toml": """
[package]
name = "emmylua_code_analysis"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true

[features]
default = []
full = []
""".lstrip(),
        "crates/emmylua_code_analysis/src/lib.rs": f"""
//! fixture {marker}
pub struct EmmyLuaAnalysis;
pub enum DiagnosticCode {{ Example }}
pub fn analyze() {{}}
""".lstrip(),
    }
    for relative, text in files.items():
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")


@unittest.skipUnless(shutil.which("git"), "Git is required")
class WowEmmySourceTests(unittest.TestCase):
    def make_remote(self, directory: Path) -> tuple[Path, Path, str]:
        seed = directory / "seed"
        remote = directory / "remote.git"
        seed.mkdir()
        git(seed, "init", "--quiet", "--initial-branch=main")
        git(seed, "config", "user.name", "Test")
        git(seed, "config", "user.email", "test@example.invalid")
        write_tree(seed)
        git(seed, "add", ".")
        git(seed, "commit", "--quiet", "-m", "initial")
        subprocess.run(
            ["git", "init", "--quiet", "--bare", str(remote)],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        git(seed, "remote", "add", "origin", remote.as_uri())
        git(seed, "push", "--quiet", "-u", "origin", "main")
        return seed, remote, git(seed, "rev-parse", "HEAD")

    def test_auto_clone_and_probe_current_revision(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _seed, remote, revision = self.make_remote(root)
            checkout = root / "checkout"
            ensured = MODULE.ensure(
                checkout,
                remote.as_uri(),
                "main",
                update_policy="auto",
            )
            self.assertEqual(ensured["relation"], "current")
            self.assertEqual(ensured["local"]["head"], revision)
            self.assertTrue(ensured["local"]["managed"])

            report = MODULE.build_report(
                checkout,
                remote.as_uri(),
                "main",
                required_symbols=["EmmyLuaAnalysis", "analyze"],
            )
            MODULE.verify_report(
                report,
                required_symbols=["EmmyLuaAnalysis", "analyze"],
            )
            self.assertEqual(report["source"]["revision"], revision)
            self.assertEqual(report["compatibility"]["status"], "compatible")
            self.assertEqual(
                report["analysis_crate"]["manifest_path"],
                "crates/emmylua_code_analysis/Cargo.toml",
            )
            self.assertTrue(report["report_sha256"].startswith("sha256:"))

    def test_auto_update_is_fast_forward_only(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            seed, remote, first_revision = self.make_remote(root)
            checkout = root / "checkout"
            MODULE.ensure(
                checkout,
                remote.as_uri(),
                "main",
                update_policy="auto",
            )
            write_tree(seed, marker="v2")
            git(seed, "add", ".")
            git(seed, "commit", "--quiet", "-m", "update")
            git(seed, "push", "--quiet", "origin", "main")
            second_revision = git(seed, "rev-parse", "HEAD")
            self.assertNotEqual(first_revision, second_revision)

            before = MODULE.status(checkout, remote.as_uri(), "main")
            self.assertEqual(before["relation"], "behind")
            self.assertTrue(before["safe_to_fast_forward"])
            after = MODULE.ensure(
                checkout,
                remote.as_uri(),
                "main",
                update_policy="auto",
            )
            self.assertEqual(after["relation"], "current")
            self.assertEqual(after["local"]["head"], second_revision)
            self.assertTrue(after["updated"])

    def test_dirty_checkout_is_never_modified(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            seed, remote, revision = self.make_remote(root)
            checkout = root / "checkout"
            MODULE.ensure(
                checkout,
                remote.as_uri(),
                "main",
                update_policy="auto",
            )
            (checkout / "local.txt").write_text("operator work\n", encoding="utf-8")
            write_tree(seed, marker="v2")
            git(seed, "add", ".")
            git(seed, "commit", "--quiet", "-m", "update")
            git(seed, "push", "--quiet", "origin", "main")

            report = MODULE.ensure(
                checkout,
                remote.as_uri(),
                "main",
                update_policy="auto",
            )
            self.assertEqual(report["relation"], "dirty")
            self.assertEqual(report["blocked_reason"], "dirty")
            self.assertEqual(git(checkout, "rev-parse", "HEAD"), revision)
            self.assertEqual(
                (checkout / "local.txt").read_text(encoding="utf-8"),
                "operator work\n",
            )

    def test_probe_reads_exact_commit_not_dirty_worktree(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _seed, remote, _revision = self.make_remote(root)
            checkout = root / "checkout"
            MODULE.ensure(
                checkout,
                remote.as_uri(),
                "main",
                update_policy="auto",
            )
            first = MODULE.build_report(
                checkout,
                remote.as_uri(),
                "main",
                network=False,
            )
            (checkout / "crates/emmylua_code_analysis/src/lib.rs").write_text(
                "pub struct DirtyOnly;\n",
                encoding="utf-8",
            )
            second = MODULE.build_report(
                checkout,
                remote.as_uri(),
                "main",
                network=False,
            )
            self.assertEqual(
                first["surface"]["surface_sha256"],
                second["surface"]["surface_sha256"],
            )
            names = {item["name"] for item in second["surface"]["symbols"]}
            self.assertIn("EmmyLuaAnalysis", names)
            self.assertNotIn("DirtyOnly", names)

    def test_missing_required_symbol_is_incompatible(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _seed, remote, _revision = self.make_remote(root)
            checkout = root / "checkout"
            MODULE.ensure(
                checkout,
                remote.as_uri(),
                "main",
                update_policy="auto",
            )
            report = MODULE.build_report(
                checkout,
                remote.as_uri(),
                "main",
                required_symbols=["MissingAdapterSeam"],
            )
            self.assertEqual(report["compatibility"]["status"], "incompatible")
            self.assertEqual(
                report["compatibility"]["missing_symbols"],
                ["MissingAdapterSeam"],
            )

    def test_tampered_report_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _seed, remote, _revision = self.make_remote(root)
            checkout = root / "checkout"
            MODULE.ensure(
                checkout,
                remote.as_uri(),
                "main",
                update_policy="auto",
            )
            report = MODULE.build_report(checkout, remote.as_uri(), "main")
            report["source"]["relation"] = "tampered"
            with self.assertRaisesRegex(MODULE.EmmySourceError, "digest"):
                MODULE.verify_report(report)

    def test_credentials_in_remote_are_rejected(self) -> None:
        with self.assertRaisesRegex(MODULE.EmmySourceError, "credentials"):
            MODULE.status(
                Path("unused"),
                "https://user:secret@example.invalid/repo.git",
                "main",
            )


if __name__ == "__main__":
    unittest.main()
