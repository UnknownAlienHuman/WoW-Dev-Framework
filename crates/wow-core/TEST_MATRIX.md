# `wow-core` E0-A test matrix

**Status:** normative implementation gate; no executable tests yet.

The first coding agent must turn these cases into tests that prove the target path executed. Test names should preserve the case IDs so failures map back to this contract.

## 1. Test rules

- Every success test asserts the canonical value, not only `is_ok`.
- Every error test asserts the stable error code and relevant safe field path.
- Randomized-order tests use recorded seeds on failure.
- Mutation tests temporarily break the target invariant and must fail for the intended reason.
- Golden JSON tests compare exact bytes.
- No test reads the network, system clock, random OS state, Git repository, or WoW client.
- No fixture uses a floating profile/revision.

## 2. Profile-ID cases

| ID | Input | Expected |
|---|---|---|
| `ID-PROFILE-001` | `profile:fixture:e0-retail-120100` | accepted, canonical |
| `ID-PROFILE-002` | `PROFILE:FIXTURE:E0-RETAIL-120100` | accepted with lowercase suggestion, `was_canonical=false` |
| `ID-PROFILE-003` | leading/trailing whitespace | `invalid_identifier` |
| `ID-PROFILE-004` | missing namespace | `invalid_identifier` |
| `ID-PROFILE-005` | empty slug | `invalid_identifier` |
| `ID-PROFILE-006` | `profile:wow:latest` | `reserved_identifier_segment` |
| `ID-PROFILE-007` | `profile:current:retail` | `reserved_identifier_segment` |
| `ID-PROFILE-008` | Unicode lookalike colon/ASCII character | `invalid_identifier` |
| `ID-PROFILE-009` | embedded NUL/control | `invalid_identifier` |
| `ID-PROFILE-010` | over maximum length | `identifier_too_long` |
| `ID-PROFILE-011` | consecutive separators in slug violating grammar | `invalid_identifier` |
| `ID-PROFILE-012` | valid label parsed twice | identical canonical value |

## 3. Dotted-ID cases

Apply to Rule, Producer, Capability, and Operation IDs.

| ID | Input | Expected |
|---|---|---|
| `ID-DOTTED-001` | `wow.api.exists` | canonical |
| `ID-DOTTED-002` | `wow.secret.local_operation` | canonical |
| `ID-DOTTED-003` | `Wow.Api.Exists` | lowercase suggestion, noncanonical |
| `ID-DOTTED-004` | one segment | `invalid_identifier` |
| `ID-DOTTED-005` | `.wow.api` | `invalid_identifier` |
| `ID-DOTTED-006` | `wow..api` | `invalid_identifier` |
| `ID-DOTTED-007` | segment begins with digit | `invalid_identifier` |
| `ID-DOTTED-008` | hyphen where dotted grammar forbids it | `invalid_identifier` |
| `ID-DOTTED-009` | whitespace around dot | `invalid_identifier` |
| `ID-DOTTED-010` | reserved segment `latest` | `reserved_identifier_segment` |
| `ID-DOTTED-011` | same text parsed as RuleId and CapabilityId | distinct typed values |
| `ID-DOTTED-012` | overlong segment | `identifier_too_long` |
| `ID-DOTTED-013` | control character | `invalid_identifier` |
| `ID-DOTTED-014` | randomized valid grammar round-trip | exact canonical round-trip |

## 4. Entity/partition key cases

| ID | Case | Expected |
|---|---|---|
| `ID-ENTITY-001` | single-segment kind `api`, key `C_UnitAuras.GetAuraDataByIndex` | accepted; exact key case preserved |
| `ID-ENTITY-001A` | multi-segment kind `api.function` | accepted |
| `ID-ENTITY-001B` | empty/invalid qualified kind | `invalid_identifier` |
| `ID-ENTITY-002` | key contains `/` | canonical `%2F` encoding |
| `ID-ENTITY-003` | key contains space | canonical `%20` encoding |
| `ID-ENTITY-004` | key contains UTF-8 non-ASCII | UTF-8 byte percent encoding |
| `ID-ENTITY-005` | lower-case percent hex | `noncanonical_percent_encoding` |
| `ID-ENTITY-006` | percent-encoded unreserved byte | `noncanonical_percent_encoding` |
| `ID-ENTITY-007` | malformed `%` sequence | `noncanonical_percent_encoding` |
| `ID-ENTITY-008` | empty key | `invalid_entity_key` |
| `ID-ENTITY-009` | control in key | `invalid_entity_key` |
| `ID-ENTITY-010` | encode/decode round-trip | exact byte-equivalent UTF-8 |
| `ID-ENTITY-011` | two keys differ only by case | remain distinct |
| `ID-ENTITY-012` | entity and partition with same payload | distinct typed IDs |
| `ID-PARTITION-001` | scope only | canonical partition without trailing colon |
| `ID-PARTITION-002` | scope + `Core/Init.lua` | canonical encoded key |
| `ID-PARTITION-003` | absent key vs empty key | absent accepted, empty rejected |
| `ID-PARTITION-004` | reserved floating scope segment | rejected |
| `ID-PARTITION-005` | duplicate logical key with different percent encoding | noncanonical variant rejected |

## 5. Digest cases

| ID | Input | Expected |
|---|---|---|
| `DIGEST-001` | valid lowercase SHA-256 | canonical |
| `DIGEST-002` | uppercase hex | lower-case canonical suggestion |
| `DIGEST-003` | 63 hex chars | `invalid_digest` |
| `DIGEST-004` | 65 hex chars | `invalid_digest` |
| `DIGEST-005` | bare hex without algorithm | `invalid_digest` |
| `DIGEST-006` | `sha1:` | `unsupported_digest_algorithm` |
| `DIGEST-007` | base64 payload | `invalid_digest` |
| `DIGEST-008` | whitespace | `invalid_digest` |
| `DIGEST-009` | truncated display digest | `invalid_digest` |
| `DIGEST-010` | same bytes, different purpose wrappers | comparison requires explicit compatible purpose |
| `DIGEST-011` | source digest used as result digest | `digest_purpose_mismatch` |
| `DIGEST-012` | parse/serialize/parse | exact equality |

Domain-separation vectors:

| ID | Case | Expected |
|---|---|---|
| `DIGEST-DOMAIN-001` | same logical value under handle/evidence domains | different digests |
| `DIGEST-DOMAIN-002` | same domain/value twice | same digest |
| `DIGEST-DOMAIN-003` | changed domain version | changed digest |
| `DIGEST-DOMAIN-004` | changed nonidentity display field | identity digest unchanged |
| `DIGEST-DOMAIN-005` | changed identity field | digest changed |

## 6. Profile identity cases

| ID | Case | Expected |
|---|---|---|
| `PROFILE-001` | complete fixture identity | accepted |
| `PROFILE-002` | fixture missing `fixture_scope` | `profile_kind_violation` |
| `PROFILE-003` | release with synthetic source | `profile_kind_violation` |
| `PROFILE-004` | release missing builder | `invalid_profile_identity` |
| `PROFILE-005` | release missing correction digest | `invalid_profile_identity` |
| `PROFILE-006` | release revision `main` | `invalid_profile_identity` |
| `PROFILE-007` | Interface zero | `invalid_profile_identity` |
| `PROFILE-008` | build zero when release requires positive build | `invalid_profile_identity` |
| `PROFILE-009` | duplicate schema ID/same version | `duplicate_schema_id` |
| `PROFILE-010` | duplicate schema ID/different version | `duplicate_schema_id` |
| `PROFILE-011` | unsorted schema entries | canonicalized sort or explicit noncanonical result per parser policy |
| `PROFILE-012` | valid profile round-trip | semantic identity preserved |
| `PROFILE-013` | same ProfileId, changed source digest | `same_label_different_identity` |
| `PROFILE-014` | different ProfileId, same material | `different_label_same_material` |
| `PROFILE-015` | Interface/build pair historically false but structurally valid | core accepts structurally; reference-layer test must reject later |
| `PROFILE-016` | fixture labeled as release by changing flag only | kind validation fails |
| `PROFILE-017` | invalid builder ID | `invalid_identifier` |
| `PROFILE-018` | invalid tool version | `invalid_profile_identity` |
| `PROFILE-019` | omitted optional fixture build | accepted |
| `PROFILE-020` | release omitted client version | rejected |

## 7. Path cases

| ID | Input | Expected |
|---|---|---|
| `PATH-001` | `Core/Init.lua` | unchanged |
| `PATH-002` | `Core\Init.lua` | `Core/Init.lua`, noncanonical |
| `PATH-003` | `./Core//Init.lua` | `Core/Init.lua`, noncanonical |
| `PATH-004` | `Core/./Init.lua` | `Core/Init.lua`, noncanonical |
| `PATH-005` | `Core/../Init.lua` | `path_escape` |
| `PATH-006` | `../Init.lua` | `path_escape` |
| `PATH-007` | `/Core/Init.lua` | `absolute_path_forbidden` |
| `PATH-008` | `C:\Core\Init.lua` | `absolute_path_forbidden` |
| `PATH-009` | `\\server\share\Init.lua` | `absolute_path_forbidden` |
| `PATH-010` | device path | `absolute_path_forbidden` |
| `PATH-011` | `file://...` or URL-like host path | `absolute_path_forbidden` |
| `PATH-012` | empty | `invalid_source_path` |
| `PATH-013` | only `.`/separators | `invalid_source_path` |
| `PATH-014` | NUL/control | `invalid_source_path` |
| `PATH-015` | valid UTF-8 non-ASCII component | preserved exactly |
| `PATH-016` | two Unicode normalization forms | remain distinct exact repository paths |
| `PATH-017` | `Foo.lua` vs `foo.lua` | remain distinct |
| `PATH-018` | trailing slash after file | canonicalized or rejected according to file-handle constructor; no silent directory assumption |
| `PATH-019` | percent signs | preserved as path text; no URL decode |
| `PATH-020` | colon in ordinary component | accepted only if not drive/device prefix and policy allows |
| `PATH-021` | extremely long path | bounded validation error |
| `PATH-022` | non-UTF-8 OS path adapter input | `unsupported_non_utf8_path` |
| `PATH-023` | canonical output reparsed | exact equality |
| `PATH-024` | randomized component mutations | no escape accepted |

## 8. Span cases

| ID | Case | Expected |
|---|---|---|
| `SPAN-001` | `unknown` only | accepted |
| `SPAN-002` | `whole_file` only | accepted |
| `SPAN-003` | `[0,0)` | accepted empty range |
| `SPAN-004` | `[0,1)` | accepted |
| `SPAN-005` | end before start | `invalid_source_span` |
| `SPAN-006` | negative offset | `invalid_source_span` |
| `SPAN-007` | unknown + offsets | `span_state_conflict` |
| `SPAN-008` | whole-file + offsets | `span_state_conflict` |
| `SPAN-009` | line/column hint field injected | strict E0 `unknown_field` at serialized boundary |
| `SPAN-010` | changed byte start | different handle ID |
| `SPAN-011` | max safe integer boundary | accepted |
| `SPAN-012` | above serialization exact-integer range | rejected by E0 constructor unless owning future schema defines decimal string |
| `SPAN-013` | round-trip | exact state preserved |
| `SPAN-014` | unknown span treated as whole file by mutation | test must fail |
| `SPAN-015` | byte offsets measured over UTF-8 multibyte text | exact byte range, not character count |
| `SPAN-016` | rendered line hints outside envelope changed | no canonical value/digest change |

## 9. Source-handle cases

| ID | Case | Expected |
|---|---|---|
| `HANDLE-001` | fixture origin + valid path/range/digest | deterministic handle ID |
| `HANDLE-002` | reference-pack origin without reference generation | `invalid_source_handle` |
| `HANDLE-002A` | reference-pack origin with project generation | `invalid_source_handle` |
| `HANDLE-002B` | repository origin with generation field | `invalid_source_handle` |
| `HANDLE-002C` | generated artifact without any generation | `invalid_source_handle` |
| `HANDLE-002D` | generated artifact with matching project/reference generations | accepted |
| `HANDLE-003` | mutable revision only | rejected for immutable handle policy |
| `HANDLE-004` | path traversal | `path_escape` |
| `HANDLE-005` | invalid span | span error propagated |
| `HANDLE-006` | invalid digest | digest error propagated |
| `HANDLE-007` | presentation line hints changed outside canonical handle | same handle ID/result semantics |
| `HANDLE-008` | same identity, different entity key | different ID |
| `HANDLE-009` | same path/content, different revision | different ID |
| `HANDLE-010` | same revision/path, different content digest | different ID |
| `HANDLE-011` | supplied host root | rejected |
| `HANDLE-012` | credentials in origin URL-like text | origin registry policy rejects/sanitizes before core; core does not echo |
| `HANDLE-013` | rebuild randomized field insertion order | same ID |
| `HANDLE-014` | verify matching digest | success |
| `HANDLE-015` | verify mismatching digest | `digest_mismatch` |
| `HANDLE-016` | compare same file/different span | expected category |
| `HANDLE-017` | compare same path/different revision | expected category, no lineage |
| `HANDLE-018` | unresolved entity key | handle still valid; resolution owned elsewhere |
| `HANDLE-019` | duplicate handle ID with different record | envelope `result_duplicate_id` |
| `HANDLE-020` | hash vector | exact expected ID |

## 10. Generation-context cases

| ID | Case | Expected |
|---|---|---|
| `CONTEXT-001` | profile + reference only | valid context |
| `CONTEXT-002` | add one project generation | valid context |
| `CONTEXT-003` | duplicate producer ID | `duplicate_producer_id` |
| `CONTEXT-004` | duplicate schema ID | `duplicate_schema_id` |
| `CONTEXT-005` | duplicate external scope/same generation | deduplicate only if record identical under explicit normalization; otherwise duplicate error |
| `CONTEXT-006` | duplicate external scope/different generation | `duplicate_external_generation_scope` |
| `CONTEXT-007` | supplied context ID mismatch | `generation_mismatch`/context ID error |
| `CONTEXT-008` | randomized list order | same derived context ID |
| `CONTEXT-009` | changed tool version | different context ID |
| `CONTEXT-010` | changed display label only | same context ID |

Merge matrix minimum:

| ID | Left | Right | Mode | Expected |
|---|---|---|---|---|
| `CONTEXT-MERGE-001` | identical | identical | strict | success |
| `CONTEXT-MERGE-002` | same ref/no project | same ref/project A | strict | failure |
| `CONTEXT-MERGE-003` | same ref/no project | same ref/project A | extend | project A |
| `CONTEXT-MERGE-004` | project A | project A | extend | success |
| `CONTEXT-MERGE-005` | project A | project B | any | `generation_mismatch` |
| `CONTEXT-MERGE-006` | profile A | profile B | any | `profile_mismatch` |
| `CONTEXT-MERGE-007` | reference A | reference B | any | `generation_mismatch` |
| `CONTEXT-MERGE-008` | external scope X/gen A | scope Y/gen B | external_union | both sorted |
| `CONTEXT-MERGE-009` | scope X/gen A | scope X/gen B | external_union | conflict |
| `CONTEXT-MERGE-010` | schema v1 | schema v2 | any | conflict |
| `CONTEXT-MERGE-011` | producer v1 | producer v2 | any | conflict |
| `CONTEXT-MERGE-012` | contexts with reordered sets | strict | success |

`require_same_generation` must reject every pair with different context IDs even when an extend/union merge could theoretically succeed.

## 11. Evidence cases

| ID | Case | Expected |
|---|---|---|
| `EVIDENCE-001` | platform source + proven + platform contract | accepted structurally |
| `EVIDENCE-002` | project source + derived + nonempty inputs | accepted |
| `EVIDENCE-003` | derived with no inputs | `derived_evidence_missing_inputs` |
| `EVIDENCE-004` | semantic candidate + proven | `evidence_authority_violation` |
| `EVIDENCE-005` | model inference + derived | `evidence_authority_violation` |
| `EVIDENCE-006` | candidate provenance + candidate | accepted |
| `EVIDENCE-007` | context mismatch | `evidence_context_mismatch` |
| `EVIDENCE-008` | missing source handle | `missing_source_handle` |
| `EVIDENCE-009` | duplicate source/evidence ref | duplicate error |
| `EVIDENCE-010` | note/rendered explanation injected into canonical evidence | strict unknown-field failure |
| `EVIDENCE-011` | changed confidence | different evidence ID |
| `EVIDENCE-012` | changed producer version | different evidence ID |
| `EVIDENCE-013` | reordered refs | same ID |
| `EVIDENCE-014` | runtime probe record | accepted only as scenario-scoped claim |
| `EVIDENCE-015` | attempt to generalize runtime scenario in core merge | no upgrade; new derivation required/higher layer |
| `EVIDENCE-016` | conflict record references fewer than two records | validation failure |
| `EVIDENCE-017` | conflict with empty affected scope | `conflict_scope_empty` |
| `EVIDENCE-018` | derived record includes candidate-only input but claims proven | authority violation |
| `EVIDENCE-019` | two possible records merged | both retained, no derived aggregate |
| `EVIDENCE-020` | deterministic derivation with explicit inputs | new derived ID |
| `EVIDENCE-021` | same derivation input order shuffled | same ID |
| `EVIDENCE-022` | missing derivation input record | `missing_evidence_reference` |
| `EVIDENCE-023` | hash vector | exact expected ID |
| `EVIDENCE-024` | duplicate evidence ID/different content | envelope failure |
| `EVIDENCE-025` | direct self derivation | `evidence_derivation_cycle` |
| `EVIDENCE-026` | multi-record derivation cycle | `evidence_derivation_cycle` |
| `EVIDENCE-027` | evidence semantic coverage ref resolves to zero records | `coverage_record_missing` |
| `EVIDENCE-028` | semantic coverage ref resolves ambiguously | `coverage_conflict` |
| `EVIDENCE-029` | project-use observation and platform-contract proof use distinct handles/records under one context | accepted; neither handle replaces the other |
| `EVIDENCE-030` | consumer assigns platform provenance to an origin not authorized by its source registry | rejected by the producer/source-registry seam before core record construction |

## 12. Conflict-record cases

| ID | Case | Expected |
|---|---|---|
| `CONFLICT-001` | two same-context evidence IDs + affected capability | deterministic ConflictId |
| `CONFLICT-002` | fewer than two evidence IDs | validation failure |
| `CONFLICT-003` | duplicate evidence ID | `duplicate_evidence_reference` |
| `CONFLICT-004` | missing evidence | `missing_evidence_reference` |
| `CONFLICT-005` | evidence contexts differ | `conflict_context_mismatch` |
| `CONFLICT-006` | no affected refs | `conflict_scope_empty` |
| `CONFLICT-007` | capability-wide affected ref | accepted |
| `CONFLICT-008` | capability + exact partition affected ref | accepted |
| `CONFLICT-009` | reordered evidence/affected refs | same ConflictId |
| `CONFLICT-010` | conflict back-reference injected into EvidenceRecord | strict schema failure; no identity cycle |
| `CONFLICT-011` | core asked to select winner | forbidden operation |
| `CONFLICT-012` | missing conflict referenced by coverage/evaluation | `missing_conflict_reference` |

## 13. Coverage-record and combination cases

Coverage-record validation minimum:

| ID | Case | Expected |
|---|---|---|
| `COVERAGE-VALIDATE-001` | complete record with no blockers | accepted, deterministic CoverageId |
| `COVERAGE-VALIDATE-002` | complete + missing input | validation failure |
| `COVERAGE-VALIDATE-003` | failed without failure code | validation failure |
| `COVERAGE-VALIDATE-004` | not_applicable with conflict/truncation | validation failure |
| `COVERAGE-VALIDATE-005` | conflict does not affect record scope | `coverage_conflict` |
| `COVERAGE-VALIDATE-006` | missing conflict | `missing_conflict_reference` |
| `COVERAGE-VALIDATE-007` | context mismatch | `coverage_context_mismatch` or enclosing result-context error |
| `COVERAGE-VALIDATE-008` | reordered set fields | same CoverageId |

Required applicable statuses:

| ID | Inputs | Expected combined |
|---|---|---|
| `COVERAGE-COMBINE-001` | complete | complete |
| `COVERAGE-COMBINE-002` | complete + complete | complete |
| `COVERAGE-COMBINE-003` | complete + partial | partial |
| `COVERAGE-COMBINE-004` | partial + unknown | unknown |
| `COVERAGE-COMBINE-005` | unknown + failed | failed |
| `COVERAGE-COMBINE-006` | complete + failed | failed |
| `COVERAGE-COMBINE-007` | not_applicable only | not_applicable |
| `COVERAGE-COMBINE-008` | not_applicable + complete | complete |
| `COVERAGE-COMBINE-009` | not_applicable + partial | partial |
| `COVERAGE-COMBINE-010` | missing required record | `coverage_record_missing` |
| `COVERAGE-COMBINE-011` | duplicate unique record | `duplicate_coverage_record` |
| `COVERAGE-COMBINE-012` | complete with affecting conflict | status complete + conflict retained; negative authority denied |
| `COVERAGE-COMBINE-013` | complete with affecting truncation | status complete + truncation retained; operation may become partial |
| `COVERAGE-COMBINE-014` | empty required set | explicit contract error, not complete |

Capability-summary validation:

- every partition ref resolves to the exact `CoverageId`;
- recomputation yields the same status/conflicts/truncation;
- summaries cannot omit a worse partition or affecting conflict;
- summaries and underlying records share one context;
- randomized record order yields identical summary bytes.

Property cases:

- commutative over set input;
- associative after record-key validation;
- idempotent only for identical records after dedup policy;
- adding `not_applicable` does not improve/degrade an applicable result;
- replacing any applicable status with a worse precedence cannot improve the result.

## 14. Capability and `NotEvaluated` cases

| ID | Case | Expected |
|---|---|---|
| `CAPABILITY-001` | all required complete | runnable |
| `CAPABILITY-002` | required partial | `NotEvaluated` with partition |
| `CAPABILITY-003` | required unknown | `NotEvaluated` |
| `CAPABILITY-004` | required failed | `NotEvaluated` |
| `CAPABILITY-005` | required not_applicable | `NotEvaluated` for applicable subject |
| `CAPABILITY-006` | optional capability partial | subject runs if descriptor does not require it; warning owned higher layer |
| `CAPABILITY-007` | multiple blocking capabilities | all returned sorted |
| `CAPABILITY-008` | conflict on complete partition | `NotEvaluated` when conflict affects subject |
| `CAPABILITY-009` | context mismatch in coverage | operation error, not `NotEvaluated` |
| `CAPABILITY-010` | same inputs reordered | same NotEvaluated ID |
| `CAPABILITY-011` | changed evaluation producer version | different NotEvaluated ID |
| `CAPABILITY-012` | blocking CoverageId missing | validation failure |
| `CAPABILITY-013` | conflict missing | `missing_conflict_reference` |
| `CAPABILITY-014` | not-evaluated record used as clean pass | envelope/status failure |

## 15. Negative-authority cases

| ID | Exact lookup | Coverage/conflict | Expected |
|---|---|---|---|
| `NEGATIVE-001` | absent | all complete, no conflict | authoritative_absent |
| `NEGATIVE-002` | absent | partial | not_authoritative/partition_partial |
| `NEGATIVE-003` | absent | unknown | not_authoritative/partition_unknown |
| `NEGATIVE-004` | absent | failed | not_authoritative/partition_failed |
| `NEGATIVE-005` | absent | complete + conflict | not_authoritative/unresolved_conflict |
| `NEGATIVE-006` | absent | complete + truncation | not_authoritative/result_truncated |
| `NEGATIVE-007` | absent | candidate-only substitute exists | not_authoritative/candidate_only_evidence |
| `NEGATIVE-008` | not run | complete | not_authoritative/capability_not_evaluated |
| `NEGATIVE-009` | absent | profile mismatch | operation context error or denial/generation_mismatch per entrypoint |
| `NEGATIVE-010` | scope unknown | complete elsewhere | not_authoritative/scope_unknown |
| `NEGATIVE-011` | subject not applicable | all N/A | not_applicable |
| `NEGATIVE-012` | found | any | no absence decision; caller handles positive result |
| `NEGATIVE-013` | absent | multiple denial reasons | all safe reasons returned sorted |
| `NEGATIVE-014` | randomized records | same decision bytes |

Mutation: force boolean `not_found=true` without decision and ensure envelope validation fails.

## 16. Finding and warning cases

| ID | Case | Expected |
|---|---|---|
| `MESSAGE-ARG-001` | unique sorted typed args | accepted |
| `MESSAGE-ARG-002` | duplicate name | `invalid_message_argument` |
| `MESSAGE-ARG-003` | arbitrary nested JSON | rejected |
| `MESSAGE-ARG-004` | float | rejected |
| `MESSAGE-ARG-005` | reordered args | canonical same order |
| `FINDING-FINGERPRINT-001` | identical semantic fields | same fingerprint |
| `FINDING-FINGERPRINT-002` | changed rendered prose only | same fingerprint |
| `FINDING-FINGERPRINT-003` | changed nonidentity arg | same fingerprint |
| `FINDING-FINGERPRINT-004` | changed identity arg | different fingerprint |
| `FINDING-FINGERPRINT-005` | changed severity/policy | same fingerprint |
| `FINDING-FINGERPRINT-006` | changed rule version | different fingerprint |
| `FINDING-BIND-001` | valid context/refs | deterministic FindingId |
| `FINDING-BIND-002` | context mismatch | `finding_context_mismatch` |
| `FINDING-BIND-003` | missing handle | `missing_source_handle` |
| `FINDING-BIND-004` | missing evidence | `missing_evidence_reference` |
| `FINDING-BIND-005` | Candidate evidence + exact_edit | `remediation_authority_violation` |
| `FINDING-BIND-006` | same fingerprint, different context | different FindingId |
| `FINDING-BIND-007` | primary project use handle plus separate platform-contract evidence | accepted; primary location remains the project span |
| `FINDING-DEDUP-001` | duplicate identical record/ID | one record after explicit dedup |
| `FINDING-DEDUP-002` | duplicate ID/different content | `result_duplicate_id` |
| `FINDING-DEDUP-003` | same message, different fingerprints | both retained |

Ordering tests randomize findings with different path/span/rule/code/fingerprint and assert exact canonical order independent of severity or insertion order.

Warning cases:

| ID | Case | Expected |
|---|---|---|
| `WARNING-001` | valid structured optional-lane warning | deterministic WarningId |
| `WARNING-002` | context mismatch | `warning_context_mismatch` |
| `WARNING-003` | missing handle/evidence | narrow reference error |
| `WARNING-004` | rendered prose injected | strict unknown-field failure |
| `WARNING-005` | warning used instead of required NotEvaluated | status/contract failure |
| `WARNING-006` | same fields reordered | same WarningId |

## 17. Budget/truncation cases

| ID | Case | Expected |
|---|---|---|
| `BUDGET-001` | all positive within maximum | accepted |
| `BUDGET-002` | zero | `budget_invalid` |
| `BUDGET-003` | above implementation maximum | `budget_invalid` |
| `BUDGET-004` | unknown dimension | `budget_invalid` |
| `BUDGET-USAGE-001` | checked addition across every declared collection | exact usage |
| `BUDGET-USAGE-002` | arithmetic overflow | `usage_overflow` |
| `TRUNCATION-001` | usage below limits, no omissions | not_truncated |
| `TRUNCATION-002` | known omitted findings | truncated + count |
| `TRUNCATION-003` | omitted count unknown | explicit count_unknown |
| `TRUNCATION-004` | silent collection clipping mutation | validation fails |
| `TRUNCATION-005` | truncation affects required capability | envelope partial and negative authority denied |
| `TRUNCATION-006` | insertion order changes | same truncation record |

## 18. Envelope cases

| ID | Case | Expected |
|---|---|---|
| `ENVELOPE-001` | canonical clean example | validates, exact bytes |
| `ENVELOPE-002` | canonical findings example | validates, status complete |
| `ENVELOPE-003` | canonical NotEvaluated example | validates, status partial |
| `ENVELOPE-003A` | canonical conflict-blocked example | validates, status partial |
| `ENVELOPE-003B` | capability summary omits referenced worse coverage record | `coverage_conflict` |
| `ENVELOPE-004` | finding context mismatch | `result_context_violation` |
| `ENVELOPE-005` | evidence context mismatch | error |
| `ENVELOPE-006` | unresolved handle ref | `result_reference_violation` |
| `ENVELOPE-007` | unresolved evidence ref | error |
| `ENVELOPE-008` | duplicate ID/same content | explicit dedup before finalization or validation failure according to operation stage |
| `ENVELOPE-009` | duplicate ID/different content | `result_duplicate_id` |
| `ENVELOPE-010` | `complete` with NotEvaluated | `result_status_violation` |
| `ENVELOPE-011` | `complete` with affecting truncation | `result_status_violation` |
| `ENVELOPE-012` | findings nonempty/status complete | accepted |
| `ENVELOPE-013` | failed but contains authoritative findings | `result_status_violation` unless schema explicitly retains partial failed output |
| `ENVELOPE-014` | unknown top-level field | `unknown_field` |
| `ENVELOPE-015` | duplicate JSON key | `duplicate_field` |
| `ENVELOPE-016` | unsupported schema major | `schema_version_unsupported` |
| `ENVELOPE-017` | unsupported canonicalization version | `schema_version_unsupported` |
| `ENVELOPE-018` | wrong canonical digest | `canonical_digest_mismatch` |
| `ENVELOPE-019` | randomized arrays/maps | canonical bytes equal golden |
| `ENVELOPE-020` | changed transport-only note/display data outside canonical envelope | canonical envelope/digest unchanged |
| `ENVELOPE-021` | changed identity field | digest changes |
| `ENVELOPE-022` | timestamp injected | unknown/forbidden field error |
| `ENVELOPE-023` | absolute host path injected | source-handle error |
| `ENVELOPE-024` | output byte count stable finalization | converges and validates |

## 19. Hash/canonical byte cases

`examples/HASH_VECTORS.json` must be executed verbatim.

Additional properties:

- one/two/N-worker producers with identical logical records produce identical bytes;
- different platform newline conventions do not change bytes;
- object construction order does not change bytes;
- transport line/column hints remain outside the canonical handle/result and cannot change its digest;
- severity/policy changes alter result digest but not finding fingerprint;
- rendered-message changes are outside E0 canonical envelope;
- domain changes always change hash;
- final `canonical_digest` verifies after round-trip.

## 20. Error-model cases

- every operation returns only catalogued errors;
- error records contain safe bounded arguments;
- malformed source input is not echoed in full;
- absolute host paths are not exposed;
- same error semantics serialize deterministically;
- an error is never placed in findings;
- NotEvaluated is never emitted as an operation error;
- `contract_violation` count remains zero for expected invalid-input cases.

## 21. E0-A completion gate

E0-A may be merged as implementation-complete only when:

```text
all normative cases above are executable
all committed JSON examples validate
all hash vectors pass
randomized ordering tests pass for recorded seeds
profile/context/source/evidence/conflict/coverage/summary/finding/warning invariants are mutation-tested
no IO/clock/random/async dependency appears in wow-core
no later-domain algorithm appears in wow-core
cargo fmt/clippy/test results are reported fresh
```

Until Rust code exists, documentation validation may report only JSON/link/hash-vector checks; it must not claim the executable E0-A gate passed.
