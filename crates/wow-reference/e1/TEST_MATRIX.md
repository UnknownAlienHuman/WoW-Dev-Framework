# `wow-reference` E1-B test matrix

**Status:** normative executable acceptance matrix for snapshot/profile identity, parser/evaluator safety, raw preservation, normalization, corrections, coverage, persistent build, and exact ReferenceView.

Tests inspect structured IDs/records/manifests/coverage/results and prove source execution does not occur. Every successful negative-authority fixture must break when a required input/coverage/conflict/store condition is mutated.

## 1. Configuration and dependency boundary

| ID | Case | Expected |
|---|---|---|
| `REF-E1-CONFIG-001` | Valid E1-B configuration | accepted |
| `REF-E1-CONFIG-002` | Framework dependency beyond core/store | architecture test fails |
| `REF-E1-CONFIG-003` | `wow-emmy`, annotations, project, search, service import | rejected |
| `REF-E1-CONFIG-004` | Raw SQL/store connection exposed | rejected |
| `REF-E1-CONFIG-005` | Network/source acquisition path | rejected |
| `REF-E1-CONFIG-006` | Implementation starts with null freeze values | fail |

## 2. Source snapshot and profile

| ID | Case | Expected |
|---|---|---|
| `SOURCE-001` | Valid fixture/candidate/release profile shapes | exact eligibility |
| `SOURCE-002` | Interface/build/flavor/source contradiction | rejected |
| `SOURCE-003` | Floating current/latest/provider HEAD | rejected |
| `SOURCE-004` | Fixture profile marked release | rejected |
| `SOURCE-005` | Missing required file | partition/profile gate fails |
| `SOURCE-006` | Optional file absent | explicit NotApplicable/not selected |
| `SOURCE-007` | Unexpected/unlisted file | recorded and policy applied, never silent |
| `SOURCE-008` | Ignored file without explicit rule/reason | rejected |
| `SOURCE-009` | File digest/length/encoding mismatch | rejected |
| `SOURCE-010` | Absolute/traversal/device/link/case-collision path | rejected |
| `SOURCE-011` | Same logical bytes, different provider | same content identity; distinct provenance |
| `SOURCE-012` | Same provider label, changed bytes | new content identity |
| `SOURCE-013` | Filesystem/thread order randomized | declared semantic order/output unchanged |
| `SOURCE-014` | PTR/live/flavor cross-use | rejected/isolation preserved |
| `SOURCE-015` | Absolute root/mtime/acquired-at change | canonical identity unchanged |
| `SOURCE-016` | Missing license/provenance release gate | candidate, not release |

## 3. Parser compatibility

| ID | Case | Expected |
|---|---|---|
| `PARSER-001` | Exact pinned parser compatibility probe | pass |
| `PARSER-002` | Unpinned/different parser | rejected |
| `PARSER-003` | Span/literal/recovery behavior changed | update blocked/last-known-good retained |
| `PARSER-004` | Fatal parse in required file | file/dependent partitions Failed |
| `PARSER-005` | Recovery node inside registration value | unsupported/no fact guess |
| `PARSER-006` | Recoverable unrelated diagnostic | exact policy/coverage impact |
| `PARSER-007` | Huge/deep/malformed source | bounded rejection |
| `PARSER-008` | Regex fallback extracts fact from invalid source | mutation fails |

## 4. Evaluator supported forms

| ID | Case | Expected |
|---|---|---|
| `EVAL-001` | nil/boolean/string/integer/number literals | exact canonical values |
| `EVAL-002` | nested bounded table constructors | exact raw tree/order/spans |
| `EVAL-003` | local binding and exact field/index access | exact resolution |
| `EVAL-004` | allow-listed constant/reference path | exact value/reference evidence |
| `EVAL-005` | each frozen constant operator | exact result |
| `EVAL-006` | each frozen pure helper | exact result/version evidence |
| `EVAL-007` | known registration call exact receiver/callee/arity | observation emitted |
| `EVAL-008` | source/registration order | exact declared sequence |
| `EVAL-009` | mixed table array/map/duplicate/nil keys | frozen semantics + raw occurrence preservation |
| `EVAL-010` | number edge cases | exact canonical policy or unsupported |
| `EVAL-011` | string escape/encoding edge cases | exact semantic bytes/text |

## 5. Evaluator unsupported/security forms

| ID | Case | Expected |
|---|---|---|
| `EVAL-SEC-001` | unknown global/call/helper/operator | unsupported record, no execution |
| `EVAL-SEC-002` | load/loadstring/dofile/require/package | forbidden/no side effect |
| `EVAL-SEC-003` | os/io/debug/coroutine | forbidden/no side effect |
| `EVAL-SEC-004` | metatable/__index/__call trick | forbidden/no side effect |
| `EVAL-SEC-005` | function/closure/control flow/loop/recursion | unsupported/bounded |
| `EVAL-SEC-006` | huge table/string/number/steps/depth | bounded coverage impact |
| `EVAL-SEC-007` | computed key/call with side effect | unsupported/no side effect |
| `EVAL-SEC-008` | source comment/prompt instruction | no policy/evaluation effect |
| `EVAL-SEC-009` | call named similar to registration/helper | not recognized |
| `EVAL-SEC-010` | filesystem/network/process/client/editor marker attempt | marker absent, security error |
| `EVAL-SEC-011` | binding use-before-def/cycle/invalidated assignment | unsupported/dependent propagation |
| `EVAL-SEC-012` | cancellation mid file/table/registration | no publication/background work |

## 6. Raw observations

| ID | Case | Expected |
|---|---|---|
| `RAW-001` | Canonical value/field-path round-trip | byte/semantic identity |
| `RAW-002` | Missing vs explicit nil vs unknown vs unsupported vs default | distinct states |
| `RAW-003` | Unknown top-level/nested field | preserved with source/value/capability impact |
| `RAW-004` | Known field unsupported value shape | raw preserved, projection absent/partial |
| `RAW-005` | Exact duplicate observations | occurrence evidence preserved, exact normalized dedup allowed |
| `RAW-006` | Conflicting duplicate observations | conflict record, no first/last collapse |
| `RAW-007` | Correction applied | raw before value unchanged |
| `RAW-008` | Schema/normalizer update | prior raw still readable/reprojectable |
| `RAW-009` | JSON-string-only canonical raw representation | architecture test fails |
| `RAW-010` | Raw budget truncation | explicit truncation/coverage downgrade |
| `RAW-011` | Raw manifest/count/digest mismatch | build/store validation fails |

## 7. Normalized facts and links

| ID | Case | Expected |
|---|---|---|
| `NORM-001` | Each supported entity kind | stable exact key/fact |
| `NORM-002` | Ordered parameters/returns/event payload/table fields | order preserved |
| `NORM-003` | Same name different kind/system/owner/signature | distinct entities |
| `NORM-004` | Exact duplicate same identity/value | canonical fact + all evidence |
| `NORM-005` | Same key conflicting value/shape | conflict |
| `NORM-006` | Unresolved named type | explicit unresolved/partial, not any |
| `NORM-007` | Ambiguous exact cross-reference | conflict |
| `NORM-008` | Cross-profile resolution fallback | rejected |
| `NORM-009` | Known restriction/predicate/deprecation/transition fields | exact structured facts |
| `NORM-010` | Unknown restriction/applicability/signature field | raw preserved, dependent blocker |
| `NORM-011` | Prose/name similarity replacement | rejected |
| `NORM-012` | Derived signature/name fact | exact input refs/version |
| `NORM-013` | Fact lacks raw/source/evidence closure | rejected |
| `NORM-014` | Random source/worker/SQL insertion order | same entity/fact/manifests |

## 8. Corrections

| ID | Case | Expected |
|---|---|---|
| `CORR-001` | Exact target/digest/replacement/evidence/reviewer | Applied |
| `CORR-002` | Raw source/value unchanged after apply | pass |
| `CORR-003` | Source file/value/shape digest changed | Expired |
| `CORR-004` | Wrong profile/build | NotApplicable |
| `CORR-005` | Missing evidence/reviewer/invalid operation | Rejected |
| `CORR-006` | Conflicting replacements same target | Conflict |
| `CORR-007` | Dependency order | exact |
| `CORR-008` | Dependency cycle | rejected |
| `CORR-009` | Invalid type/entity/restriction replacement | rejected |
| `CORR-010` | Wildcard/fuzzy/product-name/runtime whitelist correction | rejected |
| `CORR-011` | Auto-update expected digest/best effort | rejected |
| `CORR-012` | Applied mandatory correction restores declared capability | exact tested policy |
| `CORR-013` | Expired/conflict blocks release/negative authority | pass |
| `CORR-014` | Correction set change | ReferenceGeneration changes |
| `CORR-015` | Independent correction order shuffled | same application results/digest |

## 9. Coverage/conflicts/negative authority

| ID | Case | Expected |
|---|---|---|
| `COV-001` | Complete exact positive | Found with evidence/coverage |
| `COV-002` | Complete exact miss | AbsentAuthoritative |
| `COV-003` | Partial relevant partition miss | NotFoundPartial |
| `COV-004` | Failed parser/evaluator/store relevant partition | NotEvaluated/partial failure |
| `COV-005` | Unknown manifest completeness | no authority |
| `COV-006` | Optional NotApplicable partition unrelated | independent query unaffected |
| `COV-007` | Complete ingestion + unresolved conflict | Conflict/no authority |
| `COV-008` | Unknown restriction/signature/applicability field | dependent authority blocked |
| `COV-009` | Expired correction | dependent authority blocked |
| `COV-010` | Store rows missing coverage/conflict records | store/build/view validation fails |
| `COV-011` | Truncated input/raw/list/query | no relevant authority; explicit truncation |
| `COV-012` | Static old profile exact query | scoped authority for that profile |
| `COV-013` | Caller expects current but old profile selected | request/config mismatch, no implicit current |
| `COV-014` | Runtime/hotfix-sensitive claim | NotEvaluated/runtime gap |
| `COV-015` | Empty result directly called absent | mutation fails |
| `COV-016` | Unrelated partial partition | exact independent authority retained |
| `COV-017` | NotEvaluated treated clean/safe | mutation fails |
| `COV-018` | Coverage record/summary dependency/order randomized | same decision/reasons |

## 10. Persistent schema and store plan

| ID | Case | Expected |
|---|---|---|
| `REFSTORE-SCHEMA-001` | Exact reference schema/operation/validation bundle | valid/digest frozen |
| `REFSTORE-SCHEMA-002` | Domain semantics imported into store | architecture test fails |
| `REFSTORE-SCHEMA-003` | Raw SQL/connection exposed | rejected |
| `REFSTORE-SCHEMA-004` | Write all record families | closure valid |
| `REFSTORE-SCHEMA-005` | Orphan/cross-generation raw/fact/member/restriction/correction/coverage/object | rejected |
| `REFSTORE-SCHEMA-006` | Count/digest/manifest mismatch | rejected |
| `REFSTORE-SCHEMA-007` | Missing coverage/conflict/correction application rows | rejected |
| `REFSTORE-SCHEMA-008` | Fuzzy/FTS/vector operation/index | rejected E1 |
| `REFSTORE-SCHEMA-009` | Build operation order randomized | canonical plan/order/digest |
| `REFSTORE-SCHEMA-010` | Source/user string as SQL identifier | rejected/no surface |
| `REFSTORE-SCHEMA-011` | Schema evolution | new bundle/generation, raw preserved |

## 11. Build/publication

| ID | Case | Expected |
|---|---|---|
| `BUILD-001` | Complete fixture/candidate/release build | exact state/eligibility |
| `BUILD-002` | Snapshot/profile preflight failure | no ReferenceGeneration/store plan |
| `BUILD-003` | Independent optional partition partial | useful facts retained, capability explicit |
| `BUILD-004` | Mandatory partition/parser/correction/conflict blocker | no release eligibility |
| `BUILD-005` | Store schema/write/integrity/seal/publication failure | no completed manifest; old active preserved |
| `BUILD-006` | Store published but ReferenceView validation fails | no completed ReferenceData manifest |
| `BUILD-007` | Published inactive generation recovery | exact revalidation required |
| `BUILD-008` | Cancellation each stage/store handoff | no completed publication/background work |
| `BUILD-009` | Build report misses declared input outcome | rejected |
| `BUILD-010` | Store success overrides partial ingestion | mutation fails |
| `BUILD-011` | Annotation/UI graph/search/runtime claim in manifest | rejected |
| `BUILD-012` | Random files/workers/operations/temp/time | same logical IDs/manifests/results |
| `BUILD-013` | Raw SQLite byte equality claimed without store proof | rejected |

## 12. ReferenceView

| ID | Case | Expected |
|---|---|---|
| `VIEW-001` | Exact open profile/generation/store/schema | pass |
| `VIEW-002` | Profile/reference/store/schema mismatch | rejected |
| `VIEW-003` | Active pointer changes | existing view remains exact generation |
| `VIEW-004` | Each exact entity/fact lookup | Found exact |
| `VIEW-005` | Same name different kind/system/owner/signature | correct exact result/no fallback |
| `VIEW-006` | Unique key duplicate rows | Conflict/validation failure, not first row |
| `VIEW-007` | Authoritative negative | exact decision |
| `VIEW-008` | Partial/conflict/NotEvaluated/invalid query | exact result variant |
| `VIEW-009` | Entity Found but one requested field partial | Found + field blocker, not all-complete |
| `VIEW-010` | Raw unknown/unsupported/correction read | bounded exact values/relations |
| `VIEW-011` | Restriction known + unknown/runtime gap | no blanket safe result |
| `VIEW-012` | Explicit transition vs inferred candidate | only explicit fact |
| `VIEW-013` | Bounded list/raw/source detail truncation | explicit, no authority over omitted data |
| `VIEW-014` | SQL row/order/thread/cache/temp changes | same result/order/digest |
| `VIEW-015` | Raw SQL/fuzzy/external/source mutation attempt | no surface/rejected |
| `VIEW-016` | Cache reused across profile/generation/detail/budget | mutation fails |

## 13. Security/privacy

| ID | Case | Expected |
|---|---|---|
| `REF-SEC-001` | Source path/root/link escape | rejected |
| `REF-SEC-002` | Lua/source/repository/client/editor/network/process execution attempt | no side effect/rejected |
| `REF-SEC-003` | Source comment/prompt changes policy | no effect |
| `REF-SEC-004` | Huge source/table/string/value/fact/raw/read output | bounded |
| `REF-SEC-005` | Corrupt/untrusted store/object/raw manifest | rejected |
| `REF-SEC-006` | Error/manifest leaks absolute root/token/private URL/excessive source/raw/runtime Secret | rejected |
| `REF-SEC-007` | Correction/source/user supplies SQL/code | rejected |
| `REF-SEC-008` | Silent repair of corrupt/unknown source/store | rejected |

## 14. Deferred capabilities

| ID | Case | Expected |
|---|---|---|
| `REF-DEFER-001` | Annotation generation/parity output | typed unavailable/owned by wow-annotations |
| `REF-DEFER-002` | Complete UI graph/TOC/XML/function/skeleton | unavailable/deferred E2/E3 |
| `REF-DEFER-003` | Fuzzy search/lineage/replacement inference | unavailable/deferred E4 |
| `REF-DEFER-004` | Runtime current spell/security probe | unavailable/deferred runtime evidence contract |
| `REF-DEFER-005` | Final pack signing/distribution | unavailable/deferred E7 |
| `REF-DEFER-006` | Empty/default success for deferred | prohibited |

## 15. Fixture/checksum freeze

| ID | Case | Expected |
|---|---|---|
| `REF-FIX-001` | Documentation-only null pins/IDs/digests | allowed only while implementation not-started |
| `REF-FIX-002` | Implementation starts with required null | fail |
| `REF-FIX-003` | Example bytes change without checksum update | fail |
| `REF-FIX-004` | Parser/source/schema/correction/store/query vector differs from frozen | fail |
| `REF-FIX-005` | All member/bundle digests and result vectors verify | pass |

## 16. Acceptance gate

E1-B implementation is incomplete until:

```text
exact source/profile/parser/environment/field/correction/schema pins frozen
no arbitrary Lua/source/network/editor/client execution
all declared inputs ingested or explicitly diagnosed/partitioned
raw known/unknown/unsupported/duplicate/conflict metadata round-trips
normalized supported facts/restrictions/predicates/deprecations deterministic and raw-linked
corrections apply/expire/conflict exactly
coverage/conflicts/NotEvaluated gate authoritative absence
static reference schema/operation/validation/build plan publishes through wow-store
immutable ReferenceStore reopens read-only and exact ReferenceView passes all result variants
profiles/generations never mix
no annotations/full UI graph/search/runtime-whitelist/storage-bypass behavior
all checksum vectors frozen and all applicable tests pass
```
