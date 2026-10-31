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

Strict raw-settings admission remains open. The existing typed configuration
constructor is not a validator: after lenient upstream deserialization, it cannot
recover discarded unknown keys. Wire the shared Emmy admission boundary before
that information is lost; do not duplicate it in each frontend.

[`EmmyAnalysisConfiguration::equivalent`](../crates/wow-emmy/src/configuration.rs)
compares analysis fields directly, without JSON or a new persisted fingerprint:
`enable` exactly; `disable`, `enables`, `globals` and `globalsRegex` by set
membership; severity by diagnostic code and upstream severity value. This matches
the pinned [set/map and regex configuration](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/diagnostic/lua_diagnostic_config.rs).
Order and duplicates do not change those list semantics. Identical lists take the
allocation-free equality path; other comparisons use temporary borrowed sets.
The retained caller settings and public configuration shape remain unchanged.

`diagnosticInterval` does not enter equality. Thus default diagnostics and decoded
`{}` compare equally despite upstream's `Some(500)` versus `None`; reordered or
duplicated set-like entries also reuse the owner. A changed analysis field still
requires a cold owner. Ignoring an LS-only interval for reuse does not admit that
key into the ordinary checker; strict admission must reject it as specified above.

The global `enable` switch is distinct from the `disable` code set. At the pinned
[donor implementation](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/diagnostic/lua_diagnostic.rs#L52),
`enable: false` makes `diagnose_file` return `None`; disabling individual codes
filters findings while diagnostic collection remains enabled. The existing
[`collect_registered`](../crates/wow-emmy/src/syntax/analyzer.rs) reads the active
engine configuration and normalizes `None` only under explicit global-off, after
checking registration and cancellation. Every other non-cancelled `None` remains
`DiagnosticsUnavailable`. Physical and virtual diagnostics use this same path;
do not add a second collector or interpret every refusal as an empty success.

With global diagnostics off, an empty report records suppression, not a clean
Lua check. Still validate exact Main and Library registration/health, preserve
cancellation, and retain Project's missing-source or scope limitations. Successful
Library parsing establishes only parse health of the supplied files, not Blizzard
API/type coverage; missing or partial Library authority remains unknown, never
proof of API absence.

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

## Resource limits and cancellation

`DiagnosticBudgetExceeded` is a resource refusal, not analyzer unavailability.
Preserve its typed cause and source origin through the configured owner to
Service; the ordinary CLI maps it to `invalid_input`/64 under the existing
[error contract](FRONTENDS.md#check-results-and-command-status). A count or
code/message-size refusal must not poison a healthy session, truncate a diagnostic
to fit, or turn an incomplete collection into a successful empty result. The
current `syntax_error` catch-all maps this case to `NativeFailure`, marks the
session unhealthy, and the lower Service maps it to `ComponentUnavailable`.
Fix that existing chain, not a parallel error framework.

The [collector](../crates/wow-emmy/src/syntax/analyzer.rs) checks adapter count and
message limits after upstream returns the file's diagnostic vector. These are
retained-report admission limits, not a bound on native peak allocation. Snapshot
byte limits and frontend encoded-output limits likewise do not cap the combined
heap of source copies, ASTs, inference and intermediate reports.

Cancellation is cooperative around framework stages. The pinned synchronous
native call is not preemptible; check before/after and do not claim a hard deadline
or bounded cancellation latency. A stop token does not make active native work
interruptible. Keep bounded framework loops and honest refusal separate from any
unmeasured native resource guarantee.

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
