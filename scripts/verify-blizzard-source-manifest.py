#!/usr/bin/env python3
"""Validate a Blizzard source manifest and optionally compare it with a current ref."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import sys
from pathlib import Path
from typing import Any, NoReturn, Sequence

SCRIPT_DIR = Path(__file__).resolve().parent
BUILDER_PATH = SCRIPT_DIR / "build-blizzard-source-manifest.py"
SPEC = importlib.util.spec_from_file_location("build_blizzard_source_manifest", BUILDER_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load manifest producer: {BUILDER_PATH}")
BUILDER = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = BUILDER
SPEC.loader.exec_module(BUILDER)

EXIT_INVALID = 2
EXIT_STALE = 3
HEX = frozenset("0123456789abcdef")


class VerificationError(RuntimeError):
    """The manifest is malformed, internally inconsistent, or mismatched."""


def fail(message: str) -> NoReturn:
    raise VerificationError(message)


def object_pairs_no_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            fail(f"duplicate JSON key: {key!r}")
        result[key] = value
    return result


def read_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(
            path.read_text(encoding="utf-8"),
            object_pairs_hook=object_pairs_no_duplicates,
            parse_constant=lambda candidate: fail(f"non-finite JSON number: {candidate}"),
        )
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        fail(f"cannot read manifest JSON: {error}")
    if not isinstance(value, dict):
        fail("manifest root must be an object")
    return value


def require_exact_keys(value: dict[str, Any], expected: set[str], label: str) -> None:
    actual = set(value)
    if actual != expected:
        missing = sorted(expected - actual)
        unknown = sorted(actual - expected)
        fail(f"{label} keys differ; missing={missing}, unknown={unknown}")


def require_string(value: Any, label: str, *, non_empty: bool = True) -> str:
    if not isinstance(value, str) or (non_empty and not value):
        fail(f"{label} must be {'a non-empty' if non_empty else 'a'} string")
    if "\x00" in value:
        fail(f"{label} contains NUL")
    return value


def require_integer(value: Any, label: str, *, minimum: int = 0) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < minimum:
        fail(f"{label} must be an integer greater than or equal to {minimum}")
    return value


def require_hex(value: Any, length: int, label: str) -> str:
    candidate = require_string(value, label)
    if len(candidate) != length or any(character not in HEX for character in candidate):
        fail(f"{label} must be {length} lowercase hexadecimal characters")
    return candidate


def validate_manifest(manifest: dict[str, Any]) -> None:
    require_exact_keys(
        manifest,
        {"schema_version", "source", "selection", "coverage", "files", "manifest_sha256"},
        "manifest",
    )
    if manifest["schema_version"] != BUILDER.SCHEMA_VERSION:
        fail(
            f"unsupported schema_version {manifest['schema_version']!r}; "
            f"expected {BUILDER.SCHEMA_VERSION}"
        )

    source = manifest["source"]
    if not isinstance(source, dict):
        fail("source must be an object")
    require_exact_keys(
        source,
        {"source_id", "selector", "revision", "git_object_format", "version", "acquisition"},
        "source",
    )
    require_string(source["source_id"], "source.source_id")
    selector = source["selector"]
    if selector is not None:
        require_string(selector, "source.selector")
    object_format = require_string(source["git_object_format"], "source.git_object_format")
    if object_format not in {"sha1", "sha256"}:
        fail(f"unsupported Git object format: {object_format!r}")
    object_length = 40 if object_format == "sha1" else 64
    require_hex(source["revision"], object_length, "source.revision")
    require_string(source["version"], "source.version")
    if source["acquisition"] != "local_git_object_database":
        fail(f"unsupported acquisition mode: {source['acquisition']!r}")

    selection = manifest["selection"]
    if not isinstance(selection, dict):
        fail("selection must be an object")
    require_exact_keys(
        selection,
        {"extensions", "version_path", "non_regular_entries", "working_tree"},
        "selection",
    )
    extensions = selection["extensions"]
    if not isinstance(extensions, list) or not extensions:
        fail("selection.extensions must be a non-empty array")
    normalized_extensions = [
        require_string(value, f"selection.extensions[{index}]")
        for index, value in enumerate(extensions)
    ]
    if normalized_extensions != sorted(set(normalized_extensions)):
        fail("selection.extensions must be unique and sorted")
    for extension in normalized_extensions:
        if not extension.startswith(".") or len(extension) < 2 or not extension[1:].isalnum():
            fail(f"invalid selected extension: {extension!r}")
    version_path = BUILDER.validate_archive_path(
        require_string(selection["version_path"], "selection.version_path")
    )
    if selection["non_regular_entries"] != "reject":
        fail("selection.non_regular_entries must be 'reject'")
    if selection["working_tree"] != "ignored":
        fail("selection.working_tree must be 'ignored'")

    files = manifest["files"]
    if not isinstance(files, list):
        fail("files must be an array")
    previous_path: bytes | None = None
    included_bytes = 0
    kind_counts: dict[str, int] = {}
    for index, record in enumerate(files):
        if not isinstance(record, dict):
            fail(f"files[{index}] must be an object")
        require_exact_keys(
            record,
            {
                "path",
                "kind",
                "bytes",
                "git_blob_algorithm",
                "git_blob_id",
                "content_sha256",
            },
            f"files[{index}]",
        )
        path = BUILDER.validate_archive_path(require_string(record["path"], f"files[{index}].path"))
        path_key = path.encode("utf-8")
        if previous_path is not None and path_key <= previous_path:
            fail("file records must be unique and bytewise path-sorted")
        previous_path = path_key
        if not BUILDER.should_include(path, frozenset(normalized_extensions), version_path):
            fail(f"file is outside the declared selection policy: {path!r}")
        kind = require_string(record["kind"], f"files[{index}].kind")
        if kind != BUILDER.classify(path, version_path):
            fail(f"file kind does not match its path: {path!r}")
        size = require_integer(record["bytes"], f"files[{index}].bytes")
        included_bytes += size
        kind_counts[kind] = kind_counts.get(kind, 0) + 1
        if record["git_blob_algorithm"] != object_format:
            fail(f"file Git object format differs from the source: {path!r}")
        require_hex(record["git_blob_id"], object_length, f"files[{index}].git_blob_id")
        require_hex(record["content_sha256"], 64, f"files[{index}].content_sha256")

    coverage = manifest["coverage"]
    if not isinstance(coverage, dict):
        fail("coverage must be an object")
    required_coverage = {"tracked_files", "included_files", "excluded_files", "included_bytes"}
    if not required_coverage.issubset(coverage):
        fail(f"coverage is missing fields: {sorted(required_coverage - set(coverage))}")
    tracked_files = require_integer(coverage["tracked_files"], "coverage.tracked_files")
    included_files = require_integer(coverage["included_files"], "coverage.included_files")
    excluded_files = require_integer(coverage["excluded_files"], "coverage.excluded_files")
    if tracked_files != included_files + excluded_files:
        fail("coverage.tracked_files must equal included_files + excluded_files")
    if included_files != len(files):
        fail("coverage.included_files differs from the file-record count")
    if require_integer(coverage["included_bytes"], "coverage.included_bytes") != included_bytes:
        fail("coverage.included_bytes differs from the file-record total")
    expected_coverage_keys = required_coverage | {f"kind_{kind}" for kind in kind_counts}
    require_exact_keys(coverage, expected_coverage_keys, "coverage")
    for kind, count in kind_counts.items():
        if require_integer(coverage[f"kind_{kind}"], f"coverage.kind_{kind}") != count:
            fail(f"coverage count differs for kind {kind!r}")

    supplied_digest = require_hex(manifest["manifest_sha256"], 64, "manifest_sha256")
    digest_input = dict(manifest)
    del digest_input["manifest_sha256"]
    expected_digest = hashlib.sha256(BUILDER.canonical_bytes(digest_input)).hexdigest()
    if supplied_digest != expected_digest:
        fail("manifest_sha256 does not match the canonical manifest content")


def rebuild_and_compare(manifest: dict[str, Any], source_root: Path) -> None:
    source = manifest["source"]
    selection = manifest["selection"]
    coverage = manifest["coverage"]
    rebuilt = BUILDER.build_manifest(
        source_root.expanduser().resolve(),
        source["revision"],
        source_id=source["source_id"],
        selector=source["selector"],
        extensions=frozenset(selection["extensions"]),
        version_path=selection["version_path"],
        limits=BUILDER.Limits(
            max_files=max(coverage["tracked_files"] + 1, BUILDER.DEFAULT_MAX_FILES),
            max_file_bytes=BUILDER.DEFAULT_MAX_FILE_BYTES,
            max_total_bytes=max(coverage["included_bytes"] + 1, BUILDER.DEFAULT_MAX_TOTAL_BYTES),
        ),
    )
    if BUILDER.canonical_bytes(rebuilt) != BUILDER.canonical_bytes(manifest):
        fail("manifest does not reproduce from the declared exact source revision")


def current_status(manifest: dict[str, Any], source_root: Path, current_ref: str) -> tuple[bool, str]:
    object_format = BUILDER.git_object_format(source_root)
    current_revision = BUILDER.resolve_commit(source_root, current_ref, object_format)
    return current_revision == manifest["source"]["revision"], current_revision


def parse_arguments(arguments: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--source", type=Path, help="local source checkout for byte verification")
    parser.add_argument(
        "--current-ref",
        help="moving ref to compare after exact-byte verification, for example origin/live",
    )
    parser.add_argument("--json", action="store_true", dest="json_output")
    return parser.parse_args(arguments)


def main(arguments: Sequence[str] | None = None) -> int:
    options = parse_arguments(arguments if arguments is not None else sys.argv[1:])
    try:
        manifest = read_json(options.manifest)
        validate_manifest(manifest)
        if options.current_ref and options.source is None:
            fail("--current-ref requires --source")
        if options.source is not None:
            rebuild_and_compare(manifest, options.source)
        is_current = True
        current_revision = manifest["source"]["revision"]
        if options.current_ref is not None:
            is_current, current_revision = current_status(
                manifest, options.source.expanduser().resolve(), options.current_ref
            )
        result = {
            "valid": True,
            "current": is_current,
            "manifest_revision": manifest["source"]["revision"],
            "current_revision": current_revision,
            "version": manifest["source"]["version"],
            "manifest_sha256": manifest["manifest_sha256"],
        }
        if options.json_output:
            print(json.dumps(result, sort_keys=True, separators=(",", ":")))
        elif is_current:
            print(
                f"valid current manifest {result['manifest_sha256']} "
                f"for {result['version']} @ {result['manifest_revision']}"
            )
        else:
            print(
                "valid but stale manifest; update the source selector and rebuild: "
                f"manifest={result['manifest_revision']} current={current_revision}",
                file=sys.stderr,
            )
        return 0 if is_current else EXIT_STALE
    except (VerificationError, BUILDER.ManifestError, OSError, ValueError) as error:
        if options.json_output:
            print(json.dumps({"valid": False, "error": str(error)}, sort_keys=True))
        else:
            print(f"error: {error}", file=sys.stderr)
        return EXIT_INVALID


if __name__ == "__main__":
    raise SystemExit(main())
