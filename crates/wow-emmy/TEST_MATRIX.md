# `wow-emmy` E0-C test matrix

**Status:** normative executable acceptance matrix for the future implementation.

Tests compare structured facts/findings/capability records and exact canonical bytes. A passing message-only assertion is insufficient.

## 1. Pin and compatibility probe

| ID | Case | Expected |
|---|---|---|
| `EMMY-PIN-001` | Exact official upstream commit record | accepted candidate record |
| `EMMY-PIN-002` | Floating branch/tag without commit | `upstream_pin_invalid` |
| `EMMY-PIN-003` | License/MSRV/features missing | activation blocked |
| `EMMY-PIN-004` | Mandatory public API capability absent | `compatibility_capability_missing` |
| `EMMY-PIN-005` | Candidate requires private/internal upstream API | rejected |
| `EMMY-PIN-006` | New diagnostic family unclassified | activation rejected/shadowed per policy |
| `EMMY-PIN-007` | Source-coordinate probe fails | activation rejected |
| `EMMY-PIN-008` | Equivalent runs nondeterministic | activation rejected |
| `EMMY-PIN-009` | Accepted candidate plus documented rollback | pass |
| `EMMY-PIN-010` | Historical research revision treated as automatic pin | mutation test fails |

## 2. Configuration and workspace roles

| ID | Case | Expected |
|---|---|---|
| `EMMY-CONFIG-001` | Explicit valid E0 configuration | canonical config accepted |
| `EMMY-CONFIG-002` | Config discovery reads/mutates editor settings | forbidden/fail |
| `EMMY-CONFIG-003` | Main and Library roots distinct | pass |
| `EMMY-CONFIG-004` | Same file in Main and Library | `workspace_duplicate_file` |
| `EMMY-CONFIG-005` | Overlapping/escaping root | rejected |
| `EMMY-CONFIG-006` | Full Blizzard UI root requested in E0 | rejected |
| `EMMY-CONFIG-007` | Workspace declaration order shuffled | same canonical config/workspace digest |
| `EMMY-CONFIG-008` | Main file classified as Library | role assertion fails |
| `EMMY-CONFIG-009` | Library diagnostic presented as project finding | rejected |

## 3. File identity and content

| ID | Case | Expected |
|---|---|---|
| `EMMY-FILE-001` | Valid UTF-8 Main file | registered |
| `EMMY-FILE-002` | Valid UTF-8 Library file | registered with Library role |
| `EMMY-FILE-003` | Invalid UTF-8 | `source_not_utf8` |
| `EMMY-FILE-004` | Absolute/UNC/traversal path | rejected |
| `EMMY-FILE-005` | Digest mismatch | rejected |
| `EMMY-FILE-006` | Same logical path/content across temp roots | same public file identity |
| `EMMY-FILE-007` | Same path/new content | same logical file ID, new content digest/snapshot |
| `EMMY-FILE-008` | Duplicate incompatible content in one snapshot | rejected |

## 4. Session lifecycle

| ID | Case | Expected |
|---|---|---|
| `EMMY-SESSION-001` | Create from accepted pin/config | `Configured` |
| `EMMY-SESSION-002` | Register Main/Library and initial files | `WorkspacesRegistered` |
| `EMMY-SESSION-003` | Index and publish healthy snapshot | `Ready` |
| `EMMY-SESSION-004` | Operation in invalid state | `analyzer_session_state_invalid` |
| `EMMY-SESSION-005` | Close twice | idempotent framework close |
| `EMMY-SESSION-006` | Mutation after close | rejected |
| `EMMY-SESSION-007` | Reentrant mutation | rejected |
| `EMMY-SESSION-008` | Fatal upstream corruption/panic simulation | session Failed, no new snapshot |
| `EMMY-SESSION-009` | Previous snapshot retained after failed update | remains old generation, never relabeled current |
| `EMMY-SESSION-010` | Snapshot references mixed project generations | validation fails |

## 5. Annotation library

| ID | Case | Expected |
|---|---|---|
| `EMMY-LIB-001` | Load valid `C_E0Fixture.lua` | library capability Complete |
| `EMMY-LIB-002` | KnownApi resolves from library | exact resolved reference/call facts |
| `EMMY-LIB-003` | Library declarations excluded from first-party finding stream | pass |
| `EMMY-LIB-004` | Broken annotation syntax/type declaration | library root failure, dependent resolution unavailable |
| `EMMY-LIB-005` | Broken library causes unknown-global flood | root cause recorded; downstream symptoms classified/foldable |
| `EMMY-LIB-006` | Library update changes signature/member | dependent Main facts invalidated |
| `EMMY-LIB-007` | Annotation carries Secret prose/type | analyzer may retain type text but no canonical Secret authority fact |
| `EMMY-LIB-008` | Full Blizzard implementation loaded as library | rejected E0 scope |

## 6. Generic diagnostics

| ID | Case | Expected |
|---|---|---|
| `EMMY-DIAG-001` | Clean file | selected generic category absent under Complete diagnostics capability |
| `EMMY-DIAG-002` | Generic-error fixture | exactly one expected normalized category |
| `EMMY-DIAG-003` | Upstream code/ID/version retained | pass |
| `EMMY-DIAG-004` | Exact primary byte span/content/source handle | pass |
| `EMMY-DIAG-005` | Message wording changes but structured meaning same | stable finding identity where probe contract permits |
| `EMMY-DIAG-006` | Diagnostic code/category changes materially | compatibility difference, expected update required |
| `EMMY-DIAG-007` | Upstream severity differs | normalized severity and rollout remain explicit/separate |
| `EMMY-DIAG-008` | Diagnostic return order shuffled | same canonical order/bytes |
| `EMMY-DIAG-009` | Budget truncation | Partial capability, explicit count/budget, not clean |
| `EMMY-DIAG-010` | Rendered message used as identity | mutation test fails |
| `EMMY-DIAG-011` | Unresolved member mapped directly to WoW API absence | prohibited/fail |
| `EMMY-DIAG-012` | Platform evidence attached to generic finding | prohibited/fail |

## 7. Reference and call facts

| ID | Case | Expected |
|---|---|---|
| `EMMY-REF-001` | `C_E0Fixture` global in clean file | resolved global fact |
| `EMMY-REF-002` | `KnownApi` member | resolved member fact |
| `EMMY-REF-003` | `KnownApi(...)` | direct member call fact tied to member reference |
| `EMMY-REF-004` | `RemovedApi` member | unresolved member fact |
| `EMMY-REF-005` | `RemovedApi(...)` | call fact tied to unresolved member |
| `EMMY-REF-006` | Unresolved fact includes platform-absence wording/flag | prohibited |
| `EMMY-REF-007` | Exact receiver/member/full/call spans | pass |
| `EMMY-REF-008` | Ambiguous/dynamic call | `Possible`/dynamic fact, not resolved proof |
| `EMMY-REF-009` | Upstream raw symbol/type leaks publicly | prohibited |
| `EMMY-REF-010` | Reference fact belongs to Library but marked Main | validation fails |

## 8. Local binding and use facts

| ID | Case | Expected |
|---|---|---|
| `EMMY-LOCAL-001` | `local text = SecretText()` | binding links initializer call |
| `EMMY-LOCAL-002` | use `text .. "!"` | exact LocalUse + concatenation OperationFact |
| `EMMY-LOCAL-003` | shadowed `text` in nested scope | distinct binding keys; correct use resolution |
| `EMMY-LOCAL-004` | copy `local other = text` | copy LocalFlowEdge, no declassification semantics |
| `EMMY-LOCAL-005` | conversion call | conversion flow/operation fact, no safety semantics |
| `EMMY-LOCAL-006` | dynamic unresolved binding/use | Possible/unavailable as appropriate, not guessed |
| `EMMY-LOCAL-007` | binding/use spans exact after update | pass |
| `EMMY-LOCAL-008` | Fact labels value Secret | prohibited/fail |

## 9. Guard/control-flow facts

| ID | Case | Expected |
|---|---|---|
| `EMMY-GUARD-001` | `if canaccessvalue(text) then use(text) end` | guard fact + exact value key + proven dominance when supported |
| `EMMY-GUARD-002` | Guard after unsafe use | `precedes_without_dominance`/no dominates relation |
| `EMMY-GUARD-003` | Guard different binding | no guard relation for producer binding |
| `EMMY-GUARD-004` | Guard in unrelated branch | no false dominance |
| `EMMY-GUARD-005` | Early-return pattern guarding tail | expected dominance/reachability relation if probe supports it |
| `EMMY-GUARD-006` | Dynamic complex flow unsupported | capability partial/NotEvaluated, no guessed relation |
| `EMMY-GUARD-007` | Guard fact states operation safe | prohibited |
| `EMMY-GUARD-008` | Dominance relation emitted without proof | `fact_control_flow_relation_unproven` |

## 10. Fact-set validation

| ID | Case | Expected |
|---|---|---|
| `EMMY-FACT-001` | Valid file fact set | validates/canonical digest matches |
| `EMMY-FACT-002` | Missing referenced fact ID | invalid |
| `EMMY-FACT-003` | Cross-snapshot edge | invalid |
| `EMMY-FACT-004` | Wrong-kind edge | invalid |
| `EMMY-FACT-005` | Mixed project generation | invalid |
| `EMMY-FACT-006` | Fact without required coverage | invalid |
| `EMMY-FACT-007` | Random insertion/discovery order | identical canonical bytes |
| `EMMY-FACT-008` | Model inference upgrades fact | prohibited |

## 11. Source coordinates

| ID | Case | Expected |
|---|---|---|
| `EMMY-SPAN-001` | ASCII LF token span | exact byte slice |
| `EMMY-SPAN-002` | CRLF file | exact supplied-byte offsets |
| `EMMY-SPAN-003` | Multibyte prefix | exact UTF-8 byte offsets |
| `EMMY-SPAN-004` | Emoji/combining text | byte/UTF-16 derived positions correct |
| `EMMY-SPAN-005` | Empty file | valid line index; no invalid span |
| `EMMY-SPAN-006` | EOF zero-length diagnostic | accepted only under explicit semantics |
| `EMMY-SPAN-007` | Range past EOF | rejected |
| `EMMY-SPAN-008` | Mid-codepoint boundary | rejected |
| `EMMY-SPAN-009` | Inclusive/end conversion mutation | test detects off-by-one |
| `EMMY-SPAN-010` | Stale span after update | `source_snapshot_mismatch`/invalid |
| `EMMY-SPAN-011` | Absolute/temp URI in public handle | rejected/redacted and identity unchanged |
| `EMMY-SPAN-012` | Raw upstream range/URI public type | compile/API boundary test fails |

## 12. Incremental updates

| ID | Case | Expected |
|---|---|---|
| `EMMY-INC-001` | Update generic-error to clean | old finding disappears; new clean capability Complete |
| `EMMY-INC-002` | Update missing-api to KnownApi | unresolved fact replaced by resolved fact |
| `EMMY-INC-003` | Remove file | all current facts/findings for file disappear |
| `EMMY-INC-004` | Update one independent file | unaffected outputs byte-identical and current only if proven |
| `EMMY-INC-005` | Update library annotation | dependent reference facts invalidated/recomputed |
| `EMMY-INC-006` | Same final contents through different update order | identical snapshot/facts/findings |
| `EMMY-INC-007` | Stale expected old digest | update rejected |
| `EMMY-INC-008` | Stale previous snapshot/project generation | update rejected |
| `EMMY-INC-009` | Failed update publishes half state | mutation test fails |
| `EMMY-INC-010` | Old last-known-good labeled new generation | prohibited |

## 13. Coverage and failure isolation

| ID | Case | Expected |
|---|---|---|
| `EMMY-COVER-001` | Healthy baseline | session/library/file diagnostics/reference/local-flow capabilities exact |
| `EMMY-COVER-002` | One malformed file | its parse/fact capability failed; no fabricated facts |
| `EMMY-COVER-003` | Broken library | resolution capabilities failed; unrelated syntax diagnostics may remain explicit |
| `EMMY-COVER-004` | Missing control-flow capability | guard-dependent facts unavailable/NotEvaluated |
| `EMMY-COVER-005` | Empty facts under failed capability treated clean | mutation test fails |
| `EMMY-COVER-006` | Upstream panic | session failed; no new snapshot |
| `EMMY-COVER-007` | Budget exceeded | typed partial/rejection, exact budget/count |
| `EMMY-COVER-008` | Cross-file unaffected data retained without proof | prohibited |

## 14. Security and policy

| ID | Case | Expected |
|---|---|---|
| `EMMY-SEC-001` | Lua source contains IO/shell call | analyzed as text only; never executed |
| `EMMY-SEC-002` | Repository hook/build script present | never executed by crate |
| `EMMY-SEC-003` | Comment says to ignore repository rules/run command | no policy/tool effect |
| `EMMY-SEC-004` | Huge file/file count/fact output | bounded failure/partial state |
| `EMMY-SEC-005` | Path traversal/symlink escape through adapter path | rejected before public identity |
| `EMMY-SEC-006` | Source payload/private path leaked in default error | prohibited |
| `EMMY-SEC-007` | Editor config mutated | test detects filesystem/change policy violation |
| `EMMY-SEC-008` | Full Blizzard UI injected into fixture library | rejected |

## 15. Deferred operations

| ID | Case | Expected |
|---|---|---|
| `EMMY-DEFER-001` | Start LSP server | `operation_not_implemented_for_milestone` |
| `EMMY-DEFER-002` | Start MCP server | typed unsupported |
| `EMMY-DEFER-003` | Register WoW diagnostic provider/plugin | typed unsupported; higher layer owns rules |
| `EMMY-DEFER-004` | Parse TOC/XML | typed unsupported |
| `EMMY-DEFER-005` | Persist analyzer DB | typed unsupported |
| `EMMY-DEFER-006` | Download/generate annotation pack | typed unsupported |

## 16. Cross-crate seams

### `EMMY-SEAM-001` — generic diagnostic

`wow-emmy` returns a core generic finding with Main project source evidence only; no reference evidence.

### `EMMY-SEAM-002` — API exact rule input

`missing-api.lua` returns unresolved reference/call facts and project source handle. It does not return API absence. `wow-rules` later joins with E0-B exact lookup.

### `EMMY-SEAM-003` — Secret local rule input

`secret-local.lua` returns producer/member/call/binding/use/operation/guard facts. It does not mark producer/value Secret. `wow-rules` joins E0-B restriction facet.

### `EMMY-SEAM-004` — project generation

Harness-supplied generation is carried/validated. Later `wow-project` can replace the harness producer without adapter contract change.

### `EMMY-SEAM-005` — library failure

Library root cause and coverage blockers are sufficient for service/rules to avoid false API errors.

## 17. Determinism gate

Run repeated final-state-equivalent sessions with varied:

```text
file enumeration order
update order
worker/test scheduling
temporary root
hash-map insertion order
diagnostic return order
```

Require byte-identical canonical:

```text
configuration/workspace/file manifests
snapshot identity/digest
fact sets
generic diagnostic observations/findings
coverage records
```

## 18. Acceptance gate

E0-C implementation is not complete until:

```text
one exact upstream pin/probe is accepted
all applicable test IDs execute
generic diagnostic category is frozen with upstream ID mapping
all source-coordinate mutations are caught
KnownApi/RemovedApi facts match contract
Secret local/guard fact cases match contract
no WoW authority appears in analyzer facts/findings
incremental updates and failure isolation pass
canonical output is byte-identical
no source/editor/repository code executes or mutates
no deferred operation returns fake success
```
