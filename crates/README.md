# Crate implementation contracts

**Documentation frontier:** E5-B complete. **Implementation frontier:** not started.

Directories contain implementation contracts for future Rust libraries. A directory is not an activated crate; no `Cargo.toml` or `.rs` placeholder is created before its freeze gate.

## Required reading

1. [`../AGENTS.md`](../AGENTS.md)
2. [`AGENTS.md`](AGENTS.md)
3. [`MANIFEST.json`](MANIFEST.json)
4. [`DEPENDENCY_GRAPH.md`](DEPENDENCY_GRAPH.md)
5. [`WORKSTREAMS.md`](WORKSTREAMS.md)
6. target crate router/contract
7. current [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) routes
8. actual addon repository/local instructions for addon-facing work

## Production crates

| Crate | Primary ownership | Documentation frontier |
|---|---|---:|
| `wow-core` | identities, profiles, evidence, coverage, conflicts, canonical results | E0-A |
| `wow-store` | SQLite lifecycle, immutable stores, publication, recovery/GC | E2-D; E5-B durable seam |
| `wow-reference` | exact ReferenceProfile/ReferenceView and transitions | E4-B seam |
| `wow-annotations` | annotation projection, rendering, maps/loss/parity | E1-C |
| `wow-emmy` | pinned analyzer adapter, snapshots, facts, diagnostics | E0-C |
| `wow-project` | source universes, TOC/XML/load, publication and owner facts | E5-B input seam |
| `wow-graph` | typed graph, lineage, migration records, impact, validation | E5 graph-validation seam |
| `wow-recognizers` | structural rules and E5-A calibration owner algorithms | E5-A / E5-B owner seam |
| `wow-rules` | diagnostics and capability gates | E0-E |
| `wow-search` | exact-generation search and candidate explanations | E4-B seam |
| `wow-context` | Project Map, L0/L1, bounded context packs | E3-B |
| `wow-cbm` | optional external semantic candidates | E6 |
| `wow-service` | multi-owner orchestration, durable effects, authorization, envelopes | E5-B |

## Completed documentation

```text
E0-A through E0-F
E1-A through E1-D
E2-A through E2-D
E3-A through E3-C
E4-A through E4-C
E5-A
E5-B
```

Next: **E5-C** — immutable core-pack publication/catalog/signing, canary, guarded activation, rollout, rollback, and last-known-good.

Executable implementation still starts at E0-A and follows [`WORKSTREAMS.md`](WORKSTREAMS.md). Later documentation never bypasses earlier implementation/freeze gates.

## E5 separation

```text
E5-A wow-recognizers
    corpus/labels/splits/pack validation, shadow matching,
    mutations, metrics, candidates and deactivation plans

E5-B wow-service + apps/wow
    exact retained acquisition, durable runs, reviewer authorization,
    sealed-holdout audit/consumption and promotion submissions

E5-C next
    immutable publication/signing/catalog, canary,
    activation, rollout, rollback and last-known-good
```

Metrics, graph validity, review authorization, holdout authorization, submission, publication, activation, and runtime correctness are independent gates.