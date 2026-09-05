#!/usr/bin/env python3
"""Build and verify a non-executing TOC/XML topology from one exact source revision."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
import unicodedata
import xml.parsers.expat as expat
from collections import defaultdict
from pathlib import Path, PurePosixPath
from typing import Any, Iterable, Mapping, Sequence

SCHEMA = "wow-dev-framework/blizzard-ui-topology-draft"
SCHEMA_VERSION = 1
PRODUCER_ID = "blizzard-ui-topology"
PRODUCER_VERSION = 2
DEFAULT_MAX_FILE_BYTES = 32 * 1024 * 1024
DEFAULT_MAX_TOTAL_BYTES = 1024 * 1024 * 1024
DEFAULT_MAX_XML_ELEMENTS = 2_000_000
DEFAULT_MAX_XML_DEPTH = 512
_INTERFACE_ROOT = "Interface/"
_REFERENCE_ISSUES = {"invalid_reference", "missing_target", "case_mismatch", "ambiguous_case"}


class TopologyError(Exception):
    """Bounded topology error with a stable code."""

    def __init__(self, code: str, message: str, *, path: str | None = None) -> None:
        super().__init__(message)
        self.code = code
        self.message = message
        self.path = path

    def record(self) -> dict[str, Any]:
        output: dict[str, Any] = {"code": self.code, "message": self.message}
        if self.path is not None:
            output["path"] = self.path
        return output


def canonical_json_bytes(value: Any) -> bytes:
    return json.dumps(
        value,
        ensure_ascii=False,
        allow_nan=False,
        sort_keys=True,
        separators=(",", ":"),
    ).encode("utf-8")


def sha256_id(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def _get_nested(value: Mapping[str, Any], path: Sequence[str]) -> Any:
    current: Any = value
    for segment in path:
        if not isinstance(current, Mapping) or segment not in current:
            return None
        current = current[segment]
    return current


def _first_nested(value: Mapping[str, Any], paths: Iterable[Sequence[str]]) -> Any:
    for path in paths:
        candidate = _get_nested(value, path)
        if candidate is not None:
            return candidate
    return None


def load_json(path: Path, *, max_bytes: int = 128 * 1024 * 1024) -> tuple[dict[str, Any], bytes]:
    try:
        size = path.stat().st_size
        if size > max_bytes:
            raise TopologyError("input_size_limit", "input JSON exceeds size limit")
        raw = path.read_bytes()
        value = json.loads(raw.decode("utf-8"))
    except TopologyError:
        raise
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise TopologyError("input_json", "input is not valid UTF-8 JSON") from error
    if not isinstance(value, dict):
        raise TopologyError("input_json_shape", "input JSON root must be an object")
    return value, raw


def _manifest_revision(manifest: Mapping[str, Any]) -> str:
    candidate = _first_nested(
        manifest,
        (
            ("source", "revision"),
            ("source", "resolved_revision"),
            ("source", "resolved_commit"),
            ("source", "commit"),
            ("source", "commit_sha"),
            ("resolved_revision",),
            ("resolved_commit",),
            ("revision",),
            ("commit",),
            ("commit_sha",),
            ("source_revision",),
        ),
    )
    if not isinstance(candidate, str) or not re.fullmatch(
        r"[0-9a-fA-F]{40}|[0-9a-fA-F]{64}", candidate
    ):
        raise TopologyError("manifest_revision", "source manifest has no exact Git object identifier")
    return candidate.lower()


def _manifest_files(manifest: Mapping[str, Any]) -> list[Mapping[str, Any]]:
    candidate = _first_nested(
        manifest,
        (("files",), ("entries",), ("inventory", "files"), ("source", "files")),
    )
    if not isinstance(candidate, list):
        raise TopologyError("manifest_files", "source manifest has no file inventory")
    records: list[Mapping[str, Any]] = []
    for index, item in enumerate(candidate):
        if not isinstance(item, Mapping):
            raise TopologyError("manifest_file_record", f"file record {index} is not an object")
        records.append(item)
    return records


def _record_value(record: Mapping[str, Any], names: Sequence[str]) -> Any:
    for name in names:
        if name in record:
            return record[name]
    return None


def _safe_inventory_path(candidate: Any) -> str:
    if not isinstance(candidate, str) or not candidate:
        raise TopologyError("manifest_path", "source file record has no path")
    if "\x00" in candidate or "\n" in candidate or "\r" in candidate or "\\" in candidate:
        raise TopologyError("manifest_path", "source path is not canonical POSIX text")
    path = PurePosixPath(candidate)
    if (
        path.is_absolute()
        or path.as_posix() != candidate
        or any(part in {"", ".", ".."} for part in path.parts)
    ):
        raise TopologyError("manifest_path", "source path is not canonical or escapes the repository")
    return candidate


def _normalize_sha256(candidate: Any) -> str:
    if not isinstance(candidate, str):
        raise TopologyError("manifest_sha256", "source file record has no SHA-256 digest")
    digest = candidate.lower().removeprefix("sha256:")
    if not re.fullmatch(r"[0-9a-f]{64}", digest):
        raise TopologyError("manifest_sha256", "source file digest is not SHA-256")
    return "sha256:" + digest


def _manifest_metadata(manifest: Mapping[str, Any]) -> dict[str, Any]:
    return {
        "source_id": _first_nested(manifest, (("source", "id"), ("source_id",))),
        "selector": _first_nested(manifest, (("source", "selector"), ("selector",))),
        "version": _first_nested(
            manifest,
            (("source", "version"), ("version",), ("reported_version",)),
        ),
        "declared_digest": _first_nested(
            manifest, (("manifest_sha256",), ("manifest_digest",), ("digest",))
        ),
    }


def _run_git(source: Path, arguments: Sequence[str]) -> bytes:
    try:
        result = subprocess.run(
            ["git", "-C", os.fspath(source), *arguments],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
    except OSError as error:
        raise TopologyError("git_unavailable", "Git is not available") from error
    if result.returncode != 0:
        message = result.stderr.decode("utf-8", errors="replace").strip()
        raise TopologyError("git_failed", message.splitlines()[-1] if message else "Git command failed")
    return result.stdout


def _validate_revision(source: Path, revision: str) -> None:
    object_format = _run_git(source, ["rev-parse", "--show-object-format"]).decode("ascii").strip()
    if object_format not in {"sha1", "sha256"}:
        raise TopologyError("git_object_format", f"unsupported Git object format {object_format!r}")
    expected = 40 if object_format == "sha1" else 64
    if len(revision) != expected:
        raise TopologyError(
            "manifest_revision_format",
            f"manifest revision length does not match repository {object_format} format",
        )
    _run_git(source, ["cat-file", "-e", f"{revision}^{{commit}}"])


class GitBatchReader:
    """Read exact Git blobs with one bounded `cat-file --batch` process."""

    def __init__(self, source: Path, revision: str) -> None:
        try:
            self.process = subprocess.Popen(
                ["git", "-C", os.fspath(source), "cat-file", "--batch"],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
        except OSError as error:
            raise TopologyError("git_unavailable", "Git is not available") from error
        self.revision = revision

    def __enter__(self) -> "GitBatchReader":
        return self

    def read(self, path: str, max_bytes: int) -> tuple[bytes, str]:
        stdin = self.process.stdin
        stdout = self.process.stdout
        if stdin is None or stdout is None:
            raise TopologyError("git_batch_closed", "Git batch reader is closed", path=path)
        query = f"{self.revision}:{path}\n".encode("utf-8")
        try:
            stdin.write(query)
            stdin.flush()
            header = stdout.readline()
        except (BrokenPipeError, OSError) as error:
            raise TopologyError("git_batch_failed", "Git batch reader failed", path=path) from error
        if not header:
            raise TopologyError("git_batch_failed", "Git batch reader returned no header", path=path)
        if header.rstrip().endswith(b" missing"):
            raise TopologyError("source_missing", "manifest path is absent from exact revision", path=path)
        parts = header.rstrip(b"\n").split(b" ")
        if len(parts) != 3 or parts[1] != b"blob":
            raise TopologyError("source_not_blob", "manifest path does not identify a Git blob", path=path)
        try:
            size = int(parts[2])
            object_id = parts[0].decode("ascii").lower()
        except (ValueError, UnicodeDecodeError) as error:
            raise TopologyError("git_batch_header", "Git returned an invalid batch header", path=path) from error
        if size < 0 or size > max_bytes:
            raise TopologyError("source_file_limit", "source file exceeds file size limit", path=path)
        data = stdout.read(size)
        trailer = stdout.read(1)
        if len(data) != size or trailer != b"\n":
            raise TopologyError("git_batch_truncated", "Git returned a truncated blob", path=path)
        return data, object_id

    def __exit__(self, exc_type: Any, exc: Any, traceback: Any) -> None:
        if self.process.stdin is not None:
            self.process.stdin.close()
        return_code = self.process.wait(timeout=30)
        if return_code != 0 and exc is None:
            stderr = b"" if self.process.stderr is None else self.process.stderr.read()
            message = stderr.decode("utf-8", errors="replace").strip()
            raise TopologyError("git_batch_failed", message or "Git batch reader failed")


def _inventory(manifest: Mapping[str, Any]) -> tuple[dict[str, dict[str, Any]], dict[str, list[str]]]:
    records: dict[str, dict[str, Any]] = {}
    lower: dict[str, list[str]] = defaultdict(list)
    for raw in _manifest_files(manifest):
        path = _safe_inventory_path(_record_value(raw, ("path", "canonical_path", "relative_path")))
        if path in records:
            raise TopologyError("manifest_duplicate_path", f"duplicate source path {path!r}")
        size = _record_value(raw, ("size", "bytes", "size_bytes"))
        if size is not None and (not isinstance(size, int) or isinstance(size, bool) or size < 0):
            raise TopologyError("manifest_size", f"source file {path!r} has an invalid size")
        git_object = _record_value(raw, ("git_object", "git_blob", "blob", "blob_id", "object_id"))
        if git_object is not None and not isinstance(git_object, str):
            raise TopologyError("manifest_git_object", f"source file {path!r} has an invalid Git object")
        records[path] = {
            "path": path,
            "sha256": _normalize_sha256(
                _record_value(raw, ("sha256", "content_sha256", "content_digest", "digest"))
            ),
            "git_object": git_object.lower() if isinstance(git_object, str) else None,
            "size": size,
        }
        lower[path.casefold()].append(path)
    for paths in lower.values():
        paths.sort(key=str.encode)
    return records, dict(lower)


def _candidate_paths(inventory: Mapping[str, Any]) -> tuple[list[str], list[str]]:
    toc = sorted(
        (path for path in inventory if path.startswith(_INTERFACE_ROOT) and path.lower().endswith(".toc")),
        key=str.encode,
    )
    xml = sorted(
        (path for path in inventory if path.startswith(_INTERFACE_ROOT) and path.lower().endswith(".xml")),
        key=str.encode,
    )
    if not toc:
        raise TopologyError("manifest_no_toc", "source manifest contains no Interface TOC files")
    if not xml:
        raise TopologyError("manifest_no_xml", "source manifest contains no Interface XML files")
    return toc, xml


def _decode_text(data: bytes, path: str) -> str:
    try:
        return data.decode("utf-8-sig")
    except UnicodeDecodeError as error:
        raise TopologyError("source_not_utf8", "source file is not UTF-8", path=path) from error


def _source_record(path: str, data: bytes, git_object: str, line_start: int = 1, line_end: int | None = None) -> dict[str, Any]:
    if line_end is None:
        line_end = max(1, data.count(b"\n") + 1)
    return {
        "path": path,
        "line_start": line_start,
        "line_end": line_end,
        "git_object": git_object,
        "sha256": sha256_id(data),
    }


def _resolve_reference(source_path: str, declared: str) -> tuple[str | None, str | None]:
    value = declared.strip().replace("\\", "/")
    if not value or any(unicodedata.category(character) == "Cc" for character in value):
        return None, "empty or control-bearing reference"
    if re.match(r"^[A-Za-z]:", value) or value.startswith("/") or "://" in value:
        return None, "absolute or external reference"
    if value.startswith(_INTERFACE_ROOT):
        components: list[str] = []
    else:
        components = list(PurePosixPath(source_path).parent.parts)
    for part in value.split("/"):
        if part in {"", "."}:
            continue
        if part == "..":
            if not components:
                return None, "reference escapes repository root"
            components.pop()
            continue
        components.append(part)
    if not components:
        return None, "reference resolves to repository root"
    target = PurePosixPath(*components).as_posix()
    if not target.startswith(_INTERFACE_ROOT):
        return None, "reference resolves outside Interface"
    return target, None


def _classify_target(
    source_path: str,
    declared: str,
    inventory: Mapping[str, Any],
    lower_inventory: Mapping[str, list[str]],
) -> tuple[dict[str, Any], dict[str, Any] | None]:
    target, resolution_error = _resolve_reference(source_path, declared)
    if target is None:
        return (
            {"declared": declared, "target": None, "resolution": "invalid"},
            {
                "code": "invalid_reference",
                "source_path": source_path,
                "declared": declared,
                "message": resolution_error,
            },
        )
    if target in inventory:
        return {"declared": declared, "target": target, "resolution": "exact"}, None
    candidates = lower_inventory.get(target.casefold(), [])
    if len(candidates) == 1:
        return (
            {"declared": declared, "target": target, "resolution": "case_mismatch", "candidate": candidates[0]},
            {
                "code": "case_mismatch",
                "source_path": source_path,
                "declared": declared,
                "target": target,
                "candidate": candidates[0],
                "message": "reference differs from the exact source path by case",
            },
        )
    if len(candidates) > 1:
        return (
            {"declared": declared, "target": target, "resolution": "ambiguous_case", "candidates": candidates},
            {
                "code": "ambiguous_case",
                "source_path": source_path,
                "declared": declared,
                "target": target,
                "candidates": candidates,
                "message": "reference has multiple case-insensitive candidates",
            },
        )
    return (
        {"declared": declared, "target": target, "resolution": "missing"},
        {
            "code": "missing_target",
            "source_path": source_path,
            "declared": declared,
            "target": target,
            "message": "reference target is absent from the source manifest",
        },
    )


def _parse_toc(
    path: str,
    data: bytes,
    git_object: str,
    inventory: Mapping[str, Any],
    lower_inventory: Mapping[str, list[str]],
) -> tuple[dict[str, Any], list[dict[str, Any]], list[dict[str, Any]]]:
    text = _decode_text(data, path)
    metadata: dict[str, list[dict[str, Any]]] = defaultdict(list)
    entries: list[dict[str, Any]] = []
    edges: list[dict[str, Any]] = []
    issues: list[dict[str, Any]] = []
    for line_number, raw_line in enumerate(text.splitlines(), 1):
        line = raw_line.strip()
        if not line:
            continue
        metadata_match = re.match(r"^##\s*([^:]+?)\s*:\s*(.*)$", line)
        if metadata_match:
            key = metadata_match.group(1).strip()
            if not key:
                issues.append({
                    "code": "invalid_metadata",
                    "source_path": path,
                    "line": line_number,
                    "message": "TOC metadata key is empty",
                })
                continue
            metadata[key].append({"value": metadata_match.group(2).strip(), "line": line_number})
            continue
        if line.startswith("#"):
            continue
        target, issue = _classify_target(path, line, inventory, lower_inventory)
        suffix = PurePosixPath(target.get("target") or line.replace("\\", "/")).suffix.lower()
        entry = {
            **target,
            "kind": {".lua": "lua", ".xml": "xml"}.get(suffix, "other"),
            "line": line_number,
        }
        entries.append(entry)
        edges.append({
            "kind": "toc_load",
            "source": path,
            "target": target.get("target"),
            "declared": line,
            "resolution": target["resolution"],
            "line": line_number,
        })
        if issue is not None:
            issue["line"] = line_number
            issues.append(issue)
    duplicate_metadata = [key for key, values in metadata.items() if len(values) > 1]
    for key in duplicate_metadata:
        issues.append({
            "code": "duplicate_metadata",
            "source_path": path,
            "key": key,
            "lines": [item["line"] for item in metadata[key]],
            "message": "TOC metadata key is declared more than once",
        })
    ordered_metadata = {
        key: metadata[key] for key in sorted(metadata, key=str.encode)
    }
    descriptor = {
        "path": path,
        "addon_directory": PurePosixPath(path).parent.name,
        "descriptor_name": PurePosixPath(path).stem,
        "metadata": ordered_metadata,
        "entries": entries,
        "source": _source_record(path, data, git_object),
    }
    return descriptor, edges, issues


class XmlTopologyParser:
    def __init__(self, path: str, max_elements: int, max_depth: int) -> None:
        self.path = path
        self.max_elements = max_elements
        self.max_depth = max_depth
        self.element_count = 0
        self.depth = 0
        self.root: str | None = None
        self.references: list[dict[str, Any]] = []
        self.templates: list[dict[str, Any]] = []
        self.inline_scripts = 0
        parser = expat.ParserCreate()
        parser.buffer_text = True
        parser.SetParamEntityParsing(expat.XML_PARAM_ENTITY_PARSING_NEVER)
        parser.StartElementHandler = self._start
        parser.EndElementHandler = self._end
        parser.EntityDeclHandler = self._entity
        parser.ExternalEntityRefHandler = self._external
        self.parser = parser

    def parse(self, text: str) -> dict[str, Any]:
        try:
            self.parser.Parse(text, True)
        except TopologyError:
            raise
        except expat.ExpatError as error:
            raise TopologyError(
                "xml_parse",
                f"XML parse failed at line {error.lineno}, column {error.offset}: {error}",
                path=self.path,
            ) from error
        if self.root is None:
            raise TopologyError("xml_empty", "XML document has no root element", path=self.path)
        return {
            "root": self.root,
            "element_count": self.element_count,
            "inline_scripts": self.inline_scripts,
            "references": self.references,
            "templates": self.templates,
        }

    def _start(self, name: str, attributes: Mapping[str, str]) -> None:
        self.element_count += 1
        self.depth += 1
        if self.element_count > self.max_elements:
            raise TopologyError("xml_element_limit", "XML document exceeds element limit", path=self.path)
        if self.depth > self.max_depth:
            raise TopologyError("xml_depth_limit", "XML document exceeds depth limit", path=self.path)
        local_name = name.rsplit(":", 1)[-1]
        if self.root is None:
            self.root = local_name
        lowered = {key.rsplit(":", 1)[-1].casefold(): value for key, value in attributes.items()}
        line = self.parser.CurrentLineNumber
        tag = local_name.casefold()
        if tag in {"include", "script"}:
            file_value = lowered.get("file")
            if file_value is not None:
                self.references.append({"kind": "xml_include" if tag == "include" else "xml_script", "declared": file_value, "line": line})
            elif tag == "script":
                self.inline_scripts += 1
        declared_name = lowered.get("name")
        inherits = [item.strip() for item in lowered.get("inherits", "").split(",") if item.strip()]
        virtual = lowered.get("virtual", "").casefold() in {"true", "1"}
        if declared_name or inherits or virtual:
            self.templates.append({
                "element": local_name,
                "name": declared_name,
                "inherits": inherits,
                "virtual": virtual,
                "line": line,
            })

    def _end(self, name: str) -> None:
        del name
        self.depth -= 1

    def _entity(self, *arguments: Any) -> None:
        del arguments
        raise TopologyError("xml_entity_forbidden", "XML entity declarations are not accepted", path=self.path)

    def _external(self, *arguments: Any) -> int:
        del arguments
        raise TopologyError("xml_external_forbidden", "XML external entities are not accepted", path=self.path)


def _parse_xml(
    path: str,
    data: bytes,
    git_object: str,
    inventory: Mapping[str, Any],
    lower_inventory: Mapping[str, list[str]],
    max_elements: int,
    max_depth: int,
) -> tuple[dict[str, Any], list[dict[str, Any]], list[dict[str, Any]]]:
    text = _decode_text(data, path)
    parsed = XmlTopologyParser(path, max_elements, max_depth).parse(text)
    references: list[dict[str, Any]] = []
    edges: list[dict[str, Any]] = []
    issues: list[dict[str, Any]] = []
    for reference in parsed["references"]:
        target, issue = _classify_target(path, reference["declared"], inventory, lower_inventory)
        normalized = {**reference, **target}
        references.append(normalized)
        edges.append({
            "kind": reference["kind"],
            "source": path,
            "target": target.get("target"),
            "declared": reference["declared"],
            "resolution": target["resolution"],
            "line": reference["line"],
        })
        if issue is not None:
            issue["line"] = reference["line"]
            issues.append(issue)
    document = {
        "path": path,
        "root": parsed["root"],
        "element_count": parsed["element_count"],
        "inline_scripts": parsed["inline_scripts"],
        "references": references,
        "templates": parsed["templates"],
        "source": _source_record(path, data, git_object),
    }
    return document, edges, issues


def _canonical_cycle(nodes: Sequence[str]) -> tuple[str, ...]:
    if not nodes:
        return ()
    rotations = [tuple(nodes[index:] + nodes[:index]) for index in range(len(nodes))]
    return min(rotations, key=lambda cycle: tuple(item.encode("utf-8") for item in cycle))


def _include_cycles(edges: Sequence[Mapping[str, Any]], xml_paths: set[str]) -> list[list[str]]:
    adjacency: dict[str, list[str]] = {path: [] for path in xml_paths}
    for edge in edges:
        if (
            edge.get("kind") == "xml_include"
            and edge.get("resolution") == "exact"
            and isinstance(edge.get("source"), str)
            and isinstance(edge.get("target"), str)
            and edge["source"] in xml_paths
            and edge["target"] in xml_paths
        ):
            adjacency[edge["source"]].append(edge["target"])
    for values in adjacency.values():
        values.sort(key=str.encode)

    state: dict[str, int] = {}
    active_index: dict[str, int] = {}
    cycles: set[tuple[str, ...]] = set()
    for start in sorted(adjacency, key=str.encode):
        if state.get(start, 0) != 0:
            continue
        path: list[str] = [start]
        active_index[start] = 0
        state[start] = 1
        stack: list[tuple[str, int]] = [(start, 0)]
        while stack:
            node, offset = stack[-1]
            neighbors = adjacency[node]
            if offset >= len(neighbors):
                stack.pop()
                state[node] = 2
                active_index.pop(node, None)
                path.pop()
                continue
            neighbor = neighbors[offset]
            stack[-1] = (node, offset + 1)
            neighbor_state = state.get(neighbor, 0)
            if neighbor_state == 0:
                state[neighbor] = 1
                active_index[neighbor] = len(path)
                path.append(neighbor)
                stack.append((neighbor, 0))
            elif neighbor_state == 1:
                cycles.add(_canonical_cycle(path[active_index[neighbor] :]))
    return [list(cycle) for cycle in sorted(cycles, key=lambda cycle: tuple(item.encode() for item in cycle))]


def _record_key(record: Mapping[str, Any]) -> tuple[Any, ...]:
    return (
        str(record.get("code", "")),
        str(record.get("source_path", record.get("path", ""))),
        int(record.get("line", 0) or 0),
        str(record.get("declared", "")),
        str(record.get("target", "")),
        str(record.get("key", "")),
    )


def _edge_key(edge: Mapping[str, Any]) -> tuple[Any, ...]:
    return (
        str(edge.get("source", "")).encode(),
        int(edge.get("line", 0) or 0),
        str(edge.get("kind", "")),
        str(edge.get("declared", "")).encode(),
        str(edge.get("target", "")).encode(),
    )


def build_topology(
    *,
    source: Path,
    manifest: Mapping[str, Any],
    manifest_bytes: bytes,
    allow_partial: bool = False,
    max_file_bytes: int = DEFAULT_MAX_FILE_BYTES,
    max_total_bytes: int = DEFAULT_MAX_TOTAL_BYTES,
    max_xml_elements: int = DEFAULT_MAX_XML_ELEMENTS,
    max_xml_depth: int = DEFAULT_MAX_XML_DEPTH,
) -> dict[str, Any]:
    revision = _manifest_revision(manifest)
    _validate_revision(source, revision)
    inventory, lower_inventory = _inventory(manifest)
    toc_paths, xml_paths = _candidate_paths(inventory)
    candidates = sorted(toc_paths + xml_paths, key=str.encode)
    descriptors: list[dict[str, Any]] = []
    xml_documents: list[dict[str, Any]] = []
    edges: list[dict[str, Any]] = []
    issues: list[dict[str, Any]] = []
    failures: list[dict[str, Any]] = []
    parsed_toc = 0
    parsed_xml = 0
    total_bytes = 0

    with GitBatchReader(source, revision) as reader:
        for path in candidates:
            record = inventory[path]
            try:
                data, git_object = reader.read(path, max_file_bytes)
                total_bytes += len(data)
                if total_bytes > max_total_bytes:
                    raise TopologyError("source_total_limit", "topology source exceeds total size limit", path=path)
                if sha256_id(data) != record["sha256"]:
                    raise TopologyError("source_digest_mismatch", "source bytes do not match manifest SHA-256", path=path)
                if record["git_object"] is not None and git_object != record["git_object"]:
                    raise TopologyError("source_git_object_mismatch", "source Git object does not match manifest", path=path)
                if record["size"] is not None and len(data) != record["size"]:
                    raise TopologyError("source_size_mismatch", "source size does not match manifest", path=path)
                if path in toc_paths:
                    descriptor, local_edges, local_issues = _parse_toc(path, data, git_object, inventory, lower_inventory)
                    descriptors.append(descriptor)
                    parsed_toc += 1
                else:
                    document, local_edges, local_issues = _parse_xml(
                        path,
                        data,
                        git_object,
                        inventory,
                        lower_inventory,
                        max_xml_elements,
                        max_xml_depth,
                    )
                    xml_documents.append(document)
                    parsed_xml += 1
                edges.extend(local_edges)
                issues.extend(local_issues)
            except TopologyError as error:
                if error.path is None:
                    error.path = path
                failures.append(error.record())

    descriptors.sort(key=lambda item: item["path"].encode())
    xml_documents.sort(key=lambda item: item["path"].encode())
    edges.sort(key=_edge_key)
    issues.sort(key=_record_key)
    failures.sort(key=_record_key)
    cycles = _include_cycles(edges, set(xml_paths))
    unresolved = sum(1 for issue in issues if issue["code"] in _REFERENCE_ISSUES)
    all_parsed = parsed_toc == len(toc_paths) and parsed_xml == len(xml_paths) and not failures
    complete = all_parsed and unresolved == 0
    if not all_parsed and not allow_partial:
        first = failures[0] if failures else {"message": "TOC/XML coverage is incomplete"}
        raise TopologyError("topology_incomplete", str(first["message"]), path=first.get("path"))

    metadata = _manifest_metadata(manifest)
    draft: dict[str, Any] = {
        "schema": SCHEMA,
        "schema_version": SCHEMA_VERSION,
        "producer": {
            "id": PRODUCER_ID,
            "version": PRODUCER_VERSION,
            "configuration": {
                "scope": "Interface/**/*.toc+xml",
                "xml_parser": "python-expat-nonexecuting-v1",
                "toc_parser": "line-oriented-v1",
            },
        },
        "source": {
            "manifest_sha256": sha256_id(manifest_bytes),
            "manifest_declared_digest": metadata["declared_digest"],
            "source_id": metadata["source_id"],
            "selector": metadata["selector"],
            "revision": revision,
            "version": metadata["version"],
        },
        "coverage": {
            "status": "complete" if complete else "partial",
            "negative_authority": complete,
            "candidate_toc_files": len(toc_paths),
            "parsed_toc_files": parsed_toc,
            "candidate_xml_files": len(xml_paths),
            "parsed_xml_files": parsed_xml,
            "failed_files": len(failures),
            "unresolved_references": unresolved,
            "failures": failures,
            "limitations": [
                "TOC/XML topology does not prove Lua semantics or runtime behavior",
                "template inheritance records names but does not infer runtime object ancestry",
                "implementation and runtime evidence remain separate lanes",
            ],
        },
        "descriptors": descriptors,
        "xml_documents": xml_documents,
        "edges": edges,
        "issues": issues,
        "include_cycles": cycles,
    }
    draft["topology_sha256"] = sha256_id(canonical_json_bytes(draft))
    return draft


def verify_topology(draft: Mapping[str, Any], *, require_complete: bool = False) -> None:
    if draft.get("schema") != SCHEMA or draft.get("schema_version") != SCHEMA_VERSION:
        raise TopologyError("draft_schema", "unsupported topology draft schema")
    digest = draft.get("topology_sha256")
    if not isinstance(digest, str):
        raise TopologyError("draft_digest", "topology draft has no digest")
    projection = dict(draft)
    projection.pop("topology_sha256", None)
    if digest != sha256_id(canonical_json_bytes(projection)):
        raise TopologyError("draft_digest", "topology draft digest does not match content")
    producer = draft.get("producer")
    if not isinstance(producer, Mapping) or producer.get("id") != PRODUCER_ID:
        raise TopologyError("draft_producer", "unexpected topology producer")
    source = draft.get("source")
    if not isinstance(source, Mapping):
        raise TopologyError("draft_source", "topology draft has no source record")
    revision = source.get("revision")
    if not isinstance(revision, str) or not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", revision):
        raise TopologyError("draft_revision", "topology source revision is not exact")
    coverage = draft.get("coverage")
    if not isinstance(coverage, Mapping):
        raise TopologyError("draft_coverage", "topology draft has no coverage")
    integers = [
        coverage.get("candidate_toc_files"),
        coverage.get("parsed_toc_files"),
        coverage.get("candidate_xml_files"),
        coverage.get("parsed_xml_files"),
        coverage.get("failed_files"),
        coverage.get("unresolved_references"),
    ]
    if not all(isinstance(value, int) and not isinstance(value, bool) and value >= 0 for value in integers):
        raise TopologyError("draft_coverage", "topology coverage counts are invalid")
    failures = coverage.get("failures")
    if not isinstance(failures, list) or len(failures) != coverage["failed_files"]:
        raise TopologyError("draft_coverage", "topology failure count is inconsistent")
    issues = draft.get("issues")
    if not isinstance(issues, list):
        raise TopologyError("draft_issues", "topology issues must be an array")
    unresolved = sum(1 for issue in issues if isinstance(issue, Mapping) and issue.get("code") in _REFERENCE_ISSUES)
    complete = (
        coverage["candidate_toc_files"] == coverage["parsed_toc_files"]
        and coverage["candidate_xml_files"] == coverage["parsed_xml_files"]
        and coverage["failed_files"] == 0
        and unresolved == 0
    )
    if coverage.get("unresolved_references") != unresolved:
        raise TopologyError("draft_coverage", "topology unresolved reference count is inconsistent")
    if coverage.get("status") != ("complete" if complete else "partial") or coverage.get("negative_authority") is not complete:
        raise TopologyError("draft_coverage", "topology authority does not match coverage")
    if require_complete and not complete:
        raise TopologyError("draft_incomplete", "complete TOC/XML topology is required")
    descriptors = draft.get("descriptors")
    documents = draft.get("xml_documents")
    edges = draft.get("edges")
    cycles = draft.get("include_cycles")
    if not all(isinstance(value, list) for value in (descriptors, documents, edges, cycles)):
        raise TopologyError("draft_shape", "topology collections must be arrays")
    if descriptors != sorted(descriptors, key=lambda item: item["path"].encode()):
        raise TopologyError("draft_order", "TOC descriptors are not canonically ordered")
    if documents != sorted(documents, key=lambda item: item["path"].encode()):
        raise TopologyError("draft_order", "XML documents are not canonically ordered")
    if edges != sorted(edges, key=_edge_key):
        raise TopologyError("draft_order", "topology edges are not canonically ordered")
    if issues != sorted(issues, key=_record_key) or failures != sorted(failures, key=_record_key):
        raise TopologyError("draft_order", "topology issues or failures are not canonically ordered")


def write_json_atomic(path: Path, value: Mapping[str, Any]) -> None:
    data = json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2, allow_nan=False).encode("utf-8") + b"\n"
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", suffix=".tmp", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(data)
            handle.flush()
            os.fsync(handle.fileno())
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


def _summary(draft: Mapping[str, Any], *, stale: bool = False) -> dict[str, Any]:
    coverage = draft["coverage"]
    return {
        "status": "stale" if stale else "ok",
        "revision": draft["source"]["revision"],
        "version": draft["source"].get("version"),
        "coverage": coverage["status"],
        "toc": [coverage["parsed_toc_files"], coverage["candidate_toc_files"]],
        "xml": [coverage["parsed_xml_files"], coverage["candidate_xml_files"]],
        "edges": len(draft["edges"]),
        "issues": len(draft["issues"]),
        "cycles": len(draft["include_cycles"]),
        "topology_sha256": draft["topology_sha256"],
    }


def build_cli(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build an exact Blizzard TOC/XML topology draft")
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--allow-partial", action="store_true")
    parser.add_argument("--max-file-bytes", type=int, default=DEFAULT_MAX_FILE_BYTES)
    parser.add_argument("--max-total-bytes", type=int, default=DEFAULT_MAX_TOTAL_BYTES)
    parser.add_argument("--max-xml-elements", type=int, default=DEFAULT_MAX_XML_ELEMENTS)
    parser.add_argument("--max-xml-depth", type=int, default=DEFAULT_MAX_XML_DEPTH)
    parser.add_argument("--json", action="store_true", dest="json_output")
    arguments = parser.parse_args(argv)
    try:
        manifest, manifest_bytes = load_json(arguments.manifest)
        draft = build_topology(
            source=arguments.source,
            manifest=manifest,
            manifest_bytes=manifest_bytes,
            allow_partial=arguments.allow_partial,
            max_file_bytes=arguments.max_file_bytes,
            max_total_bytes=arguments.max_total_bytes,
            max_xml_elements=arguments.max_xml_elements,
            max_xml_depth=arguments.max_xml_depth,
        )
        verify_topology(draft)
        write_json_atomic(arguments.output, draft)
    except TopologyError as error:
        output = {"status": "error", "error": error.record()}
        print(json.dumps(output, ensure_ascii=False) if arguments.json_output else f"{error.code}: {error.message}", file=sys.stderr)
        return 2
    summary = _summary(draft)
    print(json.dumps(summary, ensure_ascii=False, sort_keys=True) if arguments.json_output else f"built {summary['edges']} edges: {summary['topology_sha256']}")
    return 0


def verify_cli(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Verify an exact Blizzard TOC/XML topology draft")
    parser.add_argument("draft", type=Path)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--source", type=Path)
    parser.add_argument("--current-ref")
    parser.add_argument("--require-complete", action="store_true")
    parser.add_argument("--json", action="store_true", dest="json_output")
    arguments = parser.parse_args(argv)
    if (arguments.manifest is None) != (arguments.source is None):
        parser.error("--manifest and --source must be supplied together")
    try:
        draft, _ = load_json(arguments.draft)
        verify_topology(draft, require_complete=arguments.require_complete)
        if arguments.manifest is not None and arguments.source is not None:
            manifest, manifest_bytes = load_json(arguments.manifest)
            rebuilt = build_topology(
                source=arguments.source,
                manifest=manifest,
                manifest_bytes=manifest_bytes,
                allow_partial=draft["coverage"]["status"] == "partial",
            )
            if canonical_json_bytes(rebuilt) != canonical_json_bytes(draft):
                raise TopologyError("draft_rebuild_mismatch", "topology draft does not match exact source snapshot")
        stale = False
        if arguments.current_ref is not None:
            if arguments.source is None:
                parser.error("--current-ref requires --source and --manifest")
            current = _run_git(arguments.source, ["rev-parse", "--verify", f"{arguments.current_ref}^{{commit}}"])
            stale = current.decode("ascii").strip().lower() != draft["source"]["revision"]
    except TopologyError as error:
        output = {"status": "error", "error": error.record()}
        print(json.dumps(output, ensure_ascii=False) if arguments.json_output else f"{error.code}: {error.message}", file=sys.stderr)
        return 2
    summary = _summary(draft, stale=stale)
    print(json.dumps(summary, ensure_ascii=False, sort_keys=True) if arguments.json_output else summary["status"])
    return 3 if stale else 0


__all__ = [
    "TopologyError",
    "build_topology",
    "verify_topology",
    "build_cli",
    "verify_cli",
    "canonical_json_bytes",
    "sha256_id",
]
