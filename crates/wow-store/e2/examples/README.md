# E2-D normative fixture shapes

- `runtime-profile.json` — SQLite/binding/platform/WAL/open/limit/checkpoint profile.
- `schema-set.json` — owner-separated store/project/graph schema and operation catalogs.
- `publication-plan.json` — baseline and incremental inactive-build/validate/activate plan.
- `generation-manifest.json` — noncyclic semantic publication, membership, store generation, and current record.
- `crash-recovery-cases.json` — partition/generation/activation crash, idempotency, response-loss, recovery, and backup classifications.
- `read-lease-cases.json` — old/new/exact readers, lease-admission races, semantic continuations, and checkpoint interactions.
- `retention-gc-cases.json` — exact roots, generation/partition/object/operation reachability, stale-plan races, and Windows sharing cases.
- `benchmark-cases.json` — selected/rejected physical models, corpora, workloads, metrics, and pending thresholds.
- `CHECKSUMS.json` — prerequisite/profile/vector/member freeze gate for the consolidated v2 contract.

Implementation-dependent pins, IDs, expected results, and SHA-256 values remain null only while `implementation_state` is `not-started`.

Superseded whole-SQLite-generation-image fixtures are intentionally absent from the current set. Their historical rationale is summarized in [`../REJECTED_ALTERNATIVES.md`](../REJECTED_ALTERNATIVES.md).
