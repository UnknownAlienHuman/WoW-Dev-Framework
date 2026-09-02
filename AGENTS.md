# AGENTS.md — WoW Dev Framework

These instructions apply to all automated and human contributors in this repository.

## Repository state

- Product: Rust-first WoW code intelligence, diagnostics, graph, search, context, and agent tooling.
- Documentation frontier: E6-B.
- Implementation frontier: not started; the active executable target remains E0.
- First runnable route: E0-A through E0-F in [`docs/LAUNCH_GATES.md`](docs/LAUNCH_GATES.md).
- Release model: exact immutable Reference Packs, project/graph generations, evidence artifacts, and guarded publications.
- License: MIT.

Do not respond to implementation uncertainty by redesigning the platform. Preserve accepted contracts and reduce the next change to one testable owned responsibility.

## Required reading

Before editing, read in this order:

1. [`README.md`](README.md)
2. [`docs/README.md`](docs/README.md)
3. [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
4. [`docs/PROVENANCE_AND_COVERAGE.md`](docs/PROVENANCE_AND_COVERAGE.md)
5. [`docs/DECISIONS.md`](docs/DECISIONS.md)
6. [`docs/ROADMAP.md`](docs/ROADMAP.md)
7. [`docs/LAUNCH_GATES.md`](docs/LAUNCH_GATES.md)
8. [`crates/MANIFEST.json`](crates/MANIFEST.json)
9. the target crate/application router and complete task package

For every World of Warcraft engineering task, also read the current external routes before drawing platform or addon conclusions:

1. [WoW Addon Engineering KB — `AGENTS.md`](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/AGENTS.md)
2. [WoW Addon Engineering KB — `INDEX_MINI.md`](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/INDEX_MINI.md)
3. the current task-specific KB route selected by that index

For work on a concrete addon, first resolve the actual repository under [UnknownAlienHuman repositories](https://github.com/UnknownAlienHuman?tab=repositories), then read its current `AGENTS.md`, `CLAUDE.md`, TOC, bootstrap, and local documentation before applying framework-wide guidance.

The knowledge base owns living WoW API, patch, security, field-note, and upstream-bug guidance. Link to it; do not copy changing content into this repository. Promote a conclusion here only as a stable contract, ADR, schema, fixture, test, or pinned release input.

## Authority order

1. Accepted contracts and ADRs in this repository.
2. The exact Reference Pack manifest and pinned Blizzard source snapshot selected by the task.
3. Generated Blizzard API documentation and Blizzard UI implementation/XML/TOC from that snapshot.
4. Project-owned fixtures and tests.
5. Runtime probe evidence tied to an exact client build and scenario.
6. The external WoW engineering knowledge base.
7. Selected third-party implementations at pinned commits.
8. General community reports, search/model output, or inference.

A lower source may reveal a gap; it may not silently override a higher source. Record conflicts instead of resolving them by newest, majority, popularity, rank, or guesswork.

## System invariants

Every contribution preserves these rules unless a later accepted ADR explicitly replaces one:

### Exact identity and evidence

- No mixed-profile or mixed-generation result.
- Resolve a permitted symbolic `current` selector once, record the exact identity, retain it, and never refresh it within the operation.
- Every fact/finding/relationship carries producer, exact profile/generation, provenance, confidence, coverage, conflicts, and required nonclaims.
- `Candidate`, `Possible`, partial, conflict, truncated, cancelled, failed, `OutcomeUnknown`, and `NotEvaluated` never become proof, clean Negative, complete, or pass.
- No clean negative answer from a stale, partial, conflicted, truncated, unsupported, or failed partition.

### Parsing, source, and project boundaries

- No default EmmyLua fork; pin upstream behind one adapter.
- No second correctness-path Lua parser.
- No arbitrary Lua, repository hook, installer, build script, or repository-local tool execution during ingestion/indexing.
- TOC/XML/source/archive/database inputs are bounded and nonexecuting.
- No hidden editor/client/project/profile discovery or editor-setting mutation.
- No full Blizzard UI tree in the normal Emmy library workspace or default agent context.

### Graph, recognizers, search, and context

- Ownership/load/object/inheritance/registration/lifecycle/state/call/lineage axes remain distinct; there is no universal parent.
- Production recognizers are universal structural rules and never branch on repository, addon, owner, path, popularity, label, split, reviewer, holdout, canary, or model identity.
- Exact identity, explicit aliases/transitions/replacements, and structured evidence rank before fuzzy/text/semantic similarity.
- Similarity, score, repetition, top rank, or sole result creates only a Candidate.
- Context accepts exact roots and owner views; rendering and source/provider text never become semantic truth or agent instruction.

### E5 governance

- Calibration metrics, graph validity, review authorization, holdout authorization, `PromotionSubmission`, signature, publication, canary, rollout, activation, distribution, and runtime correctness are independent gates.
- A candidate artifact is never relabeled as a core artifact.
- A signature proves exact bytes/key/profile binding only.
- Publication produces `PublishedInactive`; activation is a separate exact-profile CAS.
- Last-known-good is explicitly designated, not inferred as previous/newest.
- Rollback/revocation/deactivation create new immutable records and never rewrite historical project/graph generations.

### E6 external providers

- `wow-cbm` remains optional and depends only on `wow-core`.
- Every external result remains `provenance=semantic_candidate`, `confidence=Candidate`, `negative_authority=unavailable`.
- Provider score/rank/label/top/sole/repetition/stable-state/zero-result never upgrades authority.
- Provider scores remain provider-local and are not numerically fused across providers.
- Provider locators remain `UnverifiedProviderLocator` until an exact project/reference owner maps them.
- Only project/reference owners map locators; service/application must not open provider paths/URLs or reproduce mapping logic.
- `ExactMapped` proves locator-to-owner-record identity only, not provider summary/trace/relation/lineage/replacement/impact/runtime correctness.
- Candidate selection is caller-supplied and auditable; it is not verification, acceptance, edit authorization, or core promotion.
- Provider metadata remains a separate `ExternalCandidateSidecar` and never enters `ContextSemanticPack` truth.
- Provider failure is lane-local and cannot lower exact local reference/project/graph/search/context/diagnostic capability.
- No hidden fallback to another provider, stale cache, model, web, local search, or broadened query.
- No provider database writes, index lifecycle, generic MCP/tool/RPC, or secret material in E6 public seams.

### Service, effects, and applications

- Owner crates never depend on `wow-service` or applications.
- Applications and future transports depend on `wow-service` only.
- One command/tool/request maps to one service operation unless a higher workflow is itself a documented service operation.
- Register `OperationId + CanonicalRequestDigest` before every externally observable or durable effect.
- Same operation ID/same digest returns or reconciles the same effect; same ID/different digest fails.
- Response loss does not prove no effect. `OutcomeUnknown` is unsafe to retry until exact owner reconciliation.
- No public success before required retention, audit, and reverse-order resource closure.
- Cancellation stops new work but preserves durable evidence and does not start detached cleanup.

### Storage, security, and release

- `wow-store` owns generic persistence only and does not interpret domain semantics.
- No raw SQL, connection, transaction callback, physical database/object key, filesystem root, parser/session/process handle, or unrestricted source body crosses service/application seams.
- No database server, vector database, graph service, or new daemon in the default path without measured unique necessity.
- Sensitive signing/provider/deployment material stays inside narrow adapters and never enters repository, fixtures, CLI, logs, errors, or canonical results.
- No component enters the default path without unique correctness responsibility or measured task benefit.
- No CI/release workflow exists merely by convention; it must execute a real frozen command, have an owner, and correspond to a launch gate.

## Work sequence

1. State the exact task, owned contract, selected profile/generation, and launch gate.
2. Inspect existing files and locate the nearest owner contract/decision.
3. Identify whether the change is normative, operational, research, fixture, or implementation.
4. Make the smallest coherent owned change; do not edit sibling crates for convenience.
5. Freeze or update fixtures before claiming correctness.
6. Run the relevant deterministic checks fresh.
7. Report exact commands/checks and `pass`, `fail`, or `skipped`.
8. Update routers, manifest, dependency graph, roadmap, launch gates, glossary, or ADRs when routing changes.

Missing tooling/evidence is `skipped` or `NotEvaluated`, never `pass`. Never claim in-client validation without an actual client-build/scenario record.

## Implementation discipline

### Crates and dependencies

- Create a crate only when the responsibility is independently testable and reusable.
- Activate only the workspace members required by the current implementation milestone.
- Do not add empty modules, placeholder traits, broad `todo!()` surfaces, fake adapters, or mock success paths merely to compile.
- Do not introduce a generic abstraction until at least two owned call sites require the same semantics.
- A new dependency edge requires concrete crossing data/control, insufficiency of current seams, cycle/security/privacy/license/evidence analysis, boundary tests, dependency-graph update, and an ADR when architecture changes.
- Never move domain semantics into `wow-core` or `wow-store` to avoid a dependency problem.

### Rust

- Prefer narrow owned domain types that enforce real invariants; avoid wrapper types with no invariant.
- Keep service/use-case logic independent of CLI/LSP/MCP/editor transports.
- Keep mutable project state in one owner/actor and publish immutable generations.
- Keep migrations explicit, deterministic, and covered by round-trip/crash/recovery tests.
- Avoid `unsafe` unless a documented invariant, focused tests, and a concrete performance/FFI need exist.

### Diagnostics and restrictions

- Separate root causes from downstream symptoms.
- A rule declares required capabilities and returns `NotEvaluated` when unavailable.
- Autofixes require exact mechanically checkable preconditions; otherwise return a plan or Candidate.
- New rule families begin in shadow/evaluation mode until false-blocking behavior is measured.
- Preserve raw restriction metadata independently of annotation projections.
- Unknown restriction facets make dependent checks `NotEvaluated`.
- Runtime spell secrecy or other data-driven state is never frozen into a permanent source-code whitelist.

## Documentation discipline

- English is the canonical repository language.
- `docs/ARCHITECTURE.md`, `docs/PROVENANCE_AND_COVERAGE.md`, public schemas, package contracts, machine manifests, and accepted ADRs are normative.
- `docs/ROADMAP.md` defines package order; `docs/LAUNCH_GATES.md` defines runnable/alpha/preview/beta/v1 thresholds.
- `docs/IDEAS.md` is explicitly non-normative.
- The v8.0 archive path is a retired-source tombstone; use Git history for archaeology.
- Avoid duplicated truth. Link to the owning document.
- Use terms from `docs/GLOSSARY.md`; do not invent synonyms for established states/contracts.
- Preserve prior approved detail when updating routers; a new package should be additive unless an explicit supersession/migration is recorded.

## External repositories and KB

- Pin repository, commit, path, and license before using code as evidence.
- Community code is implementation evidence, never Blizzard API authority.
- Do not vendor third-party source without an accepted decision and license review.
- Store only manifests, checksums, permitted fixtures, and stable source handles.
- Recheck patch-sensitive conclusions through the current KB, pinned Blizzard source, and required runtime probes before release/profile claims.

## GitHub and automation

### Connector capability verification

Before claiming that GitHub is read-only or that a write cannot be performed:

1. reload the complete GitHub tool catalog without a query filter;
2. call `get_repo` and verify `permissions.push == true` for the target repository;
3. if write capability is still uncertain, call `create_blob` as a harmless probe and do not attach that blob to a tree or commit;
4. use GitHub API write actions even when the local VM has no network route, Git credentials, or authenticated `gh` session.

Never infer connector capabilities from a filtered or partially loaded tool list. Local Git failure is not evidence that connector writes are unavailable.

- Pull requests list affected contracts, exact validation performed, profile/build assumptions, blocked/skipped checks, and unresolved coverage.
- Keep commits coherent: one contract or independently reviewable implementation slice per commit.
- Do not add scheduled jobs, publishing, CodeQL, Dependabot, or release automation during bootstrap without an explicit owner and launch-gate need.

## Completion report

A task is not complete until the report states:

```text
owned crate/package and responsibility
files changed
contract operations/decisions affected
new dependency edges, if any
fixtures/evidence added
checks/commands with pass/fail/skipped
selected profile/build assumptions
NotEvaluated capabilities and known gaps
launch gate advanced or still blocked
follow-up seam request only when outside assigned ownership
```
