# Crate implementation contracts

**Documentation frontier:** E6-B complete. **Implementation frontier:** not started.

Directories contain contracts for future Rust libraries. A directory is not an activated crate; no `Cargo.toml` or `.rs` placeholder is created before its freeze gate.

## Required reading

1. [`../AGENTS.md`](../AGENTS.md)
2. [`AGENTS.md`](AGENTS.md)
3. [`MANIFEST.json`](MANIFEST.json)
4. [`DEPENDENCY_GRAPH.md`](DEPENDENCY_GRAPH.md)
5. [`WORKSTREAMS.md`](WORKSTREAMS.md)
6. [`../docs/LAUNCH_GATES.md`](../docs/LAUNCH_GATES.md)
7. target crate router/contract
8. current [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) routes
9. actual addon repository/local instructions for addon-facing work

## Production ownership

| Crate | Primary ownership | Frontier |
|---|---|---:|
| `wow-core` | identities/evidence/coverage/results | E0-A |
| `wow-store` | SQLite/objects/publication/recovery/GC | E6-B generic external-result seam |
| `wow-reference` | exact ReferenceProfile/View/transitions/mapping | E6-B mapping seam |
| `wow-annotations` | annotation projection/maps/loss/parity | E1-C |
| `wow-emmy` | pinned analyzer adapter/facts/diagnostics | E0-C |
| `wow-project` | source/TOC/XML/load/publication/reindex/mapping | E6-B mapping seam |
| `wow-graph` | typed graph/lineage/impact/partition validation | E5-C closure seam; E6-B exact context input |
| `wow-recognizers` | structural rules/calibration/core pack semantics | E5-C owner seam |
| `wow-rules` | diagnostic providers/capability gates | E0-E |
| `wow-search` | exact-generation retrieval/candidates | E4-B seam |
| `wow-context` | Project Map/L0/L1/context packs | E6-B exact mapped-root handoff |
| `wow-cbm` | optional external Candidate-only bridge | E6-A |
| `wow-service` | orchestration/durable effects/authorization/mapping/context envelopes | E6-B |

Completed documentation: E0-A–E6-B. Next: **E7-A** supported transport/session and developer-preview release boundary.

Executable implementation still starts at E0-A. Later documentation never bypasses earlier implementation/freeze gates.

## E5 gate separation

```text
E5-A calibration evidence and shadow candidates
E5-B durable runs, review, sealed holdout and PromotionSubmission
E5-C immutable core artifact, signing, inactive publication, canary,
     guarded activation, finite rollout, LKG, rollback/revocation/closure
```

Submission, signature, publication, canary, activation, rollout, rollback, distribution and runtime correctness are separate. Public distribution remains E7-B.

## E6 gate separation

```text
E6-A wow-cbm
    reviewed provider descriptors and exact external-state classes
    bounded allow-listed candidate queries
    Candidate-only normalization and provider-local scoring
    unverified locators, zero-result honesty, continuation/cache
    optional degradation and no provider/database/session ownership

E6-B wow-service + apps/wow + owner seams
    exact provider configuration/session authorization references
    durable external query/result/artifact operations and retention
    exact project/reference locator mapping
    explicit candidate selection receipt
    exact mapped-root context handoff with separate Candidate sidecar
    canonical envelopes and thin CLI
```

E6-A cannot create exact local authority. E6-B mapping proves only locator-to-owner-record identity; selection and context do not verify provider summaries/relations. E6 remains optional and may ship disabled.

## Launch order

```text
first runnable: E0-A -> E0-F
useful internal alpha: E1 + E2 + E3
external developer preview: E4 + minimal E7-A frontend
governed beta: E5; optionally E6
public supported v1: selected beta scope + E7-A/E7-B release gates
```

See [`../docs/LAUNCH_GATES.md`](../docs/LAUNCH_GATES.md).