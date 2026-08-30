# `wow-rules` E0-E test matrix

**Status:** normative executable acceptance matrix for the future two-rule implementation.

Tests compare structured rule descriptors, contexts, gates, outcomes, findings, evidence, root causes, remediation, and canonical bytes. They must prove the provider path executed and fail under deliberate authority/guard/coverage mutations.

## 1. Registry and descriptors

| ID | Case | Expected |
|---|---|---|
| `RULES-REG-001` | Registry contains exactly API and Secret E0 descriptors | valid |
| `RULES-REG-002` | Duplicate RuleId/version | rejected |
| `RULES-REG-003` | Incompatible active versions | rejected |
| `RULES-REG-004` | Descriptor missing capabilities/budgets/fixtures | rejected |
| `RULES-REG-005` | Planned non-E0 rule enabled | rejected |
| `RULES-REG-006` | Graph-dependent rule while graph inactive | rejected |
| `RULES-REG-007` | Severity error + rollout advisory | preserved independently |
| `RULES-REG-008` | Provider changes rollout silently | rejected |
| `RULES-REG-009` | Registry insertion order shuffled | identical registry ID/bytes |
| `RULES-REG-010` | Fixture policy mismatched profile/rule version | rejected |

## 2. Context coherence

| ID | Case | Expected |
|---|---|---|
| `RULES-CTX-001` | Matching profile/reference/project/analyzer/registry context | pass |
| `RULES-CTX-002` | Profile mismatch | context error |
| `RULES-CTX-003` | Reference generation mismatch | context error |
| `RULES-CTX-004` | Project generation mismatch | context error |
| `RULES-CTX-005` | Analyzer snapshot mismatch | context error |
| `RULES-CTX-006` | Stale project SourceHandle/digest | context error |
| `RULES-CTX-007` | Stale reference lookup/facet | context error/block according to exact record |
| `RULES-CTX-008` | Last-known-good substituted for requested target | rejected |
| `RULES-CTX-009` | Executor retries another implicit generation | prohibited |
| `RULES-CTX-010` | Invalid budget/cancellation state | rejected |

## 3. Provider selection and execution

| ID | Case | Expected |
|---|---|---|
| `RULES-EXEC-001` | Request both E0 rules | canonical selection/order |
| `RULES-EXEC-002` | Request one E0 rule | only requested/applicable provider |
| `RULES-EXEC-003` | Nonapplicable ordinary symbol scope | skipped-nonapplicable, not clean/NotEvaluated |
| `RULES-EXEC-004` | Capability blocked scope | NotEvaluated before provider body |
| `RULES-EXEC-005` | Provider attempts IO/mutation | fail security contract |
| `RULES-EXEC-006` | Provider hidden ordering dependency | mutation test fails |
| `RULES-EXEC-007` | Cancellation before provider | Cancelled, no outcome |
| `RULES-EXEC-008` | Cancellation during canonical scopes | no late/background output |
| `RULES-EXEC-009` | Scope/output budget exceeded | explicit blocker/failure, never clean |
| `RULES-EXEC-010` | Provider/fact/coverage input order shuffled | identical execution report/outcomes |

## 4. Capability gating — universal

| ID | Case | Expected |
|---|---|---|
| `RULES-GATE-001` | All exact required capabilities Complete | Runnable |
| `RULES-GATE-002` | Required partition Partial | NotEvaluated |
| `RULES-GATE-003` | Required partition Failed | NotEvaluated |
| `RULES-GATE-004` | Required partition Unknown | NotEvaluated |
| `RULES-GATE-005` | Required upstream NotEvaluated | NotEvaluated |
| `RULES-GATE-006` | Relevant conflict present despite Complete ingestion | NotEvaluated |
| `RULES-GATE-007` | Input/scope truncated | NotEvaluated/partial report, not clean |
| `RULES-GATE-008` | Unrelated broader partition partial, exact positive fact complete | rule can evaluate with warning/context retained |
| `RULES-GATE-009` | Empty fact list under failed capability | not clean |
| `RULES-GATE-010` | Provider uses private `all_complete` shortcut ignoring exact partition | mutation test fails |
| `RULES-GATE-011` | Context mismatch downgraded to ordinary NotEvaluated | mutation test fails |
| `RULES-GATE-012` | NotEvaluated has no exact blocker | invalid |

## 5. API rule — found and absence

| ID | Case | Expected |
|---|---|---|
| `API-001` | KnownApi direct resolved/use + exact found | no missing finding; clean/nonapplicable according to selected unresolved-use scope |
| `API-002` | RemovedApi unresolved direct call + authoritative_absent | exactly one `wow.api.exists` finding |
| `API-003` | RemovedApi absent_without_authority / partial coverage | NotEvaluated |
| `API-004` | RemovedApi reference conflict | NotEvaluated |
| `API-005` | Reference capability unavailable | NotEvaluated |
| `API-006` | Profile mismatch | context error |
| `API-007` | Annotation library failure prevents exact project reference fact | NotEvaluated |
| `API-008` | Dynamic/computed member | NotEvaluated/nonapplicable, no guessed query |
| `API-009` | Ordinary unresolved local/global outside API namespace | nonapplicable |
| `API-010` | Case-different `KnownAPI` exact absent | authoritative finding only if exact selected partition proves absence; no correction |
| `API-011` | Unresolved fact alone, no ReferenceView lookup | no finding |
| `API-012` | Empty reference result but no authority decision | no finding; NotEvaluated |
| `API-013` | All-global-summary Complete but exact partition partial | NotEvaluated |
| `API-014` | Exact entity found while broader system partial | narrow existence clean allowed with partial warning retained |

## 6. API finding contract

| ID | Case | Expected |
|---|---|---|
| `API-FIND-001` | Primary span exact member name | pass |
| `API-FIND-002` | Whole-file span despite exact member span | rejected |
| `API-FIND-003` | Project source used as platform evidence | rejected |
| `API-FIND-004` | Fabricated absent reference source handle | rejected |
| `API-FIND-005` | Reference authority/coverage/query IDs retained | pass |
| `API-FIND-006` | Finding identity message-based | mutation test fails |
| `API-FIND-007` | Same canonical observation duplicated | one finding |
| `API-FIND-008` | Distinct source spans for same missing API | distinct findings |
| `API-FIND-009` | Replacement candidate inferred | rejected |
| `API-FIND-010` | Exact edit emitted | rejected |
| `API-FIND-011` | Remediation plan_only with no replacement | pass |
| `API-FIND-012` | Severity/rollout preserved error/advisory | pass |
| `API-FIND-013` | API clean record claims signature/security correctness | rejected |

## 7. API generic causal hints

| ID | Case | Expected |
|---|---|---|
| `API-CAUSE-001` | Same generation/file/span/reference fact and compatible generic symptom | causal hint allowed |
| `API-CAUSE-002` | Generic symptom different span/file | no hint |
| `API-CAUSE-003` | Generic finding old generation | no hint/context failure |
| `API-CAUSE-004` | API miss nonauthoritative | no causal API root finding/hint |
| `API-CAUSE-005` | Hint built by message similarity only | rejected |
| `API-CAUSE-006` | Rule suppresses generic finding | rejected; service owns folding |

## 8. Secret rule — core fixture cases

| ID | Case | Expected |
|---|---|---|
| `SECRET-001` | Unsafe direct concatenation, exact facet, no guard | exactly one finding |
| `SECRET-002` | Exact-value accepted guard dominates concat | EvaluatedClean |
| `SECRET-003` | Guard after use | finding with `after_use` |
| `SECRET-004` | Guard on another value | finding with `different_value` |
| `SECRET-005` | Guard in unrelated/non-dominating branch | finding with `non_dominating` |
| `SECRET-006` | Copy then concat, exact flow retained | finding without dominating guard |
| `SECRET-007` | Conversion/format then supported concat/use, exact flow retained | no declassification; finding/continued flow as fixture defines |
| `SECRET-008` | Guard/value binding identity unresolved | NotEvaluated |
| `SECRET-009` | Shadowed same-name binding | exact binding keys prevent false guard |
| `SECRET-010` | Producer unresolved/ambiguous/dynamic | NotEvaluated |
| `SECRET-011` | Unsupported operation kind | NotEvaluated/nonapplicable per scope |
| `SECRET-012` | Interprocedural/dynamic callback/container flow | NotEvaluated |

## 9. Secret facet/reference gating

| ID | Case | Expected |
|---|---|---|
| `SECRET-FACET-001` | Exact unconflicted secret.return return 1 | usable |
| `SECRET-FACET-002` | Facet conflict with Complete ingestion | NotEvaluated |
| `SECRET-FACET-003` | Facet partition Partial/Failed/Unknown | NotEvaluated |
| `SECRET-FACET-004` | Facet subject/return position mismatch | NotEvaluated/input invalid |
| `SECRET-FACET-005` | Annotation/name/type only, no facet | no Secret finding; NotEvaluated/ordinary according to authority |
| `SECRET-FACET-006` | Authoritative no matching facet for ordinary producer | narrow clean allowed |
| `SECRET-FACET-007` | No matching facet without negative authority | NotEvaluated |
| `SECRET-FACET-008` | Conditional/runtime facet unsupported by fixture | NotEvaluated |
| `SECRET-FACET-009` | Facet from another profile/reference generation | context error |
| `SECRET-FACET-010` | Static spell/value whitelist used instead of facet | rejected |

## 10. Secret guard/control-flow gating

| ID | Case | Expected |
|---|---|---|
| `SECRET-GUARD-001` | Fixture policy matches and exact access_single fact | applicable |
| `SECRET-GUARD-002` | Fixture policy missing/mismatch | NotEvaluated/context error |
| `SECRET-GUARD-003` | Guard variable name same but binding key different | not applicable guard |
| `SECRET-GUARD-004` | Guard after use accepted by mutation | test fails |
| `SECRET-GUARD-005` | Guard without proven dominance accepted | test fails |
| `SECRET-GUARD-006` | Control-flow capability Partial | NotEvaluated |
| `SECRET-GUARD-007` | Same exact value + proven dominance | clean |
| `SECRET-GUARD-008` | Dynamic/ambiguous guard call | NotEvaluated |
| `SECRET-GUARD-009` | `issecretvalue`/other predicate substituted without policy | not accepted |
| `SECRET-GUARD-010` | Copy/conversion treated as declassification | mutation test fails |

## 11. Secret finding and clean contract

| ID | Case | Expected |
|---|---|---|
| `SECRET-OUT-001` | Primary span exact operation | pass |
| `SECRET-OUT-002` | Producer/facet/flow/guard evidence retained separately | pass |
| `SECRET-OUT-003` | Finding leaks raw produced value/source payload | rejected |
| `SECRET-OUT-004` | Finding claims runtime/general Secret safety | rejected |
| `SECRET-OUT-005` | Clean record exact guarded fixture scope/capabilities | pass |
| `SECRET-OUT-006` | Clean claim broadened to all Secret uses/runtime | rejected |
| `SECRET-OUT-007` | Automatic guard insertion/edit | rejected |
| `SECRET-OUT-008` | Remediation plan_only with static and runtime follow-up | pass |
| `SECRET-OUT-009` | Same operation duplicate facts | one finding |
| `SECRET-OUT-010` | Distinct operation spans | distinct findings |
| `SECRET-OUT-011` | Finding under facet/control-flow partial coverage | rejected |

## 12. Outcome exclusivity

| ID | Case | Expected |
|---|---|---|
| `RULES-OUT-001` | Findings outcome and clean together | invalid |
| `RULES-OUT-002` | Findings and NotEvaluated together same scope | invalid |
| `RULES-OUT-003` | Empty findings with no clean/NotEvaluated | invalid |
| `RULES-OUT-004` | Clean without examined scope/input/coverage/budget | invalid |
| `RULES-OUT-005` | NotEvaluated without blocker | invalid |
| `RULES-OUT-006` | Failed provider leaks partial finding | invalid |
| `RULES-OUT-007` | Cancelled provider publishes late outcome | invalid |
| `RULES-OUT-008` | Canonical scope has exactly one primary outcome | pass |

## 13. Evidence and provenance

| ID | Case | Expected |
|---|---|---|
| `RULES-EVID-001` | Project/reference/derivation evidence independent | pass |
| `RULES-EVID-002` | Evidence IDs resolve in same context | pass |
| `RULES-EVID-003` | Cross-generation evidence | rejected |
| `RULES-EVID-004` | Candidate/model/external evidence upgraded to Proven | rejected |
| `RULES-EVID-005` | Conflict/coverage IDs dropped after evaluation | rejected |
| `RULES-EVID-006` | Absent query authority represented without fake source | pass |
| `RULES-EVID-007` | Rule creates new platform observation instead of derived evidence | rejected |
| `RULES-EVID-008` | Related evidence over budget/truncated silently | rejected/explicit partial |

## 14. Root cause and ordering

| ID | Case | Expected |
|---|---|---|
| `RULES-ROOT-001` | Structured API root key deterministic | pass |
| `RULES-ROOT-002` | Structured Secret root key deterministic | pass |
| `RULES-ROOT-003` | Root key includes rendered message/timestamp/temp path | mutation test fails |
| `RULES-ROOT-004` | Provider outcome return order shuffled | identical canonical report |
| `RULES-ROOT-005` | Fact/coverage/conflict insertion order shuffled | identical outcomes/IDs |
| `RULES-ROOT-006` | Distinct source uses improperly deduplicated | test fails |
| `RULES-ROOT-007` | Final stream folded/suppressed in rule crate | rejected |

## 15. Remediation

| ID | Case | Expected |
|---|---|---|
| `RULES-REM-001` | API plan_only without candidate/edit | pass |
| `RULES-REM-002` | Secret plan_only without auto-guard edit | pass |
| `RULES-REM-003` | E0 exact_edit | rejected |
| `RULES-REM-004` | validated_recipe in E0 | rejected unless contract revised |
| `RULES-REM-005` | Fuzzy/semantic/external replacement presented as fact | rejected |
| `RULES-REM-006` | Runtime claim without runtime evidence | rejected |
| `RULES-REM-007` | Required post-checks structured | pass |
| `RULES-REM-008` | Source mutation attempted | rejected |

## 16. Budgets/cancellation/security

| ID | Case | Expected |
|---|---|---|
| `RULES-SEC-001` | Provider attempts file/database/network IO | rejected |
| `RULES-SEC-002` | Provider attempts process/shell/editor/client access | rejected |
| `RULES-SEC-003` | Provider executes source/repository code | rejected |
| `RULES-SEC-004` | Source comment tries to change policy | no effect |
| `RULES-SEC-005` | Raw Secret-capable value/local path/token leaked | rejected |
| `RULES-SEC-006` | Scope/facts/lookups/output over budget | explicit NotEvaluated/Failed, never clean |
| `RULES-SEC-007` | Cancellation | no late/background result |
| `RULES-SEC-008` | Hidden cross-request mutable cache changes result | mutation test fails |

## 17. Deferred operations/rules

| ID | Case | Expected |
|---|---|---|
| `RULES-DEFER-001` | `wow.api.deprecated` | typed unavailable |
| `RULES-DEFER-002` | `wow.api.arguments` | unavailable |
| `RULES-DEFER-003` | event/widget/load/TOC rules | unavailable |
| `RULES-DEFER-004` | overlay/hook/framework duplicate rules | unavailable |
| `RULES-DEFER-005` | graph-dependent provider | unavailable |
| `RULES-DEFER-006` | autofix application | unavailable |
| `RULES-DEFER-007` | runtime verification | unavailable |
| `RULES-DEFER-008` | search/replacement lane | unavailable |

No default/empty success.

## 18. Cross-crate seams

### `RULES-SEAM-001` — API exact join

Project unresolved fact/source plus independent ReferenceView authoritative absence -> one finding.

### `RULES-SEAM-002` — API non-authoritative block

Same project fact plus partial/conflicted reference -> NotEvaluated.

### `RULES-SEAM-003` — Secret exact join

Project producer/binding/use/operation facts plus exact reference facet -> evaluation.

### `RULES-SEAM-004` — Guard join

Project exact-value guard/dominance plus fixture policy -> guarded clean; after/different value -> finding.

### `RULES-SEAM-005` — Generic symptom

Rule emits optional proven causal hint; service retains/folds final stream.

### `RULES-SEAM-006` — Service orchestration

Service acquires coherent reference/project context, calls registry/executor, and builds final envelope; rules do not select/retry snapshots.

## 19. Determinism gate

Run equivalent final inputs with varied:

```text
provider selection input order
scope/file order
fact/coverage/conflict order
reference result internal order
generic finding order
worker scheduling
temporary root/message wording
```

Require byte-identical:

```text
registry/descriptors
evaluation IDs
findings/clean/NotEvaluated records
root-cause/causal hints
remediation
execution coverage/report
```

## 20. Fixture/checksum freeze

| ID | Case | Expected |
|---|---|---|
| `RULES-FIX-001` | Documentation-only null prerequisite/output IDs | allowed only while implementation_state not-started |
| `RULES-FIX-002` | Implementation starts with null required ID/digest | fail |
| `RULES-FIX-003` | Fixture byte changes without checksum update | fail |
| `RULES-FIX-004` | Rule input facts/lookups differ from frozen E0-B/C/D examples | fail |
| `RULES-FIX-005` | Finding/evaluation/root-cause vectors validate after freeze | pass |

## 21. Acceptance gate

E0-E implementation is not complete until:

```text
only the two E0 descriptors are active
all applicable non-deferred IDs execute
API finding occurs only under exact authoritative absence
all API partial/conflict/profile/library cases block correctly
Secret unsafe/guarded/after/different/copy/conversion cases classify correctly
no coverage/authority/guard mutation bypasses the gate
project/reference/derivation evidence remain separate
findings/clean/NotEvaluated are mutually exclusive and complete
all remediation is plan_only and no edits/search/runtime claims occur
canonical outputs are byte-identical under randomized order
security/no-IO/no-mutation/no-source-execution cases pass
all prerequisite/input/output/checksum IDs are frozen and verified
```
