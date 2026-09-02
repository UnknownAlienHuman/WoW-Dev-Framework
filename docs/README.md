# Documentation index

English documentation is the canonical navigation surface.

```text
documentation frontier: E5-B complete
next documentation package: E5-C core-pack publication/signing/canary/rollout/rollback
implementation frontier: not started
```

## Read first

1. [Project vision](PROJECT_VISION.md)
2. [Architecture](ARCHITECTURE.md)
3. [Provenance, confidence, and coverage](PROVENANCE_AND_COVERAGE.md)
4. [Architecture decisions](DECISIONS.md)
5. [Roadmap](ROADMAP.md)
6. [Machine manifest](../crates/MANIFEST.json)
7. [Dependency graph](../crates/DEPENDENCY_GRAPH.md)
8. [Workstreams](../crates/WORKSTREAMS.md)

## Active routes

- [E3-B context engine](../crates/wow-context/e3/README.md)
- [E3-C context service](../crates/wow-service/e3/README.md)
- [E4-A exact-generation search](../crates/wow-search/e4/README.md)
- [E4-B lineage/migration/static impact](../crates/wow-graph/e4/README.md)
- [E4-C search/lineage/impact service](../crates/wow-service/e4/README.md)
- [E5-A calibration owner](../crates/wow-recognizers/e5/README.md)
- [E5-B durable calibration orchestration](../crates/wow-service/e5/README.md)
- [E5-B CLI](../apps/wow/e5/README.md)

## E5 authority boundary

```text
E5-A: exact calibration artifacts and shadow evaluation
E5-B: durable orchestration, independent review, sealed holdout, submission
E5-C: independent publication/signing/canary/activation/rollout/rollback
```

Metrics, graph validity, review authorization, holdout authorization, disclosure, submission, publication, activation, and runtime correctness never imply each other. Response loss is not proof of no effect. A consumed or contamination-unknown holdout is never called untouched.

## Next package

E5-C must independently revalidate one exact E5-B `PromotionSubmission`, create a distinct immutable core-pack artifact/catalog entry, define detached signing/provenance/SBOM/license attestations without committed private keys, publish inactive and read back, define exact canary cohorts and observations, use guarded current/default activation, stage rollout, retain explicit last-known-good, and perform exact rollback/revocation/stale-partition closure. It cannot rewrite E5-A/B evidence or infer global runtime correctness.

Patch-sensitive WoW facts remain in the separate [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb); stable framework docs link rather than duplicate them.