# Crate contract map

**Status:** documentation frontier E3-B; Rust workspace not activated.

Each directory below is an implementation contract, not yet a Rust crate. A future agent must satisfy its exact freeze gate before adding `Cargo.toml` or `.rs` files.

| Crate | Responsibility | Documentation frontier |
|---|---|---|
| [`wow-core`](wow-core/README.md) | shared identities, generations, evidence, coverage, findings, canonical results | E0-A |
| [`wow-store`](wow-store/README.md) | SQLite substrate, ReferenceStore, ProjectStore, publication/recovery/GC | E2-D |
| [`wow-reference`](wow-reference/README.md) | exact Reference Profiles, API facts, corrections, coverage, ReferenceView | E1-B |
| [`wow-annotations`](wow-annotations/README.md) | annotation semantic model, lowering, rendering, parity, source maps | E1-C |
| [`wow-emmy`](wow-emmy/README.md) | pinned analyzer adapter, facts, diagnostics, coordinates | E0-C |
| [`wow-project`](wow-project/README.md) | addon/source universes, TOC/XML/load, analyzer/recognizer orchestration, publication, Blizzard UI index | E3-A |
| [`wow-graph`](wow-graph/README.md) | graph registries, assertions, producer partitions, snapshots, bounded queries | E2-A/E2-D seam |
| [`wow-recognizers`](wow-recognizers/README.md) | declarative typed structural matching and graph proposals | E2-B |
| [`wow-rules`](wow-rules/README.md) | diagnostic providers, findings, remediation tiers | E0-E |
| [`wow-context`](wow-context/README.md) | Project Map, L0/L1, bounded evidence-preserving context packs | E3-B |
| [`wow-search`](wow-search/README.md) | exact/migration/shape/FTS/graph retrieval and ranking | deferred E4 |
| [`wow-cbm`](wow-cbm/README.md) | optional external Codebase Memory candidate bridge | deferred E6 |
| [`wow-service`](wow-service/README.md) | coherent public use-case orchestration | E1-D; later operations deferred |

## Required route for implementation agents

1. repository [`AGENTS.md`](../AGENTS.md);
2. [`AGENTS.md`](AGENTS.md), [`DEPENDENCY_GRAPH.md`](DEPENDENCY_GRAPH.md), [`WORKSTREAMS.md`](WORKSTREAMS.md), and [`MANIFEST.json`](MANIFEST.json);
3. target crate router and active work-package contract;
4. current `wow-addon-engineering-kb` `AGENTS.md` and `INDEX_MINI.md` task route when WoW semantics are involved;
5. exact prerequisite implementations, fixture manifests, and checksums;
6. actual pinned addon/source repositories required by the work package.

## Global implementation rules

- no placeholder crates or fake successful operations;
- allowed dependency edges are maxima, not defaults;
- one agent owns one work package and primary crate;
- every artifact and result binds exact profiles/generations;
- partial/conflicted/truncated/failed/cancelled/Candidate/NotEvaluated state remains explicit;
- no repository/addon/path/provider-name production special cases;
- no source/repository script execution;
- no patch-sensitive WoW facts copied into stable algorithms;
- no CI unless the owner explicitly asks for it.

## Current next decision

After E3-B documentation, either define E3-C service/application context orchestration or proceed to E4 search/lineage contracts. Rust implementation still starts from E0-A and follows the dependency/freeze order; documentation order does not waive prerequisites.
