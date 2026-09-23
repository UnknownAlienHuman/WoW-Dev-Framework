# `wow-core` E0-A test matrix

**Status:** normative implementation gate with partial executable coverage.

`tests/e0_source_conformance.rs` covers PATH-001–021, PATH-023–024 and
SPAN-001–014 through the actual path/span/handle APIs. PATH-024 includes
1,248 drive-prefix mutations through both path and handle admission plus
20 parent-component mutations. PATH-022 (an OS non-UTF-8 adapter) is not
implemented by this UTF-8-only core surface and is not claimed as tested.
`tests/e0_examples.rs` compares every committed envelope against canonical
fixture bytes; `tests/hash_vectors.rs` checks the public digest operation and
exact typed-ID families. `tests/e0_schema_conformance.rs` covers ENVELOPE-016/017
on decoded and finalized envelopes, with independently resealed digests/byte
counts so a stale hash cannot mask missing schema admission. These focused
suites do not close the whole E0-A gate.

`tests/e0_coverage_conformance.rs` and `tests/coverage/` exercise raw coverage,
checked capability availability and negative authority. Coverage truth-table
cases run all 125 status triples under all six input orders. Native regressions
cover digest-interleaved duplicate logical keys; distinct producer preservation;
recomputed invalid record metadata; missing/forged/omitted summary input; context
and conflict closure; shared blockers; optional-lane isolation; nonapplicable,
not-run, candidate and truncation decisions; exact denial IDs and deterministic
ordering. This is a scoped acceptance slice, not complete E0-A certification.
`NEGATIVE-012` remains a caller obligation: the pure function is invoked only for
reported misses, not positive lookups. Conflict evidence closure and source
eligibility remain responsibilities of the complete envelope/owner registry.

`tests/e0_evidence_conformance.rs` adds 24 grouped public-path tests: local evidence
and conflict admission, one-context DAG closure, every confidence edge, direct and
transitive runtime restrictions, hostile cyclic wire IDs, 64 deterministic orders,
a 16,384-record chain on a 256 KiB test-thread stack and shared-input DAGs. Golden
record/envelope bytes remain unchanged. These tests do not certify provenance
acquisition, standalone source/coverage joins, input-size ceilings or full E0-A.

`tests/e0_budget_conformance.rs` adds 23 grouped public-path tests under
`tests/budget/`: all nine numeric dimensions and overflow boundaries; strict wire
types; explicit known/unknown counts; decoded truncation admission through every
consumer; 64 deterministic input permutations; byte/count limits; and resealed
envelope mutations. Golden fixture bytes are unchanged. Producer honesty and
host decoding-size ceilings are outside this pure validation slice.

`tests/e0_finding_conformance.rs` adds 26 grouped tests under `tests/finding/`:
typed argument boundaries, golden finding reconstruction, fingerprint/context
separation, all three diagnostic binding/validation paths, retained registry
integrity/ancestry/source closure, decoded reference sets and remediation, warning
subject/ID admission, explicit deduplication, 64 deterministic input permutations
and independently resealed envelopes. Correct fixture bytes remain unchanged.
Standalone source-generation/coverage/provenance and recipe execution/safety are
not certified by these tests. Remaining E0-A acceptance stays separate.

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

### Profile admission completion slice

`tests/e0_profile_conformance.rs` exercises PROFILE-001–026 through builders,
decoded validation and checked comparisons, including both operand directions
and invalid self-comparison. The original golden profile is reconstructed and
compared byte-for-byte; schema ordering uses all six three-entry permutations.
Resealed context IDs cannot hide invalid nested profiles. Existing five envelope
golden consumers remain unchanged. This is not full E0-A/source certification.

| ID | Case | Expected |
|---|---|---|
| `PROFILE-021` | optional fixture build explicitly zero | `invalid_profile_identity` at `client_build` |
| `PROFILE-022` | decoded half builder pair or orphan correction | missing-field error; absent pair remains valid |
| `PROFILE-023` | empty/blank/padded/control/overlong scope | `profile_kind_violation`; exact 4,096-byte boundary accepted |
| `PROFILE-024` | invalid or overlong revision text, either kind | `invalid_profile_identity`; exact 1,024-byte boundary accepted |
| `PROFILE-025` | unknown wire field/enum, invalid typed identity/numeric field | structural decode rejection without repair |
| `PROFILE-026` | bad profile inside resealed context and raw E0 input | nested field error before identity acceptance |

The previously referenced comparison cases are made explicit:

| ID | Case | Expected |
|---|---|---|
| `PROFILE-COMPARE-001` | identical valid values | Identical and exact-match success |
| `PROFILE-COMPARE-002` | same label, each material field changed separately | SameLabelDifferentIdentity and exact differing field |
| `PROFILE-COMPARE-003` | different label, same material | DifferentLabelSameMaterial; exact-match rejection |
| `PROFILE-COMPARE-004` | different label and material | Different |
| `PROFILE-COMPARE-005` | comparison operands reversed | symmetric category and differing fields |
| `PROFILE-COMPARE-006` | invalid left or right operand | original profile-field error |
| `PROFILE-COMPARE-007` | invalid profile compared with itself | original profile-field error, never Identical |
| `PROFILE-COMPARE-008` | real-snapshot fixture vs structurally valid release | remains distinct; no historical build proof |

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

### Checked handle consumer cases

Executable targets: `tests/e0_handle_conformance.rs` and
`tests/e0_handle_compare_conformance.rs`, with shared fixture reconstruction in
`tests/handle/support.rs`. The construction suite executes HANDLE-001–020 and
all four origin classes under every generation-presence combination; existing
path/span tests remain required. HANDLE-012 covers bounded text and credential
rejection only, not external registry eligibility. SPAN-015/016 are exercised
with multibyte fixture text and a separate transport-hint projection; core does
not become a content resolver.

| Case | Required invariant |
|---|---|
| HANDLE-VERIFY-001/002 | Matching full-artifact digest succeeds; mismatch retains its narrow code |
| HANDLE-VERIFY-003 | A substituted handle ID cannot pass on matching content |
| HANDLE-VERIFY-004 | A resealed illegal origin/generation combination rejects |
| HANDLE-VERIFY-005 | A resealed invalid span rejects before digest comparison |
| HANDLE-VERIFY-006 | The digest identifies the entire supplied artifact, not the selected span |
| HANDLE-COMPARE-001 | Identical/round-tripped valid handles compare identically |
| HANDLE-COMPARE-002 | Unknown, whole-file, empty and nonempty byte spans remain distinct (HANDLE-016) |
| HANDLE-COMPARE-003/004 | Revision and content differences keep their existing categories |
| HANDLE-COMPARE-005 | Origin, path and entity changes alone are unrelated |
| HANDLE-COMPARE-006 | Changed generations cannot be classified as a span-only change |
| HANDLE-COMPARE-007 | Revision/content/span classification precedence is explicit |
| HANDLE-COMPARE-008/009 | Forged IDs reject in either position and on self-comparison through both APIs |
| HANDLE-COMPARE-010 | Resealed invalid fields reject rather than producing a category |

Three negative controls restore the entire previous `source.rs` and execute only
the HANDLE-VERIFY-003/004/005 cases. The separate comparison target is not compiled
under the old infallible signature; no comparison-baseline execution is claimed.
The compile-fail doctest preserves source/result digest type separation. Sixty-four
recorded-seed wire-key permutations preserve the exact normative handle vector.
Existing E0 envelope consumers and all golden bytes/digests remain unchanged.
Full E0-A/R0, PATH-022's OS adapter and source-owner attestation remain separate.

## 10. Generation-context cases

Regression source: `tests/e0_generation_conformance.rs` exercises constructors,
wire validation, ID retrieval, both merge entrypoints and both strict-guard
entrypoints. It includes independently resealed invalid contexts so a stale hash
cannot mask a missing nested check, and 64 fixed-seed input permutations. Test
source is not a passing execution receipt; these cases still require fresh native
CI results before their acceptance status can advance.

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
| `CONTEXT-MERGE-013` | schema/producer inventory subset | complete inventory | strict | `merge_mode_violation` |
| `CONTEXT-MERGE-014` | schema/producer inventory subset | complete inventory | extend | `merge_mode_violation` |
| `CONTEXT-MERGE-015` | schema/producer inventory subset | complete inventory | external_union | `merge_mode_violation` |
| `CONTEXT-MERGE-016` | disjoint version inventories | disjoint version inventories | any | `merge_mode_violation` |
| `CONTEXT-MERGE-017` | distinct external scopes | distinct external scopes | strict/extend | `merge_mode_violation`; only external_union admits them |
| `CONTEXT-MERGE-018` | external scope/gen A/revision A | same scope/gen A/revision B | external_union | `duplicate_external_generation_scope` |

Strict same-generation guard:

| ID | Case | Expected |
|---|---|---|
| `CONTEXT-SAME-001` | independently rebuilt and decoded golden context | success; exact bytes and ID preserved |
| `CONTEXT-SAME-002` | unchanged supplied ID, mutated right-hand identity fields | validation failure before equality |
| `CONTEXT-SAME-003` | unchanged supplied ID, mutated left-hand identity fields | validation failure before equality |
| `CONTEXT-SAME-004` | two references to the same stale-ID record | validation failure, not self-equality success |
| `CONTEXT-SAME-005` | valid distinct contexts and valid distinct IDs | `generation_mismatch` |
| `CONTEXT-SAME-006` | equal IDs but invalid nested profile or external entry | narrow validation error propagated |

Nested external-record admission (constructor, context builder and decoded context):

| ID | Case | Expected |
|---|---|---|
| `CONTEXT-EXTERNAL-001` | provider differs from typed generation's provider | `generation_mismatch`, even with recomputed context digest |
| `CONTEXT-EXTERNAL-002` | invalid, overlong or reserved provider | existing identifier error; no normalization into another provider |
| `CONTEXT-EXTERNAL-003` | empty, padded, control-containing or overlong scope | `invalid_identifier` |
| `CONTEXT-EXTERNAL-004` | empty, padded, control-containing or overlong supplied revision | `invalid_identifier` |
| `CONTEXT-EXTERNAL-005` | exact byte limit, multibyte scope and absent optional revision | valid; round-trip and guard agree |

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


## Coverage admission regression extensions

| ID | Case | Expected |
|---|---|---|
| `CAPABILITY-015` / `NEGATIVE-015` | Empty required summaries or evidence | `coverage_record_missing`, never runnable/absence |
| `CAPABILITY-016` / `NEGATIVE-016` | Decoded complete summary over partial raw coverage | `coverage_conflict` |
| `CAPABILITY-017` / `NEGATIVE-017` | Consistent subset summary omits a supplied worse partition | `coverage_conflict` |
| `CAPABILITY-018` | Complete path with invalid subject | Same constructor identifier error |
| `COVERAGE-COMBINE-015` | Same owner key separated by unrelated digest-sorted ref | `duplicate_coverage_record` |
| `COVERAGE-COMBINE-016` | Same partition, distinct producer statements | Both retained; conservative precedence |
| `COVERAGE-VALIDATE-009` | Resealed invalid missing-input/truncation metadata | Same validation as construction |
| `COVERAGE-VALIDATE-010` | Affecting known conflict omitted from records and summary | `coverage_conflict` |
| `NEGATIVE-018` | Nonapplicable with candidate/truncation blocker | `not_authoritative`, blocker retained |
| `NEGATIVE-019` | Several partial/unknown/failed partitions in one summary | Every denial reason retained |
| `NEGATIVE-020` | Evaluation context or blocker record differs | Context/reference error |

`COVERAGE-COMBINE-013` denotes an outer operation's truncation affecting complete
source coverage. It does not authorize a contradictory `complete` raw record
with a nonempty `truncation_refs` field; see `validate_coverage_record`.

## Evidence-DAG executable cases

| ID | Case | Expected |
|---|---|---|
| `EVIDENCE-GRAPH-001/002` | Empty registry, leaf and diamond | Acyclic, unchanged input; empty is not absence |
| `EVIDENCE-GRAPH-003` | Connected or disconnected mixed contexts | `evidence_context_mismatch` |
| `EVIDENCE-GRAPH-004` | All parent/child confidence combinations | No strengthening, including Possible→Derived and Candidate→Possible |
| `EVIDENCE-GRAPH-005` | Runtime→project→project→platform, direct graph and envelope | `evidence_authority_violation` |
| `EVIDENCE-GRAPH-006` | One scenario-tainted branch in a diamond | Platform conclusion rejected |
| `EVIDENCE-GRAPH-007` | Safe platform derivation plus unrelated runtime component | Accepted |
| `EVIDENCE-GRAPH-008` | Missing input; duplicate ID, same/different bytes | Narrow missing/duplicate error |
| `EVIDENCE-GRAPH-009/010` | Direct and two-record cyclic wire mutations | `evidence_derivation_cycle`; no hash fixed point assumed |
| `EVIDENCE-GRAPH-011` | Acyclic registry with an invalid parent hash | `canonical_digest_mismatch` |
| `EVIDENCE-GRAPH-012` | 64 seeded registry orders | Identical finalized envelope bytes |
| `EVIDENCE-GRAPH-013` | 16,384-deep evidence chain | Succeeds on bounded test-thread stack; no recursive traversal |
| `EVIDENCE-GRAPH-014` | Explicit synthetic context, correctly resealed | Structurally valid, no provenance certificate |
| `EVIDENCE-GRAPH-015` | 64 shared-input layers with exponentially many paths | Each input visited once, no path enumeration |

`tests/evidence/records.rs` exercises the committed examples and EVIDENCE local
constructor/decoded-validation cases with independently resealed IDs. In particular,
coverage-ref duplicates and reversed order must reject, not disappear through
hash recomputation. `tests/evidence/conflicts.rs` checks capability-wide/partition
scopes, minimum membership, all canonical sets, unknown fields and hash admission.
Conflict-reference and source-registry ownership checks remain distinct consumer
tests, not conclusions from a conflict's local structural validity.

## Budget/truncation admission regression extensions

| ID | Case | Expected |
|---|---|---|
| `BUDGET-005` | Each usage dimension exceeds limit with valid truncation | `budget_exceeded`, no waiver |
| `BUDGET-006` | Replace output-byte usage | Revalidated budget, all other truth unchanged |
| `TRUNCATION-007` | Decoded empty, duplicate or unordered outer entries | `contract_violation` |
| `TRUNCATION-008` | Decoded contradictory known/unknown count | Same rejection as entry construction |
| `TRUNCATION-009` | Negative-authority caller supplies malformed truncation | Error, not a denial receipt |
| `TRUNCATION-010` | Decoded unordered/duplicate capability IDs | Reject without silent repair |
| `TRUNCATION-011` | Invalid decoded collection name | Same grammar/code as constructor, no raw echo |
| `TRUNCATION-012` | Same collection, different payload | Duplicate collection rejected |
| `TRUNCATION-013` | Unknown variant/field or duplicate field | Strict wire decoding rejects |
| `ENVELOPE-025` | Resealed partial envelope with empty truncation | Invalid retained truth rejected |
| `ENVELOPE-026` | Resealed contradictory/nonnormal nested truncation | Same entry admission at envelope/finalizer |
| `FINALIZE-001` | All four committed check envelopes | Exact golden bytes and byte counts |
| `FINALIZE-002` | Final byte limit with omission metadata and digest | Exact fit accepted, excess rejected |

`TRUNCATION-004` detects collection clipping when retained usage contradicts actual
records, with an independently correct result hash. It is not proof against an
upstream producer falsifying both counts and omission claims. `TRUNCATION-005`
also checks that valid omission metadata requires partial status and denies
negative authority; no complete/clean result follows from truncation.

The Linux job in budget/truncation run `35622629254` passed 22 focused cases and exposed
an additional `TRUNCATION-013` wire defect: serde's tagged unit variant ignored
payload fields on `not_truncated`. The test is retained; a private empty-struct
wire variant now preserves strict admission without changing canonical output.
The final accepted checkpoint must rerun all focused and workspace checks; this
failed run is not acceptance evidence.

## Envelope coverage/evaluation join regressions

`tests/e0_coverage_join_conformance.rs` contains 22 grouped tests, with independently
resealed envelope/record hashes and byte/count bookkeeping. It covers the following
cases through local validation, finalization, canonical reordering and the existing
negative-authority operation where applicable:

| Case | Required behavior |
|---|---|
| ENVELOPE-JOIN-001 | All four existing golden envelopes retain exact canonical bytes from the raw fixture JSON, not its pretty-print whitespace |
| ENVELOPE-JOIN-002 | A summary cannot omit a retained partial/unknown/failed partition; full recomputation passes |
| ENVELOPE-JOIN-003 | Logical duplicate raw statements reject even without a summary |
| ENVELOPE-JOIN-004 | Retained affecting conflicts cannot disappear from raw coverage and its summary |
| ENVELOPE-JOIN-005/006 | Same-owner version collisions reject; independent producers remain valid |
| ENVELOPE-JOIN-007/008 | Optional capability isolation and empty required-scope rejection remain intact |
| ENVELOPE-JOIN-009/010 | Missing summary refs reject; 64 seeded permutations preserve golden bytes |
| NOT-EVALUATED-JOIN-001/002 | Compact golden and explicit exact nested conflicts both pass |
| NOT-EVALUATED-JOIN-003/004 | Missing enclosing conflicts and invented nested references reject |
| NOT-EVALUATED-JOIN-005/006 | Duplicate CoverageIds and undeclared blocker capabilities reject locally and at envelope boundary |
| NOT-EVALUATED-JOIN-007/008 | Healthy complete records and unrelated conflicts cannot explain a blocker |
| NOT-EVALUATED-JOIN-009/010 | Missing/misdescribed owner records and noncanonical nested sets reject |
| NOT-EVALUATED-JOIN-011/012 | Unknown partition/nonapplicable denial remains valid; wrong identity still rejects |

This slice does not certify source completeness, request-scope exhaustiveness,
producer provenance, full E0-A closure or public R0 availability. No new public
operation, dependency, schema version or golden fixture is introduced.

The coverage-join golden oracle uses the committed JSON as an untyped `Value`,
then applies the existing canonical JSON encoder. It never obtains expectations
from the typed envelope, its finalizer or mutated test input. No fixture fields,
array order, identities, digests or counters are rewritten. The permutation test
computes its expected bytes once, before any shuffle. See
[COVERAGE_JOIN_RECOVERY.md](COVERAGE_JOIN_RECOVERY.md) for the interrupted-run audit.

## Canonical serialization admission regression extensions

`tests/e0_canonical_conformance.rs` contains 19 grouped executable tests. Run the
focused target without sibling feature unification and again in the full workspace.
The existing `tests/hash_vectors.rs` remains the exact canonical-text/hash oracle.

| Case | Executed invariant |
|---|---|
| CANONICAL-001 | Equal/different duplicate map values reject through bytes, text, domain hashes and typed handle derivation. |
| CANONICAL-002 | Nested arrays/tuples/options/structs/newtypes/enum containers retain duplicate rejection. |
| CANONICAL-003 | Flattened fields cannot overwrite declared fields; disjoint fields retain exact bytes. |
| CANONICAL-004 | Integer, boolean and character keys collide with equivalent JSON string keys. |
| CANONICAL-005 | Manual SerializeStruct duplicate fields reject. |
| CANONICAL-006/007 | Custom serializer diagnostic payloads are not exposed; custom value-error prose is not formatted. |
| CANONICAL-008/009 | Exact scalar bytes, unsigned bounds, null/negative/float/oversized rejection. |
| CANONICAL-010/011 | All Serde container forms and existing key spellings keep bytewise ordering. |
| CANONICAL-012/013 | Invalid keys reject before their values are serialized; malformed map emission returns an error. |
| CANONICAL-014 | Payload serialized once; forged sequence/map capacity hints do not allocate those capacities. |
| CANONICAL-015 | 64 seeded permutations and 1/2/4 workers give identical output; arrays are not reordered. |
| CANONICAL-016 | All five committed envelopes match independently encoded raw JSON, with unchanged stored digests. |
| CANONICAL-017 | Feature-unified arbitrary-precision Number handling retains the E0 subset; a normal marker-looking map is not a Number. |
| CANONICAL-018/019 | Raw JSON cannot bypass entry admission; bytes/display/omitted optional fields keep their wire contract. |

Negative controls restore the prior canonical serializer and prove duplicate maps,
flattened collisions, coerced keys, repeated struct fields and custom-error echo
are detected. Baseline controls do not run the forged allocation-hint test.
Full E0-A, host duplicate-aware decoding/size limits and producer provenance remain
separate gates. No fixture is regenerated or expected output derived from a typed
actual envelope during this checkpoint.

## Bounded raw E0 envelope admission

Executable cases: `tests/e0_decode_conformance.rs`; existing golden consumers:
`tests/e0_examples.rs`. Run the focused cases in both core-only and unified
`serde_json/arbitrary_precision` feature profiles.

| Case | Required invariant |
|---|---|
| DECODE-001/002 | All five golden envelopes and canonical round trips; whitespace, key order and escaped key identity preserved |
| DECODE-003/004 | Duplicate decoded keys at every depth; optional null cannot disappear through structural decoding |
| DECODE-005 | Raw negative zero, sign/leading-zero/fraction/exponent/overflow rejection before numeric normalization |
| DECODE-006–010 | Exact external byte, token, depth and raw-string boundaries; zero/above-ceiling/usize-max limits reject |
| DECODE-011/012 | UTF-8/BOM/escape/surrogate/grammar/trailing input and strict nested schema admission |
| DECODE-013 | Existing schema and digest errors propagate, never replaced by success |
| DECODE-014/015 | No unsafe error echo; quoted structure and independent objects do not create false duplicates |

Full E0-A/R0, host acquisition limits, source provenance and producer completeness
remain separate. No fixture is rewritten or derived from the actual typed output.
