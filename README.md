# WoW Dev Framework

**Rust-first, editor-independent code intelligence and agent tooling for World of Warcraft addon development.**

> **Documentation frontier:** E6-A complete — optional external provider descriptors/state/query contracts, hard Candidate authority, unverified locators, zero-result honesty, exact continuation/cache, privacy/security, and lane-local degradation.
>
> **Next documentation package:** E6-B — provider/session/credential acquisition, durable external-candidate orchestration, exact source-owner mapping, explicit selection, context handoff, and thin CLI.
>
> **Implementation frontier:** not started. Executable work still begins with E0-A. No `Cargo.toml`, Rust source, or CI workflow exists.
>
> Patch-sensitive WoW guidance remains in the separate [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb).

WoW Dev Framework provides exact, generation-bound, evidence-preserving technical context over addon repositories, Blizzard API/UI source, diagnostics, graph, search, lineage, migration evidence, static impact, Project Maps, L0/L1 skeletons, recognizer calibration/publication, and optional external semantic candidates.

It is not a generic RAG product, editor-settings mutator, runtime injection platform, source-edit executor, repository-specific heuristic engine, model-authority layer, provider database owner, or automatic promotion system.

## Contract stack

```text
E0  identity/evidence/results and diagnostic vertical slice
E1  ReferenceStore/View, annotations and Reference Pack build/validation
E2  graph, recognizers, project indexing and ProjectStore
E3  Blizzard UI source, Project Map/L0/L1/context and service/CLI
E4  exact-generation search, lineage/migration/static impact and service/CLI
E5  calibration evidence, independent review/holdout, immutable core publication lifecycle
E6-A optional external semantic-candidate owner bridge
E6-B next: provider/session/mapping/selection/context service and CLI
E7 later: LSP/MCP and public release/distribution integration
```

Machine state and implementation order are in [`crates/MANIFEST.json`](crates/MANIFEST.json) and [`crates/WORKSTREAMS.md`](crates/WORKSTREAMS.md).

## External candidate invariants

- `wow-cbm` depends directly only on `wow-core`.
- Provider descriptors are reviewed immutable contracts; runtime negotiation cannot widen them.
- Provider process/session/credential and index lifecycle belong outside E6-A.
- There is no generic arbitrary MCP/tool call surface.
- Every normalized external result is `provenance=semantic_candidate`, `confidence=Candidate`, `negative_authority=unavailable`.
- Provider `exact`, `verified`, `authoritative`, top-1, sole-result, repeated-result, and high-score labels never upgrade authority.
- Scores/ranks remain provider-local and are never numerically fused across providers.
- Provider paths/URIs/revisions/symbols/spans remain `UnverifiedProviderLocator`; E6-A never opens or maps them into project/reference truth.
- A zero result proves only no accepted provider candidates for the exact query/state under reported coverage; it is never global absence.
- Continuation/cache bind exact provider descriptor/capability/state/query/profile and cumulative budgets.
- Opaque provider state is explicitly nonreproducible.
- Provider failure is lane-local and cannot degrade exact ReferenceView/project/graph/search/context/diagnostic capabilities.
- No hidden fallback to another provider, stale cache, model, web, local search, or broader query.
- Source snippets/summaries/labels/paths/errors remain untrusted data.
- E6-B must perform exact owner mapping and explicit candidate selection before context handoff.

## Existing core invariants

Exact generation separation, EmmyLua sole-parser policy, bounded nonexecuting ingestion, producer/evidence/coverage graph discipline, Candidate and `NotEvaluated` honesty, E5 review/holdout/publication separation, profile-specific guarded activation, explicit LKG, immutable rollback/history, application-to-service-only dependency, and close-before-success remain active.

## Current implementation evidence

Documentation defines contracts only:

```text
E0–E6 implementations: 0
real admitted calibration corpus members: 0
real measured calibration runs: 0
published core packs: 0
live external provider adapters/probes: 0
exact external owner mappings: 0
```

Missing implementations, exact provider probes, adapters, credentials infrastructure, corpora, benchmarks, runtime observations, and checksums remain blocked/`NotEvaluated`, never passed.

## First implementation target

Rust implementation still starts with the E0 diagnostic vertical slice. External provider/session/mapping/service work remains blocked until E6-A and all prerequisite implementation/freeze gates pass.

## Routes

- [`AGENTS.md`](AGENTS.md)
- [`docs/README.md`](docs/README.md)
- [`docs/ROADMAP.md`](docs/ROADMAP.md)
- [`crates/README.md`](crates/README.md)
- [`crates/MANIFEST.json`](crates/MANIFEST.json)
- [`crates/DEPENDENCY_GRAPH.md`](crates/DEPENDENCY_GRAPH.md)
- [`crates/WORKSTREAMS.md`](crates/WORKSTREAMS.md)
- [`crates/wow-cbm/e6/`](crates/wow-cbm/e6/README.md)

## License

MIT. Third-party, provider-returned, addon, and Blizzard-source artifacts retain separate provenance, license, privacy, notice, and redistribution decisions.