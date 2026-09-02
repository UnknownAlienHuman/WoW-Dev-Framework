# AGENTS.md — WoW Dev Framework

These instructions apply to all automated and human contributors.

## Repository state

```text
product: Rust-first WoW code intelligence, diagnostics, graph, search, context, editor and release tooling
planned architecture/documentation: complete through E7-B
implementation frontier: not-started
next implementation package: I0-A / wow-core E0-A
first runnable gate: R0 / E0-A through E0-F
first supported target intent: x86_64-pc-windows-msvc after complete evidence
license: MIT
```

Do not continue speculative architecture by default. Implement the accepted contracts in dependency order. Change architecture only when a concrete implementation or test failure proves that an existing seam is insufficient.

## Required reading

Before editing:

1. [`README.md`](README.md)
2. [`docs/README.md`](docs/README.md)
3. [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
4. [`docs/PROVENANCE_AND_COVERAGE.md`](docs/PROVENANCE_AND_COVERAGE.md)
5. [`docs/DECISIONS.md`](docs/DECISIONS.md)
6. [`docs/ROADMAP.md`](docs/ROADMAP.md)
7. [`docs/LAUNCH_GATES.md`](docs/LAUNCH_GATES.md)
8. [`docs/WORKSPACE_AND_BUILD_PLAN.md`](docs/WORKSPACE_AND_BUILD_PLAN.md)
9. [`docs/IMPLEMENTATION_HANDOFF.md`](docs/IMPLEMENTATION_HANDOFF.md)
10. [`docs/CONFORMANCE_COMMANDS.md`](docs/CONFORMANCE_COMMANDS.md)
11. [`docs/PROJECT_COMPLETION_MATRIX.md`](docs/PROJECT_COMPLETION_MATRIX.md)
12. [`crates/MANIFEST.json`](crates/MANIFEST.json)
13. [`crates/DEPENDENCY_GRAPH.md`](crates/DEPENDENCY_GRAPH.md)
14. [`crates/WORKSTREAMS.md`](crates/WORKSTREAMS.md)
15. the target crate, application, or tool router and complete owned package

For every World of Warcraft engineering task, also read:

1. [WoW Addon Engineering KB — `AGENTS.md`](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/AGENTS.md)
2. [WoW Addon Engineering KB — `INDEX_MINI.md`](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/INDEX_MINI.md)
3. the exact task-specific current route selected there

For a concrete addon, first open the actual repository under [UnknownAlienHuman](https://github.com/UnknownAlienHuman?tab=repositories), then read its current instructions, TOC, bootstrap, and local documentation.

The KB owns living patch, API, security, runtime, field-note, and upstream-bug guidance. Link it; do not copy changing facts into stable framework contracts. Patch-sensitive claims require the exact target source/build and any required runtime probe.

## Authority order

1. accepted framework contracts and ADRs;
2. exact selected Reference Pack manifest and pinned Blizzard source;
3. generated Blizzard API documentation and Blizzard UI implementation, XML, and TOC;
4. project-owned fixtures, tests, and exact owner artifacts;
5. exact runtime probe evidence;
6. current external engineering KB;
7. pinned third-party implementations;
8. community, search, provider, model, or inference candidates.

A lower source may expose a gap but cannot silently override a higher source. Preserve conflict instead of choosing by newest, majority, popularity, score, or convenience.

## Core invariants

### Exact identity and evidence

- No mixed profile or generation result.
- Resolve a permitted symbolic current selector once, record and retain the exact identity, and never refresh it within the operation.
- Every fact, finding, relation, result, effect, session, release, and installation record carries exact producer/profile/generation identities, provenance, confidence, coverage, conflicts, omissions, and required nonclaims.
- `Candidate`, `Possible`, partial, conflict, truncated, cancelled, failed, `OutcomeUnknown`, `ResynchronizationRequired`, and `NotEvaluated` never become proof, clean Negative, complete, or pass.
- No clean negative from stale, partial, conflicted, truncated, unsupported, or failed coverage.

### Source, project, and analyzer

- Pin upstream EmmyLua behind one adapter; no default fork and no second correctness-path Lua parser.
- Do not execute arbitrary Lua, XML scripts, repository hooks, installers, build scripts, tasks, package managers, generated code, or repository tools during ingestion and indexing.
- TOC, XML, source, archive, database, and protocol input is bounded and nonexecuting.
- No hidden cwd, Git, editor, WoW, addon, project, profile, provider, installation, or current-state discovery.
- User project, Blizzard UI source, Reference Pack, calibration, provider, runtime, history, overlay, release, and installation universes remain separate.

### Graph, recognizers, search, and context

- Lexical, ownership, load, object, inheritance, registration, lifecycle, state, call, and lineage axes remain distinct; there is no universal parent.
- Production recognizers are universal structural rules and never branch on repository, addon, owner, path, popularity, label, split, reviewer, holdout, canary, provider, or model identity.
- Exact identity and explicit structured evidence rank before fuzzy, text, or semantic candidates.
- Similarity, score, repetition, top rank, or sole result remains Candidate evidence.
- Context accepts exact roots and exact owner views. Rendering and source/provider/model text never become semantic truth or agent instructions.
- Autofixes require exact mechanical guards; otherwise return a plan, Candidate, or disabled action.

### E5 governance

- Calibration metrics, graph validity, review authorization, holdout authorization, `PromotionSubmission`, signature, publication, canary, rollout, activation, distribution, and runtime correctness remain independent.
- A candidate is never relabeled as a core artifact.
- Signature proves bytes, key, and profile binding only.
- Publication produces `PublishedInactive`; activation is a separate exact-profile CAS.
- Last-known-good is explicitly designated, not inferred as previous or newest.
- Rollback, revocation, and deactivation create new immutable records and never rewrite historical project or graph generations.

### E6 external providers

- `wow-cbm` is optional and depends only on `wow-core`.
- Every result remains `semantic_candidate + Candidate` with no negative authority.
- Provider label, rank, score, top, sole, repetition, stable-state, or zero-result never upgrades authority.
- Provider scores remain provider-local.
- Locators remain `UnverifiedProviderLocator` until exact project/reference owner mapping.
- `ExactMapped` proves locator identity only; selection is explicit caller intent, not verification, acceptance, edit authorization, lineage, replacement, impact, or core promotion.
- Provider metadata remains an `ExternalCandidateSidecar`, never `ContextSemanticPack` truth.
- Provider failure is lane-local. No hidden fallback to another provider, stale cache, model, web, local search, or broader query.
- No provider database/index lifecycle, generic MCP/tool/RPC, or secret material in E6 public seams.

### E7-A frontends

- The canonical contracts are `wow-service/e7-a/frontend-session-operation-registry` and `apps/wow/e7-a/frontend-transports`.
- CLI, daemon, LSP, and MCP depend on `wow-service` only.
- One semantic command, method, or tool call maps to one service operation unless the composite workflow itself is a documented service operation.
- Exposed operations come only from an exact immutable reviewed registry; no reflection or generic `call_service`, arbitrary tool, RPC, shell, script, plugin, or model proxy.
- Advertise only implemented capabilities for the exact session, workspace, project, and profile.
- Workspaces are explicit. Unsaved documents are exact immutable project-owned overlays with strict versions and content digests.
- LSP 3.18 uses incremental `textDocument/didChange`; a full-document change is an exact replacement. Positions bind the exact overlay and negotiated encoding. Stale or out-of-order changes require resynchronization.
- MCP 2025-11-25 defaults to fixed read-only tools and resources. Prompts, sampling, elicitation, tasks, arbitrary tools, provider effects, governance effects, source mutation, and release effects are absent.
- Local daemon uses current-user named pipe or Unix-domain socket. Optional MCP HTTP is loopback-only, Origin-validated, and disabled by default.
- Client, editor, model, OS, Git, repository, and transport identity is not semantic or effect authorization.
- Disconnect is not cancellation. Progress is not completion. Response replay never reexecutes service.
- Backpressure is bounded, and final results, errors, and state changes outrank progress and logs.
- Sessions isolate workspaces, overlays, authorization, source, provider access, operations, results, streams, and journals.
- No automatic edit application, editor-setting mutation, default remote listener, or public release action in E7-A.

### E7-B release and update lifecycle

- A compiled binary is not a release. Source closure, build, reproducibility, tests/evidence, signatures, bundle, support, channel publication, installation, update, and runtime remain independent gates.
- The first target intent is Windows x86-64 MSVC and remains unsupported until the complete target, client, path, IPC, signing, install, migration, update, rollback, and clean-machine suites pass.
- Source tree, `Cargo.lock`, Rust toolchain, target, features, dependencies, build scripts, environment, deterministic profile, and executor are exact inputs.
- Release builds use a narrow typed executor; no arbitrary command, environment, network callback, SQL, provider API, or installer surface crosses the service seam.
- At least two independent unsigned builds are required for a reproducibility claim.
- Platform signing occurs after unsigned digest freeze and never redefines unsigned build identity.
- SBOM, provenance, licenses/notices, checksums, tests, benchmarks, signatures, bundle, support matrix, and update manifests are independent immutable artifacts.
- Reference Packs, core packs, and provider adapters keep independent identities, signatures, compatibility, and update lifecycles.
- GitHub Releases may be a provider adapter, but repository, tag, CI job, account, successful upload, and asset name are not authorization or trust.
- Channel changes use exact expected-current CAS; no in-place asset replacement and no latest/newest/age/download/no-complaint promotion shortcut.
- Baseline has no hidden startup update check, download, install, telemetry, crash upload, or remote configuration.
- Check, materialize, verify, stage, back up, migrate, activate, self-check, designate LastKnownRunnable, clean up, and roll back are separate exact states.
- The public app never overwrites its running executable or executes arbitrary helper commands. Windows replacement is owned by the exact verified installation/helper plan.
- Store and configuration migrations use registered owner operations, verified backup/restore, crash recovery, and explicit rollback compatibility; never raw SQL or scripts.
- LastKnownRunnable and rollback targets are exact retained qualified records, never inferred from previous, newest, version, or directory position.
- Revocation, retirement, and incident records are distinct and immutable.
- Support claims are limited to exact tested target, OS, protocol, store, schema, data-pack, WoW-profile, client, feature, and resource matrices.

### Service, effects, and storage

- Owner crates never depend on service, applications, transports, or tools.
- `wow-service` orchestrates narrow owner ports and never reimplements owner algorithms.
- `wow-store` owns generic persistence only and interprets no domain semantics.
- Register `OperationId + CanonicalRequestDigest` before every durable or external effect.
- Same ID and same digest returns or reconciles the same effect; same ID and different digest fails.
- Response loss never proves no effect. `OutcomeUnknown` blocks blind retry.
- No public success before required validation/read-back, retention, audit, and reverse-order resource closure.
- Cancellation stops new work but preserves durable evidence and starts no detached cleanup.
- No raw SQL, connection, transaction callback, physical key, filesystem root, parser, session, process, provider, signing, build, distribution, or installer handle crosses public seams.

## Work sequence

1. State exact implementation package, owner, contract, selected profile/generation, launch gate, and branch/worktree.
2. Inspect current owner files, prerequisites, and nearest accepted decisions.
3. Verify the package freeze gate before the first Rust implementation commit.
4. Implement the smallest coherent owner responsibility; do not patch sibling semantics for convenience.
5. Freeze or update fixtures before claiming correctness.
6. Run fresh deterministic conformance checks.
7. Report each check as pass, fail, skipped, or `NotEvaluated` with exact inputs.
8. Update routers, manifest, dependency graph, roadmap, launch gates, completion matrix, glossary, and ADRs only when routing or actual state changes.
9. Merge, quarantine, or delete the worktree before starting another primary task.

Never claim client, runtime, platform, provider, signing, installation, or release validation that was not actually performed.

## Implementation discipline

- Activate only crates required by the current implementation package.
- Do not add empty modules, broad placeholder traits, fake adapters, fake success, or `todo!()` surfaces merely to compile.
- Introduce a shared abstraction only after at least two owned call sites require identical semantics.
- New dependency edges require exact crossing data/control, insufficiency analysis, cycle, security, privacy, license, supply-chain, and evidence review, tests, and routing updates.
- Do not move domain semantics into `wow-core` or `wow-store` to evade dependency direction.
- Prefer narrow Rust types that enforce real invariants; avoid wrappers without invariants.
- Keep mutable owner state inside one owner and publish immutable generations.
- Keep migrations explicit and test forward, rollback, crash, recovery, and read-back.
- Avoid `unsafe` without a documented invariant, focused tests, and a concrete FFI or measured performance need.
- Tests verify committed fixtures; they never rewrite them.

## Documentation and release discipline

- English is canonical.
- Package contracts, schemas, architecture, provenance, machine manifest, implementation handoff, conformance commands, completion matrix, and accepted ADRs are normative.
- Avoid duplicated truth; link the owner.
- Preserve approved detail unless explicit supersession and migration are recorded.
- Documentation-only work never claims executable implementation evidence.
- No CI or release workflow by convention. It must invoke real frozen commands, have an explicit owner, protect secret material, and correspond to an implementation or release gate.
- Public release artifacts exclude agent instructions, architecture work files, TODOs, fixtures, and development-only files unless the exact release manifest explicitly requires them.
- After E7-B, default work is implementation. New architecture requires a concrete implementation-discovered contract failure and the smallest tested seam or ADR change.

## GitHub connector access

Before claiming GitHub is read-only:

1. reload the full GitHub tool catalog without a query filter;
2. call `get_repo` and verify `permissions.push == true`;
3. if still uncertain, use `create_blob` as a harmless unattached probe;
4. use GitHub API write actions even when local Git or network credentials fail.

Never infer connector capability from a filtered tool list or local VM failure.

## Completion report

```text
owned package and responsibility
files, contracts, operations, and dependency edges changed
exact prerequisites, profiles, generations, targets, and fixtures
dependencies, features, and platform behavior introduced
fixtures, checksums, and evidence added
commands with pass, fail, skipped, or NotEvaluated
idempotency, response loss, retention, audit, and close behavior
security, privacy, license, and authorization state
launch/completion gate advanced or unchanged
remaining exact blockers
```
