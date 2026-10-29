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

Public result contains no snapshot/analyzer/Project/report/proof identities.
Paths are relative to configured addon root; text may show `root/path`, JSON keeps
relative identity. Absolute/path-authority data never enters output.

Service classifies failures and limitations under the
[scope table](IMPLEMENTATION.md#requested-scope-limitations). Main Lua syntax/type
errors are findings, not invalid input. A partial result preserves independent
findings; it never becomes a clean result by discarding unavailable input.

| Exit | Meaning |
| --- | --- |
| `0` | complete, no finding at/above threshold |
| `1` | finding at/above threshold |
| `2` | requested scope partial |
| `4` | internal/serialization/output failure |
| `64` | invalid input/usage/resource refusal |
| `130` | observed cancellation |

Precedence: 64, 130, 4, 2, 1, 0. Rank Error > Warning > Information > Hint
explicitly; visible `Unknown` meets every threshold.

Pre-encode at most 32 MiB, then write once. Complete/partial result -> stdout.
Admitted-format pre-result error -> stderr; early grammar error -> text stderr.
Public pre-result error codes are exactly `invalid_input`, `cancelled` and
`internal`. Optional `role` is `config`, `main` or `library`. JSON error is one
bounded object such as
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
exact unsaved text/version/root, UTF-8/UTF-16 conversion, version gaps, stale
rejection, standard empty/null and protocol-only stdout. Never construct a lower
request/second workspace or attach limitations. Publish only exact physical/mapped
user locations. LS debounce (including interval) remains adapter state and does
not alter analyzer configuration/reuse. General Lua intelligence stays upstream.

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
