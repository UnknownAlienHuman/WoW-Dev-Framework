# WoW Dev Framework

**Rust-first, editor-independent code intelligence and agent tooling for World of Warcraft addon development.**

> **Documentation frontier:** E5-B complete — durable calibration artifact acquisition, idempotent runs, independent reviewer authorization, sealed-holdout access/audit/consumption, immutable promotion submissions, and thin CLI contracts.
>
> **Next documentation package:** E5-C — immutable core-pack publication/catalog/signing, canary, guarded activation, rollout, rollback, revocation, and last-known-good.
>
> **Implementation frontier:** not started. Executable work still begins with E0-A. No `Cargo.toml`, Rust source, or CI workflow exists.
>
> Patch-sensitive WoW guidance remains in the separate [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb).

WoW Dev Framework is designed to provide exact, generation-bound, evidence-preserving technical context over addon repositories, Blizzard API contracts, Blizzard UI source, diagnostics, graphs, search, lineage, migration evidence, static impact, Project Maps, L0/L1 skeletons, and calibration evidence for universal structural recognizers.

It is not a generic RAG product, editor-settings mutator, runtime injection platform, source-edit executor, repository-specific heuristic engine, model-authority layer, or automatic pack-promotion system.

## Contract stack

```text
E0  identity/evidence/results + fixture analyzer/project/rules/service/CLI slice
E1  ReferenceStore/ReferenceView, annotations, Reference Pack build/validation
E2  typed graph, declarative recognizers, TOC/XML/load project indexing, ProjectStore
E3  Blizzard UI source universe, Project Map/L0/L1/context, context service/CLI
E4  exact-generation search, lineage/change/migration/static impact, service/CLI
E5-A calibration corpora, shadow packs, mutations, metrics, candidates/deactivation
E5-B durable runs, independent review, sealed holdout, promotion submissions
E5-C next: immutable core-pack publication/signing/canary/rollout/rollback
E6+ optional external Codebase Memory candidates, later LSP/MCP/release integration
```

Machine state and implementation order are in [`crates/MANIFEST.json`](crates/MANIFEST.json) and [`crates/WORKSTREAMS.md`](crates/WORKSTREAMS.md).

## Non-negotiable invariants

- One result binds one exact coherent profile/generation set; no mixed `current/latest` data.
- User project, Blizzard UI source, Reference Pack, search candidate, calibration corpus, external candidate, runtime, and history remain distinct universes.
- EmmyLua is the sole correctness-path Lua parser/analyzer.
- TOC/XML/reference/repository ingestion is bounded and nonexecuting.
- Graph assertions retain producer, evidence, provenance, confidence, coverage, conflicts, and generation.
- Ownership, load, object, inheritance, registration, lifecycle, state, call, and lineage axes remain distinct.
- A reason path never becomes a direct edge.
- Production recognizers never branch on repository/addon/owner/path/popularity identity.
- A repository commit pin is not corpus admission.
- Labels, splits, expected outputs, reviewer notes, holdout data, and model output never become matcher inputs.
- `Unknown`, `Possible`, `Candidate`, `Partial`, `Conflict`, `Truncated`, `OutcomeUnknown`, cancelled, and `NotEvaluated` never become proof, Negative, or pass.
- Search ranking and calibration metrics retrieve/evaluate candidates; they do not authorize intent, lineage, review, publication, activation, safety, or runtime behavior.
- Reviewer authorization, graph validity, holdout authorization, submission, publication, activation, and runtime correctness are independent.
- Response loss never proves no effect; exact reconciliation is required before retry.
- Consumed or contamination-unknown holdout evidence is never presented as untouched.
- Promotion submissions are not published or active packs.
- Applications depend on `wow-service` only.
- No public success precedes mandatory retention and resource closure.

## E5-B public surface

The documented CLI does not exist yet. E5-B defines:

```text
wow calibration status
wow calibration source validate
wow calibration corpus validate|admit
wow calibration split validate
wow calibration run submit|get|list|cancel|retry
wow calibration case explain
wow calibration candidate build|validate
wow calibration review validate|record
wow calibration holdout request|execute|audit
wow calibration promotion prepare|validate|get
wow calibration deactivation validate
```

There is no E5-B command for core-pack publish, activate, canary, rollout, rollback, or automatic promotion.

## Current calibration evidence state

E5-A records eight exact user-repository revisions as candidate inputs, but currently:

```text
real admitted corpus members: 0
real measured calibration runs: 0
sealed holdout generations executed: 0
promotion submissions produced by implementation: 0
```

Exact tree/source inventories, owner publications, provenance groups, license/privacy decisions, independent labels, split eligibility, implementations, adapters, authorization/vault infrastructure, benchmarks, and checksums remain blocking.

## First implementation target

Rust implementation still starts with the E0 vertical slice:

```text
pinned upstream EmmyLua analysis
+ one frozen ReferenceView fixture
+ one generated annotation fixture
+ one generic Lua diagnostic
+ one WoW API diagnostic
+ one bounded Secret-local diagnostic
-> one deterministic wow check result
```

Later stores, graph, search, context, calibration, authorization, holdout, publication, LSP/MCP, and release code remain blocked until prerequisite implementation/freeze gates pass.

## Documentation routes

- [`AGENTS.md`](AGENTS.md)
- [`docs/README.md`](docs/README.md)
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- [`docs/PROVENANCE_AND_COVERAGE.md`](docs/PROVENANCE_AND_COVERAGE.md)
- [`docs/DECISIONS.md`](docs/DECISIONS.md)
- [`docs/ROADMAP.md`](docs/ROADMAP.md)
- [`crates/README.md`](crates/README.md)
- [`crates/MANIFEST.json`](crates/MANIFEST.json)
- [`crates/DEPENDENCY_GRAPH.md`](crates/DEPENDENCY_GRAPH.md)
- [`crates/WORKSTREAMS.md`](crates/WORKSTREAMS.md)
- [`crates/wow-recognizers/e5/`](crates/wow-recognizers/e5/README.md)
- [`crates/wow-service/e5/`](crates/wow-service/e5/README.md)
- [`apps/wow/e5/`](apps/wow/e5/README.md)

## License

MIT. Third-party and Blizzard-source artifacts retain separate provenance, license, notice, privacy, and redistribution decisions.