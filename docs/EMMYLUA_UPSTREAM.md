# EmmyLua integration guide

This guide records the durable contract with the pinned EmmyLua donor. Delivery
state lives in [CURRENT_STATUS.md](CURRENT_STATUS.md); implementation order lives
in [REFACTOR_QUEUE.md](REFACTOR_QUEUE.md).

<a id="current-checkpoint"></a>
<a id="pinned-upstream-contract"></a>

## Pinned donor

The workspace pins `emmylua_code_analysis` and `emmylua_parser` to revision
`aaaca68425d9362876228649b0b8d92f07654daa` of
[EmmyLua Analyzer Rust](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/tree/aaaca68425d9362876228649b0b8d92f07654daa).
Read behavior at that revision rather than a moving default branch.

EmmyLua owns Lua parsing, type inference, semantic identities and language
diagnostics. Do not build a parallel parser/type checker or import private
`emmylua_ls` handlers as an SDK.

## Source roles and engine setup

Use one owner with explicit roles:

- **Main** — selected addon source diagnosed for the user;
- **Library** — Blizzard/addon declarations used to resolve Main;
- **Virtual Main** — mapped selected XML/binding Lua after exact projection;
- **STD** — upstream Lua declarations, not user source or WoW API authority.

Set Lua 5.1 explicitly and call `init_std_lib(None)` once. Workspace roots do not
discover files automatically. STD typing is not proof that the client exposes a
host API.

<a id="configured-analyzer-session"></a>

## Configured owner

`ConfiguredAnalyzerSession` owns exact Main, explicit Libraries, optional virtual
snapshot and diagnostic configuration. Diagnostics-only collection does not run
optional semantic collectors.

Reuse requires:

- exact Main snapshot ID;
- same Library snapshot-ID set;
- exact optional virtual snapshot ID;
- equivalent **analysis-relevant** diagnostic configuration;
- healthy owner.

Library snapshots are sorted by exact ID before registration and compared as the
same set; caller argument order is irrelevant. Project generation is not part of
diagnostic-owner construction/reuse and is supplied only for explicit virtual
semantic reports.

For the direct path use `EmmyBackendIdentity::compiled_pinned()`, Main universe
`Project`, one `BlizzardUi` Library snapshot per root and retained limits. Snapshot
identity is not selected-host-path authority: Project/Service rejects duplicate
nonserialized normalized root+relative-path authorities before Emmy opens. Do not
infer authority from bytes, snapshot ID or inode/file ID.

A Library parse/health failure is caller invalid input. Carry source role,
Library snapshot ID and relative file through `ConfiguredSessionError`; Service
maps the ID to request-local root ordinal. The current
`LibraryHealthFailed -> NativeFailure -> ComponentUnavailable` path is a defect.
Never recover origin by message parsing or synthetic analyzer paths.

## Diagnostic configuration

Pinned `EmmyrcDiagnostic` lacks `deny_unknown_fields`; `LuaDiagnosticConfig::new`
logs then drops invalid `globalsRegex`. The direct checker therefore:

1. prevalidates exactly analysis-relevant keys: `disable`, `enable`, `globals`,
   `globalsRegex`, `severity`, `enables`;
2. compiles every supplied regex;
3. rejects unknown keys/invalid expressions;
4. rejects `diagnosticInterval` — at this revision it is consumed only by
   `emmylua_ls` debounce, not code analysis;
5. uses no `.emmyrc` discovery or lenient fallback.

`EmmyAnalysisConfiguration::equivalent` must compare a canonical semantic
projection, not raw serialized `EmmyrcDiagnostic`. Preserve `enable`; convert
`disable` and `enables` to sorted/deduplicated diagnostic-code sets; sort/dedup
`globals` and `globalsRegex`; sort severity entries by diagnostic code; omit
`diagnosticInterval`. Pinned analysis turns the first three list families into
sets/OR-matched regexes, so order and duplicates have no semantic effect. Thus
omitted diagnostics, `{}`, reordered lists and duplicate entries reuse the same
owner when analysis semantics are identical, while a real analysis change forces
a cold owner. LS debounce remains frontend state.

Keep separate:

1. Emmy diagnostic selection/severity;
2. selected WoW checks and semantic fact requests;
3. frontend `fail_on`, which changes exit only.

Do not gate findings through a framework allowlist. Rank Error > Warning >
Information > Hint explicitly at the frontend; preserved `Unknown` conservatively
meets every threshold.

<a id="diagnostics-and-source-locations"></a>

## Diagnostics and coordinates

Preserve upstream code, exact bounded message, configured severity, source path/
content identity and half-open UTF-8 byte span. Pinned columns count Unicode scalar
values, not negotiated LSP units; convert against exact captured text, then the
overlay converts UTF-8/UTF-16. Preserve CRLF/final empty lines and never apply old
ranges to changed text.

Library/STD diagnostics are input health, not Main findings. Virtual diagnostics
become public only after exact original-source mapping; synthetic workspace paths
are never user locations.

<a id="addon-chunk-context"></a>

## Addon chunk context

Pinned Emmy infers unbound chunk `...` as homogeneous `any...`; registration does
not provide WoW chunk arguments. Blizzard documents ordinary external chunks as
receiving `(addonName, addonTable)`; `AllowAddOnTableAccess` is separate later
retrieval permission.

For supported chunks, later Slice 1B creates one analysis-only stable table per
addon, rewrites only chunk-owned ellipsis in memory, preserves exact maps and
hides helpers. Nested function varargs, strings/comments, free inline scripts,
handlers, bindings, secure/bootstrap and suppressed-table contexts are distinct.

Slice 1A adds only a bounded requirement collector on the same configured owner:
walk registered Main expression nodes, match `LuaLiteralExpr`/
`LuaLiteralToken::Dots`, exclude matches with an enclosing `LuaClosureExpr`, and
return the lowest-starting exact `{path, content_sha256, span}` per file. Named,
local and anonymous functions all contain `LuaClosureExpr` at the pinned parser
revision. Do not raw-scan text or launch another parse/analysis. Service verifies
records against exact Main and limits only affected files.

<a id="updates-and-cancellation"></a>

## Updates

Use conservative semantic rebuild until narrower invalidation is observed.
Reuse only when exact snapshot/config semantics match. TOC/XML/dependency/channel/
context/Library/analysis-config changes rebuild cold. LS debounce is not analyzer
identity. Remove declarations, virtual units, maps and diagnostics together;
failed/cancelled/stale work never publishes.

Cancellation is cooperative around framework stages. The pinned synchronous
native call is not preemptible; check before/after and do not claim more.

<a id="editor-boundary"></a>

## Editor boundary

General completion, hover, definition, references, rename, signature help and
symbols remain upstream EmmyLua. Framework LSP/MCP expose project/check/search/
context through the same Service owners as CLI. The overlay owns exact unsaved
text/version/root/coordinate state and rejects stale publication. LS debounce is
adapter state and does not enter analysis configuration.

<a id="verification-boundary"></a>

## Verification boundary

Source inspection proves contracts, not a run. Exact generated-Library/addon/
editor observations belong in the trace register.
