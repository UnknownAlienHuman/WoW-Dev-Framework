# E3-A normative fixtures

- `source-profile.json` — exact provider/materializer/client/profile/security/license configuration shape.
- `source-snapshot.json` — closed configured-root inventory and content/provenance manifest shape.
- `package-load-model.json` — synthetic package, TOC variant, XML/script, Lua-unit, and direct-load model.
- `analyzer-graph-handoff.json` — exact analyzer/fact/recognizer/producer-partition/graph-validation closure.
- `update-cases.json` — exact reuse/invalidation/removal/no-change/fingerprint vectors.
- `publication-and-skeleton-input.json` — E2-D publication and bounded context-input read vectors.
- `CHECKSUMS.json` — prerequisite, provider, profile, vector, member, and bundle freeze gate.

All implementation-dependent IDs, exact provider revisions, expected generated manifests, benchmark reports, and SHA-256 values may remain null only while `implementation_state` is `not-started`.
