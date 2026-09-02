# WoW Dev Framework

**Rust-first, editor-independent code intelligence and agent tooling for World of Warcraft addon development.**

> **Documentation frontier:** E5-C complete — independent promotion-submission revalidation, distinct immutable core packs, detached signing/provenance, inactive publication/read-back, exact canary evidence, finite rollout, guarded activation, explicit last-known-good, rollback/revocation/deactivation, and stale producer-partition closure.
>
> **Next documentation package:** E6-A — optional external semantic-candidate bridge in `wow-cbm`.
>
> **Implementation frontier:** not started. Executable work still begins with E0-A. No `Cargo.toml`, Rust source, or CI workflow exists.
>
> Patch-sensitive WoW guidance remains in the separate [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb).

WoW Dev Framework provides exact, generation-bound, evidence-preserving technical context over addon repositories, Blizzard API contracts and UI source, diagnostics, typed graphs, search, lineage, migration evidence, static impact, Project Maps, L0/L1 skeletons, and universal structural-recognizer calibration/publication.

It is not a generic RAG product, editor-settings mutator, runtime injection platform, source-edit executor, repository-specific heuristic engine, model-authority layer, or automatic unreviewed promotion system.

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
E6 next: optional external semantic candidates
E7 later: LSP/MCP and public release/distribution integration
```

Machine state and implementation order are in [`crates/MANIFEST.json`](crates/MANIFEST.json) and [`crates/WORKSTREAMS.md`](crates/WORKSTREAMS.md).

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

## Current evidence state

Documentation defines artifact shapes and gates only. Current real implementation evidence remains:

```text
real admitted calibration corpus members: 0
real measured calibration runs: 0
sealed holdout executions: 0
implemented PromotionSubmissions: 0
published core packs: 0
canary observations: 0
active execution profiles: 0
```

Missing implementations, adapters, credentials infrastructure, exact corpora, benchmarks, runtime observations, and checksums remain blocked or `NotEvaluated`, never passed.

## First implementation target

Rust implementation still starts with the E0 diagnostic vertical slice, not publication or rollout. Later stores, graph, search, context, calibration, signing, canary, LSP/MCP, and release code remain blocked until prerequisite implementation/freeze gates pass.

## Routes

- [`AGENTS.md`](AGENTS.md)
- [`docs/README.md`](docs/README.md)
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- [`docs/PROVENANCE_AND_COVERAGE.md`](docs/PROVENANCE_AND_COVERAGE.md)
- [`docs/ROADMAP.md`](docs/ROADMAP.md)
- [`crates/README.md`](crates/README.md)
- [`crates/MANIFEST.json`](crates/MANIFEST.json)
- [`crates/DEPENDENCY_GRAPH.md`](crates/DEPENDENCY_GRAPH.md)
- [`crates/WORKSTREAMS.md`](crates/WORKSTREAMS.md)
- [`crates/wow-service/e5c/`](crates/wow-service/e5c/README.md)
- [`apps/wow/e5c/`](apps/wow/e5c/README.md)

## License

MIT. Third-party and Blizzard-source artifacts retain separate provenance, license, notice, privacy, and redistribution decisions.