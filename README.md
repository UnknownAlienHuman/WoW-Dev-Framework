# WoW Dev Framework

**Rust-first, editor-independent code intelligence and agent tooling for World of Warcraft addon development.**

> **Documentation frontier:** E5-A is complete: exact Reference Pack, analyzer/project/graph/persistence, separately published Blizzard UI source, deterministic Project Maps and L0/L1 context, immutable exact-generation search, explicit lineage/migration/static-impact contracts, service/CLI orchestration, and audited calibration-corpus/named-shadow-pack contracts.
>
> **Next documentation package:** E5-B — retained calibration artifact acquisition, durable run orchestration, reviewer authorization, sealed-holdout access audit, promotion submissions, canonical service envelopes, and thin CLI transport.
>
> **Implementation frontier:** not started. The first executable work remains E0-A, not E4 or E5. No `Cargo.toml`, Rust source, or CI workflow exists.
>
> **Compatibility model:** every result binds one exact World of Warcraft profile and exact project/reference/source generations. Live patch guidance remains routed through the separate [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb).

WoW Dev Framework is designed to give coding agents and addon developers a compact, exact, explainable technical surface over addon repositories, Blizzard API contracts, Blizzard UI implementation source, static structure, diagnostics, retrieval candidates, cross-generation evidence, migration artifacts, static impact, bounded source context, and calibration evidence for universal structural recognizers.

It is not a generic RAG product, editor-settings mutator, runtime injection platform, source-edit executor, repository-specific heuristic engine, replacement for Codebase Memory, or model-authority layer.

## What the framework should answer

- Does an API, event, method, template, mixin, package, field, file, or project entity exist in one exact profile?
- What exact project/source/reference evidence supports the answer?
- Which package, TOC, XML document, file, module, registry, state root, or lifecycle surface owns an entity?
- How is it reached through the real TOC/XML/dependency/load graph?
- Which relation is direct, derived, possible, conflicted, omitted, or not evaluated?
- What is the smallest trustworthy Project Map, L0/L1 skeleton, reason path, or source excerpt needed for a task?
- Which exact and approximate search lanes found an entity, and why did it outrank another candidate?
- Is a search miss authoritative, partial, candidate-only, truncated, or nonauthoritative?
- Which entities have accepted lineage across two exact generations, and which pairs remain candidates or conflicts?
- Was an entity moved, renamed, split, merged, introduced, removed, deprecated, or replaced under the exact proof ceiling?
- Which migration recipe is only advisory, which validates statically, and what validation still remains?
- Which entities are statically affected through exact direct edges or reason paths, without claiming runtime breakage?
- Which operation is statically permitted, and which still requires exact runtime evidence?
- Which structural recognizer convention generalizes beyond a donor identity, and which evidence remains donor-local, correlated, conflicted, or not evaluated?
- Did a candidate calibration pack pass repository/owner/addon/path/local-name invariance, decisive structural sensitivity, near-miss, leakage, graph-validation, security, and determinism gates?
- Can a candidate pack be disabled without touching core or foreign producer partitions, and what exact coverage is lost?

## Contract stack

```text
E0
    wow-core identities/evidence/coverage/results
    fixture ReferenceView
    pinned EmmyLua adapter
    minimal project generation
    two bounded WoW rules
    status/check service and CLI

E1
    SQLite/ReferenceStore foundation
    persistent ReferenceView build
    annotation generation and parity
    Reference Pack build/validation

E2
    typed assertion graph
    declarative structural recognizers
    full TOC/XML/load/analyzer project candidate
    WAL ProjectStore coherent publication

E3-A
    exact separately scoped Blizzard UI source universe
    project/analyzer/graph publication
    bounded SkeletonInputView

E3-B
    exact ContextUniverseSet
    Project Map and L0/L1 skeletons
    deterministic expansion, pruning, source boundaries and context packs

E3-C
    exact/current publication selection
    coherent retained views and leases
    context service operations
    thin apps/wow CLI projection

E4-A
    immutable SearchShard per exact owner generation
    exact/alias/member/prefix/text/similarity/shape/graph retrieval
    authority-banded deterministic candidate ranking
    complete explanations, honest misses and stable continuation

E4-B
    explicit cross-generation lineage overlay
    independent project/reference/search/review producer partitions
    proof ceilings, ambiguity components and immutable review promotion
    typed change/absence/replacement/migration/static-impact records

E4-C
    exact/current search and lineage acquisition
    shard build/validation/query orchestration
    explicit search candidate selection receipts
    review authorization plus independent graph validation
    migration validation and bounded static-impact use cases
    exact selected search root to context handoff
    thin service-only CLI

E5-A
    exact candidate-source and corpus admission contracts
    conservative provenance/fork/copy/vendor/generated grouping
    independent labels and atomic train/dev/test/holdout splits
    E2-B-compatible named calibration shadow packs
    anti-overfitting, near-miss, leakage, graph and resource gates
    immutable per-case/metric/candidate/deactivation artifacts

E5-B next
    retained calibration artifact acquisition
    durable runs, idempotency and response-loss recovery
    reviewer authorization and sealed-holdout access audit
    promotion submission preparation
    thin service-only CLI

E5-C+
    immutable core-pack publication, canary, rollout and rollback
    optional Codebase Memory candidates
    LSP/MCP and release operations
```

The machine-readable state and exact implementation order are in [`crates/MANIFEST.json`](crates/MANIFEST.json) and [`crates/WORKSTREAMS.md`](crates/WORKSTREAMS.md).

## Architecture

```text
exact Reference Pack
    raw APIDocumentation and metadata
    normalized API/type/event/restriction facts
    corrections, conflicts and coverage
    deterministic annotation artifacts

exact user addon publication
    source snapshot
    selected TOC flavor
    bounded XML and virtual Lua units
    EmmyLua facts and diagnostics
    recognizer producer partitions
    ProjectSnapshot + GraphSnapshot

exact Blizzard UI source publication
    separate source universe and ProjectStore
    package/load/XML/template/mixin/analyzer/graph records
    implementation evidence, never automatic API/runtime authority

context layer
    separate exact universe binding
    Project Maps
    L0/L1 typed projections
    bounded source excerpts and evidence closure
    semantic context packs and deterministic renderers

search layer
    separate immutable user/Blizzard/reference shards
    typed bounded documents and exact field origins
    safe structured query AST
    exact and candidate retrieval lanes
    deterministic rank tuples and explanations
    exact miss/partial/truncation/continuation state

lineage/change layer
    exact before/after generations in one universe
    independent project/reference/search/review evidence partitions
    bounded ambiguity components and explicit proof ceilings
    immutable accepted/rejected/deferred/conflicted review state
    typed change, absence, replacement, migration and static-impact records

calibration layer
    exact candidate source/publication/fact identities
    immutable provenance groups, labels, splits and mutations
    E2-B declarative shadow matching
    independent graph proposal validation
    per-case-first metrics and hard gates
    candidate-owned partitions and exact deactivation plans

service/application layer
    exact/current publication selection
    retained view/lease lifecycle
    explicit search candidate selection
    review authorization adapter invocation
    context, migration and static-impact use-case sequencing
    later calibration review/holdout/promotion-submission sequencing
    canonical envelopes and thin CLI transport

later publication/integration layer
    immutable core-pack publication/canary/rollback
    optional external semantic candidates
    LSP/MCP and release controls
```

## Non-negotiable invariants

- One result uses one exact coherent generation/profile set; no mixed current/latest data.
- `CurrentPublished` is resolved only at the service boundary and never refreshed mid-request.
- Independent project, Blizzard UI, reference, search, lineage, and calibration stores are not falsely described as one distributed atomic snapshot.
- User project, Blizzard UI implementation, Reference Pack, search candidate, calibration corpus, external candidate, runtime, and historical universes remain distinct.
- EmmyLua is the sole correctness-path Lua parser/analyzer and is pinned behind one adapter.
- TOC/XML parsing and repository materialization are bounded and nonexecuting; source/repository scripts never run.
- Graph assertions retain producer, evidence, provenance, confidence, coverage, conflicts, and generation identity.
- Ownership, lexical, load, object, inheritance, registration, lifecycle, state, call, and lineage axes remain distinct.
- A reason path never becomes a silent direct edge.
- Named addon/framework packs can calibrate universal recognizers; production semantics never branch on repository/addon/owner/path/popularity names.
- A repository commit pin is not an admitted calibration corpus member.
- Raw source, normalized facts, labels, splits, mutations, pack bytes, run results, and candidate artifacts remain separate immutable identities.
- Forks, copies, vendored code, generated templates, near-duplicates, and mutation families do not cross ordinary evaluation splits.
- Expected labels, split assignments, reviewer notes, search/model output, and donor metadata never enter matcher semantics.
- `Unknown`, `Possible`, `NotEvaluated`, conflict, partial, truncated, and cancelled are never coerced to Negative or pass.
- E5-A calibration packs are `calibration` + `shadow_only`, emit universal registered graph proposals, and are limited to `Derived`/`Possible` confidence.
- `ShadowValidated` and `PromotionEligibleByMetrics` are not reviewer authorization, publication, core activation, or runtime proof.
- Disabling a candidate removes only its exact owned shadow partitions and downgrades only their coverage.
- API/reference contract, implementation source, runtime observation, review authorization, calibration metrics, and community evidence are different authority classes.
- Missing coverage never becomes a clean negative answer.
- Project Map and context are projections, not a second graph or new authority.
- Source/query/review/migration/label text remains structurally isolated untrusted data and cannot control profiles, tools, ranking, proof ceilings, or agent instructions.
- Exact token counts require a frozen exact tokenizer and framing profile over exact final bytes.
- One SearchShard binds one exact owner generation; no mutable global/current corpus.
- Exact identifiers are case-sensitive; a folded or fuzzy match remains approximate.
- Raw FTS/BM25 values are shard-local and never compared across corpora.
- Search ranking retrieves candidates; it never proves user intent, alias, lineage, replacement, impact, safety, or runtime behavior.
- Search selection is explicit; top-1, sole candidate, rank, display name, or query text cannot be silently selected.
- Search and context remain separate: service passes an explicitly selected exact root.
- Cross-generation entities remain distinct; lineage is an explicit overlay.
- Same name, path, signature, fingerprint, graph neighborhood, uniqueness, or rank is Candidate evidence only.
- One-to-many, many-to-one, split, merge, copy, and ambiguity are preserved rather than forced into a bijection.
- Review authorization does not create proof, and review cannot exceed the minimum proof ceiling.
- Same lineage does not imply deprecation, replacement, edit compatibility, or migration success.
- Migration validation does not apply an edit or prove runtime correctness.
- Static impact preserves exact reason paths and does not claim runtime breakage, severity, performance, taint, combat, Secret behavior, or fixability.
- Applications depend on `wow-service` only and never reproduce domain algorithms.
- No public success precedes mandatory resource closure.

## E5-A current evidence state

The contract records these exact user-repository revisions as candidate inputs:

```text
UnknownAlienHuman/roth-ui           1656d4b9d33be914be2058460520e7423668d95c
UnknownAlienHuman/roth-chat         3c995183626002965043e38a837346fb290acd8a
UnknownAlienHuman/roth-tooltip      28426fef16daadc5808fec6d38b445a97f42a71a
UnknownAlienHuman/interrupt-glow    786ef9f11059b28541007af92963bc9e2234f154
UnknownAlienHuman/old-runes         9938d95759970953a7ac178a95bb5ad7aa62cb81
UnknownAlienHuman/trash-panda       f27ba9f09be0f716cb2c5f7605ed697d8aabb320
UnknownAlienHuman/gcd-optimizer     00d8bd22f03b1136841f548c0a4a5a776c1a7c71
UnknownAlienHuman/roth-blizz-plates 61de4d4d49ccf229ff3b7bff1ae1b5f97351b762
```

Current status:

```text
exact revisions pinned: 8
real admitted corpus members: 0
real measured calibration runs: 0
sealed holdout generations: 0
promotion submissions: 0
```

Admission remains blocked until exact tree/content inventories, source/project/analyzer/graph/fact publications, upstream/fork/copy/vendor/generated provenance groups, license/privacy/notice decisions, independent labels, and split eligibility are frozen. Closed synthetic fixtures validate artifact shapes only; they do not claim donor generalization or recognizer performance.

## Documented CLI surface

The executable CLI does not exist yet. Current contracts define:

```text
wow status
wow check

wow context status
wow context map
wow context inspect
wow context build
wow context continue
wow context validate
wow context render

wow search index status
wow search index build
wow search index validate
wow search query
wow search continue
wow search explain
wow search select
wow search context

wow lineage status
wow lineage build
wow lineage validate
wow lineage review validate
wow lineage review apply
wow lineage compare
wow lineage trace
wow lineage explain

wow migration candidates
wow migration validate

wow impact plan
wow impact run
wow impact continue
wow impact explain

wow-reference-builder build
wow-reference-builder validate
wow-reference-builder rebuild-compare
```

E5-B will define calibration commands only after the service orchestration, reviewer-authorization, sealed-holdout, retention, output, and exit contracts are frozen. There is no migration-apply command, automatic candidate selection, automatic pack promotion, runtime-impact command, or tool-execution escape hatch.

## First implementation target

Rust implementation still begins with the E0 vertical slice:

```text
pinned upstream EmmyLua analysis
+ one frozen ReferenceView fixture
+ one generated annotation fixture
+ one generic Lua diagnostic
+ one WoW API diagnostic
+ one bounded Secret-local diagnostic
-> one deterministic wow check result
```

Agents must not start persistent stores, the full graph, Blizzard UI indexing, context, search, lineage, calibration, LSP, MCP, or release code before prerequisite implementation and fixture/checksum gates pass.

## Documentation routes

Start with:

- [`AGENTS.md`](AGENTS.md)
- [`docs/README.md`](docs/README.md)
- [`crates/README.md`](crates/README.md)
- [`crates/MANIFEST.json`](crates/MANIFEST.json)
- [`crates/DEPENDENCY_GRAPH.md`](crates/DEPENDENCY_GRAPH.md)
- [`crates/WORKSTREAMS.md`](crates/WORKSTREAMS.md)
- [`docs/ROADMAP.md`](docs/ROADMAP.md)

Major domain contracts:

- [`crates/wow-core/`](crates/wow-core/README.md)
- [`crates/wow-reference/`](crates/wow-reference/README.md)
- [`crates/wow-emmy/`](crates/wow-emmy/README.md)
- [`crates/wow-store/`](crates/wow-store/README.md)
- [`crates/wow-project/`](crates/wow-project/README.md)
- [`crates/wow-graph/`](crates/wow-graph/README.md)
- [`crates/wow-recognizers/`](crates/wow-recognizers/README.md)
- [`crates/wow-recognizers/e5/`](crates/wow-recognizers/e5/README.md)
- [`crates/wow-rules/`](crates/wow-rules/README.md)
- [`crates/wow-context/`](crates/wow-context/README.md)
- [`crates/wow-search/`](crates/wow-search/README.md)
- [`crates/wow-service/`](crates/wow-service/README.md)
- [`apps/wow/`](apps/wow/README.md)

## Repository layout

```text
crates/      production library contracts and, later, Rust crates
apps/        thin CLI/LSP/MCP/service adapters
schemas/     versioned public data contracts
tools/       builders, evaluators, corpus, migration and release utilities
tests/       fixtures, golden tests, evaluations and compatibility probes
docs/        architecture, operating guidance, roadmap, research and archive
```

A directory is not an activated crate. Empty placeholder implementations and default-success stubs are forbidden.

## Related knowledge base

Patch-sensitive WoW engineering research, current API/security guidance, field reports, active upstream issues, and historical evidence live separately:

**[WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb)**

This framework links the knowledge base. A conclusion is copied here only when it becomes a stable contract, ADR, schema, fixture, test, or exact release input.

## Repository policy

- English is the canonical architecture/contract language.
- Addon localization content can remain in its target languages.
- No CI/workflow is added without an explicit owner request.
- Missing tools, probes, benchmarks, evaluation, client tests, authorization adapters, holdout infrastructure, or implementation evidence are reported as skipped/blocked/NotEvaluated, never pass.

## License

MIT. See [`LICENSE`](LICENSE).

## Author

Neomorph / [UnknownAlienHuman](https://github.com/UnknownAlienHuman)
