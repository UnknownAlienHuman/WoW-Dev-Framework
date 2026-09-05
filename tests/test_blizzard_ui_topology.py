from __future__ import annotations

import hashlib
import importlib.util
import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).resolve().parents[1] / "scripts" / "wow_ui_topology.py"
SPEC = importlib.util.spec_from_file_location("wow_ui_topology", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


def run_git(root: Path, *arguments: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(root), *arguments],
        check=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    return result.stdout.strip()


def write_files(root: Path, files: dict[str, str]) -> None:
    for relative, text in files.items():
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")


@unittest.skipUnless(shutil.which("git"), "Git is required")
class BlizzardUiTopologyTests(unittest.TestCase):
    def fixture(
        self,
        root: Path,
        *,
        toc: str | None = None,
        main_xml: str | None = None,
        templates_xml: str | None = None,
        extra: dict[str, str] | None = None,
    ) -> tuple[dict[str, object], bytes, str]:
        files = {
            "Interface/AddOns/Blizzard_Test/Blizzard_Test.toc": toc
            or "## Interface: 99999\n## Title: Test\nMain.xml\nLogic.lua\n",
            "Interface/AddOns/Blizzard_Test/Main.xml": main_xml
            or (
                "<Ui>"
                '<Include file="Templates.xml"/>'
                '<Script file="Logic.lua"/>'
                '<Frame name="Root" inherits="Base, Mix" virtual="true"/>'
                "</Ui>"
            ),
            "Interface/AddOns/Blizzard_Test/Templates.xml": templates_xml
            or '<Ui><Frame name="Base" virtual="true"/></Ui>',
            "Interface/AddOns/Blizzard_Test/Logic.lua": "local loaded = true\n",
            "version.txt": "99.1.2.34567\n",
        }
        files.update(extra or {})
        run_git(root, "init", "--quiet")
        run_git(root, "config", "user.email", "test@example.invalid")
        run_git(root, "config", "user.name", "Test")
        write_files(root, files)
        run_git(root, "add", ".")
        run_git(root, "commit", "--quiet", "-m", "fixture")
        revision = run_git(root, "rev-parse", "HEAD")
        records = []
        for relative, text in sorted(files.items()):
            data = text.encode("utf-8")
            records.append(
                {
                    "path": relative,
                    "size": len(data),
                    "sha256": hashlib.sha256(data).hexdigest(),
                    "git_object": run_git(root, "rev-parse", f"{revision}:{relative}"),
                }
            )
        manifest: dict[str, object] = {
            "schema_version": 1,
            "source": {
                "id": "test-source",
                "selector": "live",
                "revision": revision,
                "version": "99.1.2.34567",
            },
            "files": records,
        }
        raw = json.dumps(manifest, sort_keys=True).encode("utf-8")
        return manifest, raw, revision

    def test_builds_exact_load_and_template_topology(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest, raw, revision = self.fixture(root)
            draft = MODULE.build_topology(
                source=root,
                manifest=manifest,
                manifest_bytes=raw,
            )
            MODULE.verify_topology(draft, require_complete=True)
            self.assertEqual(draft["source"]["revision"], revision)
            self.assertEqual(draft["coverage"]["status"], "complete")
            self.assertTrue(draft["coverage"]["negative_authority"])
            self.assertEqual(len(draft["descriptors"]), 1)
            self.assertEqual(len(draft["xml_documents"]), 2)
            kinds = [edge["kind"] for edge in draft["edges"]]
            self.assertEqual(kinds.count("toc_load"), 2)
            self.assertEqual(kinds.count("xml_include"), 1)
            self.assertEqual(kinds.count("xml_script"), 1)
            main = next(
                document
                for document in draft["xml_documents"]
                if document["path"].endswith("Main.xml")
            )
            template = main["templates"][0]
            self.assertEqual(template["name"], "Root")
            self.assertEqual(template["inherits"], ["Base", "Mix"])
            self.assertTrue(template["virtual"])

    def test_reads_committed_revision_not_dirty_worktree(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest, raw, _revision = self.fixture(root)
            first = MODULE.build_topology(
                source=root,
                manifest=manifest,
                manifest_bytes=raw,
            )
            (root / "Interface/AddOns/Blizzard_Test/Main.xml").write_text(
                "<Ui><Script>raise('must not execute')</Script></Ui>",
                encoding="utf-8",
            )
            second = MODULE.build_topology(
                source=root,
                manifest=manifest,
                manifest_bytes=raw,
            )
            self.assertEqual(
                MODULE.canonical_json_bytes(first),
                MODULE.canonical_json_bytes(second),
            )

    def test_case_mismatch_blocks_negative_authority(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest, raw, _revision = self.fixture(
                root,
                toc="## Interface: 99999\nMain.xml\nlogic.lua\n",
            )
            draft = MODULE.build_topology(
                source=root,
                manifest=manifest,
                manifest_bytes=raw,
            )
            MODULE.verify_topology(draft)
            self.assertEqual(draft["coverage"]["status"], "partial")
            self.assertFalse(draft["coverage"]["negative_authority"])
            self.assertEqual(draft["coverage"]["unresolved_references"], 1)
            self.assertEqual(draft["issues"][0]["code"], "case_mismatch")
            with self.assertRaisesRegex(MODULE.TopologyError, "complete"):
                MODULE.verify_topology(draft, require_complete=True)

    def test_parent_reference_may_resolve_inside_interface(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest, raw, _revision = self.fixture(
                root,
                toc=(
                    "## Interface: 99999\n"
                    "Main.xml\n"
                    "Logic.lua\n"
                    "../Blizzard_Shared/Shared.xml\n"
                ),
                extra={
                    "Interface/AddOns/Blizzard_Shared/Shared.xml": "<Ui/>",
                },
            )
            draft = MODULE.build_topology(
                source=root,
                manifest=manifest,
                manifest_bytes=raw,
            )
            MODULE.verify_topology(draft, require_complete=True)
            edge = next(
                item
                for item in draft["edges"]
                if item["declared"] == "../Blizzard_Shared/Shared.xml"
            )
            self.assertEqual(
                edge["target"],
                "Interface/AddOns/Blizzard_Shared/Shared.xml",
            )
            self.assertEqual(edge["resolution"], "exact")

    def test_entity_declaration_is_rejected_without_execution(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest, raw, _revision = self.fixture(
                root,
                main_xml=(
                    '<!DOCTYPE Ui [<!ENTITY unsafe "expanded">]>'
                    "<Ui><Frame name=\"&unsafe;\"/></Ui>"
                ),
            )
            with self.assertRaisesRegex(MODULE.TopologyError, "entity"):
                MODULE.build_topology(
                    source=root,
                    manifest=manifest,
                    manifest_bytes=raw,
                )
            partial = MODULE.build_topology(
                source=root,
                manifest=manifest,
                manifest_bytes=raw,
                allow_partial=True,
            )
            self.assertEqual(partial["coverage"]["status"], "partial")
            self.assertFalse(partial["coverage"]["negative_authority"])
            self.assertEqual(partial["coverage"]["failed_files"], 1)

    def test_include_cycle_is_preserved(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest, raw, _revision = self.fixture(
                root,
                main_xml='<Ui><Include file="Templates.xml"/></Ui>',
                templates_xml='<Ui><Include file="Main.xml"/></Ui>',
            )
            draft = MODULE.build_topology(
                source=root,
                manifest=manifest,
                manifest_bytes=raw,
            )
            MODULE.verify_topology(draft, require_complete=True)
            self.assertEqual(len(draft["include_cycles"]), 1)
            self.assertEqual(len(draft["include_cycles"][0]), 2)

    def test_duplicate_metadata_is_reported_without_hiding_load_graph(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest, raw, _revision = self.fixture(
                root,
                toc=(
                    "## Interface: 99999\n"
                    "## Interface: 100000\n"
                    "Main.xml\n"
                    "Logic.lua\n"
                ),
            )
            draft = MODULE.build_topology(
                source=root,
                manifest=manifest,
                manifest_bytes=raw,
            )
            MODULE.verify_topology(draft, require_complete=True)
            duplicate = [
                issue for issue in draft["issues"] if issue["code"] == "duplicate_metadata"
            ]
            self.assertEqual(len(duplicate), 1)
            self.assertEqual(duplicate[0]["lines"], [1, 2])

    def test_tampered_draft_fails_digest_validation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest, raw, _revision = self.fixture(root)
            draft = MODULE.build_topology(
                source=root,
                manifest=manifest,
                manifest_bytes=raw,
            )
            draft["descriptors"][0]["descriptor_name"] = "Tampered"
            with self.assertRaisesRegex(MODULE.TopologyError, "digest"):
                MODULE.verify_topology(draft)


if __name__ == "__main__":
    unittest.main()
