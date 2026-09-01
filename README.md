# WoW Dev Framework

**Rust-first, editor-independent code intelligence and agent tooling for World of Warcraft addon development.**

> **Documentation frontier:** E4-A is complete: exact Reference Pack, analyzer/project/graph/persistence, separately published Blizzard UI source, deterministic Project Maps and L0/L1 context, service/CLI context orchestration, and immutable exact-generation search shards with explainable retrieval.
>
> **Implementation frontier:** not started. The first executable work remains E0-A, not E4. No `Cargo.toml`, Rust source, or CI workflow exists.
>
> **Compatibility model:** every result binds one exact World of Warcraft profile and exact project/reference/source generations. Live patch guidance remains routed through the separate [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb).

WoW Dev Framework is designed to give coding agents and addon developers a compact, exact, explainable technical surface over addon repositories, Blizzard API contracts, Blizzard UI implementation source, static structure, diagnostics, search candidates, migration evidence, and bounded source context.

It is not a generic RAG product, an editor-settings mutator, a runtime injection platform, a replacement for Codebase Memory, or a model-authority layer.

## What the framework should answer

- Does an API, event, method, template, mixin, package, field, file, or project entity exist in one exact profile?
- What exact project/source/reference evidence supports the answer?
- Which package, TOC, XML document, file, module, registry, state root, or lifecycle surface owns an entity?
- How is it reached through the real TOC/XML/dependency/load graph?
- Which relation is direct, derived, possible, conflicted, omitted, or not evaluated?
- What is the smallest trustworthy Project Map, L0/L1 skeleton, reason path, or source excerpt needed for a task?
- Which exact and approximate search lanes found an entity, and why did it outrank another candidate?
- Is a search miss authoritative, partial, candidate-only, truncated, or nonauthoritative?
- Which migration/lineage/impact conclusion is proven, and which is only a candidate?
- Which operation is statically permitted, and which still requires exact runtime evidence?

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

E4-B next
    explicit cross-generation lineage, rename/move/replacement/removal/introduction
    migration compatibility and bounded static impact

E4-C+
    search/lineage/impact service and CLI
    calibration packs
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

service/application layer
    exact/current publication selection
    retained view/lease lifecycle
    explicit candidate selection and use-case sequencing
    canonical envelopes and thin CLI transport

later history/integration layer
    explicit lineage/migration/impact assertions
    audited recognizer calibration
    optional external semantic candidates
    LSP/MCP and release controls
```

## Non-negotiable invariants

- One result uses one exact coherent generation/profile set; no mixed current/latest data.
- `CurrentPublished` is resolved only at the service boundary and never refreshed mid-request.
- Independent project, Blizzard UI, reference, and search stores are not falsely described as a distributed atomic snapshot.
- User project, Blizzard UI implementation, Reference Pack, search candidate, external candidate, runtime, and historical universes remain distinct.
- EmmyLua is the sole correctness-path Lua parser/analyzer and is pinned behind one adapter.
- TOC/XML parsing is bounded and nonexecuting; source/repository scripts never run.
- Graph assertions retain producer, evidence, provenance, confidence, coverage, conflicts, and generation identity.
- Ownership, lexical, load, object, inheritance, registration, lifecycle, state, call, and later lineage axes remain distinct.
- A reason path never becomes a silent direct edge.
- Named addon/framework packs can calibrate universal recognizers; production semantics never branch on repository/addon/path/popularity names.
- API/reference contract, implementation source, runtime observation, and community evidence are different authority classes.
- Missing coverage never becomes a clean negative answer.
- `Possible`, `Candidate`, conflict, partial, truncated, cancelled, and `NotEvaluated` are never upgraded or hidden.
- Project Map and context are projections, not a second graph or new authority.
- Source text remains structurally isolated untrusted data and cannot control profiles, tools, headings, query syntax, ranking, or agent instructions.
- Exact token counts require a frozen exact tokenizer and framing profile over exact final bytes.
- One SearchShard binds one exact owner generation; no mutable global/current corpus.
- Exact identifiers are case-sensitive; a folded or fuzzy match remains approximate.
- Raw FTS/BM25 values are shard-local and never compared across corpora.
- Search ranking retrieves candidates; it never proves user intent, alias, lineage, replacement, impact, safety, or runtime behavior.
- Search and context remain separate: service passes explicitly selected exact roots.
- Applications depend on `wow-service` only and never reproduce domain algorithms.
- No public success precedes mandatory resource closure.

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

wow-reference-builder build
wow-reference-builder validate
wow-reference-builder rebuild-compare
```

Search CLI operations are deferred to E4-C. Documented search architecture does not make a command available.

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

Agents must not start persistent stores, the full graph, Blizzard UI indexing, context, search, LSP, MCP, or release code before prerequisite implementation and fixture/checksum gates pass.

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
- Missing tools, probes, benchmarks, evaluation, client tests, or implementation evidence are reported as skipped/blocked/NotEvaluated, never pass.

## License

MIT. See [`LICENSE`](LICENSE).

## Author

Neomorph / [UnknownAlienHuman](https://github.com/UnknownAlienHuman)
