# Project completion matrix

**Status:** normative state ledger. Updated 2026-09-02.

## Meaning of states

```text
DocumentationComplete
    owner, operations, data, boundaries, tests, implementation order and freeze gate exist.

ImplementationNotStarted
    no executable owner code or passing implementation evidence exists.

Implemented
    exact package implementation, fixtures/checksums and required tests pass.

LaunchGateComplete
    every package/evidence requirement for the named launch scope passes.
```

Documentation completion never implies implementation or release readiness.

## Current repository state

```text
documentation frontier: E7-B / planned architecture complete
implementation frontier: not started
next owned work: I0-A / wow-core E0-A
Cargo workspace: absent
Rust source: absent
Cargo.lock: absent
rust-toolchain: absent
CI/workflows: absent
supported public release: none
```

## Work-package matrix

| Work package | Owner | Documentation | Implementation | First launch gate using it |
|---|---|---|---|---|
| E0-A | `wow-core` | Complete | Not started | R0 |
| E0-B | `wow-reference` fixture slice | Complete | Not started | R0 |
| E0-C | `wow-emmy` | Complete | Not started | R0 |
| E0-D | `wow-project` fixture slice | Complete | Not started | R0 |
| E0-E | `wow-rules` | Complete | Not started | R0 |
| E0-F | `wow-service` + `apps/wow` | Complete | Not started | R0 |
| E1-A | `wow-store` | Complete | Not started | A0 |
| E1-B | `wow-reference` persistent | Complete | Not started | A0 |
| E1-C | `wow-annotations` | Complete | Not started | A0 |
| E1-D | service + `wow-reference-builder` | Complete | Not started | A0 |
| E2-A | `wow-graph` | Complete | Not started | A0 |
| E2-B | `wow-recognizers` | Complete | Not started | A0 |
| E2-C | `wow-project` full index | Complete | Not started | A0 |
| E2-D | `wow-store` ProjectStore | Complete | Not started | A0 |
| E3-A | Blizzard UI source in `wow-project` | Complete | Not started | A0 |
| E3-B | `wow-context` | Complete | Not started | A0 |
| E3-C | context service + CLI | Complete | Not started | A0 |
| E4-A | `wow-search` | Complete | Not started | A1 |
| E4-B | graph lineage/migration/impact | Complete | Not started | A1 |
| E4-C | search/lineage/impact service + CLI | Complete | Not started | A1 |
| E5-A | recognizer calibration owner | Complete | Not started | B0 governed recognizers |
| E5-B | review/holdout/submission service | Complete | Not started | B0 governed recognizers |
| E5-C | core publication/canary/rollout/rollback | Complete | Not started | B0 governed recognizers |
| E6-A | optional `wow-cbm` Candidate bridge | Complete | Not started | B0 optional external lane |
| E6-B | provider session/mapping/selection/context | Complete | Not started | B0 optional external lane |
| E7-A | service sessions/overlays + CLI/daemon/LSP/MCP | Complete | Not started | A1/V1 |
| E7-B | build/sign/bundle/channel/install/update/support | Complete | Not started | V1 |

## Crate/application/tool matrix

| Component | Final responsibility | Documentation | Cargo/Rust | Implementation evidence |
|---|---|---|---|---|
| `wow-core` | exact shared semantic primitives | Complete | Absent | None |
| `wow-store` | generic durable objects/catalogs/effects/leases/GC | Complete | Absent | None |
| `wow-reference` | Reference Pack/View and exact platform mapping | Complete | Absent | None |
| `wow-annotations` | deterministic annotation projections | Complete | Absent | None |
| `wow-emmy` | pinned upstream Lua analyzer adapter | Complete | Absent | None |
| `wow-project` | source/TOC/XML/load/project generations/overlays | Complete | Absent | None |
| `wow-graph` | typed graph, lineage, impact, partitions | Complete | Absent | None |
| `wow-recognizers` | universal recognizers and calibration owner | Complete | Absent | None |
| `wow-rules` | diagnostics/capability gates/remediation tiers | Complete | Absent | None |
| `wow-search` | exact-generation search/ranking/explanations | Complete | Absent | None |
| `wow-context` | Project Map/L0/L1/L2/context | Complete | Absent | None |
| `wow-cbm` | optional external Candidate normalization | Complete | Absent | None |
| `wow-service` | all multi-owner use cases/effects/envelopes | Complete through E7-B | Absent | None |
| `apps/wow` | public one-shot CLI/daemon/LSP/MCP/update client | Complete through E7-B | Absent | None |
| `apps/wow-reference-builder` | Reference Pack build/validation client | Complete | Absent | None |
| `tools/wow-release` | internal release publisher client | Complete | Absent | None |

## Launch-gate matrix

### R0 — first runnable executable

Required:

```text
E0-A through E0-F implemented
Cargo workspace/toolchain/lockfile frozen
wow status and wow check work against frozen fixtures
canonical output/exit/cancellation/broken-pipe/resource tests pass
```

Current: **Blocked — all implementation absent.**

### A0 — useful internal alpha

Required:

```text
R0
E1 Reference Pack stack
E2 project/graph/recognizers/store
E3 Blizzard UI source and context
one exact real addon/WoW profile evaluation
```

Current: **Blocked by R0 and all E1–E3 implementation.**

### A1 — developer preview

Required:

```text
A0
E4 search/lineage/impact
implemented selected E7-A CLI + at least one LSP or MCP frontend
real task usefulness and platform/resource evidence
```

Current: **Blocked by A0, E4 and E7-A implementation.**

### B0 — governed beta/full planned intelligence

Required for governed recognizer evolution:

```text
A1
E5-A/B/C real calibration/review/holdout/signing/canary/rollback evidence
```

Optional external lane additionally requires E6-A/B and a real provider adapter/benefit evaluation. E6 may remain disabled without blocking the local product.

Current: **Blocked by prior gates and E5/E6 implementation.**

### V1 — public supported release

Required:

```text
selected implemented product scope
E7-A product host/client conformance
E7-B reproducible build/evidence/sign/bundle/support/channel/install/update/rollback
complete Windows x86-64 support matrix and clean-machine rehearsal
signed public artifact and exact read-back/verification
```

Current: **Blocked — no implementation or release evidence.**

## Required implementation evidence families

None currently exists. Every family below remains `NotEvaluated`:

```text
implementation commits and public API closure
Rust/toolchain/dependency/vendor/source pins
fixture/member/bundle SHA-256 closure
unit/property/integration/mutation tests
crash/response-loss/recovery tests
security/fuzz/path/resource tests
Reference Pack and real project generations
Blizzard UI/reference/current profile evidence
graph/search/context correctness and benchmarks
calibration/review/holdout/signing/canary evidence
external provider adapter/mapping/benefit evidence
session/overlay/daemon/LSP/MCP client conformance
independent reproducible Windows builds
SBOM/provenance/license/notices/signatures
installation/update/migration/rollback rehearsal
public channel/read-back/support/incident readiness
```

## Documentation closure requirements

Architecture is considered planned-complete only while these remain true:

- every package has an owner and machine contract;
- every public operation is routed to an owner/service/frontend;
- every effect has idempotency/reconciliation/retention/audit/close behavior;
- exact/Candidate/coverage/authorization/runtime boundaries are explicit;
- session/editor/release/install/update/support surfaces are documented;
- final workspace, conformance commands and implementation order are defined;
- global routers/manifest/dependency graph/roadmap/glossary are consistent;
- patch-sensitive WoW facts remain in the external KB rather than copied here.

A later implementation-discovered gap is handled by the smallest seam/ADR change with tests; it does not reopen unlimited architecture planning.

## Next action

```text
create I0-A implementation branch/worktree
freeze exact Rust toolchain and minimal dependencies
activate only crates/wow-core in Cargo workspace
implement wow-core E0-A contract and fixtures
run its complete acceptance/checksum gate
merge before starting the next owned package
```

Do not create all empty crates, fake operations, placeholder traits or CI workflows first.

## Project-complete definition

For the selected V1 scope, project completion requires:

```text
all required matrix rows Implemented
all selected launch gates Complete
no required blocker/skipped/NotEvaluated state
complete checksum and compatibility manifests
one reproducible signed bundle
one public read-back-verified release
one clean supported Windows install, update and rollback rehearsal
one admitted real addon/client end-to-end evaluation
current support/incident/revocation/retirement policy active
```

Until then, state the exact achieved gate rather than “ready” or a percentage.