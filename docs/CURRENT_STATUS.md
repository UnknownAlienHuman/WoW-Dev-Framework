# Current implementation status

**Documentation/source audit base:** `main` after PR #269
(`f674ccc7615d63c76c0781aef2150e49ea442bf8`)  
**Baseline register:** PR #244 / `c36a058`  
**Active traces:** T01–T39 in
[IMPLEMENTATION_TRACE_REGISTER.md](IMPLEMENTATION_TRACE_REGISTER.md)

This is the only live delivery checkpoint. The queue orders work; owner/domain
guides define stable contracts; the trace register records source decisions and
exact observations. README, AGENTS, skills and Issues must link here rather than
copy a moving delivery list.

## Verified documentation boundary

- all 329 baseline Markdown files remain represented exactly once in the baseline
  register: 287 `read / traced`, 42 `read / not_applicable`, no partial or
  unchecked row;
- all 60 active Markdown files and all 21 open implementation Issues were checked
  against the current Project, Emmy and Service owners;
- local Markdown paths/anchors and all four skill copies remain valid;
- obsolete E1/E2/E3/platform specifications remain outside the active tree;
- historical audits and the supplied Claude review remain evidence, not live
  product instructions.

`traced` means a producer/handoff/reader chain was inspected. `implemented` means
code exists. `observed` requires an ordinary command or protocol sequence and its
visible result. Compilation, Clippy and source review never make a workflow
observed.

## Implemented internal owners

| Boundary | State |
| --- | --- |
| configured Emmy diagnostics preserve upstream code, exact message, severity and byte range; Lua 5.1 and embedded STD share one owner | implemented, user workflow unexecuted |
| exact Main + explicit Library + optional virtual `ConfiguredAnalyzerSession`; diagnostics and selected semantic collection share it | implemented, user workflow unexecuted |
| direct low-level `wow-service::addon_check` result without Reference, Rules, Project publication, graph, store or presentation construction | implemented, ordinary caller open |
| compiled Emmy identity is distinct from compatibility-report evidence | implemented, user workflow unexecuted |
| exact Main/canonical Library set/virtual/config compatibility gates owner reuse | implemented, user workflow unexecuted |
| finite located limitation categories and reachable package-plan issue mapping | implemented, standalone TOC mapper open |
| one three-valued TOC predicate reduction | implemented, ordinary workflow unexecuted |

Library argument order is not an open defect: snapshots are sorted by exact ID
before registration and the same canonical set is compared for reuse.

## Exact open boundary: Slice 1A

There is still no ordinary public addon checker. The executable uses the
transitional `LocalCommand -> LocalProjectBackend -> generic Service` route.
Slice 1A must connect ordinary input through one direct Service operation without
preserving that platform.

The implementation boundary is now explicit:

1. **Profile-free target and plan.** Add Project-owned `AddonLoadTarget` plus a
   projection scope; replace retained `ProfileIdentity`/mandatory-Interface plan
   identity and validation. Slice 1A supplies flavor `retail`, no target Interface
   and no condition context.
2. **Physical projection.** Parse the explicit TOC and direct `.lua`; retain exact
   XML records/spans but do not open XML. Deferred selected XML is a located
   partial limitation and cannot erase physical Lua diagnostics.
3. **Requested-scope limitations.** Add a standalone `ProjectLoadPlan` mapper for
   `LuaTypes`. Required dependencies, unresolved included conditions, missing
   selected Lua and deferred XML block that scope. Optional absence, unused source
   Interface and nonblocking metadata remain Project observations.
4. **Input-root authority.** Resolve path bases lexically without following root
   links, then open every addon and Library root once through Project's
   component-wise held-root/no-follow readers. Reject duplicate roots, empty Library
   inventories, duplicate snapshot IDs and any same admitted physical Lua file
   appearing in more than one role or Library root. Mere ancestor/descendant roots
   are not rejected when their admitted file sets are disjoint. Relative-path
   equality across distinct roots is allowed.
5. **One Service orchestration boundary.** A single narrow public Service
   operation owns Project preparation, plan/Main binding, scope limitation
   projection, input-authority validation, snapshot construction and the call to
   the existing low-level `check_addon`. Frontends must not construct
   `AddonCheckRequest`, attach arbitrary limitations or combine a plan with
   unrelated snapshots.
6. **Structured input failures.** Preserve source role, snapshot/root token and
   root-relative path through Project/Emmy/ConfiguredSession/Service. In
   particular, `LibraryHealthFailed` is invalid caller input and maps its snapshot
   back to the configured Library root; it is not `ComponentUnavailable` and is
   never recovered by parsing message text or synthetic analyzer paths.
7. **Exact result and frontend contract.** Main uses the compiled pin and universe
   `Project`; each Library root becomes one `BlizzardUi` snapshot under retained
   limits. Canonical paths are root-relative. Empty physical Main with deferred
   XML is an empty Project snapshot plus a located partial limitation. Successful
   or partial results go to stdout; pre-result input/cancel/internal errors after
   format admission go to stderr, with one bounded JSON error object when
   `--format json` is selected. CLI syntax/option errors before format admission
   use one text usage error on stderr. Unknown config fields are rejected; no
   implicit config discovery.

`check_addon` must stop adding unconditional chunk-context incompleteness and
hard-coded `Partial`. Completeness follows only facts required by selected
`LuaTypes` units.

## Ordered work

1. **Slice 1A:** explicit root + TOC + `retail` + Library roots -> strict config ->
   one Service operation -> profile-free physical Project plan -> scope blockers
   -> collision-free exact snapshots -> existing configured owner/check ->
   root-relative text/JSON result and deterministic exit.
2. **Slice 1B:** dependency closure, exact selected order, mapped XML/chunk/handler
   context and coherent updates.
3. **Slice 1C:** channel, `LoadWith`, implicit bindings, positive target Interface
   and secure/special contexts.
4. Generated-Library metadata and selected WoW checks.
5. Thin LSP/MCP adapters, then the direct SQLite body/symbol index and readable
   context.

The exact acceptance conditions are in
[REFACTOR_QUEUE.md](REFACTOR_QUEUE.md).

## Readiness verdict

- baseline/current documentation reconciliation: **complete at the named base**;
- canonical Slice 1A plan: **ready after T32–T39 are implemented together**;
- first ordinary `wow check`: **not implemented**;
- any addon/editor/search workflow observed: **no**;
- guarantee of no future runtime defect: **no**. The documentation now closes the
  known profile, eager-XML, global-completeness, input-authority, arbitrary-request
  assembly and error-origin traps without pretending the runtime is proven.
