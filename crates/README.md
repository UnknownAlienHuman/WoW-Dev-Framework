# Crate implementation contracts

**Documentation frontier:** E6-A complete. **Implementation frontier:** not started.

A directory is not an activated crate; no `Cargo.toml` or `.rs` placeholder is created before its implementation freeze gate.

## Required reading

1. [`../AGENTS.md`](../AGENTS.md)
2. [`AGENTS.md`](AGENTS.md)
3. [`MANIFEST.json`](MANIFEST.json)
4. [`DEPENDENCY_GRAPH.md`](DEPENDENCY_GRAPH.md)
5. [`WORKSTREAMS.md`](WORKSTREAMS.md)
6. target crate router/contract
7. current [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) routes
8. actual addon repository/local instructions for addon-facing work

## Ownership/frontier

| Crate | Primary ownership | Frontier |
|---|---|---:|
| `wow-core` | identities/evidence/coverage/results | E0-A |
| `wow-store` | storage/publication/recovery/GC | E5-C seam |
| `wow-reference` | exact ReferenceProfile/View/transitions | E4-B seam |
| `wow-annotations` | annotation projection/maps/loss/parity | E1-C |
| `wow-emmy` | pinned analyzer adapter/facts/diagnostics | E0-C |
| `wow-project` | source/index/publication/reindex | E5-C seam; E6-B mapping owner |
| `wow-graph` | graph/lineage/impact/partition validation | E5-C seam |
| `wow-recognizers` | structural rules/calibration/core semantics | E5-C seam |
| `wow-rules` | diagnostics/capability gates | E0-E |
| `wow-search` | exact-generation local retrieval | E4-A/B |
| `wow-context` | Project Map/L0/L1/context packs | E3-B; E6-B context owner |
| `wow-cbm` | optional external Candidate-only bridge | E6-A |
| `wow-service` | orchestration/durable effects/authorization/publication | E6-B next |

Completed documentation: E0-A–E6-A. Next: **E6-B** in `wow-service` + `apps/wow`.

Executable implementation remains E0-first.

## E6 separation

```text
E6-A wow-cbm
    reviewed provider descriptors and exact external-state classes
    bounded allow-listed candidate queries
    Candidate-only normalization and provider-local scoring
    unverified locators, zero-result honesty, continuation/cache
    optional degradation and no provider/database/session ownership

E6-B next: wow-service + apps/wow
    configured provider/session/credential acquisition
    durable external-candidate operations and retention
    exact project/reference owner mapping
    explicit candidate selection receipt
    exact mapped-root context handoff
    canonical envelopes and thin CLI
```

E6-A cannot create exact local authority. E6-B mapping can prove only locator-to-owner-record identity; it cannot verify provider summaries/relations or select a candidate implicitly.