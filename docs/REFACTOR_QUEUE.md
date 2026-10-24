# Implementation order

The [manifesto](ARCHITECTURE.md) defines the product. This is the only ordered
queue; [IMPLEMENTATION.md](IMPLEMENTATION.md) supplies code routes and
[WOW_LINTER.md](WOW_LINTER.md) supplies behavior. Existing Issue numbers identify
related work, not permission to restore historical contracts or test campaigns.

## 1. A useful ordinary addon check

Related: [#200](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/200),
[#201](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/201),
[#197](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/197).

Connect root/TOC/flavor/Library input to the existing shared Emmy session and
public CLI. Resolve one analyzer configuration, initialize Lua51/STD, and
preserve upstream message/code/severity/range
and remove the four-code allowlist. Select checks before doing their work.
Return known findings separately from input/analysis limitations. Remove
compatibility-report and Project publication requirements from this path.
Derive completeness from selected-check dependencies; the current global
component reducer makes native checks partial independently of their findings.
Keep an explicitly selected TOC when its Interface differs: change the current
`load/toc.rs::parse` early rejection into a compatibility diagnostic while
retaining usable file records. Do not silently select another TOC or flavor.
Implement the [concrete configuration rules](IMPLEMENTATION.md#ordinary-configuration):
initial retail/Mainline selection, explicit ambiguity handling, optional target
Interface, upstream `emmy_diagnostics` and default Error threshold. Acquire only
the selected dependency closure. Preserve located malformed-XML failures and
independent usable inputs instead of turning every failed unit into invalid
configuration; recognize the reviewed `Dep` dependency spelling in both readers.

### Source-traced replacement boundary

The existing executable check path is a transitional platform path, not the
owner to extend for the new public operation:

- `apps/wow/src/args.rs` binds the CLI directly to `LocalCommand`;
- `LocalProjectInput::assemble_with_platform` and `LocalProjectBackend` require
  Reference, analyzer-compatibility and Project publication state;
- `Service::check` treats `wow-reference` and `wow-rules` as mandatory healthy
  components even for a type-only request;
- `local/projection.rs::check_context` expands an empty rule selection to every
  registry descriptor, while the target empty `checks` means no WoW rule;
- `GenericFinding` has no explanatory message, and current text output primarily
  renders internal component/capability identities and JSON fragments.

Factor one direct configured addon-check owner in the existing crates. It reuses
bounded Project acquisition/load selection and the shared Emmy mechanisms, but
constructs Reference/Rules only for explicitly selected WoW checks. Its public
result contains target information, source diagnostics and scoped limitations.
Do not fabricate an empty `ReferenceView`, compatibility report, RuleRegistry or
Project generation to satisfy old constructors, and do not translate old global
component health into the new result. Keep the old local Service route only for
transitional callers, then delete it with those callers.

#### Exact owner split

Implement the direct route without a new crate or reverse dependency:

- **`wow-project` owns acquisition and load preparation.** Reuse
  `ProjectInputDirectory`, `ProjectInputFile`, TOC/package resolution and XML
  source maps. `ProjectInputFile` is already a useful exact-byte handoff to
  `LuaWorkspaceFileInput`; do not require `ProjectInputInventory::build`, which
  pulls in the publication configuration and rejects virtual-only programs.
- **Extract a small load target in `wow-project::load`.** It carries flavor,
  optional target Interface and `TocLoadContext`. The current public TOC reader
  accepts only `ProfileIdentity` and binds its digest into the plan; legacy
  profile callers become adapters to the new target. The direct check must not
  invent profile IDs, source digests or schema versions when the user supplied
  only a flavor and optional Interface.
- **Extract pure XML-unit preparation from `wow-project::xml_lua`.** It consumes
  the selected load/package plans, exact captured bytes, compiled analyzer
  identity and direct limits. The current `prepare` function also requires the
  entire `ProjectConfiguration`, generation IDs, capability budgets and
  publication identity. Keep a legacy wrapper that adds those receipts for old
  callers; do not carry them into the direct result.
- **`wow-emmy::session` owns one configurable native engine.** Promote the
  existing registered Main/Library/virtual owner, accept the resolved Emmy
  configuration, set Lua 5.1, initialize embedded STD once, and collect syntax
  unconditionally with member/flow/call facts only when a selected check needs
  them. Do not instantiate independent syntax/reference/flow analyses.
- **The direct session uses the compiled backend internally.** The current
  `EmmyBackendIdentity` requires compatibility-report and surface hashes even
  though runtime pin validation compares only crate name/version/revision/tree.
  Do not fabricate hashes. Provide a compiled identity/direct-workspace path in
  `wow-emmy`; retain the report-derived identity only for legacy replay/probe
  callers until they are removed.
- **`wow-service` owns the use case and public DTOs.** An `addon_check` module
  resolves ordinary configuration, calls Project preparation and the Emmy
  session, optionally invokes selected WoW checks, and returns diagnostics plus
  scoped limitations. It does not depend on generic component health or
  publication status.
- **`wow-cli` owns only transport concerns:** arguments/config-file precedence,
  text/JSON rendering, cancellation wiring and exit policy. LSP/MCP later call
  the same Service owner rather than rebuilding analysis.

The first usable result is a typed addon check with an existing Library. It
requires neither a persistent source index nor completion of all old cleanup.
Its [scope and input selection](IMPLEMENTATION.md#first-implementation-slice)
are explicit: Lua/type diagnostics over admitted selected physical and mapped
XML units, with no WoW rule enabled by default. Missing chunk/handler context
and unsupported XML-only input remain visible limitations until item 3;
API/Secret checks are connected after their semantic fixes in item 4. Do not
advertise this first slice as the completed WoW addon workflow.

## 2. Correct Blizzard declarations for that check

Related: #200/#201; reuse the delivered builder and native Ketho port.

Separate client flavor from API environment; handle member-level `SecureOnly`.
Model `MayReturnNothing`, `NilableContents`, ordered repeated stride groups,
mixin relations and relevant restriction metadata.
Keep unresolved types/omissions visible. Use one source-to-Library pipeline,
consistent reader limits, batched cancellable Git reads and a simple metadata/map
handoff. Do not recreate the removed example producer or `/1` verifier.

The concrete current-source cases are `C_ArtifactUI.GetMetaPowerInfo` (repeated
triples) and `C_EquipmentSet.GetItemIDs` (zero results versus nullable table
contents). Connect the small metadata handoff to the Library reader; leaving it
as an ignored raw sidecar does not finish this step.

## 3. Correct project context and changes

Related: [#109](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/109),
[#117](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/117).

Connect TOC/XML/dependency selection, addon chunk namespace, supported handler
contexts and precise virtual-source maps to the same owner. Distinguish cosmetic
metadata from unknown load conditions. Handle changed/deleted files and changed
load/Library selections without stale results. Retain conservative semantic
reindexing until a narrower invalidation scheme is demonstrably correct.
Allow XML-only input through the physical-inventory boundary. Reuse the existing
single-TOC/package inline analysis and maps; implement the separate per-addon
chunk context rather than mistaking package path prefixes for that context.
Use the [mapped helper design](EMMYLUA_UPSTREAM.md#addon-chunk-context) only for
supported chunk contexts, mark its synthetic calls as context-only, and keep XML
handler signatures and special Blizzard environments separate. Bind adapter
versions in both load routes' configuration keys while retaining the existing
final snapshot/replay identity checks.

Remove the stale `InlineLuaNotAnalyzed` file-closure blocker for supported
extracted bodies without claiming implicit handler context is already complete.
Connect the TOC/package-selected update path, which currently refuses live
updates, to selection/context rebuilding.

## 4. Small WoW checks and editor/agent reuse

Related: [#103](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/103),
[#195](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/195),
[#107](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/107),
[#198](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/198),
[#199](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/199).

Use direct typed checks for supported API context, widget/template, event/callback
and restricted-use cases. API absence and runtime safety must not be guessed.
Bind API uses through Emmy lexical/source identity before exact absence lookup;
the current typed-local shadowing path can misattribute a Lua member error.
Separate unresolved Secret producer semantics from malformed fact identities;
the former limits the affected check, while the latter remains an input error.
Replace the one-file LSP path with the configured project; expose only useful
read-only MCP operations. Keep standard framing, empty results and increasing
LSP document versions, including valid gaps. Remove generic pack/matcher/proof
and frontend profile wrappers as these consumers move. Preserve evaluated-empty
references, accept bounded MCP metadata, and ignore valid unknown notifications.
Keep original error causes and locations through analyzer/frontend projection.
#103 owns useful recognizer/check selection, not a new
engine; early execution selection belongs to the check route above.

## 5. Direct source lookup and remaining platform removal

Related: [#131](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/131),
[#169](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169),
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122),
[#106](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/106).

Store current files/body text/symbols/useful relations in one SQLite database.
Read after reopen without rebuilding analysis; return readable matching excerpts.
Remove nested databases, replay recapture, self-sizing pages and Context packs.
Keep the useful outline/direct-neighbor query, exact scope selection and
empty/partial/truncated result distinctions in the direct implementation.
Reuse existing direct retained-shard SQL reads, opt-in Reference documentation
search, one-scan neighbor batching and exact source-excerpt validation. Add actual
SQL interruption, empty-index handling and transactional changed/deleted rows.
Preserve primary errors alongside cleanup failures instead of replacing them.
The known remaining fingerprint code is removal work with its consumers under
[#118](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/118), not an
already-deleted subsystem or a reason to repair its historical product.

Attach remaining Store lifecycle/DTO/format cleanup
([#111](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/111),
[#194](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194),
[#112](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/112)) to the
reader it replaces. Measure the actual index/query path when it is usable
([#113](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/113)); do not
invent performance targets from line counts.

## Working policy

One manager/worktree, bounded delegation, code first, formatting and affected
strict Clippy. Tests are deferred until functional completion by owner direction;
then reconcile relevant known failures and exercise the advertised workflows.
The supplied audit's test failures remain evidence, not instructions to restore
retired scope. A component or compilation pass alone never completes
[#125](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/125).
