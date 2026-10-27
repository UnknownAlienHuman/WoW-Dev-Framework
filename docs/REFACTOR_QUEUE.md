# Implementation order

Read [current status](CURRENT_STATUS.md) first. Architecture defines scope,
[IMPLEMENTATION.md](IMPLEMENTATION.md) defines owners, and
[WOW_LINTER.md](WOW_LINTER.md) defines domain behavior. This is the only ordered
queue; PR and Issue history cannot expand it.

A slice is complete only when ordinary input reaches the real owner and produces
the visible result named below. Build gates are not product observations.

<a id="1-first-usable-ordinary-addon-check"></a>

## 1. First usable ordinary addon check

Do **not** implement the complete Blizzard loading model before the first command.
Deliver the smallest correct `LuaTypes` path, then add context in separate slices.

### 1A. Explicit physical-Lua `wow check`

Input:

- addon root;
- explicit TOC;
- exact flavor `retail`;
- one or more explicit annotation-Library directories;
- optional `emmy_diagnostics`, output format and `fail_on`.

Path:

```text
CLI/config
  -> strict resolved ordinary input
  -> one Service prepared-addon operation
  -> AddonLoadTarget + physical_lua Project plan
  -> scope-aware standalone limitations + selected physical Lua
  -> collision-free compiled-pin Main + per-directory Library snapshots
  -> existing ConfiguredAnalyzerSession / low-level check_addon
  -> root-relative text/JSON output and exit status
```

Requirements:

- replace profile-bound plan identity/validation, not only the reader argument:
  `ProjectLoadPlan` binds flavor, optional positive target Interface, optional
  TOC context and projection scope directly; legacy callers adapt into that
  target; the direct route never fabricates `ProfileIdentity` or its digests;
- with no target Interface, retain missing/zero/positive source Interface without
  compatibility rejection; malformed numeric syntax remains invalid;
- classify metadata by effect before completeness;
- use `physical_lua` scope: capture direct TOC `.lua`, retain but do not open XML,
  and emit exact located incompleteness for each included deferred XML branch;
  malformed XML cannot suppress physical diagnostics; when deferred XML exists
  without physical Lua, use an empty Project snapshot and return partial—never a
  dummy file or clean result;
- add `project_load_limitations(plan, LuaTypes)`. Required dependencies,
  unresolved included conditions, missing/unsupported direct files and deferred
  XML are blocking; optional absence, unused Interface metadata and nonblocking
  package observations are not limitations;
- never derive the direct result from `ProjectLoadPlan::external_files_complete()`
  alone;
- decode one explicit bounded config strictly: reject unknown fields and
  duplicate scalar CLI options; repeated CLI `--library` replaces the config
  array; no implicit config discovery;
- resolve path bases lexically without filesystem canonicalization, then open
  roots through Project's component-wise no-follow boundary and reuse its
  inventory/Library readers; require each Library root to yield at least one Lua
  file;
- use `EmmyBackendIdentity::compiled_pinned()`, Main universe `Project`, and one
  separate `BlizzardUi` snapshot per explicit Library directory under the
  existing Main/Library ceilings;
- reject duplicate resolved roots/snapshot IDs and any same admitted physical Lua
  file across Main/Library or two Library roots; allow overlapping roots only
  when their admitted file sets are disjoint, and allow equal relative paths in
  distinct roots;
- keep canonical root-relative paths in snapshot/result identity; join the root
  only for human rendering;
- preserve structured role + snapshot/root token + relative path for every
  file-specific acquisition/analyzer failure; map malformed caller Library input
  to invalid input, not internal component failure;
- expose one narrow public Service operation that binds plan, admitted files,
  scope limitations, snapshots and lower `check_addon`; frontends never construct
  `AddonCheckRequest` or call `with_input_limitations`;
- no Project IDs, generations, reports, proof objects, Reference/Rules, graph,
  store or publication;
- empty/omitted WoW `checks` runs no WoW rule while Emmy diagnostics remain on;
- preserve upstream code, exact message, severity and half-open byte range;
- return findings even when selected input has a relevant limitation;
- remove the unconditional addon-chunk limitation/hard-coded `Partial`;
- source/config is never rewritten or executed;
- apply deterministic exits: invalid input `64`, observed cancellation `130`,
  internal/output failure `4`, partial result `2`, threshold finding `1`, else `0`;
  preserve findings in partial output;
- successful/partial results use stdout; pre-result input/cancel/internal errors
  after format admission use stderr. Under `--format json`, stderr contains one
  bounded typed error object and stdout remains empty; CLI syntax errors before
  format admission use one text usage error. Never emit a second error after a
  partial stdout write.

Observe one valid typed call, one invalid argument, one required-dependency or
included-XML partial result that still preserves diagnostics, one malformed
Library/input case, and one invalid/missing source input. Record exact command,
input and result.

### 1B. Dependency, XML and chunk context

After 1A works:

- add required and available optional dependency closure;
- preserve exact selected package/file order beside path-sorted snapshots;
- pass only source-projectable XML virtual units and map diagnostics back to exact
  XML source;
- bind ordinary `(addonName, addonTable)` only where top-level `...` requires it;
- add one reviewed XML handler adapter;
- admit physical-only, physical+virtual, XML-only and valid empty language scopes;
- preserve independent units after one malformed included XML unit;
- rebuild coherently on add/edit/remove/rename and TOC/XML/Library changes.

A missing handler/chunk fact limits only affected units or conclusions.

### 1C. Special load behavior

Then add, independently:

- Stable/PTR/Beta channel and `OnlyBetaAndPTR`;
- `LoadWith` as a reverse trigger, never a dependency;
- implicit bounded `Bindings*.xml` discovery and mapped `keystate` callbacks;
- `UseSecureEnvironment`, per-unit `LoadIntoEnvironment`, bootstrap/load-first
  context and unsupported `SuppressLocalTableRef`;
- source `Interface: 0` as unknown compatibility;
- raw unsupported `Secure` semantics;
- known lifecycle/error/retrieval metadata and machine SavedVariables
  declarations without user-data reads.

Donor-modeled behavior remains explicitly scoped; do not claim exact client
timing or availability without authoritative evidence.

<a id="source-traced-replacement-boundary"></a>

### Source-traced replacement boundary

The target route is the direct `wow-project -> wow-emmy ->
wow-service::addon_check` path. Do not adapt
`LocalCommand -> LocalProjectBackend -> generic Service::check` or fabricate its
Reference/Rules/publication prerequisites. Remove that route with its last caller.

<a id="exact-owner-split"></a>

### Exact owner split

- Project owns selection, bytes, order/context and source projection.
- Emmy owns Lua parsing, types, diagnostics and selected semantic facts.
- Service owns operation orchestration and the direct result.
- Frontends own config precedence, rendering, protocol framing and exit status.
- Path-sorted snapshots own analysis identity, not WoW runtime order.

## 2. Generated Library semantics and selected WoW checks

1. Make the public builder and ordinary Library reader one real path.
2. Consume one optional typed `source-report.json`; keep plain Lua valid for
   typing.
3. Correct source-backed Blizzard projection semantics:
   `Environment`, `MayReturnNothing`, `NilableContents`, stride groups, mixins and
   selected Secret/restriction facts.
4. Implement one useful direct WoW check only after semantic symbol binding is
   correct. Run only selected checks; no provider/capability/evidence platform.
5. Preserve declaration/member maps and unresolved/lossy projections honestly.

Observe generated and plain-Lua Libraries through the same public check.

## 3. Thin editor and agent adapters

1. LSP uses the same configured project/check owner with exact unsaved text,
   negotiated positions, valid version gaps, standard empty results and stale
   result rejection.
2. MCP exposes only implemented check/search/context tools through one profile;
   bounded `_meta` and valid unknown notifications do not break the session.
3. General Lua completion/hover/definition/references/rename/signature/symbols
   remain upstream EmmyLua.

Record exact framed sequences before marking either adapter observed.

## 4. Direct current-source index, search and context

<a id="direct-source-lookup-and-remaining-platform-removal"></a>

1. Store exact selected file bodies, symbols/spans and useful relations in one
   rebuildable SQLite database.
2. Query a later process without Project/Emmy/Graph replay.
3. Add source-body FTS and exact symbol lookup with deterministic located hits.
4. Render readable exact indexed source and direct relations.
5. Replace/update/delete rows transactionally and distinguish complete empty,
   partial/unavailable and truncated results.
6. Implement real SQLite interruption/lock-wait cancellation where supported.

Observe a body term absent from filenames, a symbol and one TOC/XML relation after
process restart.

## 5. Remove displaced platform code

Only after real callers move:

- compatibility/certification report paths;
- Project replay/publication wrappers not used by retained workflows;
- generic component/capability/presentation results;
- recognizer pack/plan/matcher/proof round trips;
- graph fingerprint/proof/partition products without a retained consumer;
- nested SQLite/hex/JSON and generic Store lifecycle administration;
- Search federation/retained-result pagination and Context pack/continuation
  machinery;
- internal CLI options, fixtures and prose owned only by those paths.

Do not replace them with renamed frameworks or compatibility aliases.

## 6. Measure the real workflow once

After the three public workflows run, measure one selected real Blizzard source:
file count, elapsed time, database size and the first demonstrated bottleneck.
Fix that bottleneck and rerun the same input once. Do not create a benchmark,
telemetry or corpus-certification platform.
