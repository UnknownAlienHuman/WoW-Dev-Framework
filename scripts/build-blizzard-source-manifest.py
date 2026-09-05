#!/usr/bin/env python3
"""Inventory one explicit Git revision without executing or checking out source.

Resolve a moving selector once per operation. Read raw Git objects, not an
archive: export-ignore/export-subst attributes must not change source evidence.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Iterable, NoReturn, Sequence

SCHEMA_VERSION = 1
DEFAULT_EXTENSIONS = (".lua", ".toc", ".xml", ".xsd")
DEFAULT_VERSION_PATH = "version.txt"
DEFAULT_MAX_FILES = 200_000
DEFAULT_MAX_FILE_BYTES = 32 * 1024 * 1024
DEFAULT_MAX_TOTAL_BYTES = 2 * 1024 * 1024 * 1024


class ManifestError(RuntimeError):
    """A bounded source, Git, or output validation failure."""


@dataclass(frozen=True)
class Limits:
    max_files: int
    max_file_bytes: int
    max_total_bytes: int


def fail(message: str) -> NoReturn:
    raise ManifestError(message)


def git_environment() -> dict[str, str]:
    # A caller's Git worktree/config overrides must not redirect the explicit root.
    environment = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
    environment.update(GIT_OPTIONAL_LOCKS="0", GIT_NO_REPLACE_OBJECTS="1", GIT_NO_LAZY_FETCH="1")
    return environment


def git_command(source: Path, arguments: Sequence[str]) -> list[str]:
    return ["git", "-c", f"core.hooksPath={os.devnull}", "-c", "core.fsmonitor=false",
            "-C", os.fspath(source), *arguments]


def run_git(source: Path, arguments: Sequence[str], *, text: bool = True) -> str | bytes:
    try:
        with tempfile.TemporaryFile() as output, tempfile.TemporaryFile() as errors:
            result = subprocess.run(git_command(source, arguments), stdout=output, stderr=errors,
                                    env=git_environment(), timeout=120, check=False)
            if result.returncode:
                fail(f"Git {arguments[0]} failed with exit code {result.returncode}")
            output.seek(0)
            data = output.read(64 * 1024 * 1024 + 1)
    except (OSError, subprocess.TimeoutExpired) as error:
        fail(f"Git is unavailable or exceeded its deadline: {type(error).__name__}")
    if len(data) > 64 * 1024 * 1024:
        fail("Git metadata exceeds the output limit")
    return data.decode("utf-8", "strict") if text else data


def git_object_format(source: Path) -> str:
    value = str(run_git(source, ["rev-parse", "--show-object-format"])).strip()
    if value not in {"sha1", "sha256"}:
        fail("Unsupported Git object format")
    return value


def is_object_id(value: str, algorithm: str) -> bool:
    return len(value) == (40 if algorithm == "sha1" else 64) and all(c in "0123456789abcdef" for c in value)


def resolve_commit(source: Path, revision: str, object_format: str | None = None) -> str:
    if not source.is_dir():
        fail("Source directory does not exist")
    if not revision or any(ord(c) < 32 for c in revision):
        fail("Invalid source revision")
    algorithm = object_format or git_object_format(source)
    resolved = str(run_git(source, ["rev-parse", "--verify", "--end-of-options", f"{revision}^{{commit}}"])).strip()
    if not is_object_id(resolved, algorithm):
        fail("Git returned a non-canonical commit identifier")
    return resolved


def validate_archive_path(candidate: str) -> str:
    if not candidate or "\\" in candidate or ":" in candidate or any(ord(c) < 32 or ord(c) == 127 for c in candidate):
        fail(f"Source contains an invalid path: {candidate!r}")
    path = PurePosixPath(candidate)
    if path.is_absolute() or any(part in {"", ".", ".."} for part in candidate.split("/")) or path.as_posix() != candidate:
        fail(f"Source path escapes or is not canonical: {candidate!r}")
    return candidate


def classify(path: str, version_path: str) -> str:
    if path == version_path:
        return "version"
    suffix = PurePosixPath(path).suffix.lower()
    if suffix == ".lua":
        return "generated_api" if "Blizzard_APIDocumentationGenerated" in PurePosixPath(path).parts else "lua"
    return {".toc": "toc", ".xml": "xml", ".xsd": "schema"}.get(suffix, "other")


def should_include(path: str, extensions: frozenset[str], version_path: str) -> bool:
    return path == version_path or PurePosixPath(path).suffix.lower() in extensions


def canonical_bytes(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def normalize_extensions(candidates: Iterable[str] | None) -> frozenset[str]:
    result = set()
    for candidate in DEFAULT_EXTENSIONS if candidates is None else candidates:
        value = candidate.strip().lower()
        if not value.startswith(".") or len(value) < 2 or not value[1:].isascii() or not value[1:].isalnum():
            fail(f"Invalid file extension: {candidate!r}")
        result.add(value)
    if not result:
        fail("At least one extension must be included")
    return frozenset(result)


def source_records(source: Path, commit: str, algorithm: str, extensions: frozenset[str],
                   version_path: str, limits: Limits) -> tuple[list[dict], str, dict]:
    listing = run_git(source, ["ls-tree", "-rlz", "--full-tree", commit], text=False)
    assert isinstance(listing, bytes)
    entries = [record for record in listing.split(b"\0") if record]
    if len(entries) > limits.max_files:
        fail("Source snapshot exceeds the file-count limit")
    selected: list[tuple[str, str, int]] = []
    total_bytes = 0
    for entry in entries:
        metadata, path_bytes = entry.split(b"\t", 1)
        mode, kind, object_bytes, size_bytes = metadata.split()
        path = validate_archive_path(path_bytes.decode("utf-8", "strict"))
        if mode not in {b"100644", b"100755"} or kind != b"blob":
            fail(f"Source snapshot contains an unsupported non-file entry: {path!r}")
        object_id = object_bytes.decode("ascii")
        if not is_object_id(object_id, algorithm):
            fail("Invalid Git blob identifier")
        if not should_include(path, extensions, version_path):
            continue
        size = int(size_bytes)
        total_bytes += size
        if size < 0 or size > limits.max_file_bytes or total_bytes > limits.max_total_bytes:
            fail("Included source exceeds the byte limit")
        selected.append((path, object_id, size))
    selected.sort(key=lambda item: item[0].encode("utf-8"))
    records: list[dict] = []
    version = None
    with tempfile.TemporaryFile() as errors:
        process = subprocess.Popen(git_command(source, ["cat-file", "--batch"]), stdin=subprocess.PIPE,
                                   stdout=subprocess.PIPE, stderr=errors, env=git_environment())
        assert process.stdin is not None and process.stdout is not None
        try:
            for path, object_id, size in selected:
                process.stdin.write(object_id.encode("ascii") + b"\n")
                process.stdin.flush()
                header = process.stdout.readline(256)
                if header != f"{object_id} blob {size}\n".encode("ascii"):
                    fail("Git blob header differs from the source inventory")
                content = process.stdout.read(size)
                if len(content) != size or process.stdout.read(1) != b"\n":
                    fail("Git blob content was truncated")
                digest = hashlib.new(algorithm, f"blob {size}\0".encode("ascii") + content).hexdigest()
                if digest != object_id:
                    fail("Git blob content does not match its object identifier")
                if path == version_path:
                    version = content.decode("utf-8", "strict").strip()
                    if not version or any(ord(c) < 32 for c in version):
                        fail("Version file must contain one non-empty value")
                records.append({"path": path, "kind": classify(path, version_path), "bytes": size,
                                "git_blob_algorithm": algorithm, "git_blob_id": object_id,
                                "content_sha256": hashlib.sha256(content).hexdigest()})
            process.stdin.close()
            if process.wait(timeout=30):
                fail("Git blob reader failed")
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
            process.stdin.close()
            process.stdout.close()
    if version is None:
        fail(f"Required version file is absent from the exact snapshot: {version_path}")
    counts: dict[str, int] = {}
    for record in records:
        key = "kind_" + record["kind"]
        counts[key] = counts.get(key, 0) + 1
    coverage = {"tracked_files": len(entries), "included_files": len(records),
                "excluded_files": len(entries) - len(records), "included_bytes": total_bytes, **counts}
    return records, version, coverage


def build_manifest(source: Path, revision: str, *, source_id: str, selector: str | None,
                   extensions: frozenset[str], version_path: str, limits: Limits) -> dict[str, object]:
    if not source_id or any(ord(c) < 32 for c in source_id):
        fail("Invalid source identifier")
    if selector is not None and (not selector or any(ord(c) < 32 for c in selector)):
        fail("Invalid selector label")
    if any(isinstance(n, bool) or not isinstance(n, int) or n <= 0 for n in (limits.max_files, limits.max_file_bytes, limits.max_total_bytes)):
        fail("Source limits must be positive integers")
    version_path = validate_archive_path(version_path)
    extensions = normalize_extensions(extensions)
    algorithm = git_object_format(source)
    commit = resolve_commit(source, revision, algorithm)
    records, version, coverage = source_records(source, commit, algorithm, extensions, version_path, limits)
    manifest: dict[str, object] = {
        "schema_version": SCHEMA_VERSION,
        "source": {"source_id": source_id, "selector": selector, "revision": commit,
                   "git_object_format": algorithm, "version": version, "acquisition": "local_git_object_database"},
        "selection": {"extensions": sorted(extensions), "version_path": version_path,
                      "non_regular_entries": "reject", "working_tree": "ignored"},
        "coverage": coverage, "files": records,
    }
    manifest["manifest_sha256"] = hashlib.sha256(canonical_bytes(manifest)).hexdigest()
    return manifest


def write_manifest(manifest: dict[str, object], output: str) -> None:
    rendered = json.dumps(manifest, ensure_ascii=False, sort_keys=True, indent=2) + "\n"
    if output == "-":
        sys.stdout.write(rendered)
        return
    destination = Path(output).expanduser()
    destination.parent.mkdir(parents=True, exist_ok=True)
    descriptor, name = tempfile.mkstemp(prefix=f".{destination.name}.", suffix=".tmp", dir=destination.parent)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8", newline="\n") as handle:
            handle.write(rendered)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(name, destination)
    finally:
        if os.path.exists(name):
            os.unlink(name)


def positive_integer(candidate: str) -> int:
    value = int(candidate)
    if value <= 0:
        raise argparse.ArgumentTypeError("value must be greater than zero")
    return value


def parse_arguments(arguments: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--revision", default="HEAD")
    parser.add_argument("--selector")
    parser.add_argument("--source-id", default="blizzard-ui")
    parser.add_argument("--version-path", default=DEFAULT_VERSION_PATH)
    parser.add_argument("--extension", action="append", dest="extensions")
    parser.add_argument("--max-files", type=positive_integer, default=DEFAULT_MAX_FILES)
    parser.add_argument("--max-file-bytes", type=positive_integer, default=DEFAULT_MAX_FILE_BYTES)
    parser.add_argument("--max-total-bytes", type=positive_integer, default=DEFAULT_MAX_TOTAL_BYTES)
    parser.add_argument("--output", default="-")
    return parser.parse_args(arguments)


def main(arguments: Sequence[str] | None = None) -> int:
    options = parse_arguments(arguments if arguments is not None else sys.argv[1:])
    try:
        manifest = build_manifest(options.source.expanduser().resolve(), options.revision,
                                  source_id=options.source_id, selector=options.selector,
                                  extensions=normalize_extensions(options.extensions), version_path=options.version_path,
                                  limits=Limits(options.max_files, options.max_file_bytes, options.max_total_bytes))
        write_manifest(manifest, options.output)
    except (ManifestError, OSError, ValueError, subprocess.TimeoutExpired) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
