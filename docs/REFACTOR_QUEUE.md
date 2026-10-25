# Implementation order

Read [current status](CURRENT_STATUS.md) before taking an item. The
[manifesto](ARCHITECTURE.md) defines the product,
[IMPLEMENTATION.md](IMPLEMENTATION.md) defines target owners, and
[WOW_LINTER.md](WOW_LINTER.md) defines domain behavior. This is the only ordered
queue. Issue history cannot restore removed platform contracts.

A queue item is complete only when its actual input reaches its real reader and
visible result. Compilation is not a user observation.

## 1. First usable ordinary addon check

Related: [#200](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/200),
[#201](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/201),
[#197](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/197).

### Delivered prerequisites — do not repeat

| PR | Delivered lower boundary | Remaining limitation |
| --- | --- | --- |
| #246 | configured Emmy diagnostics retain code, exact message, severity and byte range; standalone Lua 5.1 STD is initialized | transitional LSP remains one-file |
| #247 | `ConfiguredAnalyzerSession` over exact Main, Library and optional virtual snapshots; caller diagnostics; one STD; explicit semantic collection | snapshots are not ordinary Project context |
| #248 | direct `wow-service::addon_check::check_addon`, findings plus limitations, no legacy Reference/Rules/publication/graph/store stack | no root/TOC/Library acquisition or CLI adapter |
| #249 | compiled Emmy identity without fabricated report hashes | legacy report-backed callers remain transitional |
| #250 | exact Main identity in immutable-owner reuse admission | no retained public CLI owner |

Exact-head runs #418, #422, #425, #440 and #445 passed. None is an observed addon
workflow.

### Remaining item-1 path

```text
root / TOC / flavor + product channel / Library paths
  -> required/available optional dependencies + reverse triggers
  -> typed package/unit context + exact selected order
  -> implicit binding documents + bounded physical/XML/Library input
  -> existing ConfiguredAnalyzerSession
  -> existing wow-service addon_check
  -> CLI text/JSON diagnostics, limitations and exit status
```

Implement in this order:

1. add the small ordinary configuration DTO and CLI/config precedence;
2. extract a load target containing flavor, explicit product channel, optional
   positive target Interface and `TocLoadContext`, without `ProfileIdentity`;
3. use the finite dependency aliases: required `Dep`, `Dependencies`,
   `DependsOn`, `RequiredDep`, `RequiredDeps`; optional `OptionalDep`,
   `OptionalDeps`; select only relevant transitive closure;
4. model `UseSecureEnvironment`, `LoadIntoEnvironment`,
   `SuppressLocalTableRef`, bootstrap and supported load-first/load-phase facts;
5. model `OnlyBetaAndPTR` against explicit `Stable | Ptr | Beta`; retain
   `LoadWith` as a source-located reverse trigger, never a dependency;
6. parse official source `Interface: 0` as an unknown/sentinel compatibility
   value rather than malformed input; retain `Secure` as source-backed
   unsupported context until reviewed semantics exist—never alias it by name to
   `UseSecureEnvironment` or another security model;
7. recognize bounded `DefaultState` lifecycle metadata without making an explicit
   source check partial; add `SavedVariablesMachine` declaration scope while
   never reading user SavedVariables content;
8. discover at most one exact `Bindings<suffix>.xml` under the held addon root,
   and map `<Binding>` Lua as callback units with only source-verified `keystate`;
   absence is normal;
9. fix tri-state predicate reduction so known allow/exclude results are not
   poisoned by irrelevant unknown aliases or gates;
10. preserve exact package/TOC/XML/callback order and context in a validated value
    beside path-sorted snapshots; never treat Emmy registration order as runtime;
11. reuse `ProjectInputDirectory`/`ProjectInputFile` safety and bounds without
    `ProjectInputInventory` or publication configuration;
12. split pure XML/binding unit-map preparation from publication wrappers;
13. preserve independent captured units and located failure when one included or
    bindings XML document is malformed;
14. admit physical-only, physical+virtual, XML-only and valid empty language
    scopes without a dummy Lua file;
15. pass exact Main/Library/virtual snapshots plus validated context to the direct
    Service result;
16. render actual code/message/severity/location plus scoped limitations and the
    documented `0/1/2/64/130/4` exit policy;
17. record the first exact command/input/result only after it is actually run.

Empty/omitted `checks` selects no WoW rule while Emmy diagnostics remain enabled.
Missing chunk, handler, binding or unresolved security context is an explicit
limitation, not a clean result.

Source evidence:

- [TOC_LOAD_CONTEXT_AUDIT.md](TOC_LOAD_CONTEXT_AUDIT.md) — dependency aliases,
  secure/private-table context, tri-state predicates and selected order;
- [TOC_TRIGGER_BINDINGS_AUDIT.md](TOC_TRIGGER_BINDINGS_AUDIT.md) — product
  channel, reverse triggers, implicit bindings, known metadata and official
  compatibility/security uncertainty.

### Source-traced replacement boundary

The current executable check route is transitional:

- `apps/wow/src/args.rs` binds to `LocalCommand`;
- `LocalProjectInput::assemble_with_platform`/`LocalProjectBackend` require
  Reference, compatibility and Project publication state;
- generic `Service::check` requires Reference/Rules for type-only work;
- empty old rule selection expands to every registered rule;
- the old component/capability result is not the direct diagnostic result.

Do not fabricate empty Reference/Rules/report/Project objects, adapt the old
result around the new one, or build another analyzer owner. Move callers, then
remove the route.

### Exact owner split

- **`wow-project`** — config resolution, safe selected acquisition, small load
  target, finite directive semantics, dependencies/triggers, product channel,
  official Interface/security metadata, exact ordered context, implicit binding
  discovery and pure mapped XML/binding units. Legacy profile/publication wrappers
  adapt while callers remain.
- **`wow-emmy`** — existing `ConfiguredAnalyzerSession`, compiled provenance,
  Lua 5.1, STD and requested semantic collection. Keep snapshots path-sorted and
  add only demonstrated mapped context seams; never invent unresolved `Secure`
  behavior.
- **`wow-service`** — existing `addon_check`; add config orchestration, validated
  load/context input and selected check dependencies.
- **`wow-cli`** — arguments, config precedence, cancellation, rendering, exit.
- **LSP/MCP** — later adapters to the same Service/Project owner.

## 2. Correct Blizzard declarations used by the check

Related: #200/#201; reuse the public builder and native Ketho port.

Carry through the real reader:

- member-level `SecureOnly` and reviewed environment facts;
- `MayReturnNothing` as a zero-result alternative;
- `NilableContents` separately from container nilability;
- ordered numeric repeated `StrideIndex` groups;
- mixin/restriction/origin facts;
- unresolved/unrepresentable forms as located limitations, never silent `any`.

Concrete cases are `C_ArtifactUI.GetMetaPowerInfo` and
`C_EquipmentSet.GetItemIDs`. Keep one optional `source-report.json` DTO. Plain
Lua Libraries remain valid for typing and limit only dependent checks.

Replace per-file Git processes with one bounded selected batch read and truthful
child cancellation/reaping. Do not restore old producers/verifiers/converters.

## 3. Correct addon/XML/binding context and updates

Related: [#109](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/109),
[#117](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/117).

Use the same configured owner plus validated Project context for:

- physical Lua and mapped UI XML Lua;
- XML-only addons;
- one private table per supported ordinary addon and lexical chunks per file;
- source-verified UI handler parameters;
- mapped `<Binding>` callbacks with `keystate`, distinct from top-level chunks;
- suppressed private tables, package/per-unit secure overrides,
  bootstrap/load-first, free-inline and binding phases;
- exact virtual-to-XML mappings including entities/discontinuous pieces;
- explicit limitations for unresolved `Secure` metadata rather than guessed
  execution context.

Synthetic helper/wrapper code is context-only and never becomes user API/flow
facts or locations. Binding callbacks do not receive addon ellipsis semantics and
are not inserted as top-level TOC execution.

Reuse an owner incrementally only when selected membership, product channel,
Interface/security metadata, triggers, exact ordered context, Library identity and
analyzer config are unchanged. TOC/XML/bindings/dependency/trigger/Library/
channel/config changes rebuild selection/context cold. Delete/rename removes
declarations, virtual units, maps, diagnostics and index rows together.

Remove stale blanket `InlineLuaNotAnalyzed` only for supported extracted bodies;
do not claim handler/binding context. Connect selected-load live updates to the
same rebuild owner.

## 4. Small WoW checks and frontend reuse

Related: [#103](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/103),
[#195](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/195),
[#107](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/107),
[#198](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/198),
[#199](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/199).

Enable only direct typed checks with explicit dependencies. API existence requires
original Emmy receiver binding and selected API origin; a local named `C_*` is
not a Blizzard namespace. Secret checking separates coherent unresolved
semantics from malformed/stale identities. TOC `Secure` uncertainty is a separate
Project context limitation and must not be conflated with API `SecureOnly` or
Secret-value flow.

LSP keeps overlay/version/coordinate ownership but passes exact overrides to the
configured project. Accept increasing version gaps, reject stale results and use
standard empty/null responses. MCP keeps one standard profile, bounded extension
metadata, unknown-notification behavior and only implemented tools. CLI/LSP/MCP
serialize common Service results.

## 5. Direct source lookup and platform removal

Related: [#131](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/131),
[#169](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169),
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122),
[#106](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/106).

After addon checking works, store one current selected source set in normal
SQLite tables: sources, files with body text, symbols/spans and only consumed
relations. Query after restart without Project/Emmy/Graph replay. FTS indexes body
text; exact names use separate lookup. Preserve exact source/revision/flavor/
channel/context.

Reuse useful direct retained-shard SQL, opt-in Reference documentation search,
one-scan neighbor batching and exact excerpt validation. Add real SQLite
interruption/bounded lock wait, transactional updates and primary-error
preservation.

Remove nested SQLite/hex/JSON, replay recapture, fixed-point pagination, Context
packs, journals, generic lifecycle and proof/fingerprint machinery only as real
readers move. Do not create another universal store/graph/result/RAG platform.

Attach Store/format cleanup (#111/#194/#112) to replaced readers. Measure the
necessary actual workflow under #113, never code size.

## Working policy

One manager/worktree, bounded delegation, code first, formatting and affected
strict Clippy. Tests remain deferred until functional completion by owner
direction; compile/Clippy is never a product pass.

Update [CURRENT_STATUS.md](CURRENT_STATUS.md) when merged code changes the next
work and [IMPLEMENTATION_TRACE_REGISTER.md](IMPLEMENTATION_TRACE_REGISTER.md)
when evidence or an exact observed result changes. The baseline register is not a
progress diary.