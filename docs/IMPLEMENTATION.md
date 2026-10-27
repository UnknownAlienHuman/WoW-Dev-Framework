# Implementation: finish the WoW linter

Read [current status](CURRENT_STATUS.md), [architecture](ARCHITECTURE.md),
[WoW behavior](WOW_LINTER.md) and the active
[queue item](REFACTOR_QUEUE.md). This guide defines durable owner boundaries and
handoffs, not PR history.

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

Project adds `AddonLoadTarget { flavor: retail, target_interface: Option<positive>,
toc_context: Option<TocLoadContext> } + projection scope` and binds it directly
into `ProjectLoadPlan`. Replace mandatory `ProfileIdentity` digest and positive-
Interface admission; do not merely change the reader argument. Legacy callers may
adapt a real profile. The ordinary route never fabricates profile/source/report/
schema/correction/generation identity.

Slice 1A supplies `retail`, no target Interface/context. Retain missing/zero/
positive source Interface; evaluate compatibility only with a positive target.
Invalid Interface syntax is invalid input.

#### Physical projection

Add identity-bound `physical_lua` to the existing Project owner:

- parse explicit TOC/selection metadata;
- capture direct selected `.lua` under retained bounds;
- retain XML records/exact TOC spans, but do not open XML;
- emit located `SourceSelectionIncomplete` per selected deferred XML branch;
- preserve physical diagnostics when XML is missing/malformed;
- reject unsafe root/path/TOC bytes or unrecoverable syntax;
- allow empty `Project` snapshot: complete with no selected unit/blocker, partial
  with deferred selected input. Never add dummy source.

Full XML expansion is Slice 1B, not a second loader.

#### Metadata and requested-scope limitations

T23 metadata-by-effect admission is a Slice 1A prerequisite. Display/localized,
custom `X-*` and known non-selection metadata remain Project observations.
Malformed keys or selected-file/environment/callback-changing directives remain
invalid or located.

The public standalone mapper belongs in Service to avoid Project -> Service
dependency:

```text
ProjectLoadPlan + AddonCheckScope -> Vec<AddonCheckLimitation>
```

For `LuaTypes`:

| Source state | Outcome |
| --- | --- |
| missing/unreadable addon root or explicit TOC; unsafe path/bytes; invalid TOC encoding/NUL; unrecoverable syntax; config/source/Library budget refusal | pre-result `invalid_input` |
| safely parsed TOC names missing direct Lua | located `SourceSelectionIncomplete`, continue, `Partial` |
| required dependency, included condition, selected XML or recoverable selected-context uncertainty unresolved | located scope blocker, `Partial` |
| optional absence; repeated TOC occurrence; Interface without target; `LoadWith`; unused lifecycle/display/custom metadata | Project observation, no limitation |
| affected file uses chunk-owned top-level `...` or selected context changes typing | located context limitation, `Partial` |
| malformed caller Library/config/source | pre-result `invalid_input` with structured origin |
| native panic/poisoning/impossible validated-owner mismatch | `internal` |

Do not derive completeness from global `blocks_complete()` or
`external_files_complete()`.

#### Safe roots and selected-path authority

Frontend resolves path bases lexically for display. Project is the trust boundary
and exposes one narrow preparation/config entry wrapping crate-private component-
wise `open_workspace`; Slice 1A must not use ambient
`ProjectInputDirectory::open`.

Each opened root retains request-local `InputRootId = role + ordinal`, original
display descriptor, nonserialized normalized absolute authority descriptor and
held no-follow capability. Reject descriptor text above 32 KiB; admit local Unix
absolute and Windows drive-absolute roots; unsupported UNC/device prefixes are
invalid.

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
paths. Frontend may join display root for text. Absolute/path-authority data never
enters semantic identity or JSON.

#### One public Service operation

The ordinary operation owns:

1. merged strict config/display descriptors;
2. Project target/scope plan, admitted Main/Libraries and path authorities;
3. plan/Main/authority validation;
4. requested-scope blockers;
5. exact snapshots and request-local snapshot/root mapping;
6. one configured Emmy owner, requirement/diagnostic collection and internal
   lower invocation;
7. one result or operation-specific error.

CLI/LSP/MCP never open source, construct `AddonCheckRequest`, call
`with_input_limitations`, build snapshots or attach blockers. Keep lower request/
check internal after callers move. This is a use-case function, not generic
backend.

#### Structured failures

Carry stable code, role, snapshot/`InputRootId`, role-relative path and bounded
message through Project -> Emmy -> configured session -> Service.
`LibraryHealthFailed` retains Library snapshot/path so Service maps it to request-
local root. Never parse messages or synthetic paths.

Public operation error:

```text
invalid_input | cancelled | internal
message <= 4 KiB
role? = main | library | config
root_index? only for library, zero-based
relative_path? relative to role root
```

File-specific Project read/UTF-8/NUL/language/digest/length/admission failure keeps
root/path. Source/workspace/diagnostic/result-size refusal from caller input is
`invalid_input`; analyzer/serialization/cancellation-handler/I/O invariants are
`internal`. Do not enlarge global `ServiceError` into provenance platform.

#### Same-owner chunk requirement

Remove unconditional chunk limitation/hard-coded `Partial`. Add one bounded
collector on the same `ConfiguredAnalyzerSession`/registered Main AST:

- walk expression nodes;
- match `LuaLiteralExpr` with `LuaLiteralToken::Dots`;
- reject matches with enclosing `LuaClosureExpr` before chunk root;
- retain lowest-starting exact `{path, content_sha256, span}` per file;
- never raw-scan or start another parser/session.

Service validates records against exact Main and limits only affected files.
Helper injection remains Slice 1B.

<a id="ordinary-configuration"></a>

### Ordinary configuration

Slice 1A fields only: `root`, explicit relative `.toc`, exact `flavor: retail`,
nonempty `library`, optional `emmy_diagnostics`, `format`, `fail_on`.

Read at most one explicitly named config through Project no-follow helper, 256 KiB.
Reuse duplicate-key/depth-bounded `StrictJson`, then outer
`deny_unknown_fields`. No implicit discovery.

Pinned `EmmyrcDiagnostic` is permissive and silently drops invalid
`globalsRegex`. Prevalidate exactly `disable`, `enable`, `globals`,
`globalsRegex`, `severity`, `enables`; compile every regex; reject unknown keys,
invalid expressions and `diagnosticInterval`. At the pinned revision interval is
EmmyLua-LS debounce only, not code analysis. No `.emmyrc` discovery/fallback.

Raw JSON equality is not diagnostic semantic equality. Define one canonical
analysis projection in `EmmyAnalysisConfiguration::equivalent`:

- preserve `enable`;
- sort/dedup `disable` and `enables` by diagnostic code;
- sort/dedup `globals` and `globalsRegex`;
- sort severity entries by diagnostic code;
- omit `diagnosticInterval` entirely.

Pinned analysis treats the list families as sets or OR-matched regexes, so order/
duplicates have no effect. Omitted diagnostics, `{}`, reordering and duplicates
must reuse one owner when analysis semantics match. A real analysis change forces
cold owner. LS debounce remains frontend state.

CLI scalars replace config scalars; repeated CLI Libraries replace config array.
Defaults: text, `fail_on: error`, upstream default diagnostics. If config fails,
only explicitly admitted CLI `--format` selects JSON error; otherwise text. Slice
1A exposes no WoW checks and runs zero WoW rules.

<a id="load-units-and-addon-context"></a>
<a id="selected-acquisition"></a>
<a id="exact-load-context-handoff"></a>
<a id="physical-and-virtual-main"></a>
<a id="implicit-binding-documents"></a>
<a id="addon-chunk-context"></a>

### Later context slices

1. required/available optional dependencies and exact selected order;
2. source-projectable XML virtual units and exact projection;
3. ordinary `(addonName, addonTable)` only for affected top-level ellipsis;
4. reviewed handlers and coherent updates;
5. channel, `LoadWith`, bindings, target Interface and secure/special context;
6. generated metadata and selected WoW checks.

Missing context limits only affected units. Path-sorted snapshots are analysis
identity, not WoW execution order; exact order/context stays separate validated
Project data.

<a id="one-direct-check-operation"></a>

### Mature direct operation

For selected scope: resolve strict input, acquire bytes, validate authority,
prepare required context, derive required blockers, build exact snapshots/maps,
reuse/open one compatible Emmy owner, collect requested facts/rules and return one
result. Optional component health is not a global gate.

<a id="updates"></a>

### Updates

Reuse only while exact Main, canonical Library set, virtual snapshot and canonical
analysis-relevant diagnostic projection match. LS debounce is not analyzer
identity. Delta only while selected membership, target, projection, context and
authority set are unchanged. TOC/XML/dependency/channel/context/Library/config
changes rebuild cold. Remove declarations/maps/diagnostics/index rows together;
failed/cancelled/stale work never publishes.

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
`MayReturnNothing`, `NilableContents`, stride, mixin, restriction/Secret and maps
or report conservative loss. Never execute donor Lua. One optional typed manifest;
plain Lua valid. No second producer/converter/certification envelope.

<a id="outputs-and-corrections"></a>
<a id="annotation-projection-rules"></a>
<a id="manifest-and-ordinary-reader"></a>

### Outputs and corrections

Emit inert bounded Lua once. Corrections are explicit/source-guarded and separate
from emission. Preserve origin/replacement/reason/span/conflicts. Accepted
correction does not prove emission. Ordinary Library reader consumes output.

<a id="blizzard-ui-index-search-and-context"></a>
<a id="small-navigation-operations"></a>

## Blizzard UI index, search and context

Checking works without index. Later use one rebuildable SQLite DB with selected
identity, exact bodies, symbols/spans and consumed relations; body FTS; later-
process query without Project/Emmy/Graph replay; transactional updates; exact
indexed context or identity-checked reread. Remove nested SQLite->hex->JSON,
replay, generic lifecycle, federation and Context packs after final readers move.

<a id="typed-source-relationships"></a>

## Typed source relationships

Keep only consumed callable origin, dependency/trigger/order, XML include/template/
script/binding and selected type/call/reference relations. Preserve exact location,
ambiguity and incomplete coverage. No pack/plan/matcher/proof round-trip.

<a id="results-and-interfaces"></a>

## Results and interfaces

Addon result = findings + completeness + located limitations; no internal receipts.
Deterministic diagnostics order: role/path/span/code/message. Limitations:
code/location/message. Threshold rank Error > Warning > Information > Hint;
preserved `Unknown` meets every threshold. Partial exit 2 precedes finding exit 1.

Pre-encode escaped text/JSON <= 32 MiB, check cancellation, then one write.
Result -> stdout; pre-result error -> stderr. First byte commits the sole envelope.
Oversize result invalid_input/64; serializer/I/O internal/4. Human text visibly
escapes controls; typed/JSON strings exact.

CLI/LSP/MCP are thin adapters; general Lua editor features remain upstream Emmy.
See [FRONTENDS.md](FRONTENDS.md).

<a id="completion-and-verification"></a>

## Removal and verification

Remove reports, publication/replay, generic results, matcher/proof graph and
lifecycle only with final caller. Do not preserve aliases/rename platform.

Trace `user input -> producer -> handoff -> reader -> visible result`. Build,
Clippy and source review are not product observation.
