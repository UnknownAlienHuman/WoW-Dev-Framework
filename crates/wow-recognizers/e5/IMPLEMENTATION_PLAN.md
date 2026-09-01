# E5-A implementation plan

**Status:** normative sequence; implementation not started.

## Phase 0 — prerequisite and byte freeze

Before the first E5-A Rust commit:

- implement and freeze `wow-core` E0-A identities/evidence/canonicalization;
- implement and freeze `wow-emmy` normalized fact profiles used by the corpus;
- implement and freeze `wow-graph` E2-A registry/proposal validation;
- implement and freeze `wow-recognizers` E2-B pack parser/compiler/matcher/output partitions;
- implement and freeze exact `wow-project` E2-C fact-publication and `wow-store` E2-D retained publication inputs used by real corpora;
- freeze candidate-source, provenance, license/privacy, label, split, mutation, evaluation, budget, threshold, canonicalization, graph-validation, deactivation, fixture, benchmark, and checksum identities;
- update manifest/workstream implementation state.

No code binds to null profile/implementation/checksum fields. No Cargo/Rust/CI is added during the documentation phase.

## Phase 1 — calibration value types and canonical identities

Implement bounded types and canonicalization for candidate sources, corpora, examples, labels/reviews, provenance groups, splits, mutations, runs, case results, metrics, anti-overfitting reports, candidate artifacts, and deactivation plans.

Tests: `CAL-CONFIG-*`, identity/canonicalization portions of `CAL-CORPUS-*` and `CAL-DET-*`.

## Phase 2 — candidate source and admission validation

Implement pure validation of exact revision/tree/source/publication/fact bindings, inventories/exclusions, provenance evidence, license/privacy artifact classes, blockers, and admission state.

Materialization remains outside `wow-recognizers`; use immutable fixture artifacts only.

Tests: `CAL-SOURCE-*`, `CAL-CORPUS-*`, security source-execution cases.

## Phase 3 — provenance graph and split engine

Implement conservative upstream/fork/copy/vendor/generated/near-duplicate connected-component closure, explicit group-key profiles, split validation, leakage candidate reporting, holdout visibility state, and consumed-generation tracking.

Similarity produces unresolved leakage candidates, not automatic independence proof.

Tests: `CAL-SPLIT-*`.

## Phase 4 — label and review-record validation

Implement universal output label schema, exact graph-type/key/attribute checks, decisive-evidence maps, Negative closed-scope gates, ambiguity/cardinality, reviewer independence/visibility, conflict retention, and versioned correction records.

Durable reviewer authorization/unsealing remains E5-B.

Tests: `CAL-LABEL-*`.

## Phase 5 — calibration-pack candidate validation

Reuse the E2-B parser/compiler and add E5-A trust/rollout, named-metadata flow audit, universal-output, convention-literal, rule-evidence, corpus/split/mutation binding, generalization-scope, and identity/version checks.

No second operator language or graph registry mutation.

Tests: `CAL-PACK-*`, metadata-injection security cases.

## Phase 6 — shadow execution and case results

Run exact visible corpus examples through E2-B, build immutable candidate-owned shadow partitions, independently validate graph proposals, classify each expected/observed item, and produce canonical per-case artifacts.

No default graph publication or service/CLI.

Tests: `CAL-RUN-*`, `CAL-GRAPH-*`, partial/cancellation cases.

## Phase 7 — mutation and anti-overfitting engine

Implement exact before/after fact-snapshot binding, invariance/sensitivity expectations, near-miss coverage, named-condition static audit, copy/fork/vendor influence reports, adversarial resource/security cases, and deterministic result comparison.

Tests: `CAL-MUT-*`, `CAL-SEC-*`, `CAL-DET-*`.

## Phase 8 — metrics and evaluation gates

Implement per-case-first aggregation, explicit denominators/exclusions, per-rule/role/split/provenance/shape reports, hard-gate evaluation, frozen quantitative thresholds, baselines, contamination history, and honest generalization conclusions.

Tests: `CAL-METRIC-*`, comparison cases.

## Phase 9 — candidate artifact and deactivation

Implement immutable candidate-artifact assembly and partition-local deactivation validation with stale-reference closure, exact coverage downgrade, foreign/core digest invariance, privacy/license tombstones, and reproducibility retention.

Tests: `CAL-PART-*`.

## Phase 10 — determinism, resource benchmarks, and freeze

Run:

```text
1/2/N workers
shuffled facts/examples/evidence/partitions
cold/warm caches
different host/temp roots
independent materialization histories reaching identical logical artifacts
ordinary and adversarial resource profiles
cancellation at every phase
```

Freeze accepted implementation/profile/report IDs, thresholds, expected bytes, member SHA-256 values, bundle digest, and benchmark report IDs. Tests verify committed fixtures and never rewrite them automatically.

## E5-B handoff

Freeze the smallest orchestration seam for:

```text
exact corpus/pack/run selection
retained artifact acquisition
durable run/idempotency/response-loss handling
reviewer authorization and sealed-holdout unsealing audit
promotion submission preparation
canonical service result envelopes
```

`wow-service` owns that orchestration. It must not reproduce recognizer, split, metric, or graph-validation algorithms.

## Deferred

- default/core-pack publication, signing, canary, rollout, rollback, last-known-good and published partition migration (E5-C);
- source/project indexing and graph publication;
- runtime WoW probes or claims;
- source edits or migration application;
- model/embedding/Codebase Memory labeling or tuning;
- CLI/LSP/MCP/release/CI.
