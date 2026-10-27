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
| exact Main/canonical Library set/virtual compatibility gates owner reuse | implemented; canonical semantic diagnostic-config equivalence open |
| finite located limitation categories and reachable package-plan issue mapping | implemented, standalone TOC mapper open |
| one three-valued TOC predicate reduction | implemented, ordinary workflow unexecuted |

Library argument order is not an open defect: snapshots are sorted by exact ID
before registration and the same canonical set is compared for reuse.

## Exact open boundary: Slice 1A

There is still no ordinary public addon checker. The executable uses the
transitional `LocalCommand -> LocalProjectBackend -> generic Service` route.
Slice 1A must connect ordinary input through one direct Service operation without
preserving that platform.

The implementation boundary is explicit:

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
4. **Input-root authority.** Project exposes one safe preparation entry using
   crate-private component-wise `open_workspace`; Slice 1A does not use ambient
   `ProjectInputDirectory::open`. Each root retains request-local role/ordinal
   `InputRootId`, display descriptor, nonserialized normalized absolute authority
   descriptor and held capability. Each unique admitted Lua gets conservative
   portable `InputPathAuthority = root descriptor + canonical relative path`,
   slash-normalized and Unicode-lowercased. This is selected-path authority, not
   inode/object/content/hard-link/snapshot identity. Reject duplicate authorities,
   empty Library inventories and duplicate whole snapshot IDs. Repeated TOC
   occurrences are load records. Nested roots/equal relative paths are allowed
   when joined authority differs.
5. **One Service orchestration boundary.** One narrow public Service operation
   owns Project preparation, plan/Main/authority validation, requested blockers,
   snapshots, snapshot/root mapping, same-owner Emmy requirements and internal
   `check_addon`. Frontends never construct lower requests or attach limitations.
6. **Structured failures.** Preserve role, snapshot/`InputRootId`, role-relative
   path, stable code and bounded message. `LibraryHealthFailed` is caller
   `invalid_input`, not `ComponentUnavailable`. Public operation errors are only
   `invalid_input`, `cancelled`, `internal`; do not expand global `ServiceError`
   into a provenance platform or recover origin from strings/synthetic paths.
7. **Strict config and result contract.** One explicit regular no-follow config,
   duplicate-key/depth-bounded JSON, 256 KiB, unknown-field rejection and no
   discovery. Main uses compiled pin/`Project`; each Library root one `BlizzardUi`
   snapshot. Aggregate Library ceilings: 64 roots, 8,192 files, 256 MiB.
   Canonical paths are role-relative. Empty Main is complete only with no selected
   unit/blocker; deferred XML makes it partial. Defaults: text, `fail_on: error`,
   upstream default diagnostics; no WoW-check option. Prevalidate exact pinned
   diagnostic keys and all regexes. Reject `diagnosticInterval`: pinned code
   analysis does not read it; EmmyLua LS uses it only for debounce. Additionally,
   replace raw-JSON configuration equivalence with one canonical analysis
   projection: keep `enable`; sort/dedup `disable`, `enables`, `globals` and
   `globalsRegex`; sort severity by diagnostic code; omit `diagnosticInterval`.
   Omitted diagnostics, `{}`, reordered or duplicated set-like fields must reuse
   the same owner when analysis semantics match. Pre-encode at most 32 MiB, sort
   output explicitly, check cancellation after encoding/before first byte and
   write one stdout result or one stderr pre-result error. Message <= 4 KiB;
   Library-only zero-based `root_index`; role-relative paths. Explicit severity
   rank Error > Warning > Information > Hint; preserved `Unknown` meets every
   threshold.

`check_addon` must stop unconditional chunk incompleteness/hard-coded `Partial`.
The same configured Emmy owner exposes a bounded Main requirement collector over
registered AST: match `LuaLiteralExpr`/`LuaLiteralToken::Dots` with no enclosing
`LuaClosureExpr`, return at most one exact path/content/span per affected file,
never raw-scan or start another parser/session. Service validates against exact
Main and limits only affected files.

## Ordered work

1. **Slice 1A:** explicit root + TOC + `retail` + Libraries -> strict config -> one
   Service operation -> profile-free physical plan -> blockers/authority -> exact
   snapshots -> configured owner/check -> root-relative result/error.
2. **Slice 1B:** dependency closure, selected order, mapped XML/chunk/handler
   context and coherent updates.
3. **Slice 1C:** channel, `LoadWith`, bindings, positive target Interface and
   secure/special contexts.
4. Generated-Library metadata and selected WoW checks.
5. Thin LSP/MCP adapters, then direct SQLite body/symbol index and context.

Exact acceptance is in [REFACTOR_QUEUE.md](REFACTOR_QUEUE.md).

## Readiness verdict

- baseline/current documentation reconciliation: **complete at the named base**;
- canonical Slice 1A plan: **ready after T23 and T32–T39 are implemented together,
  including the active T31/T33/T36/T38/T39 refinements in the trace register**;
- first ordinary `wow check`: **not implemented**;
- any addon/editor/search workflow observed: **no**;
- guarantee of no future runtime defect: **no**. Known profile, eager-XML,
  completeness, input-authority, arbitrary assembly, error-origin and diagnostic-
  identity traps are explicit without pretending runtime proof.
