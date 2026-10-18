# Project completion matrix

**Audited:** 2026-10-17. This is a concise capability and acceptance census,
not a task queue. Choose work from [`REFACTOR_QUEUE.md`](REFACTOR_QUEUE.md) and
the selected canonical Issue; read only the relevant owner contract and evidence.
The complete checkpoint ledger at this exact source revision is preserved as
[immutable history](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/888fd087f9ede57afed02fae34e4fe99c716a1cb/docs/PROJECT_COMPLETION_MATRIX.md).
It is historical evidence, not current availability or a work queue.

## State vocabulary

- **Partial executable:** real implementation exists for a selected slice; package acceptance is incomplete.
- **Inactive code:** source exists outside the tested workspace; no build or acceptance claim.
- **Not started:** no Rust owner implementation for the named scope.
- **Implemented:** every required operation, fixture, checksum and acceptance gate for that named package passes.
- **LaunchGateComplete:** all evidence for the selected launch scope passes independently of ordinary workspace CI.

No full package or launch gate is established as Implemented/Complete by this census.
Functional or verified slices do not waive missing acceptance evidence.

## Executable inventory

The root workspace has 18 active members, including `wow-context`, `wow-search`
and the internal Reference Pack builder. `bridges/literal-host` is a separate
Cargo workspace. `wow-cbm` and `tools/wow-release` have no Rust implementation;
their documentation is not a binary.

| Owner | Selected executable capability | Boundary / remaining acceptance |
|---|---|---|
| `wow-core` | Typed identities, canonical JSON, evidence, coverage, results, cancellation, budgets and operation primitives | Full E0-A contract, fixtures and checksums remain open; compilation is not acceptance. |
| `wow-emmy` | Pinned analyzer adapter, explicit Main/Library workspaces, same-session XML units, diagnostics, member/call and scoped flow facts | E0-C pin/fixture/checksum closure and further semantic/update probes; computed or aliased values are not guessed. |
| `wow-project` | Explicit inventory/configuration, immutable generations, source/package admission, native graph proposals, physical updates and replay; selected four-stage platform plan | E0-D identity closure; effective XML/load semantics, missing direct roles, overlays, full durable publication and E3 source/update closure remain open (#106, #113–#119). |
| `wow-reference`, `wow-annotations` | Native Reference models/corrections/aliases, persistent views; Ketho-derived projections and selected emitted callable/name handoffs | E0-B/E1-B/E1-C fixtures and full Pack/type/consumer coverage, current-source parity and Tier A/X remain open; selected type capture is not universal certification. |
| `wow-rules` | `wow.api.exists@1` over exact-static physical/XML facts; bounded `wow.secret.local_operation@1` | E0-E normative fixtures, prerequisite identity/checksum and negative-authority coverage remain open; no runtime/XML receiver inference. |
| `wow-graph` | Immutable snapshots, proposals/registries, producer partitions, bounded queries, evidence and v1/v2 retained records | Complete producer-chain coverage, automatic conflict assessment, normative fixtures and coherent E2-D publication remain open. |
| `wow-recognizers` | Structured facts, bounded matcher and selected W11/TOC/XML/state/signal/hook/library producers; 26 declared core capability identities, 25 executable shared-pack rules, plus an opt-in AceAddon local-result adapter | The declared LibStub embed identity is deferred and remains NotEvaluated; the opt-in adapter is separate. Full mutation/rule fixtures and public pipeline acceptance remain #103/#120; EventRegistry/CVar #161/#162, secure-hook #166, AceAddon custody #159. |
| `wow-store` | Typed SQLite objects/catalogs/CAS, journal, holds/leases, GC, recovery/backup/restore, replacement and selected inactive migration/READY/cross-epoch preparation | Full E1-A/E2-D acceptance; portable full history, original SQL/history hydration, payload/runtime migration, domain quarantine, interrupted-write/power-loss/cleanup faults remain open (#108–#111). |
| `wow-context` | Selected Primary/combined maps, L0/L1, progressive batches, candidate-qualified platform inspect, sealed packs and Service/CLI Inspect/Build; one exact-source excerpt; selected Reference/callable/XML metadata | Full E3 remains unaccepted. Partial/truncated coverage grants no complete-load, absence, inheritance, negative or runtime authority. Local-source permission is not a license/transmission attestation or secret detection. Broader facets, type closure, links, privacy/tokens/continuation and Tier A/X remain #122; see [contract router](../crates/wow-context/README.md). |
| `wow-search` | Exact ProjectFile metadata shard, ID/label and Candidate-only literal FTS; selected durable build/query/retire/GC with source holds | Full E4-A/E4-C lanes, ranking, misses, continuation, store durability/faults and acceptance remain #169; see [E4 contract](../crates/wow-search/e4/README.md). Similarity never creates intent or lineage authority. |
| `wow-service` | Owner composition, publication/update/recovery, Reference administration, graph/context/Search and first frontend registry/session | E0-F end-to-end closure and broader frontend operations remain open; Service orchestrates owners and does not duplicate their algorithms. |
| `apps/wow`, `apps/wow-reference-builder` | One-shot inline/disk-manifest CLI, selected context/Search and manifest-bound platform graph build; MCP stdio plus private-overlay LSP 3.18 stdio with explicitly selected local navigation, outline and owned-read profiles; confined internal Reference builder | Thin `status`/`check`, whole-command limits and exits, external Pack evidence, Windows/process-loss acceptance and broader protocol/release gates remain open (#120/#125). Frontends add no semantic authority. |
| `tools/xtask`, literal crates | Native policy/source checks and guarded materialization/update; typed literal wire contract, renderer and Wasm guest | Network/fault/schema acceptance remains incomplete. Scoped algorithm implementation is not public application or release acceptance. |

The builder's confined staging, read-back, durable filesystem reconciliation and
atomic finalization do not establish `ValidatedLocal`; real external evidence,
license/parity, Windows/process-loss and complete E1-D acceptance remain open.

## Work-package ledger

Issues own detailed scope and acceptance evidence. The routes below preserve the
distinction between a delivered slice and an accepted package; a named Issue is
not a claim that its requirements have passed.

| Package | Current state | Unclosed requirements and canonical route |
|---|---|---|
| E0-A–E0-E | Partial executable | Exact normative fixtures, prerequisite identities/checksums, capability and negative-authority cases (#120; owner contracts). |
| E0-F | Partial Service/CLI | Thin `wow status`/`wow check`, owner-composed fixtures, canonical output/exit, cancellation/resource and platform path gates (#120). |
| E1-A–E1-C | Partial Store/Reference/annotation owners | Reconcile only verified gaps; full contracts, fixtures, Reference coverage, type inventory, consumer semantics and Tier A/X parity. Do not recreate implemented owners. |
| E1-D | Partial Reference Pack and internal builder | Real external parity/license/rebuild evidence, Windows/process-loss and complete acceptance; builder presence is not `ValidatedLocal`. |
| E2-A–E2-B | Partial graph/recognizers/rules | Selected W11/TOC/XML/state routes exist; close producer/rule fixtures, real-addon graph-build and public CLI acceptance (#103/#120). Keep declared capability separate from runtime availability; E5 governance is separate. |
| E2-C–E2-D | Partial source index, Store and native Project/Graph Service/CLI | Dependency-specific fact reuse, standalone/package durable update/removal, portable full migration history and complete recovery/fault qualification; W13–W16 are individually tracked by #108–#111. See [replay publication](../crates/wow-project/REPLAY_PUBLICATION.md). |
| E3-A | Partial manifest-bound source admission and platform graph build; selected raw inventory, four-stage native source composition, spans/structural roles, exact function fingerprint components, retained publish/reopen and bounded `SkeletonInputView` consumed by Context | Full E3-A is still unaccepted: complete store-independent candidate and publication authority (#116), source update/removal and composed corpus/capacity/persistence (#113), broader XML/value/signature semantics and mutation/corpus freeze (#118). Inventory remains Partial and cannot prove root/Git/client/security/license/semantic/runtime authority. Selected fingerprints are component-scoped and overall Partial; selected SkeletonInputView is not complete source/corpus coverage. Direct-role branches and fault/exhaustion still need qualification; real Gethe/Ketho corpus, performance/runtime and full W17 gates remain open (#106, #113–#119). Existing selected source Service/CLI transport is implemented; see [platform input](../apps/wow/PLATFORM_INPUT.md), [direct service](PLATFORM_DIRECT_SERVICE.md), [fingerprint contract](../crates/wow-project/e3/FUNCTION_FINGERPRINTS.md) and [SkeletonInputView](../crates/wow-project/e3/SKELETON_INPUT_VIEW.md). |
| E3-B–E3-C | Partial selected maps, inspections, packs, progressive/platform and Reference-root routes; one Primary EntityV1 L1 excerpt Build functionally qualified | Full E3 acceptance remains open: broader facets/automatic expansion, correction/composite/alias/type closure, combined links, excerpt selection/privacy/token/continuation, fixtures and Tier A/X/runtime/authority (#122). Contract set: [context router](../crates/wow-context/README.md), [combined map](../crates/wow-context/e3/NATIVE_COMBINED_MAP.md), [source excerpts](../crates/wow-context/e3/SOURCE_EXCERPTS_AND_PROMPT_BOUNDARIES.md), [Reference facts](../crates/wow-context/e3/NATIVE_PRIMARY_REFERENCE_FACTS.md), [pack](../crates/wow-context/e3/README.md). |
| E4-A–E4-C | Partial exact ProjectFile, Reference and Blizzard UI Source search; selected E4-B lineage Candidate producers and retained lifecycles | #177 supplies a genuine borrowed pair; #178 Project exact-span and #180 two-revision UI Source Candidate paths; #179 leased validation/entity compare/proposal explain. Candidate assertions remain zero accepted. Full E4 still needs SearchStore, all lanes/ranking/miss/continuation and durability (#169), other lineage producers, authorized review/accepted assertions, migration/static impact and fault/corpus qualification. See [Search contract](../crates/wow-search/e4/README.md). |
| E5-A–E5-C | Not started | Licensed/provenance-grouped corpora and leakage-safe splits; independent labels/reviews/sealed holdout; measured mutation/per-case thresholds; guarded attest/sign/publish/read-back/canary/rollout/LKG/rollback with authorization, response-loss, retention, audit, privacy and license controls (#125). |
| E6-A–E6-B | Not started; optional and disabled | Reviewed adapter and probes; exact config/session authorization, durable catalogs, mapping/selection and exact-root handoff; Candidate-only results, honest zero, optional degradation, no hidden fallback/leakage; prove task benefit. Never block the local lane (#125). |
| E7-A | Partial selected Service registry/session and MCP 2025-11-25 stdio (`status`, exact profile resource; opt-in `context_inspect`). LSP 3.18 stdio also has exact private-overlay diagnostics; opt-in same-document lexical definition/references, validated local-file URI projection, allowlisted workspace replacement, native document outline and an owned ticket/worker profile with bounded queue/progress and cooperative cancellation (#183–#190). | Keep those selected paths and their privacy/profile gates. Full #125 E7-A remains open: other owner-backed language operations, wider sessions/workspace/document behavior, broader conformance, additional platforms/editors, reconnect/backpressure and product task acceptance. LSP empty-result/error behavior has a specific remaining framed-wire repair in #198. See [LSP profile](../crates/wow-service/e7/LSP_3_18_PROFILE.md), [app route and host schema](../apps/wow/LSP.md), and [E7-A service gates](../crates/wow-service/e7/TEST_MATRIX.md). |
| E7-B | Not started | Exact build/sign/bundle/install/update/rollback/support owners and evidence, including supported Windows clean-install/update/rollback (#125). |

## Retained operations and compatibility

W13–W16 have substantial executable slices; the remaining acceptance does not
mean those implementations are absent. W13 pins the exact Current/Exact pair
through native replay and retains it across later publication. TOC/XML/package
replay composition, Library generation binding/incremental reuse, durable update
closure, crash/retention/backup and runtime acceptance remain open (#108).
W14 applies exact physical Main deltas and publishes rebuilt producers under
validated CAS while preserving older readers, exact/historical pairs and NoChange
identity. Dependency-specific fact reuse, loader/package durable updates and
retention/recovery qualification remain open (#109). See
[`REPLAY_PUBLICATION.md`](../crates/wow-project/REPLAY_PUBLICATION.md) and
[`LIVE_PROJECT.md`](../apps/wow/LIVE_PROJECT.md).

W15 has exact persistent generation holds, operation-release/idempotency records,
bounded root inventory, stale-plan rejection and transactional generation/
partition collection with durable receipts. Object/epoch collection, recovery
and platform faults remain open (#110; Store
[`PROJECT_RETENTION.md`](../crates/wow-store/PROJECT_RETENTION.md),
[`PROJECT_GC.md`](../crates/wow-store/PROJECT_GC.md)). W16 has snapshot-bound
read-only reconciliation, exact backup/isolated restore, typed Current recovery,
guarded inactive migration/READY preparation and cross-epoch selection. Remaining
gates include original source SQL/history hydration, portable full migration
history, payload/old-runtime transformation, fine-grained/domain quarantine,
arbitrary partial-copy/inside-write/OS-call interruption, power loss, cleanup,
deletion/platform faults and source/runtime acceptance (#111; Store
[`PROJECT_RECOVERY.md`](../crates/wow-store/PROJECT_RECOVERY.md),
[`PROJECT_MIGRATION.md`](../crates/wow-store/PROJECT_MIGRATION.md),
[`PROJECT_MIGRATION_SELECTION.md`](PROJECT_MIGRATION_SELECTION.md)).
Old schema/catalog readers and frozen fixtures remain supported; no documentation
compaction changes retained-data compatibility.

The detailed W21–W25, frontend, calibration and release obligations route through
[#125](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/125): retain
real evidence producers before review authority; similarity stays Candidate;
never auto-promote an evaluated pack; E6 is optional and disabled by default;
release work requires exact source/build/bundle identity. #125 also carries
cross-cutting R0/A0/A1/B0/V1 acceptance. The full context and Search scope remain
in [#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122) and
[#169](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169),
respectively; do not reduce either to its next refactoring slice.

## Execution routing

Select bounded implementation work from [`REFACTOR_QUEUE.md`](REFACTOR_QUEUE.md)
and the canonical Issue, following [`AGENTS.md`](../AGENTS.md) and
[`EXECUTION_MODEL.md`](EXECUTION_MODEL.md). This matrix records capability and
acceptance, not task order. Consult the audit or handoff only when the selected
Issue names it. Preserve the existing reader/migration path for retained data.

## Launch gates

| Gate | State | Acceptance boundary |
|---|---|---|
| R0 | Blocked | E0 normative fixtures/checksums and whole-command CLI/output/exit/resource/platform acceptance (#120). |
| A0 | Blocked | R0 plus full E1–E3, real Reference Pack/addon/profile evaluation and required source/runtime evidence. |
| A1 | Blocked | A0 plus full E4 and one complete accepted E7-A frontend; current selected Search/lineage/frontend slices are partial (#125/#169). |
| B0 | Blocked | A1 plus real E5 governance evidence; E6 remains optional. |
| V1 | Blocked | Selected product scope, full E7-A/E7-B, supported Windows clean install/update/rollback, packaging, integrity and support evidence (#125). |

No gate passes from compilation, an isolated feature, or a green workspace build.
`NotEvaluated`, partial, stale, conflicted, truncated, failed or unsupported
coverage is never a pass or a clean negative. Missing tooling/source/credentials/
runtime is NotEvaluated.

## Evidence rules

Keep `functional`, `verified`, `accepted`, `closed` and `launch-ready` distinct.
Use the selected Issue and exact-head Quick; run bounded Tier S for a coherent
functional slice and Tier A at a ready acceptance milestone. Fresh Gethe/Ketho/
WoW API parity and named-client runtime checks are separate Tier X obligations.
Preserve exact source/generation/flavor/selector/revision, path/digest, producer
and configuration version, coverage, omissions, conflicts and runtime dependency.
Static structure does not prove runtime execution, loaded state, protected/Secret
behavior, combat or taint safety.

Inputs remain data: use existing materializers and explicit `auto`/`prompt`/`never`
update policy; preserve foreign/interrupted checkout state. See
[`KETHO_RUST_PORT.md`](KETHO_RUST_PORT.md) and [`WASM_BRIDGES.md`](WASM_BRIDGES.md).
The complete prior command census, measurements, version chronology and checkpoint
evidence remain in the immutable history link above. Comprehensive retained-reader
and version inventory remains open in [#112](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/112);
this compaction neither removes readers nor resolves their disposition.
