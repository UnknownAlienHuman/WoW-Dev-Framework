# `wow-reference` E0-B test matrix

**Status:** normative executable acceptance matrix for the future implementation.

A test passes only when it proves the target path executed and compares structured data, not only human-readable messages.

## 1. Test ID rules

- IDs are stable and unique.
- Removed tests remain documented as retired rather than silently reused for different behavior.
- Every fixture case names profile, reference generation, variant, capability/partition expectations, and exact outcome.
- Randomized/property tests record the seed on failure but seed/order do not enter canonical output.

## 2. Profile identity

| ID | Case | Expected |
|---|---|---|
| `REF-PROFILE-001` | Validate the declared E0 fixture profile | accepted as `fixture`, `release_eligible=false` |
| `REF-PROFILE-002` | Missing Interface | `fixture_profile_invalid` |
| `REF-PROFILE-003` | Floating source revision `main` | `fixture_profile_floating_identity` |
| `REF-PROFILE-004` | Mark fixture as release eligible | `fixture_profile_release_masquerade` |
| `REF-PROFILE-005` | Reference generation differs from canonical fixture identity | `reference_generation_mismatch` |
| `REF-PROFILE-006` | Use fixture where release profile required | typed rejection, no fallback |
| `REF-PROFILE-007` | Two otherwise identical profiles differ by profile kind | identities remain distinct |

## 3. Input inventory

| ID | Case | Expected |
|---|---|---|
| `REF-INPUT-001` | Declared complete inventory | canonical inventory accepted |
| `REF-INPUT-002` | Input discovery order shuffled | same canonical inventory bytes/digest |
| `REF-INPUT-003` | Declared semantic order changed | deterministic but different inventory/model identity where order is semantic |
| `REF-INPUT-004` | Missing declared registration input | exact affected partitions degraded or model rejected per variant |
| `REF-INPUT-005` | Undeclared input supplied | `undeclared_input` |
| `REF-INPUT-006` | Digest mismatch | `input_digest_mismatch` |
| `REF-INPUT-007` | Byte length mismatch | `input_length_mismatch` |
| `REF-INPUT-008` | `../` traversal path | `input_path_invalid` |
| `REF-INPUT-009` | absolute Windows/POSIX/UNC path | `input_path_invalid` |
| `REF-INPUT-010` | duplicate input ID | `input_inventory_invalid` |
| `REF-INPUT-011` | duplicate logical path with incompatible digest | classified conflict/error; no arbitrary winner |

## 4. Raw canonical values

| ID | Case | Expected |
|---|---|---|
| `REF-RAW-001` | Valid null/bool/integer/string/array/object values | accepted and round-trip |
| `REF-RAW-002` | Object key order permuted | identical canonical bytes |
| `REF-RAW-003` | Array order changed | semantic/canonical bytes change |
| `REF-RAW-004` | Duplicate object key | `raw_value_invalid` |
| `REF-RAW-005` | Excess depth/entries/bytes | `evaluator_budget_exceeded` or `raw_value_invalid` with exact limit |
| `REF-RAW-006` | NaN/infinity/ambiguous decimal | rejected |
| `REF-RAW-007` | Function/userdata/runtime value | rejected; never executed |

## 5. Restricted evaluator

| ID | Case | Expected |
|---|---|---|
| `REF-EVAL-001` | Closed allow-listed fixture registration records | accepted raw canonical records |
| `REF-EVAL-002` | Literal/table/local binding forms | accepted within limits |
| `REF-EVAL-003` | Arbitrary function call | `unsupported_declarative_construct` |
| `REF-EVAL-004` | IO/environment/module loading request | `dynamic_execution_forbidden` |
| `REF-EVAL-005` | Loop or recursion | rejected without execution |
| `REF-EVAL-006` | Unknown registration target | `registration_target_unknown` |
| `REF-EVAL-007` | Invalid registration shape | `registration_shape_invalid` |
| `REF-EVAL-008` | One bad isolated record among valid records | valid unrelated partition retained; affected partition explicit |
| `REF-EVAL-009` | Budget exhausted mid-partition | partial/failed coverage; no clean authority |
| `REF-EVAL-010` | Comments contain agent instructions | treated as source text only, no behavioral effect |

## 6. Unknown-field preservation

| ID | Case | Expected |
|---|---|---|
| `REF-UNKNOWN-001` | Unknown nested field in raw function record | exact raw value/path retained |
| `REF-UNKNOWN-002` | Safe projection-only unknown | typed fact usable, notice retained |
| `REF-UNKNOWN-003` | Capability-blocking unknown | dependent coverage partial/unknown |
| `REF-UNKNOWN-004` | Unknown field silently removed mutation | test fails |
| `REF-UNKNOWN-005` | Unknown field attached to wrong source/context | validation fails |
| `REF-UNKNOWN-006` | Field classifier omits affected capability | `invalid_unknown_field_classification` |

## 7. Lowering

| ID | Case | Expected |
|---|---|---|
| `REF-LOWER-001` | Lower `C_E0Fixture` system | exact entity/raw/source/evidence links |
| `REF-LOWER-002` | Lower `KnownApi` signature | one string argument, one boolean return |
| `REF-LOWER-003` | Lower `SecretText` signature | no arguments, one string return |
| `REF-LOWER-004` | Lower `secret.return` | subject `SecretText`, return position 1, fixture applicability |
| `REF-LOWER-005` | Function references missing system | `lowering_contract_invalid` |
| `REF-LOWER-006` | Return position duplicate/non-contiguous | validation failure |
| `REF-LOWER-007` | Lowered fact points to missing raw/evidence record | `lowered_fact_reference_invalid` |
| `REF-LOWER-008` | Unsupported facet shape | raw preserved, facet capability degraded |
| `REF-LOWER-009` | Attempt to infer alias/replacement | no such output; test fails if created |

## 8. Duplicates and conflicts

| ID | Case | Expected |
|---|---|---|
| `REF-DUP-001` | One observation | `unique` |
| `REF-DUP-002` | Equivalent duplicate records in reversed order | one normalized fact, all provenance retained, deterministic |
| `REF-DUP-003` | Incompatible signature duplicate | explicit conflict, no winner |
| `REF-DUP-004` | Incompatible Secret facet duplicate | symbol may remain found; facet lookup conflict |
| `REF-DUP-005` | Implementation chooses first/last observation | mutation test fails with `conflict_winner_forbidden` |
| `REF-DUP-006` | Conflict has one evidence record | `conflict_record_invalid` |
| `REF-DUP-007` | Conflict evidence from another generation | validation fails |

## 9. Coverage records

| ID | Case | Expected |
|---|---|---|
| `REF-COVER-001` | Complete variant | exact symbol/facet records `Complete` |
| `REF-COVER-002` | Partial symbol variant | symbol-domain record `Partial` with blocker |
| `REF-COVER-003` | Conflict facet variant | source coverage may be `Complete`; conflict remains separate blocker |
| `REF-COVER-004` | Missing inventory capability | dependent records partial/failed; unrelated partition unchanged |
| `REF-COVER-005` | Coverage references wrong producer/context | `coverage_record_invalid` |
| `REF-COVER-006` | Broad profile coverage substituted for system partition | `coverage_partition_selection_invalid` |
| `REF-COVER-007` | Local boolean bypasses core authority decision | mutation test fails |

## 10. Model assembly

| ID | Case | Expected |
|---|---|---|
| `REF-MODEL-001` | Assemble complete model | validates and canonical digest matches fixture |
| `REF-MODEL-002` | Raw record omitted after lowering | `reference_model_invalid` |
| `REF-MODEL-003` | Unknown field lost | validation/checksum failure |
| `REF-MODEL-004` | Project-side source origin inserted | `reference_source_handle_invalid` |
| `REF-MODEL-005` | Mixed profile/reference generation | validation fails |
| `REF-MODEL-006` | Unresolved evidence/coverage/conflict ID | validation fails |
| `REF-MODEL-007` | Model contains winner for conflicted dimension | validation fails |
| `REF-MODEL-008` | Randomized raw/fact discovery order | identical canonical model bytes |
| `REF-MODEL-009` | Declared semantic registration order changes | expected model identity change where applicable |
| `REF-MODEL-010` | Canonical digest altered | `reference_model_digest_mismatch` |

## 11. Exact symbol lookup

| ID | Variant/query | Expected |
|---|---|---|
| `REF-LOOKUP-001` | complete / `KnownApi` | `found` exact entity/evidence/coverage |
| `REF-LOOKUP-002` | complete / `SecretText` | `found` |
| `REF-LOOKUP-003` | complete / `RemovedApi` | `authoritative_absent` |
| `REF-LOOKUP-004` | partial / `KnownApi` intact | `found` with partial surrounding coverage exposed |
| `REF-LOOKUP-005` | partial / `RemovedApi` | `absent_without_authority` + blocker |
| `REF-LOOKUP-006` | conflict / `SecretText` symbol, facet-only conflict | symbol `found`; conflict IDs exposed as relevant metadata |
| `REF-LOOKUP-007` | profile mismatch | `reference_view_profile_mismatch`, not miss |
| `REF-LOOKUP-008` | noncanonical/lowercase guessed key | `exact_query_noncanonical` |
| `REF-LOOKUP-009` | wrong entity kind | exact miss/error per grammar; no cross-kind fallback |
| `REF-LOOKUP-010` | miss with alias-like name | no alias/fuzzy candidate |
| `REF-LOOKUP-011` | result references project source handle | validation fails |
| `REF-LOOKUP-012` | lookup order randomized | identical canonical result bytes |

## 12. Restriction lookup

| ID | Variant/query | Expected |
|---|---|---|
| `REF-FACET-001` | complete / `SecretText` | `found` one `secret.return` |
| `REF-FACET-002` | complete / `KnownApi` | authoritative none only if facet domain complete |
| `REF-FACET-003` | partial facet partition | `unavailable`, not authoritative none |
| `REF-FACET-004` | conflict / `SecretText` | `conflict`, all evidence retained |
| `REF-FACET-005` | facet targets wrong return slot | validation/lowering failure |
| `REF-FACET-006` | fixture facet interpreted as real runtime wrapper | prohibited/mutation test fails |
| `REF-FACET-007` | unknown sibling facet | raw retained; dependent capability classified |

## 13. Negative authority

| ID | Case | Expected |
|---|---|---|
| `REF-AUTH-001` | exact absent + complete system partition + no blockers | authoritative |
| `REF-AUTH-002` | exact absent + partial partition | unavailable with `query_partition_partial` |
| `REF-AUTH-003` | exact absent + failed partition | unavailable with `query_partition_failed` |
| `REF-AUTH-004` | exact absent + complete source + unresolved conflict | unavailable with conflict blocker |
| `REF-AUTH-005` | exact absent + truncation | unavailable |
| `REF-AUTH-006` | exact absent + stale/mismatched generation | request rejected |
| `REF-AUTH-007` | exact absent + complete unrelated broad partition only | unavailable |

## 14. Source handles and evidence

| ID | Case | Expected |
|---|---|---|
| `REF-SOURCE-001` | Resolve system/function/facet to fixture source | valid reference handle |
| `REF-SOURCE-002` | Absolute/local/project path | rejected |
| `REF-SOURCE-003` | Source digest differs from registered input | digest mismatch |
| `REF-SOURCE-004` | Platform/reference provenance attached to unregistered origin | evidence validation fails |
| `REF-SOURCE-005` | Evidence derivation references missing/cyclic input | core validation fails |
| `REF-SOURCE-006` | Reference lookup creates finding | compile/API boundary test rejects operation |

## 15. Fixture bundle/checksums

| ID | Case | Expected |
|---|---|---|
| `REF-FIXTURE-001` | Validate declared bundle | pass |
| `REF-FIXTURE-002` | Change one normalized field without checksum update | `fixture_checksum_mismatch` |
| `REF-FIXTURE-003` | Add undeclared variant overlay | `fixture_variant_invalid` |
| `REF-FIXTURE-004` | Lookup case references unknown variant | `fixture_lookup_case_invalid` |
| `REF-FIXTURE-005` | Reorder semantically unordered maps | same digest |
| `REF-FIXTURE-006` | Reorder semantic registration array | different digest/expected behavior |

## 16. Security tests

| ID | Case | Expected |
|---|---|---|
| `REF-SEC-001` | Source asks evaluator to execute shell/IO | rejected, no execution |
| `REF-SEC-002` | Deep/huge table | bounded failure |
| `REF-SEC-003` | Path traversal/device/UNC path | rejected |
| `REF-SEC-004` | Prompt injection in comments/docs | no policy/tool effect |
| `REF-SEC-005` | Source contains token/private URL | not copied into default errors/model unless required safe fixture data; fixture rejects secret material |
| `REF-SEC-006` | Retry dynamic forbidden operation | no retry; deterministic failure |

## 17. Deferred-operation tests

| ID | Case | Expected |
|---|---|---|
| `REF-DEFER-001` | Request full pack build | `operation_not_implemented_for_milestone` |
| `REF-DEFER-002` | Request SQLite open | typed unsupported |
| `REF-DEFER-003` | Request correction apply | typed unsupported |
| `REF-DEFER-004` | Request annotation generation | typed unsupported |
| `REF-DEFER-005` | Request alias/fuzzy/replacement lookup | typed unsupported; no empty success |

## 18. Integration handoff tests

### `REF-SEAM-001` — API existence seam

Prove that:

```text
wow-reference returns authoritative exact absence for RemovedApi
wow-reference does not know the addon use site
wow-rules later receives separate project use evidence
```

### `REF-SEAM-002` — partial miss seam

Prove that partial absence carries blockers sufficient for `wow-rules` to return `NotEvaluated` rather than an API-not-found finding.

### `REF-SEAM-003` — Secret facet seam

Prove that `SecretText` facet lookup returns first-class reference evidence and no project/control-flow claim.

### `REF-SEAM-004` — conflict seam

Prove that complete ingestion plus facet conflict remains conflict and blocks dependent rule evaluation.

### `REF-SEAM-005` — generation seam

Prove that mixing complete and partial variant generations fails rather than merging coverage/facts.

## 19. Acceptance gate

E0-B code is not complete until:

```text
all non-deferred test IDs above are executable
all normative fixture lookup cases pass byte-exactly
randomized order produces identical canonical output
complete/partial/conflict states remain distinct
raw unknown fields survive round-trip
no arbitrary source code executes
no project source handle or finding can be created by wow-reference
no unsupported E1 operation returns fake success
```
