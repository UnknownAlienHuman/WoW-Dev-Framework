# Documentation index

English documentation is the canonical navigation surface.

```text
documentation frontier: E5-C complete
next documentation package: E6-A optional external semantic-candidate bridge
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

- [E4-A exact-generation search](../crates/wow-search/e4/README.md)
- [E4-B lineage/migration/static impact](../crates/wow-graph/e4/README.md)
- [E4-C service/CLI](../crates/wow-service/e4/README.md)
- [E5-A calibration owner](../crates/wow-recognizers/e5/README.md)
- [E5-B calibration orchestration/review/holdout/submission](../crates/wow-service/e5/README.md)
- [E5-B CLI](../apps/wow/e5/README.md)
- [E5-C core-pack publication lifecycle](../crates/wow-service/e5c/README.md)
- [E5-C CLI](../apps/wow/e5c/README.md)

## E5 lifecycle

```text
PromotionSubmission
-> independent E5-C revalidation
-> distinct immutable CorePackArtifact
-> attestations + detached signatures
-> PublishedInactive
-> fresh read-back validation
-> exact scoped canary
-> finite rollout
-> profile-specific guarded current activation
-> explicit last-known-good
-> exact rollback/revocation/deactivation/partition closure
```

None of submission, signature, inactive publication, canary pass, rollout stage, active pointer, or rollback proves global runtime correctness. E5-C internal catalog publication is not public distribution.

## Next package

E6-A must define an optional degradable `wow-cbm` bridge over exact provider descriptors/generations and bounded queries, normalize all external results as `semantic_candidate + Candidate`, preserve provider-local scores/coverage/failure, prohibit zero-result negative authority and provider-database mutation, and expose only unverified source locators for a later E6-B owner-mapping/service seam.

Patch-sensitive WoW facts remain in the separate [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb); stable contracts link rather than duplicate them.