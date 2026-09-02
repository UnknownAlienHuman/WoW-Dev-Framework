# AGENTS.md — WoW Dev Framework

These instructions apply to all automated and human contributors.

## Repository state

```text
product: Rust-first WoW code intelligence, diagnostics, graph, search, context and agent tooling
documentation frontier: E7-A
implementation frontier: not-started
next documentation package: E7-B release/distribution lifecycle
first executable package: E0-A wow-core
first runnable gate: E0-A through E0-F
license: MIT
```

Do not redesign the platform because implementation has not started. Preserve accepted contracts and reduce work to one testable owned responsibility.

## Required reading

Before editing:

1. [`README.md`](README.md)
2. [`docs/README.md`](docs/README.md)
3. [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
4. [`docs/PROVENANCE_AND_COVERAGE.md`](docs/PROVENANCE_AND_COVERAGE.md)
5. [`docs/DECISIONS.md`](docs/DECISIONS.md)
6. [`docs/ROADMAP.md`](docs/ROADMAP.md)
7. [`docs/LAUNCH_GATES.md`](docs/LAUNCH_GATES.md)
8. [`crates/MANIFEST.json`](crates/MANIFEST.json)
9. [`crates/DEPENDENCY_GRAPH.md`](crates/DEPENDENCY_GRAPH.md)
10. [`crates/WORKSTREAMS.md`](crates/WORKSTREAMS.md)
11. target crate/application router and complete package

For every World of Warcraft engineering task, also read:

1. [WoW Addon Engineering KB — `AGENTS.md`](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/AGENTS.md)
2. [WoW Addon Engineering KB — `INDEX_MINI.md`](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/INDEX_MINI.md)
3. the task-specific current route selected there

For a concrete addon, first open the actual repository under [UnknownAlienHuman](https://github.com/UnknownAlienHuman?tab=repositories), then read its current `AGENTS.md`, `CLAUDE.md`, TOC, bootstrap and local documentation.

The KB owns living patch/API/security/runtime/field/upstream-bug guidance. Link it; do not copy changing facts into stable framework contracts. Patch-sensitive claims require the exact target source/build and any required runtime probe.

## Authority order

1. accepted framework contracts and ADRs;
2. exact selected Reference Pack manifest and pinned Blizzard source;
3. generated Blizzard API documentation and Blizzard UI implementation/XML/TOC;
4. project-owned fixtures/tests;
5. exact runtime probe evidence;
6. current external engineering KB;
7. pinned third-party implementations;
8. community, search, model or inference candidates.

A lower source may expose a gap but cannot silently override a higher source. Preserve conflict instead of choosing by newest, majority, popularity, score or convenience.

## Core invariants

### Exact identity and evidence

- No mixed profile/generation result.
- Resolve a permitted symbolic current selector once, record the exact identity, retain it and never refresh it within the operation.
- Every fact/finding/relation/result carries exact producer/profile/generation, provenance, confidence, coverage, conflicts, omissions and required nonclaims.
- `Candidate`, `Possible`, partial, conflict, truncated, cancelled, failed, `OutcomeUnknown`, `ResynchronizationRequired` and `NotEvaluated` never become proof, clean Negative, complete or pass.
- No clean negative from stale, partial, conflicted, truncated, unsupported or failed coverage.

### Source, project and analyzer

- Pin upstream EmmyLua behind one adapter; no default fork and no second correctness-path Lua parser.
- Do not execute arbitrary Lua, XML scripts, repository hooks, installers, build scripts, tasks, package managers, generated code or repository tools during ingestion/indexing.
- TOC/XML/source/archive/database/protocol input is bounded and nonexecuting.
- No hidden cwd/Git/editor/WoW/addon discovery or editor-setting mutation.
- No full Blizzard UI source in the normal analyzer library workspace or default agent context.

### Graph, recognizers, search and context

- Lexical, ownership, load, object, inheritance, registration, lifecycle, state, call and lineage axes remain distinct; there is no universal parent.
- Production recognizers are universal structural rules and never branch on repository/addon/owner/path/popularity/label/split/reviewer/holdout/canary/model identity.
- Exact identity and explicit structured evidence rank before fuzzy/text/semantic candidates.
- Similarity, score, repetition, top rank or sole result remains Candidate evidence.
- Context accepts exact roots and exact owner views. Rendering and source/provider/model text never become semantic truth or agent instructions.

### E5 governance

- Calibration metrics, graph validity, review authorization, holdout authorization, `PromotionSubmission`, signature, publication, canary, rollout, activation, distribution and runtime correctness remain independent.
- A candidate is never relabeled as a core artifact.
- Signature proves bytes/key/profile binding only.
- Publication produces `PublishedInactive`; activation is a separate exact-profile CAS.
- LKG is explicitly designated, not inferred as previous/newest.
- Rollback/revocation/deactivation create new immutable records and never rewrite historical project/graph generations.

### E6 external providers

- `wow-cbm` is optional and depends only on `wow-core`.
- Every result remains `semantic_candidate + Candidate` with no negative authority.
- Provider label/rank/score/top/sole/repetition/stable-state/zero-result never upgrades authority.
- Provider scores remain provider-local.
- Locators remain `UnverifiedProviderLocator` until exact project/reference owner mapping.
- `ExactMapped` proves locator identity only; selection is explicit caller intent, not verification, acceptance, edit authorization or core promotion.
- Provider metadata remains an `ExternalCandidateSidecar`, never `ContextSemanticPack` truth.
- Provider failure is lane-local. No hidden fallback to another provider, stale cache, model, web, local search or broader query.
- No provider database/index lifecycle, generic MCP/tool/RPC or secret material in E6 public seams.

### E7-A frontends

- CLI/daemon/LSP/MCP depend on `wow-service` only.
- One semantic command/method/tool call maps to one service operation unless the workflow itself is a documented service operation.
- Exposed operations come only from an exact immutable reviewed registry; no reflection or generic `call_service`/tool/RPC proxy.
- Advertise only implemented capabilities for the exact session/workspace/profile.
- Workspaces are explicit. Unsaved documents are exact immutable project-owned overlays with strict versions and content digests.
- LSP positions bind the exact overlay and negotiated encoding. Stale/out-of-order changes require resynchronization.
- Initial LSP profile is 3.18 over stdio. Initial MCP profile is revision 2025-11-25 over stdio with fixed read-only tools/resources.
- MCP prompts, sampling, elicitation, tasks, arbitrary roots, generic tools and effecting default tools are absent.
- Local daemon uses current-user named pipe/Unix socket. Optional MCP HTTP is loopback-only, Origin-validated and disabled by default.
- Client/editor/model/OS/Git identity is not semantic or effect authorization.
- Disconnect is not cancellation. Progress is not completion. Response replay never reexecutes service.
- Backpressure is bounded and final results/errors/state changes outrank progress/logs.
- Sessions isolate workspaces, overlays, authorization, source, provider access, operations, results and journals.
- No automatic edit application, editor-setting mutation, remote listener or public release action in E7-A.

### Service, effects and storage

- Owner crates never depend on service/applications/transports.
- `wow-service` orchestrates narrow owner ports and never reimplements owner algorithms.
- `wow-store` owns generic persistence only and interprets no domain semantics.
- Register `OperationId + CanonicalRequestDigest` before every durable or external effect.
- Same ID/same digest returns or reconciles the same effect; same ID/different digest fails.
- Response loss never proves no effect. `OutcomeUnknown` blocks blind retry.
- No public success before required retention, audit and reverse-order resource closure.
- Cancellation stops new work but preserves durable evidence and starts no detached cleanup.
- No raw SQL/connection/transaction callback/physical key/filesystem root/parser/session/process handle crosses public seams.

## Work sequence

1. State exact task, owner, contract, selected profile/generation and launch gate.
2. Inspect current owner files and nearest accepted decision.
3. Classify the change: normative, operational, research, fixture or implementation.
4. Make the smallest coherent owner change; do not patch sibling semantics for convenience.
5. Freeze/update fixtures before claiming correctness.
6. Run fresh deterministic checks.
7. Report exact checks as pass/fail/skipped and preserve `NotEvaluated` capabilities.
8. Update routers, manifest, dependency graph, roadmap, launch gates, glossary and ADRs when routing changes.

Never claim client/runtime validation that was not actually performed.

## Implementation discipline

- Activate only crates required by the current implementation milestone.
- Do not add empty modules, broad placeholder traits, fake adapters, fake success, or `todo!()` surfaces merely to compile.
- Introduce a shared abstraction only after at least two owned call sites require identical semantics.
- New dependency edges require exact crossing data/control, insufficiency analysis, cycle/security/privacy/license/evidence review, tests and routing updates.
- Do not move domain semantics into `wow-core`/`wow-store` to evade dependency direction.
- Prefer narrow Rust types that enforce real invariants; avoid wrappers without invariants.
- Keep mutable project state in one owner and publish immutable generations.
- Keep migrations explicit and test round-trip/crash/recovery.
- Avoid `unsafe` without a documented invariant, focused tests and concrete FFI/performance need.
- Diagnostics declare capabilities and return `NotEvaluated` when unavailable.
- Autofixes require exact mechanical guards; otherwise return a plan/Candidate/disabled action.
- New rule families begin in shadow/evaluation mode until false-blocking behavior is measured.
- Unknown restriction facets remain raw and block dependent checks.

## Documentation and release discipline

- English is canonical.
- Package contracts, schemas, `docs/ARCHITECTURE.md`, `docs/PROVENANCE_AND_COVERAGE.md`, `crates/MANIFEST.json` and accepted ADRs are normative.
- `docs/ROADMAP.md` defines package order; `docs/LAUNCH_GATES.md` defines executable/alpha/preview/beta/release thresholds.
- Avoid duplicated truth; link the owner.
- Preserve approved detail when adding a package unless explicit supersession/migration is recorded.
- No Cargo/Rust/workflow placeholder during documentation-only packages.
- No CI/release workflow by convention. It must run real frozen commands, have an explicit owner, protect secret material and correspond to a launch/release gate.
- Public release artifacts must exclude agent instructions, architecture work files, TODOs, fixtures and other development-only files unless the release manifest explicitly requires them.

## GitHub connector access

Before claiming GitHub is read-only:

1. reload the full GitHub tool catalog without a query filter;
2. call `get_repo` and verify `permissions.push == true`;
3. if still uncertain, use `create_blob` as a harmless unattached probe;
4. use GitHub API write actions even when local Git/network credentials fail.

Never infer connector capability from a filtered tool list or local VM failure.

## Completion report

```text
owned package/responsibility
files and contracts changed
operations/decisions/dependency edges affected
fixtures/evidence added
checks with pass/fail/skipped
selected profile/build/protocol assumptions
NotEvaluated capabilities and known gaps
launch gate advanced or still blocked
follow-up seam only when outside assigned ownership
```