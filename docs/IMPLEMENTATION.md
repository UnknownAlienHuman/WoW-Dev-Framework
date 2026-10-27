# Implementation: finish the WoW linter

Read [current status](CURRENT_STATUS.md), [architecture](ARCHITECTURE.md),
[WoW behavior](WOW_LINTER.md) and the active
[queue item](REFACTOR_QUEUE.md). This guide defines durable ownership and direct
handoffs. It does not copy the moving PR/status ledger.

<a id="code-ownership"></a>

## Code ownership

| Responsibility | Reuse / target owner |
| --- | --- |
| Blizzard checkout/ref/TOC/API input | `apps/wow-reference-builder` plus bounded source helpers |
| Normalized Blizzard facts, aliases and corrections | `wow-reference` |
| Ketho-style `.lua` declarations and source maps | `wow-annotations`, `wow-ketho-literals` |
| Lua 5.1 parsing, types and diagnostics | `wow-emmy::ConfiguredAnalyzerSession` over pinned EmmyLua |
| Addon root, TOC/load selection, exact bytes, XML/maps and context | `wow-project` |
| Direct addon check orchestration/result | one narrow public operation in `wow-service::addon_check`; lower exact-snapshot request/check remain internal |
| Current source index | direct `rusqlite`/FTS5 owner, ultimately `wow-store` or a smaller retained replacement |
| CLI/LSP/MCP | thin adapters in `apps/wow` to the same Service owners |
| Shared primitives | `wow-core` only when multiple retained owners genuinely share them |

Do not add another analyzer, result hierarchy, Project publication path, graph
proof layer or compatibility-report adapter for ordinary checking. Move callers,
then remove displaced owners.

<a id="addon-configuration-and-emmy-analysis"></a>

## Addon configuration and Emmy analysis

<a id="first-implementation-slice"></a>

### First usable vertical slice

The first public result is an offline `LuaTypes` check over explicit local input:

```text
root + explicit TOC + flavor + Library directories
  -> strict resolved ordinary input
  -> one Service prepared-addon operation
  -> AddonLoadTarget + physical_lua Project plan
  -> standalone scope-aware limitations + admitted physical Main
  -> collision-free compiled-pin Main/Library snapshots
  -> existing ConfiguredAnalyzerSession / low-level check_addon
  -> root-relative text/JSON result and exit status
```

No WoW rule, dependency universe, XML execution context, search index, Reference,
Rules, publication, graph, replay or source regeneration is required.

#### Project target and plan identity

Add a small validated `AddonLoadTarget` owned by Project:

- exact canonical flavor `retail` (the only Slice 1A flavor);
- optional positive target Interface;
- optional `TocLoadContext`.

`ProjectLoadPlan` identity and admission must bind this target directly. Replace
its mandatory `target_interface` and `target_profile_digest` dependency on
`ProfileIdentity`; changing target or projection scope changes plan identity.
Legacy profile callers may adapt their existing profile into `AddonLoadTarget`.
The ordinary route never constructs a profile ID, source/report digest, schema
registry, correction identity or generation.

Slice 1A supplies flavor only. Parse and retain missing, zero and positive source
Interface values independently; compatibility is evaluated only when a positive
target Interface is supplied. Invalid Interface syntax remains invalid input.

#### Physical projection scope

The existing loader recursively opens XML while selecting Main. That cannot be
the first-slice boundary because malformed XML currently aborts otherwise usable
physical Lua.

Add an explicit plan scope such as `physical_lua` and bind it into the plan digest:

- parse the selected TOC and metadata/conditions;
- capture included direct `.lua` files under existing Project bounds;
- retain included XML records and exact TOC spans, but do not open or parse XML;
- emit a located `SourceSelectionIncomplete` limitation for each included XML
  branch deferred to Slice 1B;
- preserve physical diagnostics despite missing/malformed/deferred XML;
- reject unsafe selection/path/input errors;
- when selected deferred XML exists but physical Lua is empty, build the supported
  empty `Project` snapshot and return a located partial result; never add a dummy
  source or report clean.

Full recursive XML expansion remains the Slice 1B plan scope.

#### Metadata and standalone limitations

T23 metadata-by-effect admission is a Slice 1A prerequisite. Display/localized,
custom `X-*` and known non-selection metadata remain source-backed observations
and do not block independent `LuaTypes`. Empty/malformed keys and directives that
can change selected files/environment/callback context remain invalid or located
limitations.

Add a public standalone mapper:

```text
ProjectLoadPlan + requested AddonCheckScope
  -> Vec<AddonCheckLimitation>
```

It uses exact retained source digests/spans and interprets issues for the selected
scope rather than copying global `blocks_complete` or
`external_files_complete()`:

- unresolved required dependency -> `DependencyResolutionIncomplete`;
- normal optional-dependency absence -> no limitation;
- unresolved included condition -> `LoadSelectionUnresolved`;
- missing/unsupported direct selected source or deferred included XML ->
  `SourceSelectionIncomplete`;
- Interface metadata with no positive target -> retained observation, no
  `CompatibilityUnresolved`;
- cosmetic/unknown metadata not consumed by `LuaTypes` -> retained observation,
  no direct limitation.

Only a reason the requested scope cannot be complete enters
`AddonCheckLimitation`. Project observations are not mislabeled as blockers.

#### Snapshot and path construction

Use existing values directly:

- `EmmyBackendIdentity::compiled_pinned()` once;
- Main snapshot: universe `Project`, admitted `ProjectInputFile` values and
  existing Project Main ceilings (1,024 files, 1 MiB/file, 16 MiB total);
- each explicit annotation-Library directory: one separate universe `BlizzardUi`
  snapshot under `wow_core::library_limits` (4,096 files, 8 MiB/file, 64 MiB
  total). Separate snapshots allow identical relative paths in different roots;
- reject duplicate resolved Library roots and duplicate snapshot IDs rather than
  silently merging them;
- caller diagnostic settings become `EmmyAnalysisConfiguration`; no `.emmyrc`
  discovery.

Keep canonical root-relative source paths in snapshots, diagnostics and exact
locations. The frontend knows the configured display root and may join it for
human text; JSON preserves the relative identity. Ambient absolute paths do not
enter semantic snapshot/result identity.

#### Input-root and file authority

Resolve config-relative and CLI-relative paths lexically to normalized absolute
descriptors; do not call a filesystem canonicalizer that can follow a root link.
Project then opens each root component-by-component without following links and
reuses `read_lua_library`/inventory admission. Configuration decoding is strict
(`deny_unknown_fields` or equivalent), bounded and explicit; there is no implicit
config discovery. Every Library root must produce at least one admitted Lua file.

Keep a nonsemantic root token and admitted root-relative path for validation and
error projection. Reject:

- duplicate resolved roots;
- duplicate Library snapshot IDs;
- the same admitted physical Lua file appearing in Main and Library, or in two
  Library roots.

Do not reject root ancestry alone: a nested generated Library is allowed when its
admitted files are disjoint from selected Main and other Libraries. Conversely,
identical relative paths in distinct roots are valid when they refer to distinct
admitted files. Validate the actual selected file authorities before Emmy
registration; universe/snapshot ID does not detect cross-role duplication.

#### One public Service operation

Add one narrow public Service operation for ordinary addon checking. It owns the
whole trusted assembly:

1. receive already-resolved ordinary config/display roots from the frontend;
2. ask Project to open/acquire the target-bound plan, admitted Main and Libraries;
3. validate plan/Main and root/file authority consistency;
4. derive requested-scope limitations;
5. build exact snapshots and map their IDs to configured roots;
6. construct the lower `AddonCheckRequest` and call existing `check_addon`.

The frontend never calls `with_input_limitations`, never manufactures snapshots
and never attaches a plan's limitations to independently supplied Main bytes.
Keep the exact-snapshot constructor/check crate-internal or otherwise unavailable
to CLI/LSP/MCP. This is a direct use-case function, not another generic backend,
registry or result hierarchy.

#### Structured input failures

Preserve structured source role, snapshot/root token and root-relative path across
Project acquisition, `EmmyMemberCallError`, `ConfiguredSessionError` and Service.
`LibraryHealthFailed` must carry the Library snapshot ID as well as the relative
file; Service maps that ID to the configured root and classifies the failure as
invalid caller input. Project attaches a relative path to every file-specific
UTF-8, NUL, read, language, digest/length and source-admission failure. Never
recover origin by parsing message strings or synthetic Emmy workspace paths.

#### Direct result

Only the Service operation may pass blocking standalone limitations into the
lower request. Preserve Emmy findings when relevant input is partial.
`AddonCheckScope::LuaTypes` is `Complete` exactly when every fact required by
selected physical units is available. Remove the unconditional addon-chunk
fallback and hard-coded `Partial`; later context limits only units/conclusions
that consume it.

<a id="load-units-and-addon-context"></a>
<a id="selected-acquisition"></a>
<a id="exact-load-context-handoff"></a>
<a id="physical-and-virtual-main"></a>
<a id="implicit-binding-documents"></a>
<a id="addon-chunk-context"></a>

### Subsequent context slices

Add context only through existing narrow owners:

1. required/available optional dependencies and exact selected package/file order;
2. source-projectable XML Lua and exact virtual-to-XML diagnostics;
3. ordinary `(addonName, addonTable)` chunk context where top-level `...` is used;
4. reviewed UI handler parameters and coherent add/edit/remove/rename updates;
5. product channel, `LoadWith`, implicit bindings, sentinel Interface and
   secure/special contexts;
6. generated-Library metadata and explicitly selected WoW checks.

Each slice preserves independent language findings. Missing context limits only
results that consume it.

<a id="ordinary-configuration"></a>

### Ordinary configuration

Slice 1A accepts only:

- `root`;
- explicit `toc`;
- exact `flavor: retail`;
- nonempty `library` array;
- optional `emmy_diagnostics`, `fail_on` and output format.

Decode one explicitly named bounded config with unknown fields rejected. Reject
unknown/duplicate scalar CLI options; repeated CLI `--library` deliberately
replaces the configured array. Resolve paths once, but Project performs root and
file admission. Each Library root must contain at least one admitted `.lua` file.

Later slices add optional unambiguous TOC selection, `channel`, positive
`target_interface`, `dependency_roots` and selected WoW `checks`. Their absence
must not be filled with guesses or synthetic legacy identities.

CLI paths resolve from cwd; config paths resolve from the config directory;
relative TOC resolves from the addon root. CLI values override config. Repeated
CLI `--library` replaces the configured array.

No user supplies Project IDs, generations, `ProfileIdentity`, profile/report
digests, receipts, proof manifests or graph budgets. Plain Lua Libraries remain
valid for typing; missing generated metadata limits only checks that require it.

### Load and context facts

When their slices are implemented, Project owns:

- finite required dependency aliases: `Dep`, `Dependencies`, `DependsOn`,
  `RequiredDep`, `RequiredDeps`;
- optional aliases: `OptionalDep`, `OptionalDeps`;
- three-valued allow/exclude predicates;
- exact package/TOC/XML/callback order separate from path-sorted snapshots;
- package/per-unit environment, phase and ordinary/special chunk context;
- `OnlyBetaAndPTR`, `LoadWith`, implicit `Bindings*.xml`, `DefaultState` and
  SavedVariables declarations;
- source `Interface: 0` as unknown compatibility, not a positive target;
- raw unsupported `Secure` semantics without guessing an alias.

`LoadWith` is a reverse trigger, not a dependency. Binding callbacks receive only
reviewed callback context such as `keystate`; they are not addon chunks. User
SavedVariables contents are never opened or inferred.

Path-sorted `LuaWorkspaceSnapshot` identity is not WoW runtime order. Context
requiring order uses a separate immutable validated Project value binding source
role/path/digest/origin/addon/environment/phase and relevant load facts.

<a id="one-direct-check-operation"></a>

### One direct check operation

A mature check performs only the stages required by its selected scope through
one Service use-case operation:

1. admit strict ordinary config and resolved root descriptors;
2. select exact source and dependencies needed by that scope;
3. capture bounded physical/Library/XML/binding bytes;
4. validate that no admitted physical Lua file appears in conflicting roles;
5. prepare mapped virtual units and relevant context;
6. derive only requested-scope limitations;
7. build exact snapshots and source projection;
8. open or reuse one compatible configured Emmy owner;
9. collect language diagnostics and only semantic facts requested by selected
   WoW checks;
10. run those checks directly;
11. return findings, scoped limitations and exit status.

Optional component health is not a global clean gate.

<a id="updates"></a>

### Updates

Reuse an immutable configured owner only while exact Main, canonical Library set,
virtual snapshot and diagnostic configuration match. A physical delta is allowed
only while selected membership and relevant context are unchanged.

TOC/XML/bindings/dependency/channel/context/Library/config changes rebuild the
selection and owner cold. Delete/rename removes declarations, virtual units,
maps, diagnostics and later index rows together. Failed, cancelled or stale work
never replaces a newer result. LSP overlays participate in the same identity.

<a id="native-annotation-generation"></a>
<a id="preserve-the-existing-pipeline"></a>
<a id="one-native-generation"></a>

## Native annotation generation

Reuse the existing source-to-output path:

```text
wow-reference-builder
  -> wow-reference native model/corrections
  -> wow-annotations + wow-ketho-literals
  -> ordinary .lua files + optional source-report.json
```

Read [KETHO_RUST_PORT.md](KETHO_RUST_PORT.md) for exact missing semantics:
callable/member environment, `MayReturnNothing`, `NilableContents`, stride groups,
mixins, Secret/restriction facts and source maps. Use only annotation syntax
supported by the pinned Emmy consumer and record conservative losses.

The builder reads source as data; it never executes donor Lua. One optional typed
manifest may carry maps/facts required by implemented checks. Do not add a second
manifest, converter or certification envelope.

<a id="outputs-and-corrections"></a>
<a id="annotation-projection-rules"></a>
<a id="manifest-and-ordinary-reader"></a>

### Outputs and corrections

Emit inert bounded `.lua` once. Corrections are explicit, source-guarded and
separate from final emission. An accepted correction does not prove the resulting
declaration was emitted successfully. Preserve original source, replacement,
reason, generated span and unresolved conflicts.

A real Library reader must consume successful output directly. Plain Lua remains
a supported Library for typing.

<a id="blizzard-ui-index-search-and-context"></a>
<a id="small-navigation-operations"></a>

## Blizzard UI index, search and context

Checking works without an index. The later source workflow uses one rebuildable
SQLite database with normal tables for selected source identity, exact file body,
symbols/spans and only useful TOC/XML/type/call relations. FTS indexes source
bodies, not filenames alone.

A later process must query without reconstructing Project/Emmy/Graph owners.
Updates replace changed rows and delete removed rows transactionally. Context
renders exact indexed text or a disk reread only after content identity matches.

Remove nested SQLite -> hex -> JSON, replay recapture, generic Store lifecycle,
Search retained-result/federation machinery and Context packs after their real
callers move.

<a id="typed-source-relationships"></a>

## Typed source relationships

Keep only relationships consumed by diagnostics, search or navigation:

- callable/source origin;
- dependency/trigger and selected file order;
- XML include/template/script/binding relations;
- selected type/call/reference relations.

Preserve exact location, ambiguity and incomplete coverage. Do not serialize
fixed Rust facts through generic pack/plan/matcher/proof layers and then convert
them back.

<a id="results-and-interfaces"></a>

## Results and interfaces

Each workflow returns one small typed result. Addon checking separates:

- diagnostics/findings;
- completeness;
- scoped limitations with optional exact location.

The public direct result contains no snapshot, analysis, Project, report or proof
receipts. CLI/LSP/MCP serialize the same owner result at their protocol edge.
Request IDs are transport correlation only.

CLI is the complete offline path. LSP/MCP are thin adapters. General Lua editor
features remain upstream EmmyLua. See [FRONTENDS.md](FRONTENDS.md).

<a id="completion-and-verification"></a>

## Removal rule

Remove old compatibility/certification reports, publication/replay wrappers,
generic result/capability models, matcher/proof graph paths and lifecycle
administration only with their last real caller. Do not preserve disabled aliases
or move the same platform into a new crate.

For every change trace:

```text
user input -> producer -> handoff -> reader -> visible result
```

Finish that path, record the evidence in the trace register, and do not confuse a
build gate with product observation.
