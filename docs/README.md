# Documentation index

English documentation in this directory is the canonical navigation surface for WoW Dev Framework.

```text
documentation frontier: E5-A complete
next documentation package: E5-B calibration orchestration, review, holdout audit and promotion submissions
implementation frontier: not started
```

## Document classes

| Class | Meaning |
|---|---|
| **Normative** | Defines behavior, public contracts, invariants, or accepted decisions. Implementation must conform. |
| **Operational** | Defines how contributors, agents, tests, and releases apply the normative contracts. |
| **Research** | Pins external inputs and records verified observations. It is not automatically a product contract. |
| **Candidate** | Contains ideas that require an experiment or ADR before implementation. |
| **Archive** | Retired routing tombstones. Superseded source content is available only through Git history and does not control current routing. |

## Read first

1. [Project vision and boundaries](PROJECT_VISION.md) — normative scope and non-goals.
2. [Architecture](ARCHITECTURE.md) — normative system design.
3. [Provenance, confidence, and coverage](PROVENANCE_AND_COVERAGE.md) — normative truth and negative-answer contract.
4. [Architecture decisions](DECISIONS.md) — accepted constraints.
5. [Roadmap](ROADMAP.md) — active documentation and implementation gate.
6. [Crate manifest](../crates/MANIFEST.json) — machine-readable frontier and active contract routes.
7. [Crate workstreams](../crates/WORKSTREAMS.md) — exact dependency/order and next package.

## Component contracts

- [Reference Pack](REFERENCE_PACK.md) — immutable Blizzard reference artifact and builder contract.
- [EmmyLua and diagnostics](EMMYLUA_AND_DIAGNOSTICS.md) — upstream integration, project generations, and rule execution.
- [Graph, search, and planning](GRAPH_SEARCH_AND_PLANNING.md) — typed graph, lineage, skeletons, search, and patch impact.
- [E4-A exact-generation search](../crates/wow-search/e4/README.md) — immutable SearchShards, safe query lanes, deterministic ranking, explanations, misses, and continuation.
- [E4-B lineage, migration records, and static impact](../crates/wow-graph/e4/README.md) — exact before/after generations, producer partitions, proof ceilings, ambiguity, review, change classes, migration candidates, and bounded reason paths.
- [E4-C search/lineage/impact service](../crates/wow-service/e4/README.md) — exact/current acquisition, shard and lineage orchestration, explicit candidate selection, review authorization, migration validation, static impact, and context handoff.
- [E4-C CLI](../apps/wow/e4/README.md) — thin `wow-service`-only command, input, output, cancellation, and exit-code contracts.
- [E5-A calibration corpora and named packs](../crates/wow-recognizers/e5/README.md) — exact candidate-source admission, provenance-aware splits, independent labels, shadow-only packs, anti-overfitting mutations, evaluation, candidate artifacts, and deactivation.
- [Codebase Memory bridge](CODEBASE_MEMORY_BRIDGE.md) — optional broad-source candidate bridge.
- [Secret Values and restrictions](SECRET_VALUES_AND_RESTRICTIONS.md) — open restriction facets and analysis levels.

## E4 authority boundary

```text
E4-A wow-search
    retrieves and ranks exact entities relative to a query
    approximate signals remain candidate evidence

E4-B wow-graph
    validates exact cross-generation proposals under proof ceilings
    publishes immutable lineage/change/migration/static-impact records

E4-C wow-service + apps/wow
    resolves outer selectors, acquires exact artifacts, invokes owners,
    requires explicit candidate selection, validates review authorization,
    preserves status/coverage/conflicts and exposes thin transport operations
```

Search does not prove lineage or intended entity. Review authorization does not create proof. Same lineage does not imply replacement. Migration validation does not edit source. Static impact does not establish runtime breakage or severity.

## E5 calibration boundary

```text
E5-A wow-recognizers
    validates exact immutable corpus/label/split/pack artifacts
    executes E2-B packs in shadow-only candidate-owned partitions
    produces per-case/mutation/metric/candidate/deactivation artifacts

E5-B wow-service + apps/wow (next)
    will acquire retained artifacts, authorize reviewers,
    audit sealed-holdout access, and prepare promotion submissions

E5-C publication owner (later)
    will publish immutable core packs and own canary/rollout/rollback
```

A repository commit pin is not corpus admission. Repository/addon/owner/path/popularity/label/split/reviewer/model metadata cannot control matcher semantics. `ShadowValidated` or metric eligibility is not authorization or publication. Unknown/Possible/NotEvaluated/Conflict/Partial/Truncated are not Negative or pass.

## Operating documents

- [Agent workflow](AGENT_WORKFLOW.md)
- [Security model](SECURITY_MODEL.md)
- [Test strategy](TEST_STRATEGY.md)
- [Contributing](../CONTRIBUTING.md)
- [Repository agent instructions](../AGENTS.md)

## Research and planning

- [Research baseline](RESEARCH_BASELINE.md)
- [Candidate ideas](IDEAS.md)
- [Glossary](GLOSSARY.md)

## Next documentation package

E5-B is owned by `wow-service`, with a thin `apps/wow` transport. It must define exact retained calibration artifact acquisition, durable operation identity and response-loss recovery, reviewer authorization independent from metrics/graph validity, sealed-holdout unsealing audit, promotion submission preparation, conservative result envelopes, cancellation/closure, and command-specific transport behavior.

E5-B must invoke E5-A owner operations rather than reproduce corpus, split, matcher, mutation, metric, graph-validation, or deactivation algorithms. It does not publish a core pack; E5-C remains the publication/canary/rollback owner.

## Archive

- [Retired v8.0 architecture source tombstone](archive/WOW_EMMY_COGNITIVE_PLATFORM_ARCHITECTURE_V8_0_RU.md)

The original v8.0 source was retired from the working tree. Use Git history only for architecture archaeology; the English contracts in this directory are the maintained repository surface.

## Update rules

- A changed public concept must update the glossary and its owning contract.
- A changed architectural invariant must update `DECISIONS.md` through an ADR.
- A changed milestone gate must update `ROADMAP.md`, `crates/MANIFEST.json`, and `crates/WORKSTREAMS.md`.
- A new document must be added to this index or its owning crate router.
- Do not duplicate live WoW patch/security knowledge from the external knowledge base. Link to it and pin only the exact release input required here.
