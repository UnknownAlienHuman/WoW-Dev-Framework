# WoW Dev Framework

**Rust-first, editor-independent code intelligence and agent tooling for World of Warcraft addon development.**

> **Documentation frontier:** E6-B complete — exact provider configuration/session orchestration, durable external Candidate results, project/reference owner mapping, explicit selection, exact-root context handoff, and thin CLI contracts.
>
> **Next documentation package:** E7-A — LSP/MCP/CLI-daemon session surfaces and developer-preview release boundary.
>
> **Implementation frontier:** not started. Executable work still begins with E0-A. No `Cargo.toml`, Rust source, or CI workflow exists.
>
> Patch-sensitive WoW guidance remains in the separate [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb).

WoW Dev Framework provides exact, generation-bound, evidence-preserving technical context over addon repositories, Blizzard API contracts and UI source, diagnostics, typed graphs, search, lineage, migration evidence, static impact, Project Maps, L0/L1 skeletons, universal structural-recognizer calibration/publication, and optional external semantic candidates.

It is not a generic RAG product, editor-settings mutator, runtime injection platform, source-edit executor, repository-specific heuristic engine, model-authority layer, provider database owner, or automatic unreviewed promotion system.

## Contract stack

```text
E0  identity/evidence/results + fixture analyzer/project/rules/service/CLI slice
E1  ReferenceStore/View, annotations, Reference Pack build/validation
E2  typed graph, recognizers, TOC/XML/load indexing, ProjectStore
E3  Blizzard UI source, Project Map/L0/L1/context, context service/CLI
E4  exact-generation search, lineage/migration/static impact, service/CLI
E5-A calibration corpora, shadow packs, mutations, metrics, candidates
E5-B durable runs, independent review, sealed holdout, PromotionSubmission
E5-C immutable core artifact, signing, publication, canary, rollout,
     activation, LKG, rollback, revocation and partition closure
E6-A optional external semantic-candidate owner bridge
E6-B provider/session/result/mapping/selection/context service and CLI
E7-A next: LSP/MCP/CLI-daemon/session transport and developer preview
E7-B later: packaging, public distribution, updates and support lifecycle
```

Machine state and implementation order are in [`crates/MANIFEST.json`](crates/MANIFEST.json), [`crates/WORKSTREAMS.md`](crates/WORKSTREAMS.md), and [`docs/LAUNCH_GATES.md`](docs/LAUNCH_GATES.md).

## Core invariants

- One result/effect binds one exact coherent profile/generation set.
- User project, Blizzard UI source, Reference Pack, search candidate, calibration corpus, external candidate, runtime, and history remain separate universes.
- EmmyLua is the sole correctness-path Lua parser/analyzer.
- TOC/XML/reference/repository ingestion is bounded and nonexecuting.
- Graph assertions retain producer, evidence, confidence, coverage, conflicts, and generation.
- Production recognizers never branch on repository/addon/owner/path/popularity identity.
- A commit pin is not corpus admission.
- Labels, splits, expected outputs, reviewer notes, holdout data, model/search output, canary cohort identity, and rollout state never become matcher semantics.
- `Unknown`, `Possible`, `Candidate`, `Partial`, `Conflict`, `Truncated`, `OutcomeUnknown`, cancelled, and `NotEvaluated` never become proof, Negative, or pass.
- Metrics, graph validity, review authorization, holdout authorization, submission, signature, publication, canary, activation, rollout, distribution, and runtime correctness are independent gates.
- Response loss never proves no effect; exact reconciliation precedes retry.
- Consumed or contamination-unknown holdout evidence is never called untouched.
- A `PromotionSubmission` is not a published pack.
- A signature proves exact bytes/key/profile binding, not semantic or runtime correctness.
- Publication creates `PublishedInactive`; activation is a separate authorized profile-specific CAS.
- Canary evidence is exact and scoped; it is not ecosystem-wide runtime proof.
- Last-known-good is explicitly designated, not inferred as previous/newest.
- Rollback creates new immutable effect/activation/closure records and never rewrites history.
- Stale producer partitions disappear only in new project/graph generations; historical generations remain immutable.
- E5-C internal catalog publication is not public distribution.
- Applications depend on `wow-service` only.
- No public success precedes retention and resource closure.

## External candidate invariants

- `wow-cbm` depends directly only on `wow-core`.
- Provider descriptors are reviewed immutable contracts; runtime negotiation cannot widen them.
- Provider process/session/credential and index lifecycle remain outside E6-A owner semantics.
- E6-B acquires sessions only through exact nonsecret configuration/authorization references and a narrow allow-listed transport.
- There is no generic arbitrary MCP/tool-call surface.
- Every normalized external result is `provenance=semantic_candidate`, `confidence=Candidate`, and `negative_authority=unavailable`.
- Provider `exact`, `verified`, `authoritative`, top-1, sole-result, repeated-result, and high-score labels never upgrade authority.
- Scores/ranks remain provider-local and are never numerically fused across providers.
- Provider paths/URIs/revisions/symbols/spans remain `UnverifiedProviderLocator` until a project/reference owner maps them under one exact retained generation.
- Exact mapping validates locator-to-owner-record identity only; it does not verify provider summaries, traces, relationships, replacements, impact, or runtime behavior.
- Candidate selection is explicit and auditable; it is not verification, acceptance, edit authorization, or core-pack admission.
- Provider metadata remains a separate Candidate sidecar and never enters `ContextSemanticPack` as exact truth.
- A zero result proves only that no accepted candidates were returned for the exact query/state under reported coverage; it is never global absence.
- Continuation/cache bind exact provider descriptor/capability/state/query/profile and cumulative budgets.
- Opaque provider state is explicitly nonreproducible.
- Provider failure is lane-local and cannot degrade exact ReferenceView/project/graph/search/context/diagnostic capabilities.
- No hidden fallback to another provider, stale cache, model, web, local search, or broader query.
- Source snippets/summaries/labels/paths/errors remain untrusted data.

## Documented E5-C CLI

The executable CLI does not exist yet. Contracts define:

```text
wow core-pack status
wow core-pack submission validate
wow core-pack artifact build|validate
wow core-pack sign request|validate
wow core-pack publication publish|get|list|validate
wow core-pack canary plan|start|status|observe|evaluate
wow core-pack rollout plan|advance|pause
wow core-pack activation get|activate
wow core-pack lkg get|designate
wow core-pack rollback
wow core-pack revoke
wow core-pack deactivate
wow core-pack partition-closure validate
```

There is no public release/download/updater command in E5-C.

## E6 contracts

E6-A owns pure external provider candidate semantics:

```text
validate_provider_descriptor
negotiate_provider_capabilities
validate_external_generation
normalize_external_candidate_query
query_external_candidates
continue_external_candidate_query
validate_external_candidate_result_set
explain_external_candidate
build_external_candidate_artifact
compare_external_candidate_results
validate_external_candidate_cache_entry
```

E6-B documents the service/CLI path:

```text
wow external-candidate status
wow external-candidate provider validate
wow external-candidate query submit|get|list|cancel|continue
wow external-candidate operation reconcile
wow external-candidate result validate|explain|compare
wow external-candidate artifact build|get
wow external-candidate mapping resolve|get
wow external-candidate selection record|get
wow external-candidate context build|continue
wow external-candidate cache validate
```

## Current evidence state

Documentation defines artifact shapes and gates only. Current real implementation evidence remains:

```text
Cargo workspace and Rust source: 0
implemented E0-E6 operations: 0
real Reference Packs/project generations: 0
real admitted calibration corpus members: 0
real measured calibration runs: 0
implemented PromotionSubmissions: 0
published core packs/canary observations/active profiles: 0
live external provider adapters/probes: 0
exact external owner mappings/context handoffs: 0
LSP/MCP/daemon transports: 0
public release artifacts: 0
```

Missing implementations, adapters, credentials infrastructure, exact corpora, provider probes, benchmarks, runtime observations, platform tests, and checksums remain blocked or `NotEvaluated`, never passed.

## Launch path

The shortest valid path to a runnable project is still:

```text
E0-A wow-core
-> E0-B ReferenceView fixture + E0-C EmmyLua adapter
-> E0-D project fixture
-> E0-E diagnostics
-> E0-F wow-service + apps/wow
```

E1–E4 create a useful internal/developer alpha. E5 enables governed recognizer promotion. E6 is optional and may remain disabled. E7-A supplies supported frontend/session transports; E7-B supplies public packaging, signing, updates, rollback, and support lifecycle. See [`docs/LAUNCH_GATES.md`](docs/LAUNCH_GATES.md).

## Routes

- [`AGENTS.md`](AGENTS.md)
- [`docs/README.md`](docs/README.md)
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- [`docs/PROVENANCE_AND_COVERAGE.md`](docs/PROVENANCE_AND_COVERAGE.md)
- [`docs/ROADMAP.md`](docs/ROADMAP.md)
- [`docs/LAUNCH_GATES.md`](docs/LAUNCH_GATES.md)
- [`crates/README.md`](crates/README.md)
- [`crates/MANIFEST.json`](crates/MANIFEST.json)
- [`crates/DEPENDENCY_GRAPH.md`](crates/DEPENDENCY_GRAPH.md)
- [`crates/WORKSTREAMS.md`](crates/WORKSTREAMS.md)
- [`crates/wow-service/e5c/`](crates/wow-service/e5c/README.md)
- [`crates/wow-cbm/e6/`](crates/wow-cbm/e6/README.md)
- [`crates/wow-service/e6/`](crates/wow-service/e6/README.md)
- [`apps/wow/e6/`](apps/wow/e6/README.md)

## License

MIT. Third-party, provider-returned, addon, and Blizzard-source artifacts retain separate provenance, license, notice, privacy, and redistribution decisions.