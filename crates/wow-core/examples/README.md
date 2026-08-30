# `wow-core` E0 examples

These files are strict internal contract fixtures for the first implementation wave. They are not released Reference Packs and do not claim live-client validation.

| File | Contract |
|---|---|
| `e0-clean-result.json` | All E0 capabilities complete; exact coverage records and derived summaries reconcile; no findings; status remains `complete`. |
| `e0-findings-result.json` | Generic, missing-API, and direct Secret-local fixture findings under one context. Project-use handles/evidence remain separate from Reference Pack contract evidence. |
| `e0-not-evaluated-result.json` | Unknown restriction capability produces an exact blocking coverage record, a typed `NotEvaluated` record, and `partial` status. |
| `e0-conflict-not-evaluated-result.json` | Source coverage is complete, but an unresolved evidence conflict is retained, attached to coverage/summary state, and blocks rule evaluation/negative authority. |
| `e0-generation-mismatch-error.json` | Context mismatch is an operation error, not a finding, warning, or `NotEvaluated` record. |
| `HASH_VECTORS.json` | Exact domain-separated canonical JSON text and SHA-256 values for IDs, records, and the clean envelope. |

Rules:

- JSON files are pretty-printed for repository review; `canonical_utf8` values and result digests use compact canonical JSON described in `../CANONICALIZATION.md`.
- `budget.usage.output_bytes` is the compact final canonical envelope byte count, not this repository file's pretty-printed byte size.
- Exact `coverage_records` remain in every result; `capability_summaries` are derived views and must reference those records.
- Evidence describing a project source use and evidence describing a platform/reference contract use different source handles even when one finding cites both.
- The canonical reference graph is acyclic: handles → evidence → conflicts → coverage → summaries/`NotEvaluated` → findings/warnings → envelope.
- All profile/source values are synthetic and deterministic.
- Changing an identity field requires regenerating dependent IDs and hash vectors.
- A coding agent must prove tests fail after mutating at least one expected digest, reference, status, or canonical field.
