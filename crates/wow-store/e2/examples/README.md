# E2-D normative fixture shapes

- `runtime-profile.json` — SQLite/binding/platform/WAL/open/limit/checkpoint profile.
- `schema-set.json` — owner-separated store/project/graph schema and operation catalogs.
- `publication-plan.json` — baseline and incremental inactive-build/validate/activate plan.
- `generation-manifest.json` — noncyclic semantic publication, membership, store generation, and current record.
- `crash-recovery-cases.json` — crash/cancel/retry/current/inactive classifications.
- `read-lease-cases.json` — old/new/exact readers and GC/checkpoint interactions.
- `retention-gc-cases.json` — generation, partition, object, and epoch reachability.
- `benchmark-cases.json` — model candidates, corpora, workloads, metrics, and pending thresholds.
- `CHECKSUMS.json` — prerequisite/profile/vector/member freeze gate.

Implementation-dependent pins, IDs, expected results, and SHA-256 values remain null only while `implementation_state` is `not-started`.
