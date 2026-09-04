#!/usr/bin/env python3
"""Safe deterministic lowering of Blizzard generated API documentation."""

from __future__ import annotations

import argparse
import bisect
import hashlib
import json
import math
import os
import re
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any, Iterable, Iterator, Mapping, Sequence

SCHEMA = "wow-dev-framework/blizzard-api-reference-draft"
SCHEMA_VERSION = 1
PRODUCER_ID = "blizzard-generated-api-reference"
PRODUCER_VERSION = 1
PARSER_ID = "declarative-lua-table-v1"
GENERATED_ROOT = "Interface/AddOns/Blizzard_APIDocumentationGenerated/"
GENERATED_SUFFIX = "Documentation.lua"
DEFAULT_MAX_FILE_BYTES = 16 * 1024 * 1024
DEFAULT_MAX_TOTAL_BYTES = 512 * 1024 * 1024
DEFAULT_MAX_TOKENS = 4_000_000
DEFAULT_MAX_DEPTH = 256


class ReferenceBuildError(Exception):
    def __init__(self, code: str, message: str, *, path: str | None = None) -> None:
        super().__init__(message)
        self.code = code
        self.message = message
        self.path = path

    def as_record(self) -> dict[str, Any]:
        record: dict[str, Any] = {"code": self.code, "message": self.message}
        if self.path is not None:
            record["path"] = self.path
        return record


@dataclass(frozen=True, slots=True)
class Token:
    kind: str
    text: str
    start: int
    end: int
    line: int
    column: int


@dataclass(frozen=True, slots=True)
class LuaField:
    key: str | int | None
    value: "LuaNode"
    start: int
    end: int


@dataclass(frozen=True, slots=True)
class LuaTable:
    fields: tuple[LuaField, ...]


@dataclass(frozen=True, slots=True)
class LuaNode:
    kind: str
    value: Any
    start: int
    end: int


@dataclass(frozen=True, slots=True)
class ParsedDocument:
    variable_name: str
    table: LuaNode
    line_starts: tuple[int, ...]

    def line_for_offset(self, offset: int) -> int:
        return bisect.bisect_right(self.line_starts, offset)

    def span(self, node: LuaNode) -> tuple[int, int]:
        return self.line_for_offset(node.start), self.line_for_offset(max(node.start, node.end - 1))


_SIMPLE_ESCAPES = {
    "a": "\a", "b": "\b", "f": "\f", "n": "\n", "r": "\r",
    "t": "\t", "v": "\v", "\\": "\\", '"': '"', "'": "'",
}


class LuaLexer:
    def __init__(self, source: str, *, max_tokens: int = DEFAULT_MAX_TOKENS) -> None:
        self.source = source
        self.length = len(source)
        self.position = 0
        self.line = 1
        self.column = 1
        self.max_tokens = max_tokens
        self.token_count = 0

    def tokens(self) -> Iterator[Token]:
        while True:
            self._skip_space_and_comments()
            if self.position >= self.length:
                yield self._make("EOF", "", self.position, self.position, self.line, self.column)
                return
            start, line, column = self.position, self.line, self.column
            char = self.source[self.position]
            if char in ('"', "'"):
                text = self._read_quoted_string(char)
                yield self._make("STRING", text, start, self.position, line, column)
                continue
            long_level = self._long_bracket_level(self.position)
            if long_level is not None:
                text = self._read_long_string(long_level)
                yield self._make("STRING", text, start, self.position, line, column)
                continue
            if char.isalpha() or char == "_":
                self._advance_one()
                while self.position < self.length and (self.source[self.position].isalnum() or self.source[self.position] == "_"):
                    self._advance_one()
                yield self._make("IDENT", self.source[start:self.position], start, self.position, line, column)
                continue
            if char.isdigit() or (char == "." and self.position + 1 < self.length and self.source[self.position + 1].isdigit()):
                text = self._read_number()
                yield self._make("NUMBER", text, start, self.position, line, column)
                continue
            if char in "{}[]()=,;.:+-*/%^#<>~":
                self._advance_one()
                if self.position < self.length:
                    pair = char + self.source[self.position]
                    if pair in {"==", "~=", "<=", ">=", "..", "//", "<<", ">>"}:
                        self._advance_one()
                        if pair == ".." and self.position < self.length and self.source[self.position] == ".":
                            self._advance_one()
                yield self._make("PUNCT", self.source[start:self.position], start, self.position, line, column)
                continue
            raise ReferenceBuildError("lua_unsupported_character", f"unsupported character U+{ord(char):04X} at line {line}, column {column}")

    def _make(self, kind: str, text: str, start: int, end: int, line: int, column: int) -> Token:
        self.token_count += 1
        if self.token_count > self.max_tokens:
            raise ReferenceBuildError("lua_token_limit", "generated document exceeds token limit")
        return Token(kind, text, start, end, line, column)

    def _advance_one(self) -> str:
        char = self.source[self.position]
        self.position += 1
        if char == "\n":
            self.line += 1
            self.column = 1
        else:
            self.column += 1
        return char

    def _skip_space_and_comments(self) -> None:
        while self.position < self.length:
            if self.source[self.position].isspace():
                self._advance_one()
                continue
            if self.source.startswith("--", self.position):
                self._advance_one(); self._advance_one()
                level = self._long_bracket_level(self.position)
                if level is not None:
                    self._read_long_string(level)
                else:
                    while self.position < self.length and self.source[self.position] not in "\r\n":
                        self._advance_one()
                continue
            return

    def _long_bracket_level(self, position: int) -> int | None:
        if position >= self.length or self.source[position] != "[":
            return None
        cursor = position + 1
        while cursor < self.length and self.source[cursor] == "=":
            cursor += 1
        return cursor - position - 1 if cursor < self.length and self.source[cursor] == "[" else None

    def _read_long_string(self, level: int) -> str:
        opening = "[" + "=" * level + "["
        closing = "]" + "=" * level + "]"
        if not self.source.startswith(opening, self.position):
            raise AssertionError("invalid long bracket position")
        for _ in opening:
            self._advance_one()
        if self.position < self.length and self.source[self.position] in "\r\n":
            if self.source.startswith("\r\n", self.position):
                self._advance_one()
            self._advance_one()
        content_start = self.position
        closing_position = self.source.find(closing, self.position)
        if closing_position < 0:
            raise ReferenceBuildError("lua_unterminated_long_string", "unterminated long string")
        text = self.source[content_start:closing_position]
        while self.position < closing_position + len(closing):
            self._advance_one()
        return text

    def _read_quoted_string(self, quote: str) -> str:
        self._advance_one()
        output: list[str] = []
        while self.position < self.length:
            char = self._advance_one()
            if char == quote:
                return "".join(output)
            if char in "\r\n":
                raise ReferenceBuildError("lua_unterminated_string", f"newline in quoted string at line {self.line}")
            if char != "\\":
                output.append(char)
                continue
            if self.position >= self.length:
                break
            escape = self._advance_one()
            simple = _SIMPLE_ESCAPES.get(escape)
            if simple is not None:
                output.append(simple); continue
            if escape in "\r\n":
                if escape == "\r" and self.position < self.length and self.source[self.position] == "\n":
                    self._advance_one()
                output.append("\n"); continue
            if escape == "z":
                while self.position < self.length and self.source[self.position].isspace():
                    self._advance_one()
                continue
            if escape == "x":
                digits = self.source[self.position:self.position + 2]
                if len(digits) != 2 or not re.fullmatch(r"[0-9A-Fa-f]{2}", digits):
                    raise ReferenceBuildError("lua_invalid_escape", "invalid hexadecimal string escape")
                self._advance_one(); self._advance_one(); output.append(chr(int(digits, 16))); continue
            if escape == "u" and self.position < self.length and self.source[self.position] == "{":
                self._advance_one(); start = self.position
                while self.position < self.length and self.source[self.position] != "}":
                    self._advance_one()
                digits = self.source[start:self.position]
                if self.position >= self.length or not re.fullmatch(r"[0-9A-Fa-f]{1,8}", digits):
                    raise ReferenceBuildError("lua_invalid_escape", "invalid Unicode string escape")
                self._advance_one(); value = int(digits, 16)
                if value > 0x10FFFF or 0xD800 <= value <= 0xDFFF:
                    raise ReferenceBuildError("lua_invalid_escape", "invalid Unicode scalar value")
                output.append(chr(value)); continue
            if escape.isdigit():
                digits = escape
                for _ in range(2):
                    if self.position < self.length and self.source[self.position].isdigit():
                        digits += self._advance_one()
                    else:
                        break
                value = int(digits, 10)
                if value > 255:
                    raise ReferenceBuildError("lua_invalid_escape", "decimal string escape exceeds 255")
                output.append(chr(value)); continue
            output.append(escape)
        raise ReferenceBuildError("lua_unterminated_string", "unterminated quoted string")

    def _read_number(self) -> str:
        start = self.position
        if self.source.startswith(("0x", "0X"), start):
            self._advance_one(); self._advance_one()
            while self.position < self.length and (self.source[self.position].isdigit() or self.source[self.position].lower() in "abcdef"):
                self._advance_one()
            if self.position == start + 2:
                raise ReferenceBuildError("lua_invalid_number", "hexadecimal number has no digits")
            return self.source[start:self.position]
        saw_dot = False
        if self.source[self.position] == ".":
            saw_dot = True; self._advance_one()
        while self.position < self.length and self.source[self.position].isdigit():
            self._advance_one()
        if self.position < self.length and self.source[self.position] == "." and not saw_dot and not self.source.startswith("..", self.position):
            self._advance_one()
            while self.position < self.length and self.source[self.position].isdigit():
                self._advance_one()
        if self.position < self.length and self.source[self.position] in "eE":
            exponent_position = self.position; self._advance_one()
            if self.position < self.length and self.source[self.position] in "+-":
                self._advance_one()
            digit_start = self.position
            while self.position < self.length and self.source[self.position].isdigit():
                self._advance_one()
            if self.position == digit_start:
                self.position = exponent_position
        return self.source[start:self.position]


class LuaTableParser:
    def __init__(self, source: str, *, max_tokens: int = DEFAULT_MAX_TOKENS, max_depth: int = DEFAULT_MAX_DEPTH) -> None:
        self.source = source.lstrip("\ufeff")
        self.tokens = tuple(LuaLexer(self.source, max_tokens=max_tokens).tokens())
        self.index = 0
        self.max_depth = max_depth

    def parse(self) -> ParsedDocument:
        local_token = self._consume_ident("local")
        variable = self._consume("IDENT")
        self._consume_punct("=")
        table = self._parse_value(0)
        if table.kind != "table":
            self._error("lua_root_not_table", "generated documentation root is not a table", local_token)
        self._consume_optional_punct(";")
        self._consume_ident("APIDocumentation"); self._consume_punct(":"); self._consume_ident("AddDocumentationTable")
        self._consume_punct("("); registered = self._consume("IDENT"); self._consume_punct(")"); self._consume_optional_punct(";"); self._consume("EOF")
        if registered.text != variable.text:
            self._error("lua_registration_mismatch", f"registered table {registered.text!r} does not match local {variable.text!r}", registered)
        line_starts = [0]
        for match in re.finditer("\n", self.source):
            line_starts.append(match.end())
        return ParsedDocument(variable.text, table, tuple(line_starts))

    def _peek(self, offset: int = 0) -> Token:
        return self.tokens[min(self.index + offset, len(self.tokens) - 1)]

    def _consume(self, kind: str) -> Token:
        token = self._peek()
        if token.kind != kind:
            self._error("lua_unexpected_token", f"expected {kind}, found {token.text!r}", token)
        self.index += 1
        return token

    def _consume_ident(self, value: str) -> Token:
        token = self._consume("IDENT")
        if token.text != value:
            self._error("lua_unexpected_identifier", f"expected {value!r}, found {token.text!r}", token)
        return token

    def _consume_punct(self, value: str) -> Token:
        token = self._peek()
        if token.kind != "PUNCT" or token.text != value:
            self._error("lua_unexpected_token", f"expected {value!r}, found {token.text!r}", token)
        self.index += 1
        return token

    def _consume_optional_punct(self, value: str) -> bool:
        if self._peek().kind == "PUNCT" and self._peek().text == value:
            self.index += 1; return True
        return False

    def _parse_value(self, depth: int) -> LuaNode:
        if depth > self.max_depth:
            raise ReferenceBuildError("lua_depth_limit", "generated document exceeds nesting limit")
        token = self._peek()
        if token.kind == "STRING":
            self.index += 1; return LuaNode("scalar", token.text, token.start, token.end)
        if token.kind == "NUMBER":
            self.index += 1; return LuaNode("scalar", self._number_value(token), token.start, token.end)
        if token.kind == "PUNCT" and token.text == "{":
            return self._parse_table(depth + 1)
        if token.kind == "PUNCT" and token.text in {"+", "-"}:
            sign = token.text; self.index += 1; number = self._peek()
            if number.kind == "NUMBER":
                self.index += 1; value = self._number_value(number)
                return LuaNode("scalar", -value if sign == "-" else value, token.start, number.end)
            self.index -= 1; return self._parse_opaque_expression()
        if token.kind == "IDENT":
            if token.text in {"true", "false", "nil"}:
                self.index += 1
                return LuaNode("scalar", {"true": True, "false": False, "nil": None}[token.text], token.start, token.end)
            return self._parse_symbol_or_expression()
        return self._parse_opaque_expression()

    def _number_value(self, token: Token) -> int | float:
        try:
            if token.text.lower().startswith("0x"):
                return int(token.text, 16)
            if any(marker in token.text for marker in ".eE"):
                value = float(token.text)
                if not math.isfinite(value):
                    raise ValueError("non-finite")
                return value
            return int(token.text, 10)
        except ValueError as error:
            self._error("lua_invalid_number", f"invalid numeric literal {token.text!r}", token)
            raise AssertionError("unreachable") from error

    def _parse_symbol_or_expression(self) -> LuaNode:
        start_index = self.index; first = self._consume("IDENT"); parts = [first.text]; end = first.end
        while self._peek().kind == "PUNCT" and self._peek().text == ".":
            self.index += 1; part = self._consume("IDENT"); parts.append(part.text); end = part.end
        if self._peek().kind == "PUNCT" and self._peek().text in {",", ";", "}"}:
            return LuaNode("symbol", ".".join(parts), first.start, end)
        self.index = start_index
        return self._parse_opaque_expression()

    def _parse_opaque_expression(self) -> LuaNode:
        token = self._peek()
        if token.kind == "EOF":
            self._error("lua_missing_value", "missing table value", token)
        start = token.start; end = token.end; round_depth = square_depth = curly_depth = consumed = 0
        while True:
            current = self._peek()
            if current.kind == "EOF": break
            if current.kind == "PUNCT":
                if current.text == "(": round_depth += 1
                elif current.text == ")":
                    if round_depth == 0: break
                    round_depth -= 1
                elif current.text == "[": square_depth += 1
                elif current.text == "]":
                    if square_depth == 0: break
                    square_depth -= 1
                elif current.text == "{": curly_depth += 1
                elif current.text == "}":
                    if round_depth == square_depth == curly_depth == 0: break
                    curly_depth -= 1
                elif current.text in {",", ";"} and round_depth == square_depth == curly_depth == 0: break
            self.index += 1; consumed += 1; end = current.end
        if consumed == 0:
            self._error("lua_missing_value", "missing table value", token)
        return LuaNode("expression", self.source[start:end].strip(), start, end)

    def _parse_table(self, depth: int) -> LuaNode:
        opening = self._consume_punct("{"); fields: list[LuaField] = []
        while not (self._peek().kind == "PUNCT" and self._peek().text == "}"):
            if self._peek().kind == "EOF":
                self._error("lua_unterminated_table", "unterminated table", self._peek())
            field_start = self._peek().start; key: str | int | None = None
            if self._peek().kind == "IDENT" and self._peek(1).kind == "PUNCT" and self._peek(1).text == "=":
                key = self._consume("IDENT").text; self._consume_punct("="); value = self._parse_value(depth)
            elif self._peek().kind == "PUNCT" and self._peek().text == "[":
                self._consume_punct("["); key_node = self._parse_value(depth); self._consume_punct("]"); self._consume_punct("=")
                plain_key = node_to_plain(key_node)
                if not isinstance(plain_key, (str, int)) or isinstance(plain_key, bool):
                    self._error("lua_unsupported_table_key", "table key must be a string or integer", self._peek())
                key = plain_key; value = self._parse_value(depth)
            else:
                value = self._parse_value(depth)
            fields.append(LuaField(key, value, field_start, value.end))
            if not (self._consume_optional_punct(",") or self._consume_optional_punct(";")) and not (self._peek().kind == "PUNCT" and self._peek().text == "}"):
                self._error("lua_missing_separator", "expected table field separator", self._peek())
        closing = self._consume_punct("}")
        return LuaNode("table", LuaTable(tuple(fields)), opening.start, closing.end)

    def _error(self, code: str, message: str, token: Token) -> None:
        raise ReferenceBuildError(code, f"{message} at line {token.line}, column {token.column}")


def node_to_plain(node: LuaNode) -> Any:
    if node.kind == "scalar": return node.value
    if node.kind == "symbol": return {"$lua_symbol": node.value}
    if node.kind == "expression": return {"$lua_expression": node.value}
    if node.kind != "table" or not isinstance(node.value, LuaTable):
        raise ReferenceBuildError("lua_internal_node", f"unknown node kind {node.kind!r}")
    fields = node.value.fields
    if all(field.key is None for field in fields):
        return [node_to_plain(field.value) for field in fields]
    if all(field.key is not None for field in fields):
        output: dict[str, Any] = {}
        for field in fields:
            key = str(field.key)
            if key in output:
                return {"$lua_table": [{"key": item.key, "value": node_to_plain(item.value)} for item in fields]}
            output[key] = node_to_plain(field.value)
        return output
    return {"$lua_table": [{"key": field.key, "value": node_to_plain(field.value)} for field in fields]}


def parse_generated_document(source: str) -> ParsedDocument:
    return LuaTableParser(source).parse()


def _table_fields(node: LuaNode) -> tuple[LuaField, ...]:
    if node.kind != "table" or not isinstance(node.value, LuaTable):
        raise ReferenceBuildError("normalization_expected_table", "expected a Lua table")
    return node.value.fields


def _field_nodes(node: LuaNode) -> dict[str, LuaNode]:
    output: dict[str, LuaNode] = {}
    for field in _table_fields(node):
        if not isinstance(field.key, str): raise ReferenceBuildError("normalization_expected_map", "expected a keyed Lua table")
        if field.key in output: raise ReferenceBuildError("normalization_duplicate_key", f"duplicate key {field.key!r}")
        output[field.key] = field.value
    return output


def _array_nodes(node: LuaNode | None, field_name: str) -> list[LuaNode]:
    if node is None: return []
    output: list[LuaNode] = []
    for field in _table_fields(node):
        if field.key is not None: raise ReferenceBuildError("normalization_expected_array", f"{field_name} must be an array table")
        output.append(field.value)
    return output


def _optional_string(node: LuaNode | None, field_name: str) -> str | None:
    if node is None or (node.kind == "scalar" and node.value is None): return None
    if node.kind != "scalar" or not isinstance(node.value, str):
        raise ReferenceBuildError("normalization_expected_string", f"{field_name} must be a string")
    return node.value


def _required_string(node: LuaNode | None, field_name: str) -> str:
    value = _optional_string(node, field_name)
    if not value: raise ReferenceBuildError("normalization_missing_field", f"missing non-empty {field_name}")
    return value


def _documentation(node: LuaNode | None) -> list[str]:
    if node is None: return []
    output: list[str] = []
    for item in _array_nodes(node, "Documentation"):
        value = _optional_string(item, "Documentation item")
        if value is None: raise ReferenceBuildError("normalization_expected_string", "Documentation item must be a string")
        output.append(value)
    return output


def _snake_case(name: str) -> str:
    first = re.sub(r"(.)([A-Z][a-z]+)", r"\1_\2", name)
    return re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", first).replace("-", "_").lower()


def _normalized_plain(node: LuaNode) -> Any:
    if node.kind != "table": return node_to_plain(node)
    fields = _table_fields(node)
    if all(field.key is None for field in fields): return [_normalized_plain(field.value) for field in fields]
    if all(isinstance(field.key, str) for field in fields):
        output: dict[str, Any] = {}
        for field in fields:
            assert isinstance(field.key, str)
            key = _snake_case(field.key)
            if key in output:
                return {"$lua_table": [{"key": _snake_case(str(item.key)) if item.key is not None else None, "value": _normalized_plain(item.value)} for item in fields]}
            output[key] = _normalized_plain(field.value)
        return output
    return node_to_plain(node)


def _source_record(*, path: str, sha256: str, git_object: str, document: ParsedDocument, node: LuaNode) -> dict[str, Any]:
    line_start, line_end = document.span(node)
    return {"path": path, "line_start": line_start, "line_end": line_end, "git_object": git_object, "sha256": sha256}


def _normalize_parameter(node: LuaNode, *, path: str, sha256: str, git_object: str, document: ParsedDocument) -> dict[str, Any]:
    output = {_snake_case(key): _normalized_plain(value) for key, value in _field_nodes(node).items()}
    output["source"] = _source_record(path=path, sha256=sha256, git_object=git_object, document=document, node=node)
    return output


def _normalize_member(node: LuaNode, *, collection: str, namespace: str | None, path: str, sha256: str, git_object: str, document: ParsedDocument) -> dict[str, Any]:
    fields = _field_nodes(node)
    name = _required_string(fields.get("Name"), f"{collection}.Name")
    member_type = _optional_string(fields.get("Type"), f"{collection}.Type")
    literal_name = _optional_string(fields.get("LiteralName"), f"{collection}.LiteralName")
    qualified_name = literal_name if collection == "events" and literal_name else f"{namespace}.{name}" if namespace else name
    child_fields = {"Arguments": "arguments", "Returns": "returns", "Payload": "payload", "Fields": "fields", "Values": "values"}
    children: dict[str, Any] = {}
    for source_name, output_name in child_fields.items():
        values = _array_nodes(fields.get(source_name), source_name)
        if values:
            children[output_name] = [_normalize_parameter(value, path=path, sha256=sha256, git_object=git_object, document=document) for value in values]
    ignored = {"Name", "Type", "LiteralName", "Documentation", *child_fields.keys()}
    restrictions: dict[str, Any] = {}; attributes: dict[str, Any] = {}
    for key, value in fields.items():
        if key in ignored: continue
        output_key = _snake_case(key); normalized = _normalized_plain(value)
        target = restrictions if any(marker in output_key for marker in ("restriction", "secret", "secure", "protected", "combat", "requires_", "forbidden", "taint")) else attributes
        target[output_key] = normalized
    output: dict[str, Any] = {"name": name, "qualified_name": qualified_name, "type": member_type, "documentation": _documentation(fields.get("Documentation")), "restrictions": restrictions, "attributes": attributes, "source": _source_record(path=path, sha256=sha256, git_object=git_object, document=document, node=node)}
    if literal_name is not None: output["literal_name"] = literal_name
    output.update(children)
    return output


def normalize_document(document: ParsedDocument, *, path: str, sha256: str, git_object: str) -> dict[str, Any]:
    fields = _field_nodes(document.table)
    name = _required_string(fields.get("Name"), "System.Name")
    namespace = _optional_string(fields.get("Namespace"), "System.Namespace")
    collections = {"Functions": "functions", "Events": "events", "Tables": "tables", "Enumerations": "enumerations", "Constants": "constants", "Predicates": "predicates"}
    output_collections: dict[str, list[dict[str, Any]]] = {}
    for source_name, output_name in collections.items():
        members = [_normalize_member(member, collection=output_name, namespace=namespace, path=path, sha256=sha256, git_object=git_object, document=document) for member in _array_nodes(fields.get(source_name), source_name)]
        members.sort(key=lambda item: (item["qualified_name"], item.get("type") or "", item["source"]["line_start"]))
        output_collections[output_name] = members
    ignored = {"Name", "Namespace", "Type", "Environment", "Documentation", *collections.keys()}
    system: dict[str, Any] = {"name": name, "namespace": namespace, "type": _optional_string(fields.get("Type"), "System.Type"), "environment": _optional_string(fields.get("Environment"), "System.Environment"), "documentation": _documentation(fields.get("Documentation")), "attributes": {_snake_case(key): _normalized_plain(value) for key, value in fields.items() if key not in ignored}, "source": _source_record(path=path, sha256=sha256, git_object=git_object, document=document, node=document.table)}
    system.update(output_collections)
    return system


def canonical_json_bytes(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, allow_nan=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def sha256_id(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def _get_nested(value: Mapping[str, Any], path: Sequence[str]) -> Any:
    current: Any = value
    for segment in path:
        if not isinstance(current, Mapping) or segment not in current: return None
        current = current[segment]
    return current


def _first_nested(value: Mapping[str, Any], paths: Iterable[Sequence[str]]) -> Any:
    for path in paths:
        candidate = _get_nested(value, path)
        if candidate is not None: return candidate
    return None


def _manifest_revision(manifest: Mapping[str, Any]) -> str:
    candidate = _first_nested(manifest, (("source", "revision"), ("source", "resolved_revision"), ("source", "resolved_commit"), ("source", "commit"), ("source", "commit_sha"), ("resolved_revision",), ("resolved_commit",), ("revision",), ("commit",), ("commit_sha",), ("source_revision",)))
    if not isinstance(candidate, str) or not re.fullmatch(r"[0-9a-fA-F]{40}|[0-9a-fA-F]{64}", candidate):
        raise ReferenceBuildError("manifest_revision", "source manifest has no exact Git object identifier")
    return candidate.lower()


def _manifest_files(manifest: Mapping[str, Any]) -> list[Mapping[str, Any]]:
    candidate = _first_nested(manifest, (("files",), ("entries",), ("inventory", "files"), ("source", "files")))
    if not isinstance(candidate, list): raise ReferenceBuildError("manifest_files", "source manifest has no file inventory")
    if not all(isinstance(item, Mapping) for item in candidate): raise ReferenceBuildError("manifest_file_record", "source manifest contains an invalid file record")
    return list(candidate)


def _record_value(record: Mapping[str, Any], names: Sequence[str]) -> Any:
    return next((record[name] for name in names if name in record), None)


def _safe_source_path(candidate: Any) -> str:
    if not isinstance(candidate, str) or not candidate: raise ReferenceBuildError("manifest_path", "source file record has no path")
    if "\x00" in candidate or "\\" in candidate: raise ReferenceBuildError("manifest_path", "source path is not canonical POSIX text")
    path = PurePosixPath(candidate)
    if path.is_absolute() or any(part in {"", ".", ".."} for part in path.parts) or path.as_posix() != candidate:
        raise ReferenceBuildError("manifest_path", "source path escapes the repository root or is not canonical")
    return candidate


def _normalize_sha256(candidate: Any) -> str:
    if not isinstance(candidate, str): raise ReferenceBuildError("manifest_sha256", "source file record has no SHA-256 digest")
    lowered = candidate.lower().removeprefix("sha256:")
    if not re.fullmatch(r"[0-9a-f]{64}", lowered): raise ReferenceBuildError("manifest_sha256", "source file digest is not SHA-256")
    return "sha256:" + lowered


def _run_git(source: Path, arguments: Sequence[str]) -> bytes:
    try:
        result = subprocess.run(["git", "-C", os.fspath(source), *arguments], stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    except OSError as error:
        raise ReferenceBuildError("git_unavailable", "Git is not available") from error
    if result.returncode:
        message = result.stderr.decode("utf-8", errors="replace").strip()
        raise ReferenceBuildError("git_failed", message.splitlines()[-1] if message else "Git command failed")
    return result.stdout


def _validate_revision_for_repository(source: Path, revision: str) -> None:
    object_format = _run_git(source, ["rev-parse", "--show-object-format"]).decode("ascii").strip()
    if object_format not in {"sha1", "sha256"}: raise ReferenceBuildError("git_object_format", f"unsupported Git object format {object_format!r}")
    if len(revision) != (40 if object_format == "sha1" else 64): raise ReferenceBuildError("manifest_revision_format", f"manifest revision length does not match repository {object_format} format")
    _run_git(source, ["cat-file", "-e", f"{revision}^{{commit}}"])


def _read_git_blob(source: Path, revision: str, path: str, max_bytes: int) -> tuple[bytes, str]:
    object_name = f"{revision}:{path}"
    if _run_git(source, ["cat-file", "-t", object_name]).decode("ascii").strip() != "blob": raise ReferenceBuildError("source_not_blob", "manifest path does not identify a Git blob", path=path)
    size = int(_run_git(source, ["cat-file", "-s", object_name]).decode("ascii").strip())
    if size < 0 or size > max_bytes: raise ReferenceBuildError("source_file_limit", "generated document exceeds file size limit", path=path)
    object_id = _run_git(source, ["rev-parse", object_name]).decode("ascii").strip().lower()
    data = _run_git(source, ["cat-file", "blob", object_name])
    if len(data) != size: raise ReferenceBuildError("source_size_mismatch", "Git blob size changed during read", path=path)
    return data, object_id


def _candidate_records(manifest: Mapping[str, Any]) -> list[dict[str, Any]]:
    output: list[dict[str, Any]] = []; seen: set[str] = set()
    for record in _manifest_files(manifest):
        path = _safe_source_path(_record_value(record, ("path", "canonical_path", "relative_path")))
        if not (path.startswith(GENERATED_ROOT) and path.endswith(GENERATED_SUFFIX)): continue
        if path in seen: raise ReferenceBuildError("manifest_duplicate_path", f"duplicate source path {path!r}")
        seen.add(path)
        output.append({"path": path, "sha256": _normalize_sha256(_record_value(record, ("sha256", "content_sha256", "content_digest", "digest"))), "git_object": _record_value(record, ("git_object", "git_blob", "blob", "blob_id", "object_id"))})
    output.sort(key=lambda item: item["path"].encode())
    if not output: raise ReferenceBuildError("manifest_no_generated_docs", f"source manifest contains no {GENERATED_ROOT}*{GENERATED_SUFFIX} files")
    return output


def _member_conflicts(systems: Sequence[Mapping[str, Any]]) -> list[dict[str, Any]]:
    seen: dict[tuple[str, str], list[Mapping[str, Any]]] = {}
    for system in systems:
        for collection in ("functions", "events", "tables", "enumerations", "constants", "predicates"):
            for member in system.get(collection, []): seen.setdefault((collection, str(member["qualified_name"])), []).append(member)
    conflicts = []
    for (collection, qualified_name), members in sorted(seen.items()):
        if len(members) > 1:
            conflicts.append({"kind": "duplicate_symbol", "collection": collection, "qualified_name": qualified_name, "sources": sorted(({"path": member["source"]["path"], "line_start": member["source"]["line_start"]} for member in members), key=lambda item: (item["path"].encode(), item["line_start"]))})
    return conflicts


def load_json(path: Path, *, max_bytes: int = 64 * 1024 * 1024) -> tuple[dict[str, Any], bytes]:
    try:
        if path.stat().st_size > max_bytes: raise ReferenceBuildError("input_size_limit", "input JSON exceeds size limit")
        raw = path.read_bytes(); value = json.loads(raw.decode("utf-8"))
    except ReferenceBuildError: raise
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error: raise ReferenceBuildError("input_json", "input is not valid UTF-8 JSON") from error
    if not isinstance(value, dict): raise ReferenceBuildError("input_json_shape", "input JSON root must be an object")
    return value, raw


def build_reference_draft(*, source: Path, manifest: Mapping[str, Any], manifest_bytes: bytes, allow_partial: bool = False, max_file_bytes: int = DEFAULT_MAX_FILE_BYTES, max_total_bytes: int = DEFAULT_MAX_TOTAL_BYTES) -> dict[str, Any]:
    revision = _manifest_revision(manifest); _validate_revision_for_repository(source, revision); records = _candidate_records(manifest)
    systems: list[dict[str, Any]] = []; failures: list[dict[str, Any]] = []; parsed_paths: list[str] = []; total_bytes = 0
    for record in records:
        path = record["path"]
        try:
            data, git_object = _read_git_blob(source, revision, path, max_file_bytes); total_bytes += len(data)
            if total_bytes > max_total_bytes: raise ReferenceBuildError("source_total_limit", "generated documentation exceeds total size limit", path=path)
            actual_sha256 = sha256_id(data)
            if actual_sha256 != record["sha256"]: raise ReferenceBuildError("source_digest_mismatch", "source bytes do not match manifest SHA-256", path=path)
            if record["git_object"] is not None and (not isinstance(record["git_object"], str) or record["git_object"].lower() != git_object): raise ReferenceBuildError("source_git_object_mismatch", "source Git object does not match manifest", path=path)
            try: text = data.decode("utf-8")
            except UnicodeDecodeError as error: raise ReferenceBuildError("source_not_utf8", "generated documentation is not UTF-8", path=path) from error
            systems.append(normalize_document(parse_generated_document(text), path=path, sha256=actual_sha256, git_object=git_object)); parsed_paths.append(path)
        except ReferenceBuildError as error:
            if error.path is None: error.path = path
            failures.append(error.as_record())
    systems.sort(key=lambda item: ((item.get("namespace") or "").encode(), item["name"].encode(), item["source"]["path"].encode())); failures.sort(key=lambda item: (item.get("path", "").encode(), item["code"], item["message"]))
    complete = not failures and len(parsed_paths) == len(records)
    if not complete and not allow_partial:
        first = failures[0] if failures else {"message": "generated documentation coverage is incomplete"}; raise ReferenceBuildError("generated_api_incomplete", str(first["message"]), path=first.get("path"))
    collections = ("functions", "events", "tables", "enumerations", "constants", "predicates")
    counts = {collection: sum(len(system[collection]) for system in systems) for collection in collections}
    metadata = {"source_id": _first_nested(manifest, (("source", "id"), ("source_id",))), "selector": _first_nested(manifest, (("source", "selector"), ("selector",))), "version": _first_nested(manifest, (("source", "version"), ("version",), ("reported_version",))), "declared_digest": _first_nested(manifest, (("manifest_sha256",), ("manifest_digest",), ("digest",)))}
    draft: dict[str, Any] = {"schema": SCHEMA, "schema_version": SCHEMA_VERSION, "producer": {"id": PRODUCER_ID, "version": PRODUCER_VERSION, "parser": PARSER_ID, "configuration": {"generated_root": GENERATED_ROOT, "generated_suffix": GENERATED_SUFFIX}}, "source": {"manifest_sha256": sha256_id(manifest_bytes), "manifest_declared_digest": metadata["declared_digest"], "source_id": metadata["source_id"], "selector": metadata["selector"], "revision": revision, "version": metadata["version"]}, "coverage": {"scope": f"{GENERATED_ROOT}*{GENERATED_SUFFIX}", "status": "complete" if complete else "partial", "negative_authority": complete, "candidate_files": len(records), "parsed_files": len(parsed_paths), "failed_files": len(failures), "parsed_paths": parsed_paths, "failures": failures, "entity_counts": counts, "limitations": ["generated documentation does not prove runtime or hotfix behavior", "implementation, XML, TOC, and runtime evidence remain separate sources"]}, "conflicts": _member_conflicts(systems), "systems": systems}
    draft["draft_sha256"] = sha256_id(canonical_json_bytes(draft)); return draft


def verify_reference_draft(draft: Mapping[str, Any], *, require_complete: bool = False) -> None:
    if draft.get("schema") != SCHEMA or draft.get("schema_version") != SCHEMA_VERSION: raise ReferenceBuildError("draft_schema", "unsupported API reference draft schema")
    supplied = draft.get("draft_sha256"); projection = dict(draft); projection.pop("draft_sha256", None)
    if not isinstance(supplied, str) or supplied != sha256_id(canonical_json_bytes(projection)): raise ReferenceBuildError("draft_digest", "API reference draft digest does not match content")
    if not isinstance(draft.get("producer"), Mapping) or draft["producer"].get("id") != PRODUCER_ID: raise ReferenceBuildError("draft_producer", "unexpected API reference producer")
    source = draft.get("source")
    if not isinstance(source, Mapping) or not isinstance(source.get("revision"), str) or not re.fullmatch(r"[0-9a-f]{40}|[0-9a-f]{64}", source["revision"]): raise ReferenceBuildError("draft_revision", "API reference draft revision is not exact")
    coverage = draft.get("coverage")
    if not isinstance(coverage, Mapping) or coverage.get("status") not in {"complete", "partial"}: raise ReferenceBuildError("draft_coverage", "API reference draft coverage is invalid")
    if require_complete and coverage["status"] != "complete": raise ReferenceBuildError("draft_incomplete", "complete generated API coverage is required")
    candidate, parsed, failed = coverage.get("candidate_files"), coverage.get("parsed_files"), coverage.get("failed_files")
    if not all(isinstance(value, int) and not isinstance(value, bool) and value >= 0 for value in (candidate, parsed, failed)): raise ReferenceBuildError("draft_coverage", "API reference file counts are invalid")
    failures, paths = coverage.get("failures"), coverage.get("parsed_paths")
    if not isinstance(failures, list) or len(failures) != failed or not isinstance(paths, list) or len(paths) != parsed or candidate != parsed + failed: raise ReferenceBuildError("draft_coverage", "API reference file counts are inconsistent")
    complete = failed == 0 and candidate == parsed
    if (coverage["status"] == "complete") != complete or coverage.get("negative_authority") is not complete: raise ReferenceBuildError("draft_coverage", "API reference authority does not match coverage")
    if paths != sorted(paths, key=lambda item: item.encode()) or len(paths) != len(set(paths)): raise ReferenceBuildError("draft_order", "API reference parsed paths are not unique and sorted")
    systems = draft.get("systems")
    if not isinstance(systems, list): raise ReferenceBuildError("draft_systems", "API reference systems must be an array")
    expected_systems = sorted(systems, key=lambda item: ((item.get("namespace") or "").encode(), item.get("name", "").encode(), item.get("source", {}).get("path", "").encode()))
    if systems != expected_systems: raise ReferenceBuildError("draft_order", "API reference systems are not canonically ordered")
    collections = ("functions", "events", "tables", "enumerations", "constants", "predicates"); observed = {name: 0 for name in collections}
    for system in systems:
        if not isinstance(system, Mapping) or not isinstance(system.get("name"), str): raise ReferenceBuildError("draft_system", "API reference system record is invalid")
        for collection in collections:
            members = system.get(collection)
            if not isinstance(members, list): raise ReferenceBuildError("draft_members", f"system {collection} must be an array")
            if members != sorted(members, key=lambda item: (item.get("qualified_name", ""), item.get("type") or "", item.get("source", {}).get("line_start", 0))): raise ReferenceBuildError("draft_order", f"system {collection} are not canonically ordered")
            observed[collection] += len(members)
    if coverage.get("entity_counts") != observed: raise ReferenceBuildError("draft_coverage", "API reference entity counts are inconsistent")


def write_json_atomic(path: Path, value: Mapping[str, Any]) -> None:
    data = json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2, allow_nan=False).encode() + b"\n"; path.parent.mkdir(parents=True, exist_ok=True)
    descriptor, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", suffix=".tmp", dir=path.parent); temporary = Path(temporary_name)
    try:
        with os.fdopen(descriptor, "wb") as handle:
            handle.write(data); handle.flush(); os.fsync(handle.fileno())
        os.replace(temporary, path)
    finally:
        try: temporary.unlink()
        except FileNotFoundError: pass


def _summary(draft: Mapping[str, Any], *, stale: bool = False) -> dict[str, Any]:
    coverage = draft["coverage"]
    return {"status": "stale" if stale else "ok", "revision": draft["source"]["revision"], "version": draft["source"].get("version"), "coverage": coverage["status"], "candidate_files": coverage["candidate_files"], "parsed_files": coverage["parsed_files"], "failed_files": coverage["failed_files"], "systems": len(draft["systems"]), "entities": coverage["entity_counts"], "conflicts": len(draft["conflicts"]), "draft_sha256": draft["draft_sha256"]}


def build_cli(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Build normalized Blizzard generated API reference")
    parser.add_argument("--manifest", required=True, type=Path); parser.add_argument("--source", required=True, type=Path); parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--allow-partial", action="store_true"); parser.add_argument("--max-file-bytes", type=int, default=DEFAULT_MAX_FILE_BYTES); parser.add_argument("--max-total-bytes", type=int, default=DEFAULT_MAX_TOTAL_BYTES); parser.add_argument("--json", action="store_true", dest="json_output")
    arguments = parser.parse_args(argv)
    try:
        manifest, raw = load_json(arguments.manifest); draft = build_reference_draft(source=arguments.source, manifest=manifest, manifest_bytes=raw, allow_partial=arguments.allow_partial, max_file_bytes=arguments.max_file_bytes, max_total_bytes=arguments.max_total_bytes); verify_reference_draft(draft, require_complete=not arguments.allow_partial); write_json_atomic(arguments.output, draft)
    except ReferenceBuildError as error:
        print(json.dumps({"status": "error", "error": error.as_record()}, ensure_ascii=False) if arguments.json_output else f"{error.code}: {error.message}", file=sys.stderr); return 2
    summary = _summary(draft); print(json.dumps(summary, ensure_ascii=False, sort_keys=True) if arguments.json_output else f"built {summary['systems']} systems from {summary['parsed_files']} files: {summary['draft_sha256']}"); return 0


def verify_cli(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Verify normalized Blizzard generated API reference")
    parser.add_argument("draft", type=Path); parser.add_argument("--manifest", type=Path); parser.add_argument("--source", type=Path); parser.add_argument("--current-ref"); parser.add_argument("--require-complete", action="store_true"); parser.add_argument("--json", action="store_true", dest="json_output")
    arguments = parser.parse_args(argv)
    if (arguments.manifest is None) != (arguments.source is None): parser.error("--manifest and --source must be supplied together")
    try:
        draft, _ = load_json(arguments.draft); verify_reference_draft(draft, require_complete=arguments.require_complete)
        if arguments.manifest is not None and arguments.source is not None:
            manifest, raw = load_json(arguments.manifest); rebuilt = build_reference_draft(source=arguments.source, manifest=manifest, manifest_bytes=raw, allow_partial=draft["coverage"]["status"] == "partial")
            if canonical_json_bytes(rebuilt) != canonical_json_bytes(draft): raise ReferenceBuildError("draft_rebuild_mismatch", "draft does not match exact source snapshot")
        stale = False
        if arguments.current_ref is not None:
            if arguments.source is None: parser.error("--current-ref requires --source and --manifest")
            stale = _run_git(arguments.source, ["rev-parse", "--verify", f"{arguments.current_ref}^{{commit}}"]).decode("ascii").strip().lower() != draft["source"]["revision"]
    except ReferenceBuildError as error:
        print(json.dumps({"status": "error", "error": error.as_record()}, ensure_ascii=False) if arguments.json_output else f"{error.code}: {error.message}", file=sys.stderr); return 2
    summary = _summary(draft, stale=stale); print(json.dumps(summary, ensure_ascii=False, sort_keys=True) if arguments.json_output else summary["status"]); return 3 if stale else 0


__all__ = ["ReferenceBuildError", "build_reference_draft", "canonical_json_bytes", "parse_generated_document", "normalize_document", "verify_reference_draft", "build_cli", "verify_cli", "node_to_plain", "sha256_id", "SCHEMA", "SCHEMA_VERSION", "PRODUCER_ID"]
