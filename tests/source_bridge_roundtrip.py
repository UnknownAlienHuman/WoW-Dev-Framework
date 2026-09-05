"""Cross-language contract test; invoked by Cargo with its actual built binaries."""
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DOCUMENT = '''local Fixture = {
    Name = "Fixture", Type = "System", Namespace = "C_Fixture",
    Functions = {{Name = "Lookup", Type = "Function", HasRestrictions = true,
                  Arguments = {{Name = "id", Type = "number", Nilable = false}},
                  Returns = {{Name = "value", Type = "number", Nilable = true}},
                  FutureMetadata = {Ratio = 1e-7, Minimum = -1}}},
    Events = {}, Tables = {}, Enumerations = {}, Constants = {}, Predicates = {}
}
APIDocumentation:AddDocumentationTable(Fixture)
'''


def command(*args: str, expected: int = 0) -> str:
    result = subprocess.run(list(args), capture_output=True, text=True, timeout=120)
    if result.returncode != expected:
        raise AssertionError(f"{args[0]} returned {result.returncode}, expected {expected}:\n{result.stdout}\n{result.stderr}")
    return result.stdout


def script(name: str, *args: str, expected: int = 0) -> str:
    return command(sys.executable, str(ROOT / "scripts" / name), *args, expected=expected)


def run(algorithm: str) -> None:
    with tempfile.TemporaryDirectory(prefix="wdf-roundtrip-") as directory:
        root = Path(directory)
        source = root / "source"
        source.mkdir()
        def git(*args: str) -> str:
            return command("git", "-C", str(source), *args).strip()
        git("init", "--initial-branch=live", f"--object-format={algorithm}")
        git("config", "user.name", "Fixture")
        git("config", "user.email", "fixture@example.invalid")
        git("config", "core.autocrlf", "false")
        files = {
            "version.txt": "99.2.0.12345\n",
            "Interface/AddOns/Blizzard_APIDocumentationGenerated/FixtureDocumentation.lua": DOCUMENT,
            "Interface/AddOns/Fixture/Fixture.toc": "## Interface: 990200\nFixture.xml\n",
            "Interface/AddOns/Fixture/Fixture.xml": '<Ui><Script file="Fixture.lua"/></Ui>\n',
            "Interface/AddOns/Fixture/Fixture.lua": "-- $Format:%H$\nlocal value = 1\n",
            ".gitattributes": "*.lua export-subst\nversion.txt export-ignore\n",
        }
        for name, text in files.items():
            path = source / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding="utf-8", newline="\n")
        git("add", ".")
        git("commit", "-qm", "synthetic input")
        revision = git("rev-parse", "HEAD")
        manifest, api, topology, bundle = (str(root / name) for name in ("manifest.json", "api.json", "topology.json", "bundle.json"))
        script("build-blizzard-source-manifest.py", "--source", str(source), "--revision", revision, "--selector", "live", "--output", manifest)
        script("verify-blizzard-source-manifest.py", manifest, "--source", str(source), "--current-ref", "live", "--json")
        script("build-blizzard-api-reference.py", "--source", str(source), "--manifest", manifest, "--output", api, "--json")
        script("verify-blizzard-api-reference.py", api, "--source", str(source), "--manifest", manifest, "--require-complete", "--json")
        script("build-blizzard-ui-topology.py", "--source", str(source), "--manifest", manifest, "--output", topology, "--json")
        script("verify-blizzard-ui-topology.py", topology, "--source", str(source), "--manifest", manifest, "--require-complete", "--json")
        api_bin, topology_bin, source_bin = (os.environ[key] for key in ("WDF_API_BIN", "WDF_TOPOLOGY_BIN", "WDF_SOURCE_BIN"))
        api_summary = json.loads(command(api_bin, "verify", api))
        assert api_summary["facts"] == 1 and api_summary["coverage"] == "complete"
        assert json.loads(command(api_bin, "lookup", api, "function", "C_Fixture.Lookup"))["status"] == "found"
        assert json.loads(command(api_bin, "lookup", api, "function", "C_Fixture.Missing"))["status"] == "absent_authoritative"
        command(topology_bin, "verify", topology)
        command(source_bin, "materialize", api, topology, bundle)
        first = Path(bundle).read_bytes()
        command(source_bin, "materialize", api, topology, bundle)
        assert Path(bundle).read_bytes() == first, "identical input changed the bundle"
        Path(bundle).write_bytes(b"existing unrelated file")
        command(source_bin, "materialize", api, topology, bundle, expected=2)
        assert Path(bundle).read_bytes() == b"existing unrelated file", "publication clobbered an existing file"
        # Dirty worktree content must not affect a committed source revision.
        (source / "version.txt").write_text("dirty worktree\n", encoding="utf-8")
        script("verify-blizzard-source-manifest.py", manifest, "--source", str(source))
        git("add", "version.txt")
        git("commit", "-qm", "advance source")
        script("verify-blizzard-source-manifest.py", manifest, "--source", str(source), "--current-ref", "live", "--json", expected=3)
        # Byte tampering must fail instead of producing a clean reference result.
        draft = json.loads(Path(api).read_text(encoding="utf-8"))
        draft["source"]["version"] = "tampered"
        Path(api).write_text(json.dumps(draft), encoding="utf-8")
        command(api_bin, "verify", api, expected=2)
        print(f"{algorithm}: manifest -> Python API/topology -> Rust lookup/bundle; tamper and stale checks passed")


if __name__ == "__main__":
    for object_format in ("sha1", "sha256"):
        run(object_format)
