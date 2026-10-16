# wow search select

## Command

```text
wow search select \
  --selection-json <wow-service/search-selection-input/1 JSON> \
  --search-store <EXISTING-STORE> \
  --operation-id <ID> \
  --allow-local-metadata-only \
  --max-output-bytes <positive-u64> \
  --store-max-object-bytes <positive-u64> \
  --store-max-batch-objects <positive-u32> \
  --store-max-catalog-mutations <positive-u32> \
  --store-max-manifest-records <positive-u32> \
  --store-max-gc-deletes <positive-u32>
```

## Input schema

Schema: `wow-service/search-selection-input/1`

| Field | Type | Description |
|-------|------|-------------|
| `explanation_operation_id` | string | Exact original explanation operation ID |
| `explanation_request_digest` | string | SHA-256 of canonical original explanation request |
| `explanation_result_id` | string | Exact original explanation result ID |
| `result_set_id` | string | Exact original result set ID |
| `candidate_id` | string | Exact original candidate ID |
| `expected_explanation_digest` | string | SHA-256 of canonical entire owner explanation |
| `expected_candidate_digest` | string | SHA-256 of canonical entire owner candidate |
| `entity_kind` | string | `project_graph_entity` \| `reference_document` \| `source_observation` |
| `expected_entity_id` | string | Exact expected entity ID |
| `selection_origin` | string | Always `explicit_caller_selection` |
| `target_action` | string | Always `receipt_only` |

## Semantics

- Consumes exact completed owner-validated journal output.
- Preserves original status and authority.
- Emits inert explicit selection receipt.
- No query/source rerun.
- No context execution.
- No native owner lease.
- No fake context native token or authority from Source facts.
- Actual context admission remains separate.

`expected_explanation_digest` hashes the complete original owner explanation as
canonical JSON. It guards every original universe/owner scope, shard, query,
profile, ranking/status and support field together; the candidate digest is an
additional exact row guard. Digests use `sha256:` followed by 64 lowercase hex
digits. The result ID is the original completed explanation's Service result ID,
not a rank or display name. All other fields, modes, current/query selectors,
Search bounds, leases and unregistered origins/attestations are rejected.

The receipt retains exact original explanation/result/candidate references,
native search binding IDs, original rank tuple, explicit owner entity and caller
origin. It does not attest a human principal or authorize an edit/tool. A Source
fact and a Reference document remain their original owner entity classes. Native
Context root admission is never manufactured from their labels or shapes.

Receipt-only selection can address a historical completed explanation after its
Search shard has been retired/collected. Its original journal object remains
immutable and retained by the completed operation; selection does not reopen a
live owner or assert current membership. The output limit may be smaller than
the original explanation's stored-object limit. Completed retry returns exact
recorded bytes; a changed request under the same operation ID fails. A missing
Store is never created. The separate `search_context` composition must reacquire
exact compatible E3 owners before any Context success.

## Service call

Exactly one: `service.select_candidate(&request, stop)`.

## Exit codes

| Code | Condition |
|------|-----------|
| 0 | Original owner status maps to success |
| 2 | CandidateOnly / Partial / ConflictBlocked / Truncated / NotEvaluated |
| 3 | Service error (identity mismatch, budget exceeded, etc.) |
| 4 | I/O, internal or output failure |
| 64 | Invalid CLI/schema/closed option or origin |
| 130 | Cancelled |
