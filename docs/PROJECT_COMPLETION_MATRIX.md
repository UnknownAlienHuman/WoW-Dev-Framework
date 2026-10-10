# Project completion matrix

**Audited:** 2026-09-19 America/New_York. Execution inventory refreshed 2026-10-09
through the scoped W16 guarded cross-epoch selection checkpoint below.

This is the current execution ledger. [IMPLEMENTATION_HANDOFF.md](IMPLEMENTATION_HANDOFF.md)
remains the normative I0–I7 plan; do not restart its historical bootstrap steps.
The [audit](AUDIT_2026-09-19.md) records evidence, defects and bounded next tasks.

## State vocabulary

- **Partial executable:** real Rust implementation exists; this is not full package acceptance.
- **Inactive code:** source exists outside the tested workspace; no build or acceptance claim.
- **Not started:** no Rust owner implementation for the named scope.
- **Implemented:** every required operation, fixture, checksum and acceptance gate for the named package passes.
- **LaunchGateComplete:** all evidence for the selected launch scope passes independently of ordinary workspace CI.

The audit advances no full package or launch gate to Implemented/Complete. Existing
code already crossed several historical pre-implementation fixture-freeze gates;
recording that code honestly does not waive those gates or populate missing evidence.

## Executable inventory

The root [Cargo.toml](../Cargo.toml) activates **16 members**, including the internal Reference Pack builder.

| Component | Observed executable slice | Remaining acceptance boundary |
|---|---|---|
| `wow-core` | Typed identities, canonical JSON, evidence, coverage, results and operation primitives | Reconcile the complete E0-A contract/test matrix; compilation is not the acceptance ledger |
| `wow-emmy` | Real pinned analyzer adapter, explicit Main/Library workspaces, same-session XML virtual units, syntax/semantic diagnostics, direct member calls and scoped local-flow facts | E0-C fixture/pin/checksum closure; additional semantic operations and update probes are not inferred from parser compatibility |
| `wow-project` | Explicit inventories, bounded TOC/XML/package receipts, analyzer bindings, immutable generations, exact source artifacts/handles, guarded updates, publication and physical/standalone/package native replay | E0-D fixture identity closure; effective XML receiver/load semantics, full load acceptance, overlays and full durable project publication |
| `wow-rules` | `wow.api.exists@1` over physical Main and exact-static XML inline facts; `wow.secret.local_operation@1` over its bounded physical flow slice | E0-E normative fixtures, exact prerequisite identities and complete capability/negative-authority cases; no inferred XML receiver/runtime authority |
| `wow-service` | E0 status/check over immutable normalized contexts, mixed physical/XML rule scopes and exact XML finding projection; separate ReferenceView administration/publication; native Project/Graph replay for guarded READY-target source-authority export | E0-F end-to-end fixture/CLI closure; full E1 Reference Pack and later public operation families |
| `wow-store` | Typed SQLite objects, catalogs/CAS, journal, leases, GC and integrity; manifested recovery, native backup, isolated restore, same-epoch replacement, whole-instance quarantine/guarded restore, portable holds and inactive physical v1/v2 to v3 migration with guarded staging, target export, private immutable mapped-root/Current preparation and portable original-epoch selector/hold authority | Full E1-A/E2-D acceptance; live cross-epoch selection/reconciliation, original source SQL/history hydration, portable full migration history, payload/runtime migration, domain quarantine, interrupted-write/power-loss and cleanup faults remain open |
| `wow-reference` | Native source/model/corrections/aliases, compatibility imports, persistent ReferenceView and publication | E0-B/E1-B normative fixture and full Reference Pack/coverage acceptance |
| `wow-annotations` | Native Ketho-derived library projection, alias/type/catalog/inheritance/navigation slices and consumer tests | Full E1-C contract/corpus parity; scoped passing consumers are not universal semantic certification |
| `wow-graph` | Immutable snapshots, proposals/registries, neighbors, producer partitions, bounded queries, exact retained derivation/conflict records, evidence resolution and v1/v2 persistence | Complete producer-chain coverage and automatic conflict assessment, normative fixtures and coherent E2-D publication |
| `wow-recognizers` | Structured facts, pack parser/compiler, bounded matcher and all 26 active E2-B rule IDs published through service partitions: three `core.lua.*`, five `core.toc.*`, four `core.xml.*`, three `core.state.*`, plus signal, callback, hook and library families | Full rule-specific fixtures/checksum freeze, public CLI/real-addon graph-build acceptance and E5 governance remain separate |
| `wow-render-contract`, `wow-ketho-literals`, `modules/ketho-literals` | Typed literal wire contract, native renderer and Wasm guest | Scoped algorithm implementation, not public application/release acceptance |
| `tools/xtask` | Native policy/source/manifest/library checks, guarded exact fast-forward, managed checkout materialization and checkout-free exact GitHub API/blob snapshots under explicit `auto`/`prompt`/`never` policy with durable replay | Lower-layer hostile-network qualification, background scheduling, live platform/fault acceptance and full schema/fixture closure remain incomplete |

`bridges/literal-host` is an intentionally separate Cargo workspace with its own
CI lane. It is not a seventeenth root member.

`apps/wow-reference-builder` is now an active root member and a service-only internal
frontend for build, validate and rebuild-compare. It performs confined staging,
independent read-back, durable filesystem-effect reconciliation and guarded atomic
finalization. This executable slice is not full E1-D acceptance: real external
evidence, Windows/process-loss acceptance and the remaining Reference Pack gates stay open.

`apps/wow` now provides a one-shot CLI over inline inputs or an explicit
[disk file manifest](../apps/wow/FILES_INPUT.md); see
[LOCAL_INPUT.md](../apps/wow/LOCAL_INPUT.md). `wow-search`, `wow-context`, `wow-cbm` and `tools/wow-release` have
no Rust implementation in the audited source. Their documentation is not a binary.

## Work-package ledger

| Packages | Actual state | Next requirement |
|---|---|---|
| E0-A–E0-E | Partial executable | Close exact normative fixtures and prerequisite identity/checksum chains, one owner at a time |
| E0-F | Partial service and inline/disk-input public CLI | Thin `wow status`/`wow check`, owner-composed fixture, output/exit/cancellation/resource gates |
| E1-A–E1-C | Partial executable | Finish only verified missing contract/acceptance slices; do not recreate existing store/reference/annotation implementations |
| E1-D | Partial executable Reference Pack service and active internal builder with durable materialization/finalization | Close external parity/license/rebuild evidence, Windows/process-loss acceptance and complete package gates; code presence is not `ValidatedLocal` |
| E2-A–E2-B | Partial executable | All 26 active E2-B rule IDs are service-published after the W11 semantic repair, TOC, XML and state slices; close public CLI/full-pipeline fixtures and package acceptance |
| E2-C–E2-D | Partial source index, manifested store, native live pair service/CLI, cancellable updates, exact Library/fact-profile generation binding, retained physical parser updates and durable physical update/removal publication; bounded retention/GC, backup/recovery, same-epoch replacement, inactive migration/READY preparation and portable source selector/hold authority | Dependency-specific fact reuse, standalone/package durable updates, live cross-epoch selection/reconciliation, portable full migration history and complete E2 acceptance remain open; see [REPLAY_PUBLICATION.md](../crates/wow-project/REPLAY_PUBLICATION.md) |
| E3-A–E3-C | Not started | Exact Blizzard source universe, context owners and service/CLI after E2 closure |
| E4-A–E4-C | Not started | Search, lineage/migration/static impact and routing after A0 prerequisites |
| E5-A–E5-C | Not started | Calibration, independent review/holdout and governed publication lifecycle |
| E6-A–E6-B | Not started; optional/disabled | External candidates never block the local lane or gain exact/negative authority |
| E7-A–E7-B | Not started | Selected frontend conformance, then build/sign/bundle/install/update/rollback/support |

## Immediate execution order

The operator's 2026-09-23 priority is functional code before expanded tests.
Existing acceptance requirements remain separate and must not be reported passed.

1. **I0-F/W10/W11 functional path:** `apps/wow` routes explicit materialized inputs through
   actual project/analyzer/reference/rule owners. Managed source snapshots and W09 load
   closure feed W10 XML semantics. All 26 W11 rules are service-published after the
   semantic-repair checkpoint, TOC, XML and state slices; continue with public CLI and
   mutation acceptance without inventing receiver, runtime or negative authority.
2. **E0 acceptance:** retain existing fixtures/checksums; close their remaining
   evidence after functional implementation, not as an endless prerequisite for it.
3. **I1:** the builder/service/materialization code path is active. Close remaining
   store/reference/annotation and Reference Pack evidence gates, then execute the
   deferred process-loss, Windows and real-input acceptance without recreating owners.
4. **I2 then I3:** finish graph/recognizers, full project indexing and coherent
   persistence before context. Run one exact real-addon/profile evaluation for A0.
5. **I4–I7:** follow the existing handoff; do not front-load optional providers,
   transports or release scaffolding ahead of the runnable local product.

[The audit task table](AUDIT_2026-09-19.md#bounded-follow-up-tasks) supplies file
boundaries and acceptance criteria. Work sequentially in `main`, without task
branches/worktrees or force pushes. Read back each published checkpoint.

## Launch gates

| Gate | State | Concrete reason |
|---|---|---|
| R0 | Blocked | Materialized-input CLI exists; required E0 fixture/checksum and whole-command acceptance remain open |
| A0 | Blocked | R0 plus full E1–E3 acceptance and real-addon/profile evaluation |
| A1 | Blocked | A0, E4 and the selected implemented E7-A frontend |
| B0 | Blocked | A1 and real E5 governance evidence; E6 remains optional |
| V1 | Blocked | Selected product scope, full E7-A/E7-B and supported Windows clean-install/update/rollback evidence |

## Evidence rules

Baseline `e2c74314bb7ccde4a9b48c049dfeacc157259960` passed
[CI 34735568141](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/actions/runs/34735568141).
The duplicate-key implementation is
[`c10579d`](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/commit/c10579d359b5f0044fc6fcdfef3b353c5caa65dc),
with exact checks in
[CI 35487232004](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/actions/runs/35487232004).
A subsequent documentation commit has a different SHA and its own CI record.
The W08 managed-source code checkpoint is
[`a69c685`](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/commit/a69c685315e32d8afde28d758ecf2b495f02a80b),
validated by focused run
[36821527175](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/actions/runs/36821527175)
and exact-tree publication run
[36821677002](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/actions/runs/36821677002).
That checkpoint ran formatting, strict `xtask` Clippy and repository policy on Linux;
tests, Windows, live-source acquisition and process/network fault injection were not run.
The checkout-free API snapshot checkpoint is
[`b6ea598`](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/commit/b6ea5981b2372468e332b30c0bdd697ce3b52301),
checked as tree `312caed69e65a5b907dfa219e2fb3aa739837ece` by focused run
[36826290606](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/actions/runs/36826290606).
Formatting, strict `xtask` Clippy and repository policy passed; tests, a live donor
call, Windows and network/process fault injection were not run.

The W10 XML rule-consumer checkpoint is
[`260cf430`](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/commit/260cf430a22fdc3f86da33a9c363946cc31aca00),
checked as tree `df3db6e01a804f44f57cbfe11f1a15cec1face60` by focused run
[36966297105](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/actions/runs/36966297105).
Formatting, strict affected-crate Clippy, existing affected-crate tests, `xtask check` and exact
file-set/tree publication passed on Linux/Rust 1.99.0. The retained artifact tree and SHA-256 files
were read back. Windows, WoW runtime, real-addon command acceptance and full workspace/package
gates were not run and remain open.

The audit used that CI's retained source artifact and verified both the archive
SHA-256 and embedded commit identity. Local Rust execution was unavailable; no
local cargo, Windows runtime, installation or product acceptance run is claimed.
Passing code tests are not evidence that dormant apps, placeholder fixture
manifests, a current WoW profile or later release gates passed.

## Verified code checkpoint

All eight jobs in CI run `35487232004` concluded success for
`c10579d359b5f0044fc6fcdfef3b353c5caa65dc`: Linux/Windows locked checks,
strict Clippy, debug/release tests and rustdoc; updated dependencies; rolling
parser; Linux/Windows semantic consumers; Linux/Windows Wasm swap/rollback.
This is code-checkpoint evidence, not a full package or public launch certificate.

## XML inline semantic implementation

TOC/XML source capture, syntax indexing and inline extraction now feed a bounded,
generation-bound EmmyLua semantic pass in the same physical Main/Library session. Mapped
syntax/semantic diagnostics and direct member/call facts participate in normal local-check
selection. `wow.api.exists@1` consumes only exact-static facts backed by captured XML source,
exact mappings and complete unit coverage. See [XML_ANALYSIS.md](../apps/wow/XML_ANALYSIS.md).
This advances functional code, not the existing E2/R0 acceptance gates. Effective callback
receiver/inheritance/dispatch semantics, richer value flow, XSD validation and runtime remain
open. XML parent/inheritance references still retain explicit conflicts, order and cycles; see
[XML_REFERENCES.md](../apps/wow/XML_REFERENCES.md).

## Retained graph application route

The exact-entity, direct-neighbor, bounded subgraph, axis, explanation and path owners have one-shot service/CLI
routing over explicit retained partition artifacts; see
[GRAPH_INPUT.md](../apps/wow/GRAPH_INPUT.md). Coherent ProjectStore acquisition and full package acceptance stay open.

## Source graph artifact path

`wow graph build` produces the first-party file/direct-load partition artifact
accepted by the graph-read CLI, from the existing local project materialization
path. It retains exact source/evidence and consumer node IDs without a second
Emmy session or persistent graph/project publication. The result remains Partial;
other recognizer families, dependency graph coverage and coherent ProjectStore remain
open. See [GRAPH_BUILD.md](../apps/wow/GRAPH_BUILD.md).


The source graph also exports XML declarations, source ownership, admitted local
inheritance and uniquely resolved Main mixin declaration links. The original XML
binding report and skipped outcomes remain in provenance. Inheritance-axis queries
are available for this source slice; runtime mixin behavior is not certified.

The callable pipeline exports Main chunk/closure nodes plus the existing direct-call
recognizer's independently owned `Calls` partition. It reuses the semantic session
and retains unresolved/Library/self-recursion outcomes, exact evidence and final
function/edge maps. Call-axis and multi-hop reads operate on the exported snapshot;
this does not close dynamic dispatch, non-call families, persistence or E2 acceptance.

The XML handler pipeline adds source inline-handler nodes and independent
`SetsScript` recognizer proposals. Exact Emmy signatures connect named handlers to
the existing function/call graph; method and inherited associations retain Possible
confidence. Direct/inherited site receipts preserve skipped queries and unresolved
dispatch ordering. Existing inline syntax units are reused, not semantically
recompiled. The v5 export rebinds all node/edge maps after both recognizer partitions.
Other recognizer families, virtual-unit semantics and full E2 acceptance remain open.

## Source read-back checkpoint

The graph bundle explanation has an explicit local source-root route: admitted
manifest/catalog binding, whole-file read-back, exact UTF-8 excerpts, bounded
work/output and drift/unsafe-source outcomes. It reuses existing graph and disk
owners without source execution or reindexing; metadata-only modes are unchanged.
See [GRAPH_EVIDENCE.md](../apps/wow/GRAPH_EVIDENCE.md). This advances the source
read seam, not full E2/E3 acceptance, authentication or persistent publication.

## Retained manifested publication checkpoint

Explicit graph publish/reconcile and current/exact stored reads use immutable
partition versions, complete generation membership, read-back owner validation
and current-record CAS. See [GRAPH_STORE.md](../apps/wow/GRAPH_STORE.md). Retained
graph/project evidence is not a rehydrated live ProjectView. No acceptance,
crash/power-loss, GC, backup/restore, runtime or release gate is closed by this code.

## W12 retained assertion records checkpoint

Exact derivation and reported conflict records are now validated at graph
publication. Explanations traverse actual supporting/rebutting assertions, retain
both conflict participants and report missing or truncated chains explicitly.
Source graph and state producers preserve exact prerequisites through captured
files; other producer chains and complete automatic conflict assessment remain
open. Legacy record-free identities and stored v1 graphs remain readable;
record-bearing batches/snapshots and explanation payloads use explicit v2 schemas.

Fresh workspace checks on 2026-10-09 passed: native policy, fmt, check, strict
Clippy, tests (842 passed, 1 ignored, 103 targets), rustdoc and build. This is a
functional W12 checkpoint, not full E2-A/E2-D acceptance or a live ProjectView.

## W13 physical-input live pair checkpoint

An owned archive now reconstructs a real physical-input ProjectView through the
existing native publisher. Project/graph source coherence and complete logical
membership are validated before store activation. Service acquisition pins one
Current/Exact transaction and generation lease through native replay and owner
checks; the actual immutable pair survives later current advancement.

Fresh workspace policy, fmt, check, strict Clippy, tests (844 passed, 1 ignored,
103 targets), rustdoc and build passed on 2026-10-09. TOC/XML/package replay and
public CLI wiring remain open, as do full W13/E2, crash, incremental, retention,
backup and runtime acceptance. See
[REPLAY_PUBLICATION.md](../crates/wow-project/REPLAY_PUBLICATION.md).

The later physical-input CLI/service checkpoint passes the complete producer
chain, publication/read-back, public read/reconciliation projections and stale
CAS rejection. Canonical pack repairs version bridge4/hooks5/library3. W14 caller
cancellation and typed Library intent are executable; current E0 policy rejects
Clear/empty replacement. Full Library generation binding and incremental reuse
are not implemented. Fresh workspace gates passed on 2026-10-09 (847 tests,
1 ignored, 103 targets). See [LIVE_PROJECT.md](../apps/wow/LIVE_PROJECT.md).

Standalone TOC/XML replay now reuses the same loader over captured documents and
selection context. Exact plan and project/analyzer IDs reproduce after source
removal; full service composition and stored Exact reopen pass. Physical v1 bytes
and legacy epoch catalogs remain compatible; loader replay uses explicit v2 and a
new store catalog. Workspace policy, fmt, check, strict Clippy, tests (849 passed,
1 ignored, 103 targets), rustdoc and build passed on 2026-10-09. Multi-package replay,
generation Library binding and full E2 acceptance remain open.

Declared-package replay now captures the complete selected corpus and all variant
TOC bytes, including unreachable Lua outside analyzer Main. Native/storage v3
rebuilds the real package closure and namespaced Main with exact original IDs.
V1/V2 epochs retain their exact catalogs. The service source-edge crosswalk now
rebinds endpoints to the final generation before returning edge IDs. Full service
publication and Exact reopen after source removal pass, as do corpus/variant/root
substitution guards. Workspace policy, fmt, check, strict Clippy, tests (850 passed,
1 ignored, 103 targets), rustdoc and build passed on 2026-10-09. Full W13/E2,
generation Library binding, incremental, retention/recovery and source/runtime
acceptance remain open.

Generation recipe v2 now binds exact sorted Library snapshot IDs and the owned
function-call-facts profile. Replacement changes the generation while retaining
Main bytes; source handles rebind correctly. New native replay v4 covers all three
input profiles. Frozen old v1/v2/v3 archives reproduce original IDs and reopen
under exact V1/V2/V3 catalogs; v4 writes preserve current and epoch on rejection.
Workspace policy, fmt, check, strict Clippy, tests (852 passed, 1 ignored,
103 targets), rustdoc and build passed on 2026-10-09. Analyzer batches, reuse,
durable updates/removal closure and full W14/E2 acceptance remain open.

Physical W14 updates now apply exact Main deltas to retained native syntax and
semantic engines. Unchanged green allocations survive; removals drop URI mappings
and exact symbol declarations. Complete semantic reindexing and fresh target
report extraction match independent cold final-state builds, including optional
function facts and same-session remove/readd. Library/configuration replacement
and standalone/package/virtual inputs use cold owners. Failure poisons/discards
the mutable cache, preserving current; NoChange keeps its exact Arc. Frozen
legacy replay IDs still match. Workspace policy, fmt, check, strict Clippy,
tests (859 passed, 1 ignored, 104 targets), rustdoc and build passed on 2026-10-09;
the strengthened declaration-removal regression also passed separately.
Dependency-specific fact reuse, durable updates/removal closure and full W14/E2
acceptance remain open.

Durable W14 physical updates now retain the native owner from exact original
publication acquisition and rebuild every graph producer before validated
current CAS. Independent cold full-graph equality verifies Add/Update/Remove;
older leased readers and Exact/historical pairs remain intact. NoChange validates
the requested project/generation without consuming an operation ID. Retried
operations keep their original base and receipt after later activation, while
target substitution conflicts. The public CLI requires explicit Library intent.
Source-function-calls v2 repairs the typed source-support/observation handoff.
Workspace policy, fmt, check, strict Clippy, tests (860 passed, 1 ignored,
104 targets), rustdoc and build passed on 2026-10-09; an additional focused Library
intent test and CLI guards passed separately. Dependency-specific fact reuse,
loader/package durable updates, retention/recovery and full W14/E2 acceptance
remain open. Publication does not close the Gethe/Ketho or named-client gates.

W15 now has persistent exact generation holds under a separately selected physical
v2 profile. SQL roots retain canonical identity, attributable holder, finite kind
and exact epoch/generation. Put is idempotent only for the complete original root;
removal checks its digest. Real store and native service reopen regressions pass;
v1 physical epochs and frozen owner catalogs remain readable without migration.
Unresolved transactions cannot prove durable commit success. No generation or
partition deletion is implemented here. Final workspace policy, fmt, check, strict
Clippy, tests (864 passed, 1 ignored, 105 targets), rustdoc and build passed on
2026-10-09. Operation release/idempotency tombstones,
complete root/shared-data closure, stale-plan rejection, GC/recovery and full W15
acceptance remain open; see
[PROJECT_RETENTION.md](../crates/wow-store/PROJECT_RETENTION.md).

The subsequent W15 v3 slice implements explicit operation release, persisted
policy CAS, bounded complete inline root inventory, stale-plan rejection and
transactional generation/partition collection with durable receipts. Released IDs
stay reserved and original activation evidence survives. Store and native service
lifecycles verify actual reclamation plus shared/current/pin/lease preservation and
reopen. Final workspace policy, fmt, check, strict Clippy, tests (866 passed,
1 ignored, 106 targets), rustdoc and build passed on 2026-10-09. Object/epoch collection, recovery and
platform faults remain open, so full W15/E2 remains unaccepted. See
[PROJECT_GC.md](../crates/wow-store/PROJECT_GC.md).

W16 adds executable read-only reconciliation under one admitted SQLite snapshot,
complete manifest/seal descriptors, canonical receipts and explicit invalid/
incomplete/not-applicable coverage. Activation acknowledgment stays unknown.
Verified native SQLite backup includes committed WAL and independently preserves
the exact inline snapshot and body identities. Restoration to a new private path
requires actual owner capabilities for every retained generation before registry
publication; native service replay checks all Project/Graph pairs and preserves
source current and older readers. See
[PROJECT_RECOVERY.md](../crates/wow-store/PROJECT_RECOVERY.md).

Workspace policy, fmt, check, strict Clippy, tests (877 passed, 1 ignored,
106 targets), rustdoc and build passed on 2026-10-09.
Quarantine, same-root epoch selection, supported migrations, process termination,
power loss and sharing/cleanup faults remain open. Full W16/E2 is unaccepted.

Subsequent selector and quarantine checkpoints preserve exact semantic IDs while
selecting confined physical instances and explicit schema-3 holds. Quarantine
binds the preceding normal selector, raw Current and immutable recovery evidence;
normal new reads/publication/activation/GC/backup refuse, existing leased pairs
remain coherent. A separate readonly inspection can hold damaged SQL without a
normal writable open. Exact selected receipts reconcile without redispatch.
Seven store regressions and one native service lifecycle pass, including actual
Windows sharing refusal and selected-instance scope with absent Current. Final
workspace policy, fmt, check, strict Clippy, tests (895 passed, 1 ignored,
107 targets), rustdoc and build pass. Guarded restore, fine-grained/domain
quarantine, supported migrations, interruption inside writes/OS calls, power
loss, cleanup and Gethe/Ketho/runtime remain separate open gates.

The subsequent guarded-restore and inactive-migration slices retain portable
source hold authority and independently replay native Project/Graph owners.
Supported physical v1/v2 to v3 migration preserves the original catalog/payload
identities and source history in an independent archive, records all generation
aliases, and stops at validated inactive state without target Current or registry.
Partial-state and final-record preflight precede resume effects; both epoch
manifests are rechecked before validation writes. Workspace policy, fmt, check,
strict Clippy, tests (906 passed, 1 ignored, 107 targets), strict rustdoc and build
pass on 2026-10-09. Cross-epoch selection/mapped roots,
payload/old-runtime transformation, pre-intent staging recovery and full W16/E2 or
platform/source/runtime acceptance stay open. See
[PROJECT_MIGRATION.md](../crates/wow-store/PROJECT_MIGRATION.md).

Typed Current-domain recovery is now executable beside the unchanged physical
report. Native owners replay its exact retained publication under a held read and
return actual pair IDs or phased Failed/Incomplete/Cancelled outcomes. Corrupt or
Unverified physical Current never becomes Absent. Explicit CLI `--domain-current`
dispatches one service operation and retains completed physical evidence on replay
cancellation. It grants no repair/activation authority or all-generation verdict.
Three native cases and workspace policy, fmt, check, strict Clippy, tests
(909 passed, 1 ignored, 107 targets), strict rustdoc and build pass on 2026-10-09.
CLI help/strict-option/missing-root checks pass; positive published-root CLI smoke,
injected availability/late cancellation and full W16/E2/source/runtime acceptance
remain separate open gates. See
[PROJECT_CURRENT_RECOVERY.md](../crates/wow-service/PROJECT_CURRENT_RECOVERY.md).

Guarded inactive staging now binds the complete live source snapshot, including pins
and freshly admitted holds, alongside exact selector/Current/epoch guards. Completed
migrations can export an independently verified target-only backup without changing
the baseline or its source archive. Native service replay validates every exported
pair. Six store migration cases and an extended native service lifecycle pass;
workspace policy, fmt, check, strict Clippy, tests (911 passed, 1 ignored,
107 targets), strict rustdoc and build pass on 2026-10-09. Automatic mapped roots and
activation-ready preparation, exact cross-epoch selection/retry and portable entire
migration history remain open; full W16/E2 acceptance is not advanced. See
[PROJECT_MIGRATION.md](../crates/wow-store/PROJECT_MIGRATION.md).

Private immutable migration preparation now reconstructs the complete source pin
set and target-local Current in a separate copy. A durable exact intent precedes
handoff; closed inventory admits only planned root subsets and one exact activation.
Fresh owners validate every retained generation. Completed output/record admission
precedes resumed effects, and no partial native copy is recopied or overwritten.
Three native store cases and the extended actual Project/Graph service lifecycle
pass, preserving baseline/source evidence, live Current and old readers. Workspace
policy, fmt, check, strict Clippy, tests (914 passed, 1 ignored, 107 targets), strict
rustdoc and build pass on 2026-10-09. Live cross-epoch selection, portable entire
migration history, interrupted-copy/power-loss and full W16/E2/source/runtime gates
remain open. See [PROJECT_MIGRATION_READY.md](../crates/wow-store/PROJECT_MIGRATION_READY.md).

Guarded READY-target export now carries independently admitted original-epoch
selector/hold evidence through backup, isolated restore, replacement and quarantine.
Source/live and immutable migration/READY evidence stays unchanged. Complete flat
dependency admission, canonical digest/length checks, inherited reference subsets
and combined aggregate limits precede staging effects. Source holds remain separate
from target holds. Three native store cases and the extended native Project/Graph
service lifecycle pass; workspace policy, fmt, check, strict Clippy, tests
(917 passed, 1 ignored, 107 targets), strict rustdoc and build pass on 2026-10-09.
See [PROJECT_SOURCE_AUTHORITY.md](PROJECT_SOURCE_AUTHORITY.md).

Live cross-epoch selection/reconciliation, original source SQL/history hydration,
portable full migration history, payload/runtime transformation, domain quarantine,
arbitrary interrupted-copy/inside-write/power-loss/deletion/platform and full
W16/E2/source/runtime acceptance remain open. W17-W26 are unchanged.

Guarded cross-epoch selection now installs the independently admitted READY target
under the original selector, optional Current and full source snapshot guards.
A durable ledger, complete native copy and exact Working handoff precede compiled
owner replay of every target generation and one outer schema-6 selector rename.
Exact staged/selected reconciliation preserves old readers, staged target leases,
writer locks and shared reader admissions. Source holds remain original-epoch
archives; lost-response acknowledgment remains Unknown. Historical receipts survive
later legitimate publication and reject quarantined roots.

Two native store cases and the extended native service lifecycle pass. Workspace
policy, fmt, check, strict Clippy, tests (919 passed, 1 ignored, 107 targets), strict
rustdoc and build pass on 2026-10-09. See
[PROJECT_MIGRATION_SELECTION.md](PROJECT_MIGRATION_SELECTION.md).
Portable entire migration history, arbitrary partial-copy/inside-write/power-loss/
deletion/platform, payload/runtime transformation, domain quarantine and full
W16/E2/source/runtime acceptance remain open. W17 source profile/inventory admission
is the next functional owner; W18-W26 remain open.
