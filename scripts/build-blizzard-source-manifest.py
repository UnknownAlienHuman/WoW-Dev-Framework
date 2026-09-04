#!/usr/bin/env python3
"""Build a deterministic manifest from one exact Blizzard UI Git snapshot.

The caller may select a moving branch before invoking this program, but this
program resolves that selector once and reads every byte from the resulting
commit. The working tree, current directory, remote URL, and wall clock never
participate in the manifest identity.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import tarfile
import tempfile
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import BinaryIO, Iterable, NoReturn, Sequence

SCHEMA_VERSION = 1
DEFAULT_EXTENSIONS = (".lua", ".toc", ".xml", ".xsd")
DEFAULT_VERSION_PATH = "version.txt"
DEFAULT_MAX_FILES = 200_000
DEFAULT_MAX_FILE_BYTES = 32 * 1024 * 1024
DEFAULT_MAX_TOTAL_BYTES = 2 * 1024 * 1024 * 1024


class ManifestError(RuntimeError):
    """A bounded source, Git, archive, or output validation failure."""


@dataclass(frozen=True)
class Limits:
    max_files: int
    max_file_bytes: int
    max_total_bytes: int


@dataclass(frozen=True)
class FileRecord:
    path: str
    kind: str
    bytes: int
    git_blob_sha1: str
    content_sha256: str

    def as_json(self) -> dict[str, object]:
        return {
            "path": self.path,
            "kind": self.kind,
            "bytes": self.bytes,
            "git_blob_sha1": self.git_blob_sha1,
            "content_sha256": self.content_sha256,
        }


def fail(message: str) -> NoReturn:
    raise ManifestError(message)


def run_git(source: Path, arguments: Sequence[str], *, text: bool = True) -> str | bytes:
    command = [
        "git",
        "-c",
        "core.hooksPath=/dev/null",
        "-c",
        "filter.lfs.smudge=",
        "-c",
        "filter.lfs.required=false",
        "-C",
        os.fspath(source),
        *arguments,
    ]
    try:
        completed = subprocess.run(
            command,
            check=False,
            capture_output=True,
            text=text,
            env={**os.environ, "GIT_OPTIONAL_LOCKS": "0"},
        )
    except FileNotFoundError as error:
        fail(f"Git executable is unavailable: {error}")
    if completed.returncode != 0:
        stderr = completed.stderr if text else completed.stderr.decode("utf-8", "replace")
        fail(f"Git command failed ({' '.join(arguments)}): {stderr.strip()}")
    return completed.stdout


def resolve_commit(source: Path, revision: str) -> str:
    if not source.is_dir():
        fail(f"Source directory does not exist: {source}")
    inside = str(run_git(source, ["rev-parse", "--is-inside-work-tree"])).strip()
    if inside != "true":
        fail(f"Source is not a Git working tree: {source}")
    resolved = str(run_git(source, ["rev-parse", "--verify", f"{revision}^{{commit}}"])).strip()
    if len(resolved) != 40 or any(character not in "0123456789abcdef" for character in resolved):
        fail(f"Git returned a non-canonical commit identifier: {resolved!r}")
    return resolved


def validate_archive_path(candidate: str) -> str:
    if not candidate or "\x00" in candidate or "\\" in candidate:
        fail(f"Archive contains an invalid path: {candidate!r}")
    path = PurePosixPath(candidate)
    if path.is_absolute() or any(part in {"", ".", ".."} for part in path.parts):
        fail(f"Archive path escapes or is not canonical: {candidate!r}")
    canonical = path.as_posix()
    if canonical != candidate:
        fail(f"Archive path is not canonical: {candidate!r}")
    return canonical


def classify(path: str, version_path: str) -> str:
    if path == version_path:
        return "version"
    suffix = PurePosixPath(path).suffix.lower()
    if suffix == ".lua":
        if "Blizzard_APIDocumentationGenerated" in PurePosixPath(path).parts:
            return "generated_api"
        return "lua"
    if suffix == ".toc":
        return "toc"
    if suffix == ".xml":
        return "xml"
    if suffix == ".xsd":
        return "schema"
    return "other"


def git_blob_sha1(content: bytes) -> str:
    header = f"blob {len(content)}\0".encode("ascii")
    return hashlib.sha1(header + content, usedforsecurity=False).hexdigest()


def should_include(path: str, extensions: frozenset[str], version_path: str) -> bool:
    return path == version_path or PurePosixPath(path).suffix.lower() in extensions


def read_member(member: tarfile.TarInfo, stream: BinaryIO, limits: Limits) -> bytes:
    if member.size < 0 or member.size > limits.max_file_bytes:
        fail(f"Source member exceeds the per-file byte limit: {member.name!r} ({member.size})")
    extracted = stream.read(member.size + 1)
    if len(extracted) != member.size:
        fail(f"Source member was truncated while reading: {member.name!r}")
    return extracted


def records_from_git_archive(
    source: Path,
    commit: str,
    *,
    extensions: frozenset[str],
    version_path: str,
    limits: Limits,
) -> tuple[list[FileRecord], str, dict[str, int]]:
    command = [
        "git",
        "-c",
        "core.hooksPath=/dev/null",
        "-c",
        "filter.lfs.smudge=",
        "-c",
        "filter.lfs.required=false",
        "-C",
        os.fspath(source),
        "archive",
        "--format=tar",
        commit,
    ]
    try:
        process = subprocess.Popen(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env={**os.environ, "GIT_OPTIONAL_LOCKS": "0"},
        )
    except FileNotFoundError as error:
        fail(f"Git executable is unavailable: {error}")
    if process.stdout is None or process.stderr is None:
        process.kill()
        fail("Git archive did not expose bounded output streams")

    records: list[FileRecord] = []
    version: str | None = None
    tracked_files = 0
    excluded_files = 0
    total_included_bytes = 0

    try:
        with tarfile.open(fileobj=process.stdout, mode="r|") as archive:
            for member in archive:
                path = validate_archive_path(member.name)
                if member.isdir():
                    continue
                if not member.isfile():
                    fail(f"Source snapshot contains an unsupported non-file entry: {path!r}")
                tracked_files += 1
                if tracked_files > limits.max_files:
                    fail(f"Source snapshot exceeds the file-count limit ({limits.max_files})")
                extracted = archive.extractfile(member)
                if extracted is None:
                    fail(f"Source member cannot be read: {path!r}")
                if not should_include(path, extensions, version_path):
                    excluded_files += 1
                    continue
                content = read_member(member, extracted, limits)
                total_included_bytes += len(content)
                if total_included_bytes > limits.max_total_bytes:
                    fail(
                        "Included source bytes exceed the total byte limit "
                        f"({limits.max_total_bytes})"
                    )
                if path == version_path:
                    try:
                        version = content.decode("utf-8").strip()
                    except UnicodeDecodeError as error:
                        fail(f"Version file is not UTF-8: {error}")
                    if not version or any(character in "\r\n\x00" for character in version):
                        fail("Version file does not contain one canonical non-empty value")
                records.append(
                    FileRecord(
                        path=path,
                        kind=classify(path, version_path),
                        bytes=len(content),
                        git_blob_sha1=git_blob_sha1(content),
                        content_sha256=hashlib.sha256(content).hexdigest(),
                    )
                )
    except (tarfile.TarError, OSError) as error:
        process.kill()
        fail(f"Cannot read exact Git archive: {error}")
    finally:
        process.stdout.close()

    stderr = process.stderr.read().decode("utf-8", "replace").strip()
    return_code = process.wait()
    if return_code != 0:
        fail(f"Git archive failed with exit code {return_code}: {stderr}")
    if version is None:
        fail(f"Required version file is absent from the exact snapshot: {version_path}")

    records.sort(key=lambda record: record.path.encode("utf-8"))
    counts: dict[str, int] = {}
    for record in records:
        counts[record.kind] = counts.get(record.kind, 0) + 1
    coverage = {
        "tracked_files": tracked_files,
        "included_files": len(records),
        "excluded_files": excluded_files,
        "included_bytes": total_included_bytes,
        **{f"kind_{kind}": count for kind, count in sorted(counts.items())},
    }
    return records, version, coverage


def canonical_bytes(value: object) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        sort_keys=True,
        separators=(",", ":"),
        allow_nan=False,
    ).encode("utf-8")


def build_manifest(
    source: Path,
    revision: str,
    *,
    source_id: str,
    selector: str | None,
    extensions: frozenset[str],
    version_path: str,
    limits: Limits,
) -> dict[str, object]:
    commit = resolve_commit(source, revision)
    records, version, coverage = records_from_git_archive(
        source,
        commit,
        extensions=extensions,
        version_path=version_path,
        limits=limits,
    )
    manifest: dict[str, object] = {
        "schema_version": SCHEMA_VERSION,
        "source": {
            "source_id": source_id,
            "selector": selector,
            "revision": commit,
            "version": version,
            "acquisition": "local_git_object_database",
        },
        "coverage": coverage,
        "files": [record.as_json() for record in records],
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
    descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{destination.name}.", suffix=".tmp", dir=destination.parent
    )
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8", newline="\n") as temporary:
            temporary.write(rendered)
            temporary.flush()
            os.fsync(temporary.fileno())
        os.replace(temporary_name, destination)
    except BaseException:
        try:
            os.unlink(temporary_name)
        except FileNotFoundError:
            pass
        raise


def positive_integer(candidate: str) -> int:
    value = int(candidate)
    if value <= 0:
        raise argparse.ArgumentTypeError("value must be greater than zero")
    return value


def parse_arguments(arguments: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", required=True, type=Path, help="local Git checkout")
    parser.add_argument(
        "--revision",
        default="HEAD",
        help="commit or selector to resolve exactly once (default: HEAD)",
    )
    parser.add_argument(
        "--selector",
        help="optional human channel/branch label; never used to read bytes after resolution",
    )
    parser.add_argument("--source-id", default="blizzard-ui", help="stable non-secret source label")
    parser.add_argument("--version-path", default=DEFAULT_VERSION_PATH)
    parser.add_argument(
        "--extension",
        action="append",
        dest="extensions",
        help="included lowercase extension; repeatable (default: .lua, .toc, .xml, .xsd)",
    )
    parser.add_argument("--max-files", type=positive_integer, default=DEFAULT_MAX_FILES)
    parser.add_argument("--max-file-bytes", type=positive_integer, default=DEFAULT_MAX_FILE_BYTES)
    parser.add_argument("--max-total-bytes", type=positive_integer, default=DEFAULT_MAX_TOTAL_BYTES)
    parser.add_argument("--output", default="-", help="destination JSON path or '-' for stdout")
    return parser.parse_args(arguments)


def normalize_extensions(candidates: Iterable[str] | None) -> frozenset[str]:
    values = candidates or DEFAULT_EXTENSIONS
    normalized: set[str] = set()
    for candidate in values:
        value = candidate.strip().lower()
        if not value.startswith(".") or len(value) < 2 or not value[1:].isalnum():
            fail(f"Invalid file extension: {candidate!r}")
        normalized.add(value)
    if not normalized:
        fail("At least one extension must be included")
    return frozenset(normalized)


def main(arguments: Sequence[str] | None = None) -> int:
    options = parse_arguments(arguments if arguments is not None else sys.argv[1:])
    try:
        manifest = build_manifest(
            options.source.expanduser().resolve(),
            options.revision,
            source_id=options.source_id,
            selector=options.selector,
            extensions=normalize_extensions(options.extensions),
            version_path=validate_archive_path(options.version_path),
            limits=Limits(
                max_files=options.max_files,
                max_file_bytes=options.max_file_bytes,
                max_total_bytes=options.max_total_bytes,
            ),
        )
        write_manifest(manifest, options.output)
    except (ManifestError, OSError, ValueError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
