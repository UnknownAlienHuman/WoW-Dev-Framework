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
| Direct addon check orchestration/result | `wow-service::addon_check` |
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

The first public result is intentionally small: an offline `LuaTypes` check over
explicit local input.

Required input:

- addon root;
- explicit TOC path;
- explicit supported flavor;
- one or more explicit Library directories;
- optional Emmy diagnostic configuration and `fail_on` threshold.

The path is:

```text
ordinary config
  -> bounded selected physical Lua bytes
  -> exact Main and canonical Library snapshots
  -> existing ConfiguredAnalyzerSession
  -> existing wow-service::addon_check
  -> projected user locations, text/JSON result and exit status
```

No WoW rule is enabled by default. Search, SQLite, Reference, Rules, publication,
graph, replay and source regeneration are not prerequisites.

This slice must return a useful result before the complete Blizzard loading model
is implemented. Unsupported selected input is a scoped limitation. Product
channel, reverse triggers, implicit bindings or an unrequested WoW check do not
make an unrelated type-only result partial.

`AddonCheckScope::LuaTypes` completeness is computed from facts actually required
by selected units. The current unconditional addon-chunk fallback and hard-coded
`Partial` result are transitional defects: unused top-level ellipsis cannot make
every addon incomplete. When chunk context is needed, report the affected files
or conclusions explicitly.

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

Target configuration contains:

- `root`;
- explicit `toc` for the first slice, later optional only when selection is
  unambiguous;
- `flavor`;
- optional `channel` (`stable`, `ptr`, `beta`) for channel-gated source;
- optional positive `target_interface`;
- nonempty `library` array;
- optional `dependency_roots`, selected WoW `checks`, `emmy_diagnostics` and
  `fail_on`.

CLI paths resolve from cwd; config paths resolve from the config directory;
relative TOC resolves from the addon root. CLI values override config. Repeated
CLI `--library` replaces the configured array.

No user supplies Project IDs, generations, profile/report digests, receipts,
proof manifests or graph budgets. Plain Lua Libraries remain valid for typing;
missing generated metadata limits only checks that require it.

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

A mature check performs only the stages required by its selected scope:

1. resolve ordinary config and target;
2. select exact source and dependencies needed by that scope;
3. capture bounded physical/Library/XML/binding bytes;
4. prepare mapped virtual units and relevant context;
5. build exact snapshots and validated source projection;
6. open or reuse one compatible configured Emmy owner;
7. collect language diagnostics and only semantic facts requested by selected
   WoW checks;
8. run those checks directly;
9. return findings, scoped limitations and exit status.

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
