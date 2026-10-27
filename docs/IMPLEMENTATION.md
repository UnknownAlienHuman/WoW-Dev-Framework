# Implementation: finish the WoW linter

Read [current status](CURRENT_STATUS.md), [architecture](ARCHITECTURE.md),
[WoW behavior](WOW_LINTER.md) and the active
[queue item](REFACTOR_QUEUE.md). This guide defines durable owner boundaries and
handoffs. It is not a PR diary.

<a id="code-ownership"></a>

## Code ownership

| Responsibility | Owner |
| --- | --- |
| Blizzard checkout/ref/TOC/API input | `apps/wow-reference-builder` plus bounded source helpers |
| Normalized Blizzard facts/corrections | `wow-reference` |
| Ketho-style Lua declarations/maps | `wow-annotations`, `wow-ketho-literals` |
| Lua 5.1 parsing/types/diagnostics | `wow-emmy::ConfiguredAnalyzerSession` over pinned EmmyLua |
| Addon root, TOC selection, exact bytes, XML/maps/context | `wow-project` |
| Ordinary addon-check use case/result | one narrow public operation in `wow-service::addon_check`; lower exact-snapshot request/check stay internal |
| Current source index/search/context | one rebuildable direct `rusqlite`/FTS5 owner |
| CLI/LSP/MCP | thin adapters in `apps/wow` |

Do not add another analyzer, loader, result hierarchy, Project publication path,
proof layer, registry or compatibility-report adapter. Change the real producer
and reader together, then remove displaced owners.

<a id="addon-configuration-and-emmy-analysis"></a>
<a id="first-implementation-slice"></a>

## Addon configuration and Emmy analysis

### Slice 1A: one trusted physical-Lua path

```text
strict root + explicit TOC + retail + Library roots
  -> one public Service ordinary-check operation
  -> profile-free target/scope Project plan + admitted path authority
  -> requested-scope blockers + exact physical Main
  -> compiled-pin Project Main + per-root BlizzardUi Libraries
  -> existing ConfiguredAnalyzerSession / internal check_addon
  -> root-relative text/JSON result or typed pre-result error
```

No WoW rule, dependency universe, XML execution context, search index, Reference,
Rules, publication, graph, replay or source regeneration is required.

#### Target and retained plan identity

Project adds:

```text
AddonLoadTarget {
    flavor: retail,
    target_interface: Option<positive>,
    toc_context: Option<TocLoadContext>
}
+ projection scope
```

`ProjectLoadPlan` stores and validates this target/scope directly. Replace its
mandatory `ProfileIdentity` digest and positive-Interface admission; do not merely
change the reader argument. Legacy profile callers may adapt a real profile into
the target. The ordinary route never fabricates profile/source/report/schema/
correction/generation identity.

Slice 1A supplies `retail`, no target Interface and no condition context. Retain
missing/zero/positive source Interface; evaluate compatibility only when a
positive target exists. Invalid Interface syntax is invalid input.

#### Physical projection

Add one identity-bound `physical_lua` projection to the existing Project owner:

- parse the explicit TOC and selection metadata;
- capture selected direct `.lua` files under existing bounds;
- retain XML records and exact TOC spans, but do not open XML;
- emit one located `SourceSelectionIncomplete` blocker per selected deferred XML
  branch;
- preserve physical diagnostics when XML is missing or malformed;
- reject unsafe root/path/TOC bytes or unrecoverable TOC syntax;
- allow an empty `Project` snapshot. With deferred XML it is partial; with no
  selected language unit/blocker it is complete. Never add a dummy source.

Full XML expansion is Slice 1B, not a second loader.

#### Metadata and requested-scope limitations

T23 metadata-by-effect admission is a Slice 1A prerequisite. Display/localized,
custom `X-*` and known non-selection metadata remain Project observations.
Malformed keys or selection/environment/callback-changing directives remain
invalid or located.

The public standalone mapper belongs in `wow-service::addon_check` to avoid a
Project -> Service dependency:

```text
ProjectLoadPlan + AddonCheckScope
  -> Vec<AddonCheckLimitation>
```

For `LuaTypes`:

| Source state | Public outcome |
| --- | --- |
| missing/unreadable addon root or explicit TOC; unsafe path/bytes; invalid TOC encoding/NUL; unrecoverable syntax; config/source/Library budget refusal | pre-result `invalid_input` |
| safely parsed TOC names a missing direct Lua | located `SourceSelectionIncomplete`, continue, `Partial` |
| required dependency unresolved; included condition unresolved; selected XML deferred; recoverable file-kind/selection uncertainty | located scope blocker, `Partial` |
| optional dependency absent; repeated TOC occurrence; Interface without positive target; `LoadWith`; unused lifecycle/display/custom metadata | Project observation, no limitation |
| affected file uses chunk-owned top-level `...` or selected context changes typing | located context limitation, `Partial` |
| malformed caller Library/config/source | pre-result `invalid_input` with structured origin |
| native panic/poisoning or impossible validated-owner mismatch | `internal` |

Do not derive direct completeness from global `blocks_complete()` or
`external_files_complete()`. Only a fact required by the requested scope enters
`AddonCheckLimitation`.

#### Safe roots and selected-path authority

The frontend resolves path bases lexically for display. Project is the trust
boundary and exposes one narrow preparation/config entry wrapping crate-private
component-wise `open_workspace`. Slice 1A must not use legacy ambient
`ProjectInputDirectory::open`.

Each opened root retains:

- request-local `InputRootId = role + ordinal` (`main` or zero-based Library
  index), used only for routing errors/display;
- an original absolute descriptor for display;
- a request-local, nonserialized normalized absolute authority descriptor;
- the held no-follow directory capability.

Reject root descriptor text above 32 KiB. Admit local Unix absolute and Windows
drive-absolute roots; unsupported UNC/device prefixes are invalid input.

For every unique admitted Lua path derive:

```text
InputPathAuthority = normalized absolute root descriptor
                   + canonical relative path
```

Slash-normalize and Unicode-lowercase using the same conservative portable policy
as Project case-collision admission. This intentionally collapses case-only
spellings, but is selected-path authority—not inode/file-ID, hard-link, content or
filesystem-canonical identity. Reject duplicate authorities across Main/Libraries.
Do not reject root ancestry alone. Nested roots are valid when selected authority
sets are disjoint; equal relative paths in distinct roots are valid when joined
authorities differ. Repeated TOC records for one Main inventory entry are not a
second authority.

Each Library root must yield at least one admitted Lua. Reject duplicate resolved
root authority keys and duplicate whole snapshot IDs. Fixed aggregate ceilings:
64 Library roots, 8,192 Library files, 256 MiB, in addition to existing per-root
limits. Expose no budget flags.

#### Exact snapshots and paths

Use:

- `EmmyBackendIdentity::compiled_pinned()` once;
- Main universe `Project`, existing Main ceilings;
- one separate universe `BlizzardUi` snapshot per Library root under shared
  Library ceilings;
- canonical root-relative paths in snapshots, diagnostics and locations.

The frontend may join its display root in human output. Ambient absolute paths and
path-authority keys never enter snapshot/result identity or JSON.

#### One public Service operation

The ordinary operation owns the entire trusted assembly:

1. receive merged strict config and display-root descriptors;
2. ask Project to open roots and produce target/scope plan, admitted Main,
   Libraries and path-authority records;
3. validate plan/Main and authority consistency;
4. derive requested-scope blockers;
5. build exact snapshots and request-local snapshot/root mapping;
6. open one configured Emmy owner, collect requirements/diagnostics and call the
   internal lower `check_addon`;
7. return one result or one operation-specific error.

CLI/LSP/MCP never open source, construct `AddonCheckRequest`, call
`with_input_limitations`, build snapshots or attach blockers. Keep the lower
request/check crate-internal after callers move. This is a direct use-case
function, not a generic backend.

#### Structured failures

Carry stable code, source role, snapshot/`InputRootId`, role-relative path and
bounded message through Project -> Emmy -> configured session -> Service.
`LibraryHealthFailed` must retain Library snapshot ID/path so Service can map it
to the request-local root. Never parse messages or synthetic workspace paths.

Use a small ordinary-check error with exactly:

```text
invalid_input | cancelled | internal
message <= 4 KiB
role? = main | library | config
a zero-based root_index only for library
relative_path? relative to that role's configured root
```

File-specific Project read/UTF-8/NUL/language/digest/length/admission failures
carry root/path. Source/workspace/diagnostic/result-size refusal caused by caller
input is `invalid_input`; analyzer invariants, serialization defects,
cancellation-handler setup and output I/O are `internal`. Do not turn global
`ServiceError` into a provenance platform.

#### Same-owner chunk requirement

Remove the unconditional chunk limitation and hard-coded `Partial`. Add one
bounded collector on the same `ConfiguredAnalyzerSession` and registered Main AST:

- walk expression nodes;
- match `LuaLiteralExpr` with `LuaLiteralToken::Dots`;
- reject matches with an enclosing `LuaClosureExpr` before chunk root;
- retain the lowest-starting exact `{ path, content_sha256, span }` per file;
- never raw-scan text or launch another parser/session.

Service validates each record against exact Main and limits only affected files.
Helper injection remains Slice 1B.

<a id="ordinary-configuration"></a>

### Ordinary configuration

Slice 1A config fields are only:

- `root`;
- explicit canonical relative `.toc`;
- exact `flavor: retail`;
- nonempty `library` array;
- optional `emmy_diagnostics`, `format`, `fail_on`.

Read at most one explicitly named config through Project's no-follow helper, capped
at 256 KiB. Reuse duplicate-key/depth-bounded `StrictJson`, then decode an outer
`deny_unknown_fields` DTO. No implicit discovery.

Pinned `EmmyrcDiagnostic` is permissive and silently drops invalid
`globalsRegex`. Prevalidate exactly the analysis-relevant pinned nested keys
(`disable`, `enable`, `globals`, `globalsRegex`, `severity`, `enables`) and compile
every regex. Reject unknown keys and invalid expressions. Reject
`diagnosticInterval`: at the pinned revision only EmmyLua LS uses it for debounce;
code analysis does not. It must not become a no-op CLI field or owner-compatibility
input. No `.emmyrc` discovery/fallback.

CLI scalar values replace config scalars. Repeated CLI `--library` replaces the
configured array. Defaults: text, `fail_on: error`, upstream default diagnostics.
If config fails, an explicitly admitted CLI `--format` selects the error format;
otherwise use text. Slice 1A exposes no WoW `checks` option and runs zero WoW
rules.

<a id="load-units-and-addon-context"></a>
<a id="selected-acquisition"></a>
<a id="exact-load-context-handoff"></a>
<a id="physical-and-virtual-main"></a>
<a id="implicit-binding-documents"></a>
<a id="addon-chunk-context"></a>

### Later context slices

1. required/available optional dependencies and exact selected order;
2. source-projectable XML virtual units and exact projection;
3. ordinary `(addonName, addonTable)` only for affected top-level chunk ellipsis;
4. reviewed handler parameters and coherent updates;
5. channel, `LoadWith`, implicit bindings, target Interface and secure/special
   contexts;
6. generated metadata and explicitly selected WoW checks.

Missing context limits only affected units/conclusions. Path-sorted snapshots are
analysis identity, not WoW execution order; keep exact order/context in a separate
validated Project value.

<a id="one-direct-check-operation"></a>

### Mature direct operation

For the selected scope: resolve strict input, acquire exact bytes, validate path
authority, prepare only required context, derive only required blockers, build
exact snapshots/maps, reuse/open one compatible Emmy owner, collect only requested
facts/rules, and return one result. Optional component health is not a global gate.

<a id="updates"></a>

### Updates

Reuse one configured owner only while exact Main, canonical Library set, virtual
snapshot and analysis-relevant diagnostic configuration match. LS debounce is not
analyzer identity. A delta is allowed only while selected membership, target,
projection scope, context and admitted authority set are unchanged.
TOC/XML/dependency/channel/context/Library/config changes rebuild cold.
Delete/rename removes declarations, virtual maps, diagnostics and index rows
together. Failed/cancelled/stale work never publishes.

<a id="native-annotation-generation"></a>
<a id="preserve-the-existing-pipeline"></a>
<a id="one-native-generation"></a>

## Native annotation generation

```text
wow-reference-builder
  -> wow-reference native model/corrections
  -> wow-annotations + wow-ketho-literals
  -> ordinary .lua + optional source-report.json
```

Read [KETHO_RUST_PORT.md](KETHO_RUST_PORT.md). Preserve environment,
`MayReturnNothing`, `NilableContents`, stride, mixin, restriction/Secret and source
map semantics or report conservative loss. Never execute donor Lua. Keep one
optional typed manifest; plain Lua remains valid for typing. No second producer,
converter or certification envelope.

<a id="outputs-and-corrections"></a>
<a id="annotation-projection-rules"></a>
<a id="manifest-and-ordinary-reader"></a>

### Outputs and corrections

Emit inert bounded Lua once. Corrections are explicit/source-guarded and separate
from final emission. Preserve origin, replacement, reason, generated span and
conflicts. An accepted correction does not prove emission. The ordinary Library
reader consumes successful output directly.

<a id="blizzard-ui-index-search-and-context"></a>
<a id="small-navigation-operations"></a>

## Blizzard UI index, search and context

Checking works without an index. Later use one rebuildable SQLite database with
selected source identity, exact bodies, symbols/spans and only consumed TOC/XML/
type/call relations. FTS indexes source bodies. A later process queries without
Project/Emmy/Graph replay. Updates replace/delete rows transactionally; context
uses exact indexed text or a disk reread only after content identity matches.
Remove nested SQLite->hex->JSON, replay recapture, generic Store lifecycle,
Search federation and Context packs after their final readers move.

<a id="typed-source-relationships"></a>

## Typed source relationships

Keep only relations consumed by diagnostics/search/navigation: callable origin,
dependency/trigger/order, XML include/template/script/binding and selected type/
call/reference. Preserve exact location, ambiguity and incomplete coverage. Do not
round-trip fixed Rust facts through pack/plan/matcher/proof layers.

<a id="results-and-interfaces"></a>

## Results and interfaces

Addon result = diagnostics/findings + completeness + located scoped limitations;
no snapshot/analysis/Project/report/proof receipts. Frontends serialize it once.

Deterministic order:

- diagnostics: role, path, byte start/end, code, message;
- limitations: code, optional location, message.

Threshold rank is explicitly Error > Warning > Information > Hint. Preserve
`Unknown`; conservatively treat it as meeting every threshold. Partial exit `2`
precedes threshold exit `1`.

Pre-encode escaped text/JSON into at most 32 MiB. Complete/partial results go once
to stdout. After encoding, check cancellation before the first byte. A stop there
returns only cancellation/130. Once writing begins, finish the single envelope or
return internal/4; never retry/replace/append. Result-size refusal is
invalid_input/64. Serializer or I/O failure is internal/4. Human output visibly
escapes control characters; typed/JSON strings remain exact.

CLI/LSP/MCP are thin adapters; general Lua editor features remain upstream Emmy.
See [FRONTENDS.md](FRONTENDS.md).

<a id="completion-and-verification"></a>

## Removal and verification

Remove compatibility/certification reports, publication/replay wrappers, generic
results, matcher/proof graph paths and lifecycle administration only with their
last real caller. Do not preserve aliases or rename the platform.

For every change trace:

```text
user input -> producer -> handoff -> reader -> visible result
```

Record exact observations in the trace register. Build/Clippy/source review is not
product observation.
