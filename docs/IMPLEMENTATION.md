# Implementation: finish the WoW linter

This guide owns source admission and Service assembly for the ordinary checker.
[Current status](CURRENT_STATUS.md) says what exists; the
[queue](REFACTOR_QUEUE.md) orders implementation. Command/config/output semantics
live in [FRONTENDS.md](FRONTENDS.md); engine-specific behavior lives in
[EMMYLUA_UPSTREAM.md](EMMYLUA_UPSTREAM.md). Read the relevant sections, not every
audit. The contract below is a target until current status marks it delivered.

<a id="code-ownership"></a>

## Code ownership

| Responsibility | Owner |
| --- | --- |
| Blizzard checkout/ref/TOC/API input | `apps/wow-reference-builder` plus bounded source helpers |
| Normalized Blizzard facts/corrections | `wow-reference` |
| Ketho-style Lua declarations/maps | `wow-annotations`, `wow-ketho-literals` |
| Lua 5.1 parsing/types/diagnostics | `wow-emmy::ConfiguredAnalyzerSession` over pinned EmmyLua |
| Addon root, TOC selection, exact bytes, XML/maps/context | `wow-project` |
| Ordinary addon-check use case/result | one narrow public operation in `wow-service::addon_check`; lower exact-snapshot request/check internal |
| Current source index/search/context | one rebuildable direct `rusqlite`/FTS5 owner |
| CLI/LSP/MCP | thin adapters in `apps/wow` |

These are responsibilities, not a requirement for a new type, file or layer at
each arrow. The [repository map](REPOSITORY_MAP.md) locates existing owners.

## Small implementation decisions

Reuse a suitable public donor/library API or existing domain function before
writing a substitute. Choose the smallest connected change that preserves source
bounds, typed failures and locations; fewer lines are not an excuse to lose them.

| Decision | Default; reason to add more |
| --- | --- |
| Function, type or module? | Concrete function and cohesive private helpers. A type protects an invariant or carries consumed data; a module owns a coherent responsibility. No wrapper/builder/getter set per contract name or line-count quota. |
| Trait, generic backend or registry? | Concrete owners and explicit matches for finite cases. Abstract actual interchangeable behavior or a required library boundary, not hypothetical providers or mockability. Repeated syntax alone is not shared domain policy. |
| Where to validate? | Admit untrusted input at its owning boundary, then pass the typed value. Recheck changed bytes, authority or binding, not unchanged admission at every layer. Wire DTOs belong at protocol boundaries. No silent fallback. |
| What state to retain? | Request-local values; move/borrow or share exact source through existing ownership. No global cache/lock/queue for one call. Retain sessions for real repeated callers; prefer conservative rebuild. |
| Errors and internal transport? | Typed cause/origin, converted at the owner/frontend boundary. No message parsing, generic success envelopes or JSON round-trips. Keep hashes needed by snapshots/maps/reuse, not new receipts per step. |
| Dependency or optimization? | Existing suitable primitive first; add a dependency for a concrete missing capability. No second parser to avoid a library. Remove redundant reads/parses/copies; add caching/concurrency/incremental machinery for a demonstrated workflow need. |

### Reuse behavior, not obsolete orchestration

If a useful algorithm is entangled with unrelated prerequisites, extract a narrow
helper within its owning crate and route the real callers through it. Do not copy
and fork the algorithm, or add mode flags/dummy inputs to make the old coordinator
pretend its prerequisites exist. For example, the current
[`ServiceBackend`](../crates/wow-service/src/backend.rs) deals in published
contexts/generations; it is not the reuse boundary for an ordinary addon check.
Reuse Project admission and Emmy analysis below it, as the slice specifies.

Update affected internal signatures, re-exports and callers in the same change
rather than introducing a second version to keep the old call shape compiling.
Keep an adapter only for a real remaining caller, translating real input; remove
it with that caller. Existing serialization or fixtures alone do not establish a
compatibility promise. Preserve genuinely supported user input and never rewrite
user files to accommodate an internal refactor.

Keep the call chain readable: neither a giant function nor forwarding wrappers
at every arrow. Comments explain invariants and non-obvious donor behavior, not
the surrounding syntax. No unused extension points or placeholder success.

<a id="addon-configuration-and-emmy-analysis"></a>
<a id="first-implementation-slice"></a>

## Addon configuration and Emmy analysis

### Slice 1A: one trusted physical-Lua path

```text
strict root + explicit TOC + retail + Library roots
  -> one public Service ordinary-check operation
  -> existing Project owner: physical plan, admitted bytes and path authority
  -> exact Main/Libraries + requested-scope blockers
  -> internal check_addon: one configured Emmy owner
  -> same-owner chunk requirements + configured diagnostics
  -> root-relative text/JSON result or typed pre-result error
```

No WoW rule, dependency universe, XML execution context, search index, Reference,
Rules, publication, graph, replay or source regeneration is required. Stages are
ordinary calls within the existing owners, not independently registered services.

#### Target and retained plan identity

Project adds `AddonLoadTarget { flavor: retail, target_interface: Option<positive>,
toc_context: Option<TocLoadContext> } + projection scope` and binds it directly
into `ProjectLoadPlan`. Replace mandatory `ProfileIdentity` digest and positive-
Interface admission; changing only the reader argument leaves the dependency.
Legacy callers may adapt a real profile. The ordinary route never fabricates
profile/source/report/schema/correction/generation identity.

Slice 1A supplies `retail`, no target Interface/context. Retain missing/zero/
positive source Interface; evaluate compatibility only with a positive target.
Invalid Interface syntax is invalid input. Existing later-context fields do not
justify implementing their selection behavior in this slice.

#### Physical projection

Add identity-bound `physical_lua` to the existing Project owner. Parse the explicit
TOC and selection metadata; capture direct selected `.lua` under retained bounds.
Retain XML records/exact TOC spans without opening XML; emit located
`SourceSelectionIncomplete` per selected deferred XML branch. Independent physical
diagnostics survive missing/malformed deferred XML. Reject unsafe root/path/TOC
bytes or unrecoverable syntax. Full XML expansion is Slice 1B, not a second loader.

Allow empty `Project` Main: complete with no selected unit/blocker, partial with
deferred selected input. Never add dummy source. Change both existing nonempty
gates: `disk.rs::read_lua_inventory` and `load/mod.rs::read_toc_source`. Retain the
nonempty-Lua requirement for each Library root.

<a id="requested-scope-limitations"></a>

#### Metadata and requested-scope limitations

T23 metadata-by-effect admission is a Slice 1A prerequisite. Malformed TOC metadata
syntax or an invalid key is `invalid_input`. Unsupported selection/environment/
callback directives limit only the affected requested scope. Cosmetic, localized,
custom `X-*` and known non-selection metadata stay Project observations. Preserve
unrecognized package keys/spans; an unfamiliar name alone is not evidence of an
execution effect or a dependency of the requested check.

Service owns the standalone `ProjectLoadPlan + AddonCheckScope -> limitations`
mapping; Project must not depend on Service. For `LuaTypes`:

| Source state | Outcome |
| --- | --- |
| missing/unreadable addon root, explicit TOC or Library root; unsafe path/bytes; invalid TOC encoding/NUL; unrecoverable TOC syntax; config/source/Library budget refusal | pre-result `invalid_input` with the selected role/path origin |
| safely parsed TOC names missing direct Lua; `NotFound` while safely traversing that selected child path or acquiring its file | located `SourceSelectionIncomplete` at the original TOC span, continue, `Partial` |
| selected child traversal/file acquisition fails because of a symlink/special file, permission, or other I/O error | pre-result `invalid_input` with the selected role/path origin; never classify every read failure as missing |
| required dependency, included condition, selected XML or recoverable selected-context uncertainty unresolved | located scope blocker, `Partial` |
| optional absence; repeated TOC occurrence; Interface without target; `LoadWith`; unused lifecycle/display/custom metadata | Project observation, no limitation |
| affected file uses chunk-owned top-level `...` or selected context changes typing | located context limitation, `Partial` |
| Main Lua syntax/type mistake | ordinary diagnostic, not an input-admission error |
| malformed Library/config or source encoding/NUL failure | pre-result `invalid_input` with structured origin |
| native panic/poisoning/impossible validated-owner mismatch | `internal` |

Do not derive completeness from global `blocks_complete()` or
`external_files_complete()`.

#### Safe roots and selected-path authority

Frontend resolves path bases lexically for display. Project is the trust boundary
and exposes narrow preparation/config entry points using crate-private component-
wise `open_workspace`; Slice 1A must not use ambient
`ProjectInputDirectory::open`.

Admit only host-native absolute roots: Unix absolute paths on Unix and drive-
absolute paths on Windows. Do not interpret a path string for another operating
system as a host path; UNC and device paths remain unsupported.

Each opened root retains request-local `InputRootId = role + ordinal`, original
display descriptor, nonserialized normalized absolute authority descriptor and
held no-follow capability. Reject descriptor text above 32 KiB.

For each unique admitted Lua:

```text
InputPathAuthority = normalized absolute root descriptor
                   + canonical relative path
```

Slash-normalize and Unicode-lowercase with Project's conservative portable case
policy. This is selected-path authority, not inode/file-ID/hard-link/content/
filesystem-canonical/snapshot identity. Reject duplicate authorities, not root
ancestry. Nested roots and equal relative paths are valid when joined authority
differs. Repeated TOC records are not duplicate files.

Each Library root must yield Lua. Reject duplicate root authority keys and
duplicate whole snapshot IDs. Fixed aggregate ceilings: 64 Library roots, 8,192
files, 256 MiB, plus existing per-root limits. No budget flags.

#### Exact snapshots and paths

Use `EmmyBackendIdentity::compiled_pinned()` once; Main universe `Project`; one
separate universe `BlizzardUi` snapshot per Library root; canonical role-relative
paths. Build from the admitted captured bytes, not an ambient reread to reconstruct
the input. Frontend may join display root for text. Absolute/path-authority data
never enters semantic identity or JSON.

#### One public Service operation

Frontend parses flags and merges config using
[the path/precedence rules](FRONTENDS.md#configuration-and-path-bases); reading the
explicit config uses Project's bounded no-follow helper. Frontends never open
addon/Library source, build snapshots, construct `AddonCheckRequest`, call
`with_input_limitations` or attach blockers.

The Service operation prepares sources through Project, validates the plan/Main/
authority bindings, maps requested-scope blockers, and builds exact snapshots plus
a request-local snapshot/root map. It then invokes the internal checker and maps
its typed outcome to one public result/error. These are local operations, not
separate preparation, validation and execution frameworks.

**Use one analyzer construction site.** The existing
[`check_addon`](../crates/wow-service/src/addon_check.rs) already calls
`ConfiguredAnalyzerSession::open`. Extend that path to collect chunk requirements
and diagnostics from that same session; the outer source-preparation operation
must not open another session before calling it. Keep the request/check internal
after callers move. A later retained caller may refactor construction away from
collection, but must still supply exactly one owner, not a parallel checker.

#### Structured failures

Carry stable code, role, snapshot/`InputRootId`, role-relative path and bounded
message through the relevant Project/Emmy -> Service conversions.
`LibraryHealthFailed` retains Library snapshot/path so Service can identify the
request-local root. It is caller invalid input, not component unavailability.
Never recover origin by parsing messages or synthetic paths.

Preserve file context on read/encoding/NUL/language/digest/length/admission errors.
Map them using the [scope table](#requested-scope-limitations) and the
[public error contract](FRONTENDS.md#check-results-and-command-status). Add only
error distinctions a consumer acts on; do not enlarge global `ServiceError` into
a provenance framework or create a new error hierarchy for each helper.

#### Same-owner chunk requirement

Remove unconditional chunk limitation/hard-coded `Partial`. Use
[the configured owner's AST collector](EMMYLUA_UPSTREAM.md#addon-chunk-context),
validate returned path/content/span against exact Main and limit only affected
files. Diagnostic suppression does not erase source/context blockers. No second
parse/session or raw-text scan; helper injection is later Slice 1B.

<a id="ordinary-configuration"></a>

### Ordinary configuration

[Frontends](FRONTENDS.md#configuration-and-path-bases) owns keys, path origins,
overrides and defaults. Merge before required-field validation; reuse `StrictJson`
for duplicate/depth bounds. The [Emmy adapter](EMMYLUA_UPSTREAM.md#diagnostic-configuration)
owns strict diagnostic admission and semantic equivalence. No implicit config
discovery; `fail_on` and debounce are presentation state, not analyzer identity.

<a id="load-units-and-addon-context"></a>
<a id="selected-acquisition"></a>
<a id="exact-load-context-handoff"></a>
<a id="physical-and-virtual-main"></a>
<a id="implicit-binding-documents"></a>
<a id="addon-chunk-context"></a>

### Later context slices

Follow [1B/1C](REFACTOR_QUEUE.md#1b-dependencies-xml-and-chunk-injection) for
dependencies, XML, chunk injection and special loading; do not prebuild their
engines in 1A. Missing context limits affected units. Path-sorted snapshots are
analysis identity, not runtime order; exact order/context remains Project data.

<a id="one-direct-check-operation"></a>
<a id="mature-direct-operation"></a>
<a id="updates"></a>

### Updates

Retained callers reuse the [same operation](#one-public-service-operation).
Reuse only while exact Main, canonical Library set, virtual snapshot and canonical
analysis-relevant diagnostic projection match. Delta only while membership,
target, projection, context and authority set are unchanged. TOC/XML/dependency/
channel/context/Library/config changes rebuild cold. Remove declarations/maps/
diagnostics/index rows together; failed/cancelled/stale work never publishes.
The one-shot CLI does not require a retained-session manager or incremental engine.

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

[KETHO_RUST_PORT.md](KETHO_RUST_PORT.md) owns projection of environment,
`MayReturnNothing`, `NilableContents`, stride, mixin, restriction/Secret and maps.
Extend the existing report only with consumed typed fields and preserve explicit
loss. Plain Lua remains valid without it. No second producer/converter or
certification envelope; donor Lua is data, never executed.

<a id="outputs-and-corrections"></a>
<a id="annotation-projection-rules"></a>
<a id="manifest-and-ordinary-reader"></a>

### Outputs and corrections

Emit inert bounded Lua once. Explicit source-guarded corrections retain origin,
replacement, reason, span and conflicts separately from emission. Acceptance of
a correction is not evidence it was emitted or consumed by the Library reader.

<a id="blizzard-ui-index-search-and-context"></a>
<a id="small-navigation-operations"></a>

## Blizzard UI index, search and context

Checking works without an index. The [direct lookup slice](REFACTOR_QUEUE.md#4-direct-source-index-search-and-context)
uses one rebuildable SQLite DB for selected identity, exact bodies, symbols/spans
and consumed relations. Serve exact indexed context or identity-checked rereads,
not Project/Emmy/Graph replay. Remove nested SQLite->hex->JSON, generic lifecycle,
federation and Context packs after their final readers move.

<a id="typed-source-relationships"></a>

## Typed source relationships

Keep only consumed callable origin, dependency/trigger/order, XML include/template/
script/binding and selected type/call/reference relations. Preserve exact location,
ambiguity and incomplete coverage. No pack/plan/matcher/proof round-trip.

<a id="results-and-interfaces"></a>

## Results and interfaces

Service returns findings, scope completeness and located limitations, with exact
public locations and no internal receipts. Frontends render this result under
[one output contract](FRONTENDS.md#check-results-and-command-status); general Lua
editor features remain upstream EmmyLua.

<a id="completion-and-verification"></a>

## Removal and verification

Trace input -> existing producer -> consumer -> visible result. Remove displaced
code, options and instructions with the last caller, not in a separate final
rewrite. Use [the change review](../CONTRIBUTING.md#review-the-handoff) and targeted
checks; compilation/source review does not prove an executed product workflow.
