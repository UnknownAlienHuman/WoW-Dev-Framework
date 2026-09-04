#!/usr/bin/env python3
"""Verify a normalized Blizzard generated-API ReferenceDraft."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any, Sequence

SCHEMA = "wow.reference-draft.blizzard-generated-api/v1"
HEX_64 = re.compile(r"sha256:[0-9a-f]{64}")
REVISION = re.compile(r"[0-9a-f]{40,64}")


class VerificationError(RuntimeError):
    pass


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def digest(value: Any) -> str:
    return "sha256:" + hashlib.sha256(canonical_bytes(value)).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise VerificationError(message)


def verify(path: Path) -> dict[str, Any]:
    try:
        root = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise VerificationError("cannot read draft JSON") from error
    require(isinstance(root, dict), "draft root must be an object")
    require(root.get("schema") == SCHEMA, "unsupported draft schema")
    supplied = root.get("draft_digest")
    require(isinstance(supplied, str) and HEX_64.fullmatch(supplied) is not None, "invalid draft_digest")
    projection = dict(root)
    projection.pop("draft_digest", None)
    require(supplied == digest(projection), "draft_digest mismatch")

    source = root.get("source")
    require(isinstance(source, dict), "source must be an object")
    revision = source.get("revision")
    require(isinstance(revision, str) and REVISION.fullmatch(revision) is not None, "source revision must be a full hexadecimal object identifier")
    manifest_digest = source.get("manifest_digest")
    require(isinstance(manifest_digest, str) and HEX_64.fullmatch(manifest_digest) is not None, "source manifest_digest is invalid")

    coverage = root.get("coverage")
    require(isinstance(coverage, dict), "coverage must be an object")
    status = coverage.get("status")
    require(status in {"complete", "partial"}, "invalid coverage status")
    candidate = coverage.get("candidate_files")
    parsed = coverage.get("parsed_files")
    failures = coverage.get("failed_files")
    files = coverage.get("files")
    require(isinstance(candidate, int) and candidate > 0, "candidate_files must be positive")
    require(isinstance(parsed, int) and 0 <= parsed <= candidate, "invalid parsed_files")
    require(isinstance(failures, list), "failed_files must be an array")
    require(isinstance(files, list) and len(files) == candidate, "coverage file count does not match candidate_files")
    require(parsed + len(failures) == candidate, "parsed and failed file counts do not cover candidates")
    require((status == "complete") == (len(failures) == 0), "coverage status does not match failures")
    require(coverage.get("negative_authority") is (len(failures) == 0), "negative_authority must be false for partial coverage")

    paths: list[str] = []
    for entry in files:
        require(isinstance(entry, dict), "coverage file entry must be an object")
        file_path = entry.get("path")
        require(isinstance(file_path, str) and file_path, "coverage path is invalid")
        require(entry.get("status") in {"parsed", "failed"}, "invalid file status")
        paths.append(file_path)
    require(paths == sorted(paths, key=lambda item: item.encode("utf-8")), "coverage files are not bytewise path-sorted")
    require(len(paths) == len(set(paths)), "duplicate coverage file path")

    systems = root.get("systems")
    require(isinstance(systems, list), "systems must be an array")
    system_keys: list[tuple[str, str]] = []
    qualified_names: set[str] = set()
    record_counts = {"systems": len(systems), "functions": 0, "events": 0, "tables": 0, "fields": 0}
    for system in systems:
        require(isinstance(system, dict), "system must be an object")
        name = system.get("name")
        namespace = system.get("namespace")
        require(isinstance(name, str) and name, "system name is invalid")
        require(namespace is None or isinstance(namespace, str), "namespace is invalid")
        system_keys.append((namespace or "", name))
        source_record = system.get("source")
        require(isinstance(source_record, dict), "system source is missing")
        source_path = source_record.get("path")
        source_sha = source_record.get("sha256")
        require(isinstance(source_path, str) and source_path in paths, "system source path is outside coverage")
        require(isinstance(source_sha, str) and HEX_64.fullmatch(source_sha) is not None, "system source SHA-256 is invalid")
        for collection, kind in (("functions", "function"), ("events", "event")):
            records = system.get(collection)
            require(isinstance(records, list), f"{collection} must be an array")
            keys: list[tuple[str, str]] = []
            for record in records:
                require(isinstance(record, dict), f"{kind} must be an object")
                require(record.get("kind") == kind, f"invalid {kind} kind")
                record_name = record.get("name")
                qualified = record.get("qualified_name")
                require(isinstance(record_name, str) and record_name, f"invalid {kind} name")
                require(isinstance(qualified, str) and qualified, f"invalid {kind} qualified_name")
                require(qualified not in qualified_names, f"duplicate callable identity: {qualified}")
                qualified_names.add(qualified)
                keys.append((qualified, record_name))
                for parameter_key in ("arguments", "returns", "payload"):
                    parameters = record.get(parameter_key, [])
                    require(isinstance(parameters, list), f"{kind}.{parameter_key} must be an array")
                    for position, parameter in enumerate(parameters, 1):
                        require(isinstance(parameter, dict), "parameter must be an object")
                        require(parameter.get("position") == position, "parameter positions must be contiguous")
                        require(isinstance(parameter.get("name"), str), "parameter name is invalid")
                        require(isinstance(parameter.get("type"), str), "parameter type is invalid")
            require(keys == sorted(keys), f"{collection} are not canonically sorted")
            record_counts[collection] += len(records)
        tables = system.get("tables")
        require(isinstance(tables, list), "tables must be an array")
        table_keys: list[tuple[str, str]] = []
        for table in tables:
            require(isinstance(table, dict), "table must be an object")
            table_name = table.get("name")
            require(isinstance(table_name, str) and table_name, "table name is invalid")
            table_keys.append((str(table.get("qualified_name") or ""), table_name))
            fields = table.get("fields", [])
            require(isinstance(fields, list), "table fields must be an array")
            for position, field in enumerate(fields, 1):
                require(isinstance(field, dict) and field.get("position") == position, "table field positions must be contiguous")
            record_counts["fields"] += len(fields)
        require(table_keys == sorted(table_keys), "tables are not canonically sorted")
        record_counts["tables"] += len(tables)
    require(system_keys == sorted(system_keys), "systems are not canonically sorted")
    require(coverage.get("record_counts") == record_counts, "coverage record_counts mismatch")
    producer = root.get("producer")
    require(isinstance(producer, dict), "producer must be an object")
    require(isinstance(producer.get("id"), str) and producer.get("id"), "producer id is invalid")
    require(isinstance(producer.get("version"), str) and producer.get("version"), "producer version is invalid")
    return {"draft_digest": supplied, "coverage": status, "record_counts": record_counts, "source_revision": revision}


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("draft", type=Path)
    result.add_argument("--require-complete", action="store_true")
    result.add_argument("--json", action="store_true")
    return result


def main(argv: Sequence[str] | None = None) -> int:
    args = parser().parse_args(argv)
    try:
        summary = verify(args.draft)
        if args.require_complete and summary["coverage"] != "complete":
            raise VerificationError("complete coverage is required")
    except VerificationError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    if args.json:
        print(json.dumps(summary, ensure_ascii=False, sort_keys=True))
    else:
        counts = summary["record_counts"]
        print(f"verified {summary['draft_digest']}: {counts['systems']} systems, {counts['functions']} functions, {counts['events']} events, {counts['tables']} tables")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
