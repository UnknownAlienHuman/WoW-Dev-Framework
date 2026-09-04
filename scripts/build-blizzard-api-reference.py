#!/usr/bin/env python3
"""Build a deterministic ReferenceDraft from Blizzard generated API documentation.

The producer never executes Lua. It parses data-only documentation tables from
one exact source revision named by a source manifest. A local Git object database
is preferred; exact-revision GitHub raw reads are an explicit fallback.
"""
from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any, Iterable, Iterator, Sequence

SCHEMA = "wow.reference-draft.blizzard-generated-api/v1"
PRODUCER_ID = "wow-reference.blizzard-generated-api"
PRODUCER_VERSION = "1"
GENERATED_ROOT = "Interface/AddOns/Blizzard_APIDocumentationGenerated/"


class DraftError(RuntimeError):
    """A bounded producer failure."""


class LuaParseError(DraftError):
    """Generated documentation stopped matching the data-table grammar."""


@dataclasses.dataclass(frozen=True)
class Token:
    kind: str
    value: str
    offset: int


@dataclasses.dataclass(frozen=True)
class Symbol:
    value: str


@dataclasses.dataclass(frozen=True)
class ParsedTable:
    fields: dict[str, Any]
    items: list[Any]


_IDENTIFIER = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
_NUMBER = re.compile(r"(?:0[xX][0-9A-Fa-f]+|(?:\d+\.\d*|\.\d+|\d+)(?:[eE][+-]?\d+)?)")
_LONG_BRACKET = re.compile(r"\[(=*)\[")


def _decode_short_string(source: str, start: int) -> tuple[str, int]:
    quote = source[start]
    cursor = start + 1
    output: list[str] = []
    escapes = {"a": "\a", "b": "\b", "f": "\f", "n": "\n", "r": "\r", "t": "\t", "v": "\v", "\\": "\\", '"': '"', "'": "'"}
    while cursor < len(source):
        char = source[cursor]
        if char == quote:
            return "".join(output), cursor + 1
        if char != "\\":
            output.append(char)
            cursor += 1
            continue
        cursor += 1
        if cursor >= len(source):
            raise LuaParseError(f"unterminated escape at byte {start}")
        escaped = source[cursor]
        if escaped in escapes:
            output.append(escapes[escaped])
            cursor += 1
        elif escaped == "z":
            cursor += 1
            while cursor < len(source) and source[cursor].isspace():
                cursor += 1
        elif escaped in "\r\n":
            if escaped == "\r" and cursor + 1 < len(source) and source[cursor + 1] == "\n":
                cursor += 1
            output.append("\n")
            cursor += 1
        elif escaped == "x":
            digits = source[cursor + 1 : cursor + 3]
            if len(digits) != 2 or not all(c in "0123456789abcdefABCDEF" for c in digits):
                raise LuaParseError(f"invalid hexadecimal escape at byte {cursor}")
            output.append(chr(int(digits, 16)))
            cursor += 3
        elif escaped == "u" and cursor + 1 < len(source) and source[cursor + 1] == "{":
            end = source.find("}", cursor + 2)
            if end < 0:
                raise LuaParseError(f"unterminated Unicode escape at byte {cursor}")
            digits = source[cursor + 2 : end]
            if not digits or not all(c in "0123456789abcdefABCDEF" for c in digits):
                raise LuaParseError(f"invalid Unicode escape at byte {cursor}")
            output.append(chr(int(digits, 16)))
            cursor = end + 1
        elif escaped.isdigit():
            end = cursor
            while end < min(cursor + 3, len(source)) and source[end].isdigit():
                end += 1
            value = int(source[cursor:end], 10)
            if value > 255:
                raise LuaParseError(f"decimal escape out of range at byte {cursor}")
            output.append(chr(value))
            cursor = end
        else:
            output.append(escaped)
            cursor += 1
    raise LuaParseError(f"unterminated string at byte {start}")


def _read_long_bracket(source: str, start: int) -> tuple[str, int] | None:
    match = _LONG_BRACKET.match(source, start)
    if match is None:
        return None
    equals = match.group(1)
    close = "]" + equals + "]"
    body_start = match.end()
    body_end = source.find(close, body_start)
    if body_end < 0:
        raise LuaParseError(f"unterminated long bracket at byte {start}")
    body = source[body_start:body_end]
    if body.startswith("\r\n"):
        body = body[2:]
    elif body.startswith(("\r", "\n")):
        body = body[1:]
    return body, body_end + len(close)


def tokenize(source: str, start: int = 0) -> Iterator[Token]:
    cursor = start
    punctuation = "{}[]=,;().:-"
    while cursor < len(source):
        char = source[cursor]
        if char.isspace():
            cursor += 1
            continue
        if source.startswith("--", cursor):
            long_comment = _read_long_bracket(source, cursor + 2)
            if long_comment is not None:
                _, cursor = long_comment
            else:
                line_end = source.find("\n", cursor + 2)
                cursor = len(source) if line_end < 0 else line_end + 1
            continue
        if char in "'\"":
            value, end = _decode_short_string(source, cursor)
            yield Token("string", value, cursor)
            cursor = end
            continue
        long_string = _read_long_bracket(source, cursor)
        if long_string is not None:
            value, end = long_string
            yield Token("string", value, cursor)
            cursor = end
            continue
        identifier = _IDENTIFIER.match(source, cursor)
        if identifier is not None:
            yield Token("identifier", identifier.group(0), cursor)
            cursor = identifier.end()
            continue
        number = _NUMBER.match(source, cursor)
        if number is not None:
            yield Token("number", number.group(0), cursor)
            cursor = number.end()
            continue
        if char in punctuation:
            yield Token(char, char, cursor)
            cursor += 1
            continue
        raise LuaParseError(f"unsupported token {char!r} at byte {cursor}")
    yield Token("eof", "", len(source))


class LuaTableParser:
    def __init__(self, tokens: Iterable[Token]) -> None:
        self._tokens = iter(tokens)
        self.current = next(self._tokens)

    def _advance(self) -> Token:
        previous = self.current
        self.current = next(self._tokens)
        return previous

    def _accept(self, kind: str) -> Token | None:
        if self.current.kind == kind:
            return self._advance()
        return None

    def _expect(self, kind: str) -> Token:
        if self.current.kind != kind:
            raise LuaParseError(f"expected {kind!r} at byte {self.current.offset}, got {self.current.kind!r}")
        return self._advance()

    def parse_value(self) -> Any:
        token = self.current
        if token.kind == "string":
            return self._advance().value
        if token.kind == "number":
            raw = self._advance().value
            if raw.lower().startswith("0x"):
                return int(raw, 16)
            if any(marker in raw for marker in ".eE"):
                return float(raw)
            return int(raw, 10)
        if token.kind == "-":
            self._advance()
            value = self.parse_value()
            if not isinstance(value, (int, float)) or isinstance(value, bool):
                raise LuaParseError(f"unary minus requires a number at byte {token.offset}")
            return -value
        if token.kind == "{":
            return self.parse_table()
        if token.kind == "identifier":
            identifier = self._advance().value
            if identifier == "true":
                return True
            if identifier == "false":
                return False
            if identifier == "nil":
                return None
            parts = [identifier]
            while self._accept(".") is not None:
                parts.append(self._expect("identifier").value)
            return Symbol(".".join(parts))
        raise LuaParseError(f"unsupported value token {token.kind!r} at byte {token.offset}")

    def parse_table(self) -> ParsedTable:
        self._expect("{")
        fields: dict[str, Any] = {}
        items: list[Any] = []
        while self.current.kind != "}":
            if self.current.kind == "eof":
                raise LuaParseError("unterminated table")
            if self.current.kind == "identifier":
                first = self.current
                self._advance()
                if self._accept("=") is not None:
                    if first.value in fields:
                        raise LuaParseError(f"duplicate table field {first.value!r}")
                    fields[first.value] = self.parse_value()
                else:
                    if first.value == "true":
                        items.append(True)
                    elif first.value == "false":
                        items.append(False)
                    elif first.value == "nil":
                        items.append(None)
                    else:
                        parts = [first.value]
                        while self._accept(".") is not None:
                            parts.append(self._expect("identifier").value)
                        items.append(Symbol(".".join(parts)))
            elif self._accept("[") is not None:
                key = self.parse_value()
                self._expect("]")
                self._expect("=")
                if not isinstance(key, (str, int)) or isinstance(key, bool):
                    raise LuaParseError("only string/integer table keys are supported")
                canonical_key = str(key)
                if canonical_key in fields:
                    raise LuaParseError(f"duplicate table field {canonical_key!r}")
                fields[canonical_key] = self.parse_value()
            else:
                items.append(self.parse_value())
            if self._accept(",") is None and self._accept(";") is None and self.current.kind != "}":
                raise LuaParseError(f"expected table separator at byte {self.current.offset}")
        self._expect("}")
        return ParsedTable(fields=fields, items=items)


def _plain(value: Any) -> Any:
    if isinstance(value, Symbol):
        return {"symbol": value.value}
    if isinstance(value, ParsedTable):
        if value.fields and value.items:
            return {"fields": {key: _plain(value.fields[key]) for key in sorted(value.fields)}, "items": [_plain(item) for item in value.items]}
        if value.fields:
            return {key: _plain(value.fields[key]) for key in sorted(value.fields)}
        return [_plain(item) for item in value.items]
    if isinstance(value, list):
        return [_plain(item) for item in value]
    if isinstance(value, dict):
        return {key: _plain(value[key]) for key in sorted(value)}
    return value


def _parse_document_table(source: str) -> tuple[str, dict[str, Any]]:
    calls = list(re.finditer(r"APIDocumentation\s*:\s*AddDocumentationTable\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)", source))
    if len(calls) != 1:
        raise LuaParseError(f"expected exactly one AddDocumentationTable call, found {len(calls)}")
    variable = calls[0].group(1)
    assignment = re.search(rf"\blocal\s+{re.escape(variable)}\s*=", source)
    if assignment is None:
        raise LuaParseError(f"missing local assignment for {variable}")
    parser = LuaTableParser(tokenize(source, assignment.end()))
    parsed = parser.parse_value()
    if not isinstance(parsed, ParsedTable):
        raise LuaParseError("documentation root must be a table")
    plain = _plain(parsed)
    if not isinstance(plain, dict):
        raise LuaParseError("documentation root must be a keyed table")
    return variable, plain


def _canonical_bytes(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":"), allow_nan=False).encode("utf-8")


def _digest(value: Any) -> str:
    return "sha256:" + hashlib.sha256(_canonical_bytes(value)).hexdigest()


def _first(mapping: dict[str, Any], *keys: str) -> Any:
    for key in keys:
        value = mapping.get(key)
        if value not in (None, ""):
            return value
    return None


def _source_field(manifest: dict[str, Any], *keys: str) -> Any:
    source = manifest.get("source")
    if isinstance(source, dict):
        value = _first(source, *keys)
        if value is not None:
            return value
    return _first(manifest, *keys)


def _manifest_revision(manifest: dict[str, Any]) -> str:
    value = _source_field(manifest, "resolved_commit", "resolved_revision", "revision", "source_revision", "commit")
    if not isinstance(value, str) or not re.fullmatch(r"[0-9a-fA-F]{40,64}", value):
        raise DraftError("source manifest does not contain a full Git revision")
    return value.lower()


def _manifest_files(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    records = manifest.get("files")
    if not isinstance(records, list):
        selection = manifest.get("selection")
        if isinstance(selection, dict):
            records = selection.get("files")
    if not isinstance(records, list):
        raise DraftError("source manifest does not contain a files array")
    result: list[dict[str, Any]] = []
    for record in records:
        if not isinstance(record, dict) or not isinstance(record.get("path"), str):
            raise DraftError("source manifest contains an invalid file record")
        result.append(record)
    return result


def _record_sha256(record: dict[str, Any]) -> str | None:
    value = _first(record, "sha256", "content_sha256", "content_digest", "digest")
    if not isinstance(value, str):
        return None
    lowered = value.lower().removeprefix("sha256:")
    if not re.fullmatch(r"[0-9a-f]{64}", lowered):
        raise DraftError(f"invalid SHA-256 for {record.get('path')}")
    return lowered


def _manifest_digest(manifest: dict[str, Any]) -> str:
    supplied = _first(manifest, "manifest_digest", "digest")
    if isinstance(supplied, str) and supplied.startswith("sha256:"):
        return supplied.lower()
    projection = dict(manifest)
    projection.pop("manifest_digest", None)
    projection.pop("digest", None)
    return _digest(projection)


class SourceReader:
    def __init__(self, revision: str, source: Path | None, github_repository: str | None, token: str | None) -> None:
        self.revision = revision
        self.source = source
        self.github_repository = github_repository
        self.token = token
        self.transport = self._select_transport()

    def _select_transport(self) -> str:
        if self.source is not None:
            completed = subprocess.run(["git", "-C", str(self.source), "cat-file", "-e", f"{self.revision}^{{commit}}"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=False)
            if completed.returncode == 0:
                return "local-git"
        if self.github_repository is not None:
            if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", self.github_repository):
                raise DraftError("invalid GitHub repository name")
            return "github-raw"
        raise DraftError("exact revision is unavailable locally and no GitHub fallback was configured")

    def read(self, path: str) -> bytes:
        if self.transport == "local-git":
            completed = subprocess.run(["git", "-C", str(self.source), "show", f"{self.revision}:{path}"], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
            if completed.returncode != 0:
                raise DraftError(f"cannot read {path!r} from exact local revision")
            return completed.stdout
        quoted = "/".join(urllib.parse.quote(part, safe="") for part in path.split("/"))
        url = f"https://raw.githubusercontent.com/{self.github_repository}/{self.revision}/{quoted}"
        request = urllib.request.Request(url, headers={"User-Agent": "wow-dev-framework"})
        if self.token:
            request.add_header("Authorization", f"Bearer {self.token}")
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                return response.read()
        except (urllib.error.HTTPError, urllib.error.URLError, TimeoutError) as error:
            raise DraftError(f"cannot read {path!r} from exact GitHub revision") from error


def _as_list(value: Any, field: str) -> list[Any]:
    if value is None:
        return []
    if not isinstance(value, list):
        raise LuaParseError(f"{field} must be an array table")
    return value


def _string(value: Any, field: str, *, optional: bool = False) -> str | None:
    if value is None and optional:
        return None
    if not isinstance(value, str) or not value:
        raise LuaParseError(f"{field} must be a non-empty string")
    return value


def _documentation(value: Any) -> list[str]:
    if value is None:
        return []
    if isinstance(value, str):
        return [value]
    if isinstance(value, list) and all(isinstance(item, str) for item in value):
        return value
    raise LuaParseError("Documentation must be a string array")


def _attributes(record: dict[str, Any], consumed: set[str]) -> dict[str, Any]:
    return {key: record[key] for key in sorted(record) if key not in consumed}


def _parameter(record: Any, position: int) -> dict[str, Any]:
    if not isinstance(record, dict):
        raise LuaParseError("parameter entry must be a keyed table")
    consumed = {"Name", "Type", "Nilable", "Documentation", "Default"}
    result: dict[str, Any] = {"position": position, "name": _string(record.get("Name"), "parameter.Name"), "type": _string(record.get("Type"), "parameter.Type"), "nilable": bool(record.get("Nilable", False)), "documentation": _documentation(record.get("Documentation"))}
    if "Default" in record:
        result["default"] = record["Default"]
    attributes = _attributes(record, consumed)
    if attributes:
        result["attributes"] = attributes
    return result


def _restriction_attributes(record: dict[str, Any]) -> dict[str, Any]:
    prefixes = ("HasRestriction", "HasRestrictions", "Requires", "Secret", "Allowed", "Restricted", "MayReturn", "NeverSecret")
    return {key: record[key] for key in sorted(record) if key.startswith(prefixes)}


def _callable(record: Any, namespace: str | None, kind: str, source: dict[str, Any]) -> dict[str, Any]:
    if not isinstance(record, dict):
        raise LuaParseError(f"{kind} entry must be a keyed table")
    name = _string(record.get("Name"), f"{kind}.Name")
    consumed = {"Name", "Type", "Arguments", "Returns", "Payload", "Documentation", "Event"}
    restrictions = _restriction_attributes(record)
    consumed.update(restrictions)
    qualified_name = f"{namespace}.{name}" if namespace else name
    result: dict[str, Any] = {"kind": kind, "name": name, "qualified_name": qualified_name, "documentation": _documentation(record.get("Documentation")), "source": source}
    if "Event" in record:
        result["event"] = _string(record.get("Event"), f"{kind}.Event")
    for input_key, output_key in (("Arguments", "arguments"), ("Returns", "returns"), ("Payload", "payload")):
        value = record.get(input_key)
        if value is not None:
            result[output_key] = [_parameter(item, position) for position, item in enumerate(_as_list(value, input_key), 1)]
    if restrictions:
        result["restrictions"] = restrictions
    attributes = _attributes(record, consumed)
    if attributes:
        result["attributes"] = attributes
    return result


def _table_record(record: Any, source: dict[str, Any]) -> dict[str, Any]:
    if not isinstance(record, dict):
        raise LuaParseError("table entry must be a keyed table")
    name = _string(record.get("Name"), "table.Name")
    table_type = _string(record.get("Type"), "table.Type", optional=True) or "Unknown"
    consumed = {"Name", "Type", "Fields", "Documentation"}
    result: dict[str, Any] = {"kind": table_type.lower(), "name": name, "documentation": _documentation(record.get("Documentation")), "source": source}
    fields = record.get("Fields")
    if fields is not None:
        result["fields"] = [_parameter(item, position) for position, item in enumerate(_as_list(fields, "Fields"), 1)]
    attributes = _attributes(record, consumed)
    if attributes:
        result["attributes"] = attributes
    return result


def _normalize_system(root: dict[str, Any], source: dict[str, Any]) -> dict[str, Any]:
    name = _string(root.get("Name"), "system.Name")
    namespace = _string(root.get("Namespace"), "system.Namespace", optional=True)
    consumed = {"Name", "Type", "Namespace", "Environment", "Documentation", "Functions", "Events", "Tables"}
    system: dict[str, Any] = {
        "name": name,
        "namespace": namespace,
        "environment": _string(root.get("Environment"), "system.Environment", optional=True),
        "documentation": _documentation(root.get("Documentation")),
        "source": source,
        "functions": [_callable(item, namespace, "function", source) for item in _as_list(root.get("Functions"), "Functions")],
        "events": [_callable(item, namespace, "event", source) for item in _as_list(root.get("Events"), "Events")],
        "tables": [_table_record(item, source) for item in _as_list(root.get("Tables"), "Tables")],
    }
    extras = _attributes(root, consumed)
    if extras:
        system["attributes"] = extras
    return system


def _sort_system(system: dict[str, Any]) -> tuple[str, str]:
    return (str(system.get("namespace") or ""), str(system["name"]))


def _sort_named(record: dict[str, Any]) -> tuple[str, str]:
    return (str(record.get("qualified_name") or ""), str(record.get("name") or ""))


def _sorted_system(system: dict[str, Any]) -> dict[str, Any]:
    result = dict(system)
    for key in ("functions", "events", "tables"):
        result[key] = sorted(result[key], key=_sort_named)
    return result


def build_draft(manifest_path: Path, source_path: Path | None, github_repository: str | None, token: str | None, allow_partial: bool) -> dict[str, Any]:
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise DraftError("cannot read source manifest") from error
    if not isinstance(manifest, dict):
        raise DraftError("source manifest root must be an object")
    revision = _manifest_revision(manifest)
    files = _manifest_files(manifest)
    candidates = [record for record in files if record["path"].startswith(GENERATED_ROOT) and record["path"].endswith("Documentation.lua")]
    candidates.sort(key=lambda record: record["path"].encode("utf-8"))
    if not candidates:
        raise DraftError("source manifest contains no generated API documentation files")
    reader = SourceReader(revision, source_path, github_repository, token)
    systems: list[dict[str, Any]] = []
    failures: list[dict[str, str]] = []
    file_coverage: list[dict[str, Any]] = []
    for record in candidates:
        path = record["path"]
        expected_sha = _record_sha256(record)
        try:
            payload = reader.read(path)
            actual_sha = hashlib.sha256(payload).hexdigest()
            if expected_sha is not None and actual_sha != expected_sha:
                raise DraftError(f"source bytes do not match manifest SHA-256 for {path}")
            try:
                text = payload.decode("utf-8")
            except UnicodeDecodeError as error:
                raise LuaParseError("generated documentation is not UTF-8") from error
            variable, root = _parse_document_table(text)
            source = {"path": path, "sha256": "sha256:" + actual_sha, "document_variable": variable}
            systems.append(_sorted_system(_normalize_system(root, source)))
            file_coverage.append({"path": path, "status": "parsed"})
        except DraftError as error:
            failures.append({"path": path, "error": str(error)})
            file_coverage.append({"path": path, "status": "failed"})
    if failures and not allow_partial:
        examples = "; ".join(f"{failure['path']}: {failure['error']}" for failure in failures[:5])
        raise DraftError(f"generated API coverage is incomplete ({len(failures)} failures): {examples}")
    systems.sort(key=_sort_system)
    counts = {
        "systems": len(systems),
        "functions": sum(len(system["functions"]) for system in systems),
        "events": sum(len(system["events"]) for system in systems),
        "tables": sum(len(system["tables"]) for system in systems),
        "fields": sum(len(table.get("fields", [])) for system in systems for table in system["tables"]),
    }
    source_identity = {
        "manifest_digest": _manifest_digest(manifest),
        "source_id": _source_field(manifest, "source_id", "id") or "blizzard-ui-source",
        "selector": _source_field(manifest, "selector", "source_selector"),
        "revision": revision,
        "version": _source_field(manifest, "version", "reported_version", "source_version"),
    }
    draft: dict[str, Any] = {
        "schema": SCHEMA,
        "producer": {"id": PRODUCER_ID, "version": PRODUCER_VERSION, "configuration": {"generated_root": GENERATED_ROOT, "parser_schema": 1}},
        "source": source_identity,
        "coverage": {"status": "complete" if not failures else "partial", "candidate_files": len(candidates), "parsed_files": len(candidates) - len(failures), "failed_files": failures, "files": file_coverage, "record_counts": counts, "negative_authority": not failures},
        "systems": systems,
    }
    draft["draft_digest"] = _digest(draft)
    return draft


def _atomic_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    rendered = json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2) + "\n"
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", newline="\n", dir=path.parent, delete=False) as handle:
        handle.write(rendered)
        temporary = Path(handle.name)
    os.replace(temporary, path)


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--source", type=Path, default=None)
    parser.add_argument("--github-repository", default=None)
    parser.add_argument("--github-token-env", default="GITHUB_TOKEN")
    parser.add_argument("--allow-partial", action="store_true")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--json", action="store_true")
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    source = args.source
    if source is None:
        configured = os.environ.get("WOW_UI_SOURCE_DIR")
        source = Path(configured) if configured else None
    token = os.environ.get(args.github_token_env) if args.github_token_env else None
    try:
        draft = build_draft(args.manifest, source, args.github_repository, token, args.allow_partial)
        _atomic_json(args.output, draft)
    except DraftError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    summary = {"output": str(args.output), "draft_digest": draft["draft_digest"], "coverage": draft["coverage"]}
    if args.json:
        print(json.dumps(summary, ensure_ascii=False, sort_keys=True))
    else:
        counts = draft["coverage"]["record_counts"]
        print(f"built {draft['draft_digest']} from {draft['coverage']['parsed_files']} files: {counts['systems']} systems, {counts['functions']} functions, {counts['events']} events, {counts['tables']} tables")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
