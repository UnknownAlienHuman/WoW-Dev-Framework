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
4. **Input-root authority.** Project must expose one safe preparation entry that
   uses its crate-private component-wise `open_workspace`; Slice 1A must not use
   legacy ambient `ProjectInputDirectory::open`. Each opened root retains a
   request-local role/ordinal `InputRootId` and a nonserialized normalized
   absolute root descriptor. Every unique admitted Lua gets a conservative portable
   `InputPathAuthority` from the lexical absolute descriptor plus its canonical
   relative path, slash-normalized and Unicode-lowercased. This intentionally
   treats case-only spellings as one authority but never claims inode/object
   identity. Reject duplicate authorities, empty Library
   inventories and duplicate snapshot IDs. Repeated TOC occurrences are load
   records, not duplicate authorities. Nested roots and equal relative paths are
   allowed when authority sets are disjoint. Distinct admitted paths are not
   collapsed by content digest, inode, hard-link or snapshot identity.
5. **One Service orchestration boundary.** A single narrow public Service
   operation owns Project preparation, plan/Main binding, scope limitation
   projection, input-authority validation, snapshot construction and the call to
   the existing low-level `check_addon`. Frontends must not construct
   `AddonCheckRequest`, attach arbitrary limitations or combine a plan with
   unrelated snapshots.
6. **Structured input failures.** Preserve source role, snapshot/`InputRootId` and
   root-relative path through Project/Emmy/ConfiguredSession/Service. In
   particular, `LibraryHealthFailed` is invalid caller input and maps its snapshot
   back to a request-local root ordinal; it is not `ComponentUnavailable`. Use one
   small addon-check operation error with only `invalid_input`, `cancelled`, or
   `internal`, not a universal `ServiceError` provenance
   expansion, and never recover origin from message text or synthetic paths.
7. **Strict config and exact result contract.** Read an explicit config only
   through a narrow Project helper wrapping the crate-private no-follow opener;
   legacy ambient `ProjectInputDirectory::open` is not a Slice 1A trust boundary.
   Main uses the compiled pin and universe
   `Project`; each Library root becomes one `BlizzardUi` snapshot. Retain per-root
   limits and add fixed aggregate ceilings: 64 roots, 8,192 Library files and
   256 MiB. Canonical paths are root-relative. Empty Main is complete only when
   no language unit/blocker was selected; deferred XML makes it partial. Config
   is one explicit regular no-follow file using existing duplicate-key/depth-
   bounded JSON, a 256 KiB cap and unknown-field rejection. CLI/config merge
   defaults to text, `fail_on: error` and upstream default Emmy diagnostics. Slice
   1A has no WoW-check option; it always runs zero WoW rules. Strictly prevalidate
   the nested diagnostic object and every `globalsRegex`, because pinned upstream
   accepts unknown keys and silently drops invalid regexes. Reject the upstream
   `diagnosticInterval` key in Slice 1A: it is consumed only by EmmyLua LS debounce,
   not code analysis, and must not create a false analyzer-configuration identity.
   Render a complete
   result/error into a bounded 32 MiB buffer before one write: results to stdout,
   admitted-format errors to stderr, early usage errors as text. Sort diagnostics
   and limitations by explicit role/path/span/code keys before either rendering.
   Diagnostic/result volume beyond the public bound is `invalid_input`;
   serializer defects, cancellation-handler setup and output I/O are `internal`
   (`4`). Check cancellation after encoding and before the first output byte;
   once output starts, never replace or append another envelope. Operation-error
   messages are capped at 4 KiB; only Library errors carry a zero-based
   `root_index`, and all paths are role-relative. Severity thresholding uses
   an explicit Error > Warning > Information > Hint ranking; `Unknown` remains
   visible and conservatively meets every threshold.

`check_addon` must stop adding unconditional chunk-context incompleteness and
hard-coded `Partial`. The same configured Emmy owner must expose a bounded Main
requirement collector over its registered AST: match only
`LuaLiteralExpr`/`LuaLiteralToken::Dots` with no enclosing `LuaClosureExpr`,
return at most one exact path/content/span record per file, and never scan raw
text or start another parser/session. Service validates that report against the
exact Main snapshot and limits only affected files. Completeness follows only
facts required by selected `LuaTypes` units.

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
- canonical Slice 1A plan: **ready after T23 and T32–T39 are implemented
  together, including the T31/T33/T36/T38/T39 refinements recorded through D80**;
- first ordinary `wow check`: **not implemented**;
- any addon/editor/search workflow observed: **no**;
- guarantee of no future runtime defect: **no**. The documentation now closes the
  known profile, eager-XML, global-completeness, input-authority, arbitrary-request
  assembly and error-origin traps without pretending the runtime is proven.
