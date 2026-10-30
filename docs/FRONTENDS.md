# CLI, editors and agents

CLI, LSP and MCP are thin adapters to the same Service owners. They do not create
parallel analyzers, Project publication state, result hierarchies or lifecycle
platforms. General Lua language intelligence remains upstream EmmyLua.

Read [current status](CURRENT_STATUS.md) and the
[queue](REFACTOR_QUEUE.md). This guide defines the stable interface contract.

<a id="current-checkpoint"></a>
<a id="target-user-operations"></a>
<a id="configuration-and-check-input"></a>

## CLI

**Planned Slice 1A interface — not implemented in the current CLI:**

```text
wow check
  [--config <wowdev.json>]
  [--root <addon-dir>]
  [--toc <explicit.toc>]
  [--flavor retail]
  [--library <dir> ...]
  [--format text|json]
  [--fail-on error|warning|information|hint]
```

Later slices add auto-selection, target Interface, channel, dependency roots and
WoW checks; they do not delay physical Lua.

### Configuration and path bases

The JSON object accepts only `root`, `toc`, `flavor`, `library`,
`emmy_diagnostics`, `format` and `fail_on`. Paths/flavor/format/threshold are
strings; `library` is an array of directory strings; `emmy_diagnostics` is the
pinned diagnostic object. Root, TOC, exact `retail` and at least one Library are
required after config/CLI merge. For example:

```json
{"root":"./MyAddon","toc":"MyAddon.toc","flavor":"retail","library":["./wow-library"]}
```

Read one explicit config only, max 256 KiB, through Project's component-wise
no-follow helper. Reuse duplicate-key/depth-bounded `StrictJson`; reject unknown
outer fields and duplicate scalar CLI options. No implicit discovery. If config
fails, only an already admitted CLI `--format` selects JSON error output.

In [Project disk](../crates/wow-project/src/disk.rs), `read_configuration` currently
admits 32 MiB; the smaller `read_json_artifact_with_limit` requires a predeclared
digest/length. Neither is the ordinary config entry unchanged. Reuse the internal
bounded read with the 256 KiB ceiling applied during acquisition. Do not read the
larger buffer then reject it, invent an artifact identity, or add another reader.

Path origin is part of deterministic configuration:

- a relative `--config` path resolves from process cwd;
- relative CLI `--root` and every CLI `--library` resolve from cwd;
- relative `root` and `library` values inside config resolve from the resolved
  config file's parent directory;
- `toc`, whether supplied by config or CLI, is always a canonical relative `.toc`
  path and resolves only after scalar merge, against the final resolved addon
  root—not cwd or the config directory;
- absolute root/Library values remain absolute;
- joining/normalization is lexical (`.`/`..` reduction as applicable) and never
  calls filesystem canonicalization or follows a root link. Project later opens
  the resolved descriptor component-by-component without following links.

CLI scalar values replace config scalar values while retaining the winning
value's origin base. One or more CLI `--library` options replace the entire config
Library array and therefore all use cwd; otherwise each config Library uses the
config directory. A CLI root override changes the final root used by either
config or CLI TOC. Merge before requiring root, TOC, exact `retail` and Libraries.

Defaults: text, `fail_on: error`, upstream default diagnostics. Slice 1A has no
WoW-check option.

Diagnostic settings use [strict Emmy admission and canonical semantics](EMMYLUA_UPSTREAM.md#diagnostic-configuration).
`fail_on` changes exit status only, not collected diagnostics or analyzer reuse.

The frontend resolves caller-facing path descriptors with the rules above, then
calls one public Service operation. It does not open addon/Library source, compute
path authority, parse TOC/XML, construct snapshots/lower requests or attach
limitations.

<a id="check-results-and-command-status"></a>

## Result and exit

Text and JSON render the same typed result: scope/completeness, exact diagnostics
and located scoped limitations. Deterministic Service order:

- diagnostics: role, canonical path, byte start/end, code, message;
- limitations: code, optional location, message.

Use explicit sort keys in Service. Derived struct field order is not this
contract: the current lower diagnostic puts category/code/message before path,
and the limitation puts message before location. Optional locations sort absent
first, then by canonical path and byte start/end.

Public result contains no snapshot/analyzer/Project/report/proof identities.
Paths are relative to configured addon root; text may show `root/path`, JSON keeps
relative identity. Absolute/path-authority data never enters output.

Service classifies failures and limitations under the
[scope table](IMPLEMENTATION.md#requested-scope-limitations). Main Lua syntax/type
errors are findings, not invalid input. A partial result preserves independent
findings; it never becomes a clean result by discarding unavailable input.

| Exit | Meaning |
| --- | --- |
| `0` | complete, no enabled finding at/above threshold |
| `1` | finding at/above threshold |
| `2` | requested scope partial |
| `4` | internal/serialization/output failure |
| `64` | invalid input/usage/resource refusal |
| `130` | observed cancellation |

Precedence: 64, 130, 4, 2, 1, 0. Rank Error > Warning > Information > Hint
explicitly; visible `Unknown` meets every threshold. Completeness and an empty
report have the [scope-relative meaning](WOW_LINTER.md#7-useful-output-and-updates)
defined for the requested checks and diagnostic configuration.

Encode the borrowed typed result once into a buffer capped at 32 MiB, including
escaped bytes and the final newline, then write it. Reuse the check-before-append
buffer algorithm in [local response](../crates/wow-service/src/local/response.rs),
not `LocalOperationResult` or its generic encoding-error mapping. The existing
[app JSON encoder](../apps/wow/src/json.rs) first counts encoded bytes and then
serializes a `Value` again; do not put the ordinary typed result through an
intermediate `Value` or a second serialization pass. Text and its newline use the
same append bound, not an unbounded string checked after growth.

Keep limit exhaustion distinguishable from serializer/I/O failure, for example
with a private writer limit-hit flag inspected by the renderer. Do not parse error
strings or classify every serialization error as oversized input. This needs a
small output helper, not a serialization framework or a rewrite of every protocol
adapter before the ordinary command can work.

Complete/partial result -> stdout. Admitted-format pre-result error -> stderr;
early grammar error -> text stderr. Public pre-result error codes are exactly
`invalid_input`, `cancelled` and `internal`. Optional `role` is `config`, `main` or
`library`. JSON error is one bounded object such as
`{"error":{"code":"invalid_input","message":"...","role":"library","root_index":0,"relative_path":"..."}}`.
Message <= 4 KiB; `root_index` only for Library and zero-based; paths role-relative.
Check cancellation after encoding/before first byte. Once output starts, no retry,
replacement or second envelope. Oversize result -> invalid input/64; serializer,
handler setup or I/O -> internal/4. Human text visibly escapes controls; typed/
JSON strings stay exact.

<a id="locations-and-document-updates"></a>
<a id="stdio-protocol-behavior"></a>
<a id="lsp-adapter"></a>

## LSP adapter

Use one LSP 3.18 stdio profile and the same public Service check as CLI. Preserve
exact unsaved text/version/root, UTF-8/UTF-16 conversion, standard empty/null and
protocol-only stdout. Never construct a lower request/second workspace or attach
limitations. Publish only exact physical/mapped user locations. LS debounce
(including interval) remains adapter state and does not alter analyzer
configuration/reuse. General Lua intelligence stays upstream.

### Document synchronization

The reviewed LSP source at `baf2db31` defines
[signed 32-bit `integer`](https://github.com/microsoft/language-server-protocol/blob/baf2db31b5fb314a8f358f9a839d3c66e9d98734/_specifications/lsp/3.18/specification.md#baseTypes),
[increasing, nonconsecutive document versions](https://github.com/microsoft/language-server-protocol/blob/baf2db31b5fb314a8f358f9a839d3c66e9d98734/_specifications/lsp/3.18/types/versionedTextDocumentIdentifier.md)
and [ordered change arrays](https://github.com/microsoft/language-server-protocol/blob/baf2db31b5fb314a8f358f9a839d3c66e9d98734/_specifications/lsp/3.18/textDocument/didChange.md).
Apply these at the wire boundary without changing Project's internal model into
a protocol schema:

| Input | Required handling |
| --- | --- |
| Version in `-2^31..=2^31-1`, including negative values and gaps | Preserve the signed client version through overlay/result binding; require an increase for a new change, not `previous + 1`. Internal generation counters remain separate; never cast a negative version to `u64`. |
| `contentChanges: []` | The protocol array has no nonempty constraint. For an admitted newer version, keep bytes unchanged and advance version under the same stale-result guards. Do not fabricate an edit or force resynchronization merely because the array is empty. |
| `[{"text":""}]` | Replace the whole document with empty text; this is not a no-op. Empty `didOpen.text` is also valid. |
| Several ranged/full changes | Apply in received order against each intermediate text, then adopt the final state atomically. |
| Deprecated `rangeLength` on a ranged change | Accept a valid LSP `uinteger`; `range` is authoritative. Validate then discard this redundant wire field before constructing `OverlayEdit`. |

Stale versions and invalid ranges remain failures; accepting valid empty input
does not bypass source bounds, cancellation or exact version matching. The current
Service rejects negative versions, Project refuses an empty edit array, and the
wire parser deserializes directly into strict `OverlayEdit`, rejecting
`rangeLength`. These remain open under T08 in the
[trace register](IMPLEMENTATION_TRACE_REGISTER.md#source-owners-behind-the-open-boundaries).
Empty source text is already accepted and must stay accepted.

<a id="mcp-adapter"></a>

## MCP adapter

Use one bounded MCP 2025-11-25 stdio profile. Advertise only implemented Service
operations: addon check, later direct source search and readable context. Accept
bounded permitted `_meta`; ignore valid unknown notifications without reply or
termination. Keep malformed/unsupported distinct. Do not expose profiles,
schema/effect IDs, Project/report identities, Context packs or receipts. JSON-RPC
IDs are correlation only.

## Configuration and output safety

Source/config remains user-owned/read-only. Project validates roots/files.
Output is bounded/deterministic. Findings survive limitations. Advertise
cancellation only at real cooperative checkpoints; native Emmy calls are not
preemptible.

<a id="completion-and-verification"></a>

## Observation

An adapter is complete only after the trace records exact command/framed sequence,
inputs and visible result. Build/Clippy/source review is not frontend acceptance.
