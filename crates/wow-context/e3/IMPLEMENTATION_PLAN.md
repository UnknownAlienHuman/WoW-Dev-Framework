# E3-B implementation plan

**Status:** normative order; implementation has not started.

## Phase 0 — prerequisites and freeze

Before any E3-B Rust file or `Cargo.toml`:

- implement and freeze `wow-core`, persistent `wow-reference`, E2 graph/project/store publication, and E3-A Blizzard UI source-index contracts required by active profiles;
- freeze exact project/graph/reference/source-slice/SkeletonInputView read catalogs;
- freeze map, L0, L1, control/effect, intent, expansion, selection, continuation, budget, tokenizer/estimator, source, privacy/license, boundary, semantic-pack, renderer, cache-key, metrics, evaluation, error, and canonicalization profiles;
- freeze synthetic, high-fanout/cyclic, pinned `roth-ui`, pinned Blizzard UI, combined-universe, and adversarial source corpora;
- populate every fixture ID, expected byte/token vector, benchmark/evaluation threshold, and SHA-256 manifest;
- verify no retired E3-A context type/operation is activated.

## Phase 1 — narrow input interfaces

Implement exact traits/adapters over public owner views only:

```text
ProjectContextRead
GraphContextRead
ReferenceContextRead
SourceSliceRead
```

No raw store/analyzer/parser handles. Prove generation binding, cancellation, bounds, record validation, and no current refresh.

## Phase 2 — profiles and universe binding

Implement closed profile schemas, canonicalization, request/root validation, `ContextUniverseSet`, compatibility reports, and exact identity DAG foundations.

Tests: universe/generation/profile/cross-universe/floating-current/unknown-field mutations.

## Phase 3 — Project Map

Implement map node/edge/group/facet projections, direct relation/path distinction, deterministic ordering, pagination, omissions, and validation.

Tests: grouping/role/name/path mutations, partial coverage, cycles, budget pages, 1/2/N order.

## Phase 4 — L0 and L1

Implement container L0, entity/local-neighborhood L1, exact signatures/types/spans, signal/hook/state/API facets, and source excerpt candidates.

No bodies by default and no second parser/CFG/SSA/data-flow.

## Phase 5 — control/effect projection

Implement only the closed published-fact registry, unknown/collapsed/omitted regions, reason paths, and exact origin closure.

Tests include native/custom/CVar/hook distinctions and runtime/safety nonclaims.

## Phase 6 — expansion and selection

Implement deterministic stages, candidate dependency DAG, deduplication, mandatory closure, optional priority tiers, no-new-evidence stopping, cancellation, and continuation.

No search/ranking/model inference.

## Phase 7 — source, privacy, and boundaries

Implement exact source slice requests, privacy/license/consumer decisions, deterministic transformations, JSON strings, Markdown `SRC` data boundaries, and boundary round-trip validation.

## Phase 8 — budgets and tokenization

Implement exact canonical/rendered byte accounting, deterministic predicted costs, optional pruning/rollback, exact tokenizer interface, deterministic estimate/upper-bound fallbacks, and overflow failures.

## Phase 9 — semantic pack and renderers

Assemble/validate `ContextSemanticPack`; implement lossless canonical JSON and deterministic Markdown with item/range trace. Rendering cannot change selection/facts.

## Phase 10 — cache identity and comparison

Implement cache keys/artifact validation only, plus semantic/renderer comparison. Physical cache storage remains outside the crate.

## Phase 11 — metrics and evaluation

Implement noncanonical operational metrics and frozen corpus/task evaluation reports. Missing external consumer/tokenizer/harness remains `NotEvaluated`.

## Phase 12 — integration and freeze verification

Run:

- all contract/unit/property/mutation/security/cancellation tests;
- 1/2/N and shuffled owner-result rebuilds;
- cold/warm external-cache validation;
- old/new generation reader tests through owner views;
- canonical JSON round trip and Markdown source-boundary parser;
- synthetic/`roth-ui`/Blizzard UI/combined corpus evaluation;
- exact checksum verification.

## Implementation slicing

Prefer independently testable modules:

```text
profiles
universe
map
l0
l1
control_effect
expansion
selection
source_boundary
budget
semantic_pack
render_json
render_markdown
cache_key
metrics_eval
validation
```

Do not create placeholder modules for deferred E4/E5/E6/E7 work.

## Deferred

- natural-language/fuzzy root resolution and search;
- lineage/migration/impact;
- named calibration packs;
- Codebase Memory candidates;
- runtime data/probes;
- diagnostics/remediation/editing;
- physical context cache/persistence;
- service/application/transport/tool orchestration;
- CI/release automation.
