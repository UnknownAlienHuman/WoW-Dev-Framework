# Implementation work queue — 2026-09-25

Audit: [PR #68](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/68), [findings](AUDIT_2026-09-25.md). Baseline: `02d893cd3b9c9dd116a3330824b9764c3d5ab919`. Machine index: [WORK_QUEUE.json](WORK_QUEUE.json).

**Published state:** 26 work PRs plus the audit PR. W01 contains a proposed code fix and regressions, **not compiler-verified**. W02–W26 are **specification-only drafts**, not implemented features. All target `main`; none was merged by this audit. Each work branch contains `docs/work-queue/Wxx.md`, and its PR body repeats the actionable scope, exact normative sections, dependencies and acceptance checks.

## Agent entrypoint

Read this index, the selected PR/task and its owner contracts. Do not load every task into one agent context. Continue implementation in the existing task branch, re-reading its remote head and current `main` first. The operator's 2026-09-25 request authorizes this PR queue and supersedes historical main-only/no-task-branch routing for these tasks. Architecture, source authority and security contracts remain binding. Older no-Rust/no-workspace/bootstrap status snapshots are not current implementation instructions; W26 reconciles those legacy registries without weakening their acceptance requirements.

Write missing functional code first; then build and run focused checks; expand package/end-to-end acceptance after the functional path exists. Preserve existing tests. Never merge a task document as feature completion. No force-push, automatic merge, CI disabling, implicit source execution, provider activation or public release is authorized.

## Ready work and critical paths

Independent owner lanes: **W01, W02, W03, W05, W09, W12 and the W26 validator**. W01 needs its stated compiler/tests/review before becoming review-ready. One worker owns a file/semantic seam at a time; in particular coordinate W02/W26 in xtask and service registration changes across functional lanes.

The shortest useful ordinary-addon path is **W03 → W04**, with W01/W02 intake safety before untrusted-input acceptance. Full pack composition is **W03 + W05 → W06 → W07**. Coherent retained project/context is **W03 + W12 → W13 → W18 → W19**, adding **W09/W10/W11/W17** for the full selected load/XML/platform-source profiles. Network acquisition, optional daemon, calibration and release work must not block the explicit-local one-shot path.

Dependencies below are **integration prerequisites**, not reasons to stop independent owner coding or wait for blanket fixture completion. Conditional/profile-specific dependencies are named explicitly.

## Work PRs

| ID / PR | Deliverable | Integrates after |
|---|---|---|
| [W01 / #69](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/69) | Reject duplicate decoded compatibility-report JSON members; code proposed | None |
| [W02 / #70](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/70) | Bound external artifact reads before allocation | None |
| [W03 / #71](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/71) | Bind native Reference/Library artifacts into status/check | None; W01/W02 before untrusted-input acceptance |
| [W04 / #72](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/72) | Capability-gated production WoW rule profiles | W03 |
| [W05 / #73](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/73) | Restore annotation/persistence ownership boundary | None |
| [W06 / #74](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/74) | Full Reference Pack build/validate/rebuild-compare service | W03, W05 |
| [W07 / #75](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/75) | Repair and activate the thin Reference Pack builder | W06 |
| [W08 / #76](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/76) | Managed source clone/fallback and interrupted-update recovery | W02 |
| [W09 / #77](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/77) | TOC/package/variant and selected load closure | None |
| [W10 / #78](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/78) | XML virtual Lua semantic analysis | W03, W09 |
| [W11 / #79](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/79) | Remaining core structural recognizer producers | W09; W10 for virtual-unit cases only |
| [W12 / #80](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/80) | Owned graph conflict/derivation explanations | None |
| [W13 / #81](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/81) | Coherent live ProjectView/GraphView publication and acquisition | W03, W12; W09–W11 for extended producer profiles |
| [W14 / #82](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/82) | Cancellable incremental updates and removal closure | W13 |
| [W15 / #83](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/83) | Generation retention roots, leases and safe GC | W13 |
| [W16 / #84](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/84) | Recovery, verified backup/restore and epoch replacement | W13, W15 |
| [W17 / #85](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/85) | Separate exact Blizzard UI source universe | W09, W11, W13 |
| [W18 / #86](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/86) | Executable Project Map and L0/L1 skeletons | W12, W13; W17 for combined platform-source context |
| [W19 / #87](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/87) | Bounded context packs, rendering and service/CLI routing | W18 |
| [W20 / #88](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/88) | Immutable exact-generation search shards and routing | W13 |
| [W21 / #89](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/89) | Exact lineage review, migration and static impact | W12, W13; W20 optional candidate input |
| [W22 / #90](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/90) | Sessions, read-only MCP, then isolated local daemon | W13, W15, W19, W20 for selected registry; status/check transport can start now |
| [W23 / #91](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/91) | LSP and private exact document overlays | W14, W22; no mandatory daemon |
| [W24 / #92](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/92) | Calibration admission/evaluation and guarded candidate lifecycle | W11, W12, W13 |
| [W25 / #93](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/93) | Selected Windows build/bundle verification and release client | W16, W22; W23 only for LSP-enabled profile |
| [W26 / #94](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/94) | Rust contract/queue validator and truthful acceptance ledger | None for validator; acceptance follows each implemented owner |

## Progress and merge discipline

Use separate states: `specified`, `coding`, `code-written-unverified`, `focused-checks-passed`, `reviewed/merged`, `package-accepted`. State changes require links to exact code and actual command/head/platform outcomes. Record unexecuted checks as `NOT-RUN` or `NotEvaluated`; a published commit, task checklist or green unrelated CI run is not evidence its functional path executed.

Keep each PR draft until real code and required focused checks/review are present. Maintain the task's existing-code reuse, boundary, compatibility, source identity and negative cases. If a task must be split further, open the concrete child PRs and update this index before closing the parent; never hide remaining work in a completed label. Incorporate prerequisite changes without force-pushing or discarding another worker's changes. Refresh this index and machine state from actual PRs at each coherent checkpoint.

## Explicitly unclosed later work

This is an executable development queue, **not a claim that 26 merges automatically finish every E0–E7 gate**. W20 must report enabled versus deferred lanes; W24 is the calibration/lifecycle foundation, not full governed core publication; W25 is the local release-engineering foundation, not signed public distribution or proven clean install/update/rollback. Those remaining gates require further concrete implementation/evidence before acceptance. Optional E6/provider integration stays disabled and does not block the local product. No credentials, human review, runtime safety or platform support may be fabricated to close a gate.

Queue check at creation: 26 unique work IDs, PR numbers and branch names; no unknown prerequisites or cycles, including conditional edges. The 78 distinct normative file/section targets used in the audit/work PRs were checked against the exact audited tree. Rust checks remain NOT-RUN in the audit environment.
