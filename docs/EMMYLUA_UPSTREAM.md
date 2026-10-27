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

Use one analysis owner with explicit roles:

- **Main** — selected addon source diagnosed for the user;
- **Library** — Blizzard/addon declarations used to resolve Main;
- **Virtual Main** — mapped selected XML/binding Lua, still user source after
  exact projection;
- **STD** — upstream Lua declarations, neither user source nor WoW API authority.

Set Lua 5.1 explicitly and call `init_std_lib(None)` once in the retained owner.
`EmmyLuaAnalysis::new()` does not load STD and workspace roots do not discover
files automatically.

STD typing is not proof that the WoW client exposes a host API. Availability
checks require the resolved symbol plus selected WoW facts.

<a id="configured-analyzer-session"></a>

## Configured owner

`wow-emmy::ConfiguredAnalyzerSession` owns exact caller-supplied Main, explicit
Library and optional virtual snapshots with caller diagnostic configuration.
Diagnostics-only collection does not run optional semantic collectors.

Owner compatibility requires:

- exact Main snapshot ID;
- the same Library snapshot-ID set;
- exact optional virtual snapshot ID;
- equivalent diagnostic configuration;
- a healthy retained owner.

`RegisteredMemberAnalysis::new` sorts Library snapshots by exact snapshot ID
before assigning roots and registering files. `compatible` sorts the same IDs
before comparison. Caller Library argument order is therefore deliberately
irrelevant; do not add another ordering work item.

A Project generation is **not** part of configured owner construction or reuse.
It is supplied only to `semantic_reports` when exact virtual semantic facts are
requested. Ordinary diagnostics over an exact virtual snapshot must not fabricate
or expose a Project publication identity.

## Diagnostic configuration

Decode explicit `emmy_diagnostics` as the pinned `EmmyrcDiagnostic` type. Invalid
values are configuration errors; do not use a lenient discovery/fallback loader.
Keep separate:

1. Emmy diagnostic selection/severity;
2. selected WoW checks and their semantic fact requests;
3. frontend `fail_on`, which changes exit status but does not delete findings.

Do not gate diagnostics through a framework code allowlist.

<a id="diagnostics-and-source-locations"></a>

## Diagnostics and coordinates

Preserve for every configured upstream finding:

- string/numeric/missing upstream code as text;
- exact bounded message;
- configured upstream severity;
- source path and content identity;
- half-open UTF-8 byte span.

Pinned Emmy line/column conversion counts Unicode scalar values, not negotiated
LSP code units. The framework converts to UTF-8 bytes against exact captured text,
then the overlay converts to UTF-8/UTF-16 editor positions. Preserve CRLF and final
empty-line behavior. Never apply a range to changed text.

Library/STD diagnostics are input-health information, not Main findings. Virtual
diagnostics become public only after exact mapping to the original XML/binding
source; synthetic workspace paths are not user locations.

<a id="addon-chunk-context"></a>

## Addon chunk context

Pinned Emmy infers an unbound chunk `...` as homogeneous `any...`; workspace
registration alone does not provide WoW chunk arguments.

Pinned Blizzard documentation supports ordinary external chunks receiving
`(addonName, addonTable)`. `AllowAddOnTableAccess` separately controls later table
retrieval. For supported chunks:

1. create one analysis-only stable table per addon;
2. identify only top-level chunk-owned ellipsis expressions with the Emmy AST;
3. replace those expressions in memory with a helper call;
4. preserve exact source maps and exclude helpers from user diagnostics/facts.

Do not rewrite nested function varargs, strings or comments. External chunks,
free inline XML scripts, reviewed handlers, binding callbacks, secure/bootstrap
contexts and `SuppressLocalTableRef` are distinct.

Context is required only where selected code consumes it. A `LuaTypes` run over
files that do not use top-level ellipsis must not be globally partial merely
because the ordinary helper is not yet implemented.

<a id="updates-and-cancellation"></a>

## Updates

The upstream bulk update API removes/reindexes touched file IDs but does not prove
that every unchanged semantic consumer of a deleted declaration was invalidated.
Use conservative semantic rebuild until narrower invalidation is observed correct.

Reuse `ConfiguredAnalyzerSession` only when its exact compatibility contract
holds. TOC/XML/dependency/channel/context/Library/config changes rebuild the
selection and owner cold. Remove declarations, virtual units, maps and diagnostics
together. Failed, cancelled or stale work never publishes.

Cancellation is cooperative around framework stages. The pinned synchronous
native call is not preemptible; check before and after it and do not claim more.

<a id="editor-boundary"></a>

## Editor boundary

General completion, hover, definition, references, rename, signature help and
symbols remain upstream EmmyLua. Framework LSP/MCP adapters expose WoW project,
check, search and context operations through the same owners as CLI.

The current overlay owns exact unsaved text, document version and negotiated
coordinate conversion. The target passes that override into the shared selected
project, accepts increasing nonconsecutive versions and rejects stale publication.

Existing same-file navigation helpers may be retained only where a surviving
WoW-specific consumer needs them; they are not whole-project language-server
coverage.

<a id="verification-boundary"></a>

## Verification boundary

Source inspection proves the contracts above. It does not prove a generated
Library, addon command or editor workflow has run. Exact observations belong in
[the trace register](IMPLEMENTATION_TRACE_REGISTER.md).
