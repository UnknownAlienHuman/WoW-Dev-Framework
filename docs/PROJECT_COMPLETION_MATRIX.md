# Project completion matrix

**Audited:** 2026-09-19 America/New_York. Execution inventory refreshed 2026-10-02
through product checkpoint `260cf430a22fdc3f86da33a9c363946cc31aca00`.

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
| `wow-project` | Explicit inventories, bounded disk/selected TOC-XML acquisition and load receipts, analyzer bindings, XML virtual-semantic reports, immutable generations, exact source artifacts/handles, guarded updates and publication | E0-D fixture identity closure; effective XML receiver/load semantics, full TOC/XML/load acceptance, overlays and durable project publication |
| `wow-rules` | `wow.api.exists@1` over physical Main and exact-static XML inline facts; `wow.secret.local_operation@1` over its bounded physical flow slice | E0-E normative fixtures, exact prerequisite identities and complete capability/negative-authority cases; no inferred XML receiver/runtime authority |
| `wow-service` | E0 status/check over immutable normalized contexts, mixed physical/XML rule scopes and exact XML finding projection; separate ReferenceView administration/publication | E0-F end-to-end fixture/CLI closure; full E1 Reference Pack and later public operation families |
| `wow-store` | Typed SQLite objects, catalogs/CAS, operation journal, leases, GC and integrity | Full E1-A migration/crash/backup acceptance; separate manifested retained ProjectStore exists, not full E2-D publication acceptance |
| `wow-reference` | Native source/model/corrections/aliases, compatibility imports, persistent ReferenceView and publication | E0-B/E1-B normative fixture and full Reference Pack/coverage acceptance |
| `wow-annotations` | Native Ketho-derived library projection, alias/type/catalog/inheritance/navigation slices and consumer tests | Full E1-C contract/corpus parity; scoped passing consumers are not universal semantic certification |
| `wow-graph` | Immutable snapshots, proposals/registries, neighbors, producer partitions, bounded paths/subgraphs, retained-support explanations and scoped persistence | Full conflict/derivation explanations, cross-owner evidence resolution, normative fixtures and coherent E2-D publication |
| `wow-recognizers` | Structured facts, pack parser/compiler, bounded matcher and 23 of 26 active E2-B rule IDs published through service partitions: three `core.lua.*`, five `core.toc.*`, four `core.xml.*`, plus signal, callback, hook and library families | Three state rules, full rule-specific fixtures/checksum freeze, real-addon graph-build acceptance and E5 governance remain separate |
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
| E2-A–E2-B | Partial executable | Twenty-three of 26 active E2-B rule IDs are service-published after the W11 semantic repair, TOC and XML slices; implement the three state rules from typed owner facts, then close full-pipeline fixtures and package acceptance |
| E2-C–E2-D | Partial executable source index and retained manifested store | Live project publication, incremental invalidation, retention/GC/backup/epoch replacement and complete acceptance remain open; see [GRAPH_STORE.md](../apps/wow/GRAPH_STORE.md) |
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
   closure feed W10 XML semantics. Twenty-three W11 rules are now service-published after the
   semantic-repair checkpoint, TOC and XML slices; continue with typed state facts and the
   three rules without inventing receiver, runtime or negative authority.
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
