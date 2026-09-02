# Documentation index

English documentation is the canonical navigation surface.

```text
documentation frontier: E6-A complete
next documentation package: E6-B external-candidate service/mapping/context/CLI
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

- [E5-A calibration owner](../crates/wow-recognizers/e5/README.md)
- [E5-B review/holdout/submission](../crates/wow-service/e5/README.md)
- [E5-C core-pack publication lifecycle](../crates/wow-service/e5c/README.md)
- [E6-A external semantic-candidate bridge](../crates/wow-cbm/e6/README.md)

## E6-A authority boundary

```text
reviewed descriptor + exact external state + bounded query
-> allow-listed provider response
-> loss-preserving normalization
-> semantic_candidate + Candidate
-> UnverifiedProviderLocator
```

Top rank, sole result, provider labels, stable generation, high score, repeated result, or zero result never creates exact source/project/reference/graph/search/runtime authority. Provider-local failures remain optional and lane-local.

## Next package

E6-B must define configured provider/session/credential-port acquisition, durable operation/idempotency/response-loss state, exact result/artifact catalogs and retention, project/reference owner mapping, explicit candidate selection receipts, exact mapped-root context handoff, conservative envelopes, privacy/license/security, cancellation/closure, and thin CLI.

It must not widen E6-A Candidate authority, compare provider scores as confidence, choose top/sole candidates, treat mapping as provider truth, expose credentials/private endpoints/provider cursors, or make exact local workflows depend on external availability.

Patch-sensitive WoW facts remain in the separate [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb).