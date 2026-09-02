# Documentation index

English documentation is the canonical navigation surface.

```text
documentation frontier: E6-B complete
next documentation package: E7-A transport/session and developer-preview release boundary
implementation frontier: not started
```

## Read first

1. [Project vision](PROJECT_VISION.md)
2. [Architecture](ARCHITECTURE.md)
3. [Provenance, confidence, and coverage](PROVENANCE_AND_COVERAGE.md)
4. [Architecture decisions](DECISIONS.md)
5. [Roadmap](ROADMAP.md)
6. [Launch gates](LAUNCH_GATES.md)
7. [Machine manifest](../crates/MANIFEST.json)
8. [Dependency graph](../crates/DEPENDENCY_GRAPH.md)
9. [Workstreams](../crates/WORKSTREAMS.md)

## Active routes

- [E4-A exact-generation search](../crates/wow-search/e4/README.md)
- [E4-B lineage/migration/static impact](../crates/wow-graph/e4/README.md)
- [E4-C service/CLI](../crates/wow-service/e4/README.md)
- [E5-A calibration owner](../crates/wow-recognizers/e5/README.md)
- [E5-B calibration orchestration/review/holdout/submission](../crates/wow-service/e5/README.md)
- [E5-C core-pack publication lifecycle](../crates/wow-service/e5c/README.md)
- [E6-A external semantic-candidate bridge](../crates/wow-cbm/e6/README.md)
- [E6-B external-candidate orchestration/mapping/context](../crates/wow-service/e6/README.md)
- [E6-B CLI](../apps/wow/e6/README.md)

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

## E6 authority and handoff

```text
reviewed descriptor + exact external state + bounded query
-> E6-A semantic_candidate + Candidate
-> UnverifiedProviderLocator
-> E6-B exact project/reference owner mapping
-> explicit caller selection
-> exact mapped root to normal context owner
-> external Candidate sidecar kept separate
```

Top rank, sole result, provider labels, stable generation, high score, repeated result, zero result, mapping, selection, or context inclusion never verifies provider interpretation. Provider-local failures remain optional and lane-local.

## Next package

E7-A must define concrete supported CLI-daemon/LSP/MCP session and transport contracts around existing `wow-service` operations:

- capability negotiation and schema/version pinning;
- project/profile/session registration;
- one transport call to one service operation;
- bounded request/response/streaming/progress/cancellation/backpressure;
- reconnect, response loss, lease/retention, and close semantics;
- no generic shell/tool/RPC escape hatch;
- security/privacy/credential and multi-client isolation;
- developer-preview packaging and compatibility manifest boundary.

Public installers, update channels, artifact signing/distribution, rollback/retirement, and long-term support remain E7-B.

Patch-sensitive WoW facts remain in the separate [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb); stable contracts link rather than duplicate them.