# Project completion matrix

**Audited:** 2026-09-19 America/New_York. Code inventory: `c10579d359b5f0044fc6fcdfef3b353c5caa65dc`.

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

The root [Cargo.toml](../Cargo.toml) activates **15 members**, not two, four or seven.

| Component | Observed executable slice | Remaining acceptance boundary |
|---|---|---|
| `wow-core` | Typed identities, canonical JSON, evidence, coverage, results and operation primitives | Reconcile the complete E0-A contract/test matrix; compilation is not the acceptance ledger |
| `wow-emmy` | Real pinned analyzer adapter, explicit Main/Library workspaces, syntax/generic diagnostics, direct member calls and scoped local-flow facts | E0-C fixture/pin/checksum closure; additional semantic operations and update probes are not inferred from parser compatibility |
| `wow-project` | Explicit input inventories and bounded disk acquisition, analyzer bindings, immutable generations, guarded updates and publication | E0-D fixture identity closure; full TOC/XML/load indexing, overlays and durable project publication |
| `wow-rules` | `wow.api.exists@1` and `wow.secret.local_operation@1` | E0-E normative fixtures, exact prerequisite identities and complete capability/negative-authority cases |
| `wow-service` | E0 status/check over immutable normalized contexts; separate ReferenceView administration/publication | E0-F end-to-end fixture/CLI closure; full E1 Reference Pack and later public operation families |
| `wow-store` | Typed SQLite objects, catalogs/CAS, operation journal, leases, GC and integrity | Full E1-A migration/crash/backup acceptance; coherent E2-D ProjectStore is not the generic store |
| `wow-reference` | Native source/model/corrections/aliases, compatibility imports, persistent ReferenceView and publication | E0-B/E1-B normative fixture and full Reference Pack/coverage acceptance |
| `wow-annotations` | Native Ketho-derived library projection, alias/type/catalog/inheritance/navigation slices and consumer tests | Full E1-C contract/corpus parity; scoped passing consumers are not universal semantic certification |
| `wow-graph` | Immutable snapshots, proposals/registries, neighbors, producer partitions, bounded paths and scoped persistence | Axes/subgraph/explanation, cross-owner evidence resolution, normative fixtures and coherent E2-D publication |
| `wow-recognizers` | Structured facts, pack parser/compiler, bounded matcher and Emmy direct-call adapter | Full E2-B producer/fixture/calibration prerequisites; E5 governance is separate |
| `wow-render-contract`, `wow-ketho-literals`, `modules/ketho-literals` | Typed literal wire contract, native renderer and Wasm guest | Scoped algorithm implementation, not public application/release acceptance |
| `tools/xtask` | Native policy/source/manifest/library checks; duplicate JSON member rejection added in this checkpoint | Full schema/contract-ID/dependency/fixture/link closure remains incomplete |

`bridges/literal-host` is an intentionally separate Cargo workspace with its own
CI lane. It is not a fifteenth root member.

`apps/wow-reference-builder` is different: Cargo/source/tests exist, but it is
neither a root member nor an independently declared workspace. Its source also
calls absent service symbols and diverges from its documented command contract.
Do not count it as tested or simply activate it to make the table look complete.

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
| E1-D | Partial ReferenceView service; inactive, inconsistent builder source | Repair the frontend/service contract and then include real package tests in CI |
| E2-A–E2-B | Partial executable | Remaining graph queries, normative recognizer coverage and owner seams |
| E2-C–E2-D | Full package implementation not started | TOC/XML/load/incremental project index and coherent ProjectStore protocol; generic snapshots are prerequisites, not substitutes |
| E3-A–E3-C | Not started | Exact Blizzard source universe, context owners and service/CLI after E2 closure |
| E4-A–E4-C | Not started | Search, lineage/migration/static impact and routing after A0 prerequisites |
| E5-A–E5-C | Not started | Calibration, independent review/holdout and governed publication lifecycle |
| E6-A–E6-B | Not started; optional/disabled | External candidates never block the local lane or gain exact/negative authority |
| E7-A–E7-B | Not started | Selected frontend conformance, then build/sign/bundle/install/update/rollback/support |

## Immediate execution order

The operator's 2026-09-23 priority is functional code before expanded tests.
Existing acceptance requirements remain separate and must not be reported passed.

1. **I0-F functional path:** `apps/wow` now routes explicit materialized inputs
   through actual project/analyzer/reference/rule owners. Finish explicit host
   source/Library materialization and supported-profile rule routing next.
2. **E0 acceptance:** retain existing fixtures/checksums; close their remaining
   evidence after functional implementation, not as an endless prerequisite for it.
3. **I1:** close remaining store/reference/annotation acceptance, then repair the
   dormant builder against its E1 contract. Workspace inclusion and isolated parser
   CI membership must be updated together; adding an app introduces a transitive
   `wow-service` dependency into that lane.
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
