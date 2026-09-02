# Crate implementation contracts

**Documentation frontier:** E6-A complete. **Implementation frontier:** not started.

Directories contain contracts for future Rust libraries. A directory is not an activated crate; no `Cargo.toml` or `.rs` placeholder is created before its freeze gate.

## Required reading

1. [`../AGENTS.md`](../AGENTS.md)
2. [`AGENTS.md`](AGENTS.md)
3. [`MANIFEST.json`](MANIFEST.json)
4. [`DEPENDENCY_GRAPH.md`](DEPENDENCY_GRAPH.md)
5. [`WORKSTREAMS.md`](WORKSTREAMS.md)
6. target crate router/contract
7. current [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) routes
8. actual addon repository/local instructions for addon-facing work

## Production ownership

| Crate | Primary ownership | Frontier |
|---|---|---:|
| `wow-core` | identities/evidence/coverage/results | E0-A |
| `wow-store` | SQLite/objects/publication/recovery/GC | E5-C store seam; E6-B catalog/retention seam next |
| `wow-reference` | exact ReferenceProfile/View/transitions | E4-B seam; E6-B mapping owner next |
| `wow-annotations` | annotation projection/maps/loss/parity | E1-C |
| `wow-emmy` | pinned analyzer adapter/facts/diagnostics | E0-C |
| `wow-project` | source/TOC/XML/load/publication/reindex | E5-C reindex seam; E6-B mapping owner next |
| `wow-graph` | typed graph/lineage/impact/partition validation | E5-C closure seam |
| `wow-recognizers` | structural rules/calibration/core pack semantics | E5-C owner seam |
| `wow-rules` | diagnostic providers/capability gates | E0-E |
| `wow-search` | exact-generation retrieval/candidates | E4-B seam |
| `wow-context` | Project Map/L0/L1/context packs | E3-B; E6-B exact-root handoff next |
| `wow-cbm` | optional external Candidate-only bridge | E6-A |
| `wow-service` | orchestration/durable effects/authorization/publication envelopes | E5-C; E6-B next |

Completed documentation: E0-A–E6-A. Next: **E6-B** external-candidate service/mapping/context/CLI integration.

Executable implementation still starts at E0-A. Later documentation never bypasses earlier implementation/freeze gates.

## E5 gate separation

```text
E5-A calibration evidence and shadow candidates
E5-B durable runs, review, sealed holdout and PromotionSubmission
E5-C immutable core artifact, signing, inactive publication, canary,
     guarded activation, finite rollout, LKG, rollback/revocation/closure
```

Submission, signature, publication, canary, activation, rollout, rollback, distribution and runtime correctness are separate. Public distribution remains E7.

## E6 gate separation

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