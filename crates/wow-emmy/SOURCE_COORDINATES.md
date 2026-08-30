# Source coordinates and content identity

**Status:** normative E0-C coordinate conversion contract.

Source spans are correctness data. A diagnostic or semantic fact with the wrong byte range, file identity, or content digest is invalid even when its message looks plausible.

## 1. Canonical framework span

```text
SourceByteSpan
    byte_start: u64
    byte_end: u64
```

Semantics:

- UTF-8 encoded source bytes;
- half-open interval `[byte_start, byte_end)`;
- `byte_start <= byte_end <= content_byte_length`;
- offsets refer to the exact content digest in the analyzer snapshot;
- zero-length spans are allowed only for insertion/EOF points explicitly supported by the diagnostic/fact contract.

This is the canonical identity-bearing coordinate form.

## 2. Supplementary positions

```text
DerivedTextPosition
    line_1_based
    utf8_column_0_based: optional
    utf16_column_0_based: optional
```

Derived positions aid UI/transports. They do not replace byte spans for identity/hash validation.

## 3. File identity inputs

Coordinate conversion requires:

```text
AnalyzerFile.file_id
normalized relative path
content digest
exact UTF-8 bytes
line index derived from those bytes
upstream source/range representation
analyzer snapshot ID
```

No conversion uses a file currently on disk unless its supplied bytes/digest are exactly the snapshot content.

## 4. Upstream range adapter

All upstream-specific coordinate/range/URI types remain in one private adapter.

The compatibility probe records:

- whether upstream ranges are byte offsets, character offsets, or another representation;
- zero/one-based line and column conventions;
- inclusive/exclusive end semantics;
- URI/path normalization behavior;
- behavior at EOF and malformed files;
- behavior after incremental updates.

Public consumers never receive an upstream range object.

## 5. UTF-8 validation

E0 source input must be valid UTF-8.

Invalid UTF-8:

- is rejected with exact file/input error;
- produces no fabricated line index/facts;
- does not invoke lossy replacement for canonical analysis;
- may be reported as failed file capability.

A future binary/legacy encoding policy requires a separate contract.

## 6. Line endings

The content digest and byte spans refer to supplied bytes, so LF and CRLF are distinct contents/spans.

Tests must cover:

```text
LF only
CRLF only
mixed line endings if accepted
final newline present/absent
empty file
```

Do not normalize line endings behind the analyzer while retaining the original digest unless an explicit source-map translation is implemented and tested. E0 avoids such translation.

## 7. Multibyte text

Tests place multibyte UTF-8 before and inside target spans:

```text
Latin-1 supplement
Cyrillic
CJK
emoji / supplementary plane
combining sequence
```

Required assertions:

- byte offsets slice the intended UTF-8 bytes;
- line/UTF-8/UTF-16 columns are independently derived correctly;
- no mid-codepoint boundary;
- source-handle span validates against content digest.

Grapheme clusters are not the canonical offset unit.

## 8. Path and URI normalization

Public source handles use:

```text
registered project/fixture origin
normalized repository/workspace-relative path
```

They reject or redact:

```text
absolute POSIX path
Windows drive path
UNC/device path
home directory
temporary directory
file:// URI with host path
credentials/query tokens
path traversal outside root
```

The adapter may map upstream internal URI to a registered file ID, then discard/redact the host-specific URI from public output.

## 9. Source handle construction

For Main workspace files:

```text
origin = project/fixture source registry
revision/project generation = supplied context
path = normalized relative path
span = canonical byte span
content digest = exact AnalyzerFile digest
symbol/entity key = optional analyzer-local/project key
```

Library files use library origin/role and cannot masquerade as first-party Main source.

## 10. Span validation

```text
validate_source_span(file, span)
    ensure bounds
    ensure UTF-8 codepoint boundaries when span represents text token/node
    ensure allowed zero-length semantics
    ensure snapshot/file digest match
    optionally verify expected source slice digest/token kind
```

A stale span after update is invalid even if it falls within the new file length.

## 11. Source slice evidence

Facts/findings may store a short source-slice digest, token kind, or stable syntax-node key for validation. They should not embed large source text by default.

Source comments/text remain untrusted evidence, not instructions.

## 12. Update behavior

On file update:

- old facts/findings retain old snapshot/content identity and are not current;
- new spans are recomputed from new upstream state;
- unchanged semantic facts may compare equal but receive new snapshot binding through explicit publication;
- consumers cannot rebase old byte spans by simple delta guessing;
- removed files invalidate all handles/facts/findings for current snapshot.

## 13. Diagnostic span conversion

For every upstream diagnostic:

1. resolve upstream file identity to registered `AnalyzerFile`;
2. convert range to byte half-open span using exact snapshot bytes;
3. validate bounds/encoding/end semantics;
4. build project source handle;
5. retain upstream coordinate details only as optional debug/probe metadata;
6. reject/quarantine diagnostic if conversion is ambiguous/invalid.

Do not clamp invalid spans silently.

## 14. Semantic fact span conversion

Reference/member/call/binding/use/operation/guard facts each have exact primary node/token/expression spans.

Where multiple spans matter, store structured fields rather than one broad enclosing range:

```text
receiver span
member-name span
full reference span
callee span
argument spans
call span
binding declaration span
initializer span
operation span
```

E0 implementations may expose only required fields, but must not substitute a broad line/file span when exact source is available.

## 15. Future LSP conversion

LSP typically uses zero-based line and UTF-16 code-unit columns. When E7 activates:

```text
canonical byte span
    <-> exact snapshot line index
    <-> LSP UTF-16 positions
```

The conversion belongs in transport/adapter utilities with round-trip tests. LSP positions do not become core identity.

## 16. Required operations

```text
build_line_index
validate_utf8_source
normalize_workspace_relative_path
map_upstream_file_identity
convert_upstream_range_to_byte_span
convert_byte_span_to_positions
convert_lsp_position_to_byte_offset (deferred transport support)
validate_source_span
build_project_source_handle
verify_source_handle_content
canonicalize_source_coordinates
```

## 17. Error classes

```text
source_not_utf8
source_path_invalid
source_file_unregistered
source_content_digest_mismatch
source_span_out_of_bounds
source_span_not_codepoint_boundary
source_span_end_semantics_ambiguous
source_coordinate_conversion_failed
source_snapshot_mismatch
source_uri_leaks_host_path
```

## 18. Test matrix highlights

- ASCII token at start/middle/end;
- EOF zero-length diagnostic;
- LF versus CRLF expected offsets;
- multibyte prefix and target;
- combining/emoji UTF-16 columns;
- invalid UTF-8;
- range past EOF;
- inclusive/exclusive upstream mismatch mutation;
- stale span after update;
- absolute/UNC/path traversal URI;
- library versus main source origin;
- randomized temp root produces identical public handles.

## 19. Hard stops

- no public raw upstream range/URI;
- no line/column-only identity;
- no source span without content digest/snapshot;
- no lossy encoding conversion;
- no silent clamp/rebase;
- no absolute path leakage;
- no use of current disk content to validate an older snapshot.
