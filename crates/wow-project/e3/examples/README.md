# E3-B normative source-universe fixtures

- `source-profile.json` — provider, build-binding, root, parser/analyzer/graph/store and policy shape.
- `source-snapshot.json` — sealed materialized roots/files/content/license/security manifest.
- `source-index-candidate.json` — source inventory, package/load/analyzer/recognizer/graph/bridge candidate closure.
- `universe-graph-proposals.json` — source-universe entities/relations and anti-merge mutations.
- `bridge-cases.json` — exact source/reference resolution, ambiguity, authority and deferred project bridges.
- `publication-cases.json` — inactive build, read-back validation, CAS, readers, recovery and selector isolation.
- `invalidation-cases.json` — file/profile/tool/reference/license changes, reuse and removal closure.
- `license-redistribution-cases.json` — local-only, notices, excerpts, derived artifacts and release blocks.
- `CHECKSUMS.json` — prerequisite, real-source, profile, vector, member and bundle freeze gate.

Fixture source code is synthetic by default. The first real source fixture is recorded as exact provider/revision/content/build/license manifests and expected generated outputs; raw real-source bytes are not committed unless an explicit positive redistribution decision permits the minimum required excerpt.
