# Crate implementation contracts

**Documentation frontier:** E7-A complete. **Implementation frontier:** not started.

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
| `wow-store` | immutable storage/effects/retention/recovery | E7-A session/journal seam |
| `wow-reference` | exact ReferenceProfile/View/transitions/mapping | E6-B mapping seam |
| `wow-annotations` | annotation projection/maps/loss/parity | E1-C |
| `wow-emmy` | pinned analyzer adapter/facts/overlay analysis | E7-A overlay seam |
| `wow-project` | source/index/publication/reindex/mapping/overlays | E7-A workspace/overlay seam |
| `wow-graph` | typed graph/lineage/impact/partition validation | E5-C/E6-B seams |
| `wow-recognizers` | structural rules/calibration/core semantics | E5-C |
| `wow-rules` | diagnostics/capability gates/live diagnostic results | E7-A live seam |
| `wow-search` | exact-generation retrieval/candidates | E4 |
| `wow-context` | Project Map/L0/L1/context packs | E6-B handoff |
| `wow-cbm` | optional external Candidate-only bridge | E6-A |
| `wow-service` | orchestration/durable effects/frontend registry/sessions | E7-A |

Completed documentation: E0-A–E7-A. Next: **E7-B** public release and support lifecycle.

Executable implementation still starts at E0-A. Later documentation never bypasses earlier implementation/freeze gates.

## E5 separation

```text
E5-A calibration evidence and shadow candidates
E5-B durable runs, review, sealed holdout and PromotionSubmission
E5-C immutable core artifact, signing, inactive publication, canary,
     guarded activation, finite rollout, LKG, rollback/revocation/closure
```

## E6 separation

```text
E6-A wow-cbm owns pure external Candidate normalization
E6-B wow-service coordinates session/result/mapping/selection/context
```

E6 remains optional and may ship disabled. Mapping and selection never verify provider interpretation.

## E7-A separation

```text
owner crates
    project workspace/overlay identity
    Emmy overlay analysis
    diagnostics/search/context results
    generic store/journal/retention

wow-service
    immutable operation registry
    session/workspace/document orchestration
    exact request/result/delivery state

apps/wow
    one-shot CLI
    foreground local daemon
    LSP 3.18 stdio
    MCP 2025-11-25 stdio
    optional local-only MCP HTTP
```

Transports do not own semantic algorithms, advertise missing capabilities, infer workspaces, expose generic tools, treat disconnect as cancellation, or mutate source automatically.

## Launch order

```text
first runnable: E0-A -> E0-F
useful internal alpha: E1 + E2 + E3
developer preview: E4 + implemented minimal E7-A frontend
governed beta: E5; optionally E6
public supported v1: selected beta scope + implemented E7-A + E7-B release gates
```

See [`../docs/LAUNCH_GATES.md`](../docs/LAUNCH_GATES.md).