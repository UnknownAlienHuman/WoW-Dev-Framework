# Implementation order

Read [current status](CURRENT_STATUS.md) before taking an item. The
[manifesto](ARCHITECTURE.md) defines the product,
[IMPLEMENTATION.md](IMPLEMENTATION.md) defines the target owners, and
[WOW_LINTER.md](WOW_LINTER.md) defines domain behavior. This file is the only
ordered queue. Issue history cannot restore removed platform contracts.

A queue item is complete only when its actual input reaches its real reader and
visible result. A component build is not a user observation.

## 1. First usable ordinary addon check

Related: [#200](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/200),
[#201](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/201),
[#197](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/197).

### Delivered prerequisites — do not repeat

| PR | Delivered lower boundary | Remaining limitation |
| --- | --- | --- |
| #246 | all configured Emmy diagnostics retain code, exact message, severity and byte range; standalone Lua 5.1 STD is initialized | transitional LSP is still one-file |
| #247 | `ConfiguredAnalyzerSession` over exact Main, Library and optional virtual snapshots; caller diagnostics; one embedded STD; diagnostics-only versus explicit semantic collection | it consumes snapshots, not ordinary project configuration or selected load context |
| #248 | direct `wow-service::addon_check::check_addon`, findings plus limitations, no legacy Reference/Rules/publication/graph/store stack | no root/TOC/Library acquisition or CLI adapter; context remains partial |
| #249 | compiled Emmy identity without fabricated compatibility-report hashes | legacy publication/replay remains report-backed while it has callers |
| #250 | exact Main identity in immutable-owner reuse admission | no retained public CLI owner yet |

These are implemented in `main`, with exact-head runs #418, #422, #425, #440
and #445 respectively. None is an observed addon workflow.

### Remaining item-1 path

Connect the ordinary user operation without rebuilding those owners:

```text
root / explicit-or-unambiguous TOC / retail flavor / Library paths
  -> selected required and available optional dependency closure
  -> exact typed package/unit context and selected package/TOC/XML order
  -> exact bounded physical inputs and mapped XML units
  -> existing ConfiguredAnalyzerSession
  -> existing wow-service addon_check
  -> CLI text/JSON diagnostics, limitations and exit status
```

The implementation slice must:

1. add the small ordinary configuration DTO and CLI/config-file precedence;
2. extract a load target containing flavor, optional target Interface and
   `TocLoadContext`, rather than requiring `ProfileIdentity`;
3. use one finite reviewed dependency table: required `Dep`, `Dependencies`,
   `DependsOn`, `RequiredDep`, `RequiredDeps`; optional `OptionalDep`,
   `OptionalDeps`; select only the requested TOC/relevant transitive closure and
   reject ambiguity explicitly;
4. model package/per-unit execution context used by selected source:
   `UseSecureEnvironment`, `LoadIntoEnvironment`, `SuppressLocalTableRef`,
   bootstrap and supported load-first/load-phase facts;
5. fix tri-state predicate reduction so known allow/exclude results are not
   poisoned by irrelevant unknown aliases or gates;
6. retain exact package/TOC/XML order in a validated context value beside the
   path-sorted snapshots; never treat snapshot/Emmy registration order as WoW
   runtime order;
7. reuse `ProjectInputDirectory` and `ProjectInputFile` safety/bounds without
   constructing `ProjectInputInventory` or publication configuration;
8. split pure XML-unit/map preparation from the publication wrapper;
9. preserve independent captured units and located failure when one included XML
   file is malformed;
10. pass exact Main/Library/virtual snapshots plus the validated context to the
    direct Service path;
11. render actual code/message/severity/location plus scoped limitations;
12. apply the documented failure threshold and `0/1/2/64/130/4` exit policy;
13. record the first exact command/input/result only after it is actually run.

The first usable result remains a Lua/type check. Empty or omitted `checks` means
no WoW rule while Emmy diagnostics remain enabled. Missing chunk or XML-handler
context remains an explicit limitation; it is not silently converted to clean.
The full source trace for steps 3–6 is
[TOC_LOAD_CONTEXT_AUDIT.md](TOC_LOAD_CONTEXT_AUDIT.md).

### Source-traced replacement boundary

The existing executable check path is transitional and must not be extended:

- `apps/wow/src/args.rs` binds the CLI to `LocalCommand`;
- `LocalProjectInput::assemble_with_platform` and `LocalProjectBackend` require
  Reference, analyzer compatibility and Project publication state;
- generic `Service::check` treats Reference and Rules as mandatory even for
  type-only work;
- `local/projection.rs::check_context` expands empty rule selection to every
  registered rule;
- the old component/capability result is not the delivered direct diagnostic
  result.

Do not fabricate empty `ReferenceView`, RuleRegistry, compatibility reports,
Project generations or proof identities to satisfy those constructors. Do not
wrap the new result inside the old result. Move callers, then delete the route.

### Exact owner split

- **`wow-project`** owns bounded acquisition, small load target, finite directive
  semantics, TOC/dependency selection, exact ordered package/unit context,
  physical files and pure XML virtual units/maps. Legacy profile/publication
  wrappers adapt while old callers exist.
- **`wow-emmy`** already owns `ConfiguredAnalyzerSession`, compiled provenance,
  Lua 5.1, embedded STD and optional semantic collection. Keep snapshots
  path-sorted/content-addressed; add only the demonstrated mapped-context seam,
  not another session or runtime emulator.
- **`wow-service`** already owns `addon_check` and its direct result. Add ordinary
  configuration resolution, validated load/context input and selected WoW-check
  dependencies; do not reintroduce global component health.
- **`wow-cli`** owns arguments, config precedence, cancellation, rendering and
  exit policy only.
- **LSP/MCP** later adapt to the same Service owner and exact project state.

## 2. Correct Blizzard declarations used by the check

Related: #200/#201; reuse the public builder and existing native Ketho port.

Separate client flavor, game/glue load context and API member environment. Model
and carry through the actual consumer:

- member-level `SecureOnly` and other reviewed environment facts;
- `MayReturnNothing` as a zero-result alternative;
- `NilableContents` separately from container nilability;
- numeric ordered repeated `StrideIndex` groups;
- mixin relationships and selected restriction/origin facts;
- unresolved or unrepresentable forms as located limitations, not silent `any`.

Concrete reviewed cases are `C_ArtifactUI.GetMetaPowerInfo` and
`C_EquipmentSet.GetItemIDs`. Keep one `source-report.json` producer/consumer DTO.
The ordinary reader must consume optional exact typed metadata/maps; plain Lua
Libraries remain valid for type checking and limit only checks that need the
manifest.

Replace per-file blocking Git calls with one bounded selected batch read and
truthful child cancellation/reaping. Do not restore the removed example producer,
old `/1` verifier, another converter or certification platform.

## 3. Correct addon/XML context and updates

Related: [#109](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/109),
[#117](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/117).

Use the same configured owner plus the validated Project load/context value for:

- physical Lua and mapped XML-derived Lua;
- XML-only addons without a dummy physical file;
- one shared private table per supported ordinary addon and separate lexical
  chunks per file;
- exact source-verified XML handler parameters;
- distinct suppressed-private-table, package-secure, per-unit environment
  override, bootstrap/load-first and free-inline contexts;
- exact virtual-to-XML mappings, including entities and discontinuous pieces.

The mapped helper design in
[EMMYLUA_UPSTREAM.md](EMMYLUA_UPSTREAM.md#addon-chunk-context) remains a design
until executed. Synthetic helper calls/declarations are context-only and must not
become user API/flow facts or diagnostics.

A physical file delta may reuse an owner only when selected membership, exact
ordered load context, Library identity and analyzer configuration are unchanged.
TOC/XML/dependency/Library/config changes rebuild selection and owner cold.
Deletion or rename removes declarations, virtual units, maps, diagnostics and
index rows coherently. Failed/cancelled candidates never advertise an old result
as current.

Remove the stale blanket `InlineLuaNotAnalyzed` blocker for supported extracted
bodies without claiming handler semantics. Connect the currently rejected
TOC/package-selected live-update route to selection/context rebuilding.

## 4. Small WoW checks and frontend reuse

Related: [#103](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/103),
[#195](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/195),
[#107](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/107),
[#198](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/198),
[#199](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/199).

Enable only direct typed checks with explicit dependencies. Before API existence
is enabled, carry/query the original Emmy lexical binding and selected API origin;
a local named `C_*` is not a Blizzard namespace by spelling. Before Secret use is
enabled, separate unresolved producer semantics from malformed/stale fact
identity. Unknown provenance remains unknown.

LSP keeps overlay text/version/coordinate ownership but passes exact overrides to
the configured project owner. Accept increasing version gaps, reject stale
results, and return standard empty/null results. General Lua completion/hover/
definition/references remain upstream EmmyLua unless a WoW-specific persisted
navigation operation is required.

MCP keeps one standard stdio profile, bounded extensible metadata, valid unknown
notification behavior and only implemented useful tools. CLI/LSP/MCP serialize
the same Service results; no profile/registry/proof platform.

## Direct source lookup and remaining platform removal

**Order: 5.** Related:
[#131](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/131),
[#169](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169),
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122),
[#106](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/106).

After addon checking is useful, store one current selected source set in normal
SQLite tables: sources, files with body text, symbols with spans and only the
relations consumed by navigation. Query after restart without Project/Emmy/Graph
replay. FTS indexes body text; exact names use a separate lookup. Return readable
source from the indexed exact text with path/range/revision/flavor.

Preserve existing useful direct retained-shard SQL, opt-in Reference documentation
search, one-scan neighbor batching and exact excerpt checks. Add real SQLite
interruption/bounded lock waiting, valid empty indexes, transactional updates and
primary-error preservation through cleanup.

Remove nested SQLite/hex/JSON, replay recapture, fixed-point pagination, Context
packs, response journals, generic lifecycle and proof/fingerprint machinery only
as their real readers move. Do not replace them with another universal store,
graph, result envelope or RAG platform.

Attach Store/format cleanup
([#111](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/111),
[#194](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194),
[#112](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/112)) to the
reader it replaces. Measure the necessary real workflow under
[#113](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/113); do not
invent targets from code size.

## Working policy

One manager/worktree, bounded delegation, code first, formatting and affected
strict Clippy. Tests remain deferred until functional completion by owner
direction. Do not use that policy to call compile/Clippy a product pass.

Update [CURRENT_STATUS.md](CURRENT_STATUS.md) when merged code changes the next
work, and [IMPLEMENTATION_TRACE_REGISTER.md](IMPLEMENTATION_TRACE_REGISTER.md)
when source evidence changes a boundary or an exact workflow becomes observed.
The baseline register is not a recurring progress diary.