# E4-C service fixtures

These JSON files are closed documentation fixtures. All implementation-dependent IDs, canonical bytes and SHA-256 values remain `null` only while `implementation_state = not-started`.

- `search-requests.json` — shard build/query/explain/select/continue and honest-miss service vectors.
- `lineage-review-requests.json` — before/after acquisition, producer, ambiguity, authorization, review and immutable publication vectors.
- `migration-impact-context-requests.json` — migration validation, static impact and explicit search-to-context handoff vectors.
- `result-and-lifecycle-cases.json` — status, validation, idempotency, response-loss, retention, close and cancellation vectors.
- `CHECKSUMS.json` — prerequisite, port, profile, corpus, canonical-byte and member freeze gate.

Fixtures never contain private review keys, access tokens, credentials or real private source. Review authentication uses synthetic public test identities and deterministic test signatures only after the exact adapter profile is selected.
