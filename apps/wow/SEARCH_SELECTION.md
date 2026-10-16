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

## Service call

Exactly one: `service.select_candidate(&request, stop)`.

## Exit codes

| Code | Condition |
|------|-----------|
| 0 | Valid receipt |
| 1 | Invalid |
| 2 | CandidateOnly / Partial / ConflictBlocked / Truncated / NotEvaluated |
| 3 | Service error (identity mismatch, budget exceeded, etc.) |
| 130 | Cancelled |
