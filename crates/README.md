# Crate implementation contracts

**Documentation frontier:** E5-C complete. **Implementation frontier:** not started.

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
| `wow-store` | SQLite/objects/publication/recovery/GC | E5-C store seam |
| `wow-reference` | exact ReferenceProfile/View/transitions | E4-B seam |
| `wow-annotations` | annotation projection/maps/loss/parity | E1-C |
| `wow-emmy` | pinned analyzer adapter/facts/diagnostics | E0-C |
| `wow-project` | source/TOC/XML/load/publication/reindex | E5-C reindex seam |
| `wow-graph` | typed graph/lineage/impact/partition validation | E5-C closure seam |
| `wow-recognizers` | structural rules/calibration/core pack semantics | E5-C owner seam |
| `wow-rules` | diagnostic providers/capability gates | E0-E |
| `wow-search` | exact-generation retrieval/candidates | E4-B seam |
| `wow-context` | Project Map/L0/L1/context packs | E3-B |
| `wow-cbm` | optional external candidates | E6-A next |
| `wow-service` | orchestration/durable effects/authorization/publication envelopes | E5-C |

Completed documentation: E0-A–E5-C. Next: **E6-A** optional external semantic-candidate bridge owned by `wow-cbm`; E6-B later provides service/CLI integration.

Executable implementation still starts at E0-A. Later documentation never bypasses earlier implementation/freeze gates.

## E5 gate separation

```text
E5-A calibration evidence and shadow candidates
E5-B durable runs, review, sealed holdout and PromotionSubmission
E5-C immutable core artifact, signing, inactive publication, canary,
     guarded activation, finite rollout, LKG, rollback/revocation/closure
```

Submission, signature, publication, canary, activation, rollout, rollback, distribution and runtime correctness are separate. Public distribution remains E7.