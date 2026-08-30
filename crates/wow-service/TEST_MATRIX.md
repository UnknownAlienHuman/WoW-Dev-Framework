# `wow-service` E0-F test matrix

**Status:** normative executable acceptance matrix for future `status`, `check`, folding, envelope, and CLI integration.

Tests compare structured identities, contexts, findings/outcomes, presentation relations, semantic statuses, canonical bytes, and CLI projections. They must prove the target path executed and fail under deliberate generation/folding/status mutations.

## 1. Configuration and component registry

| ID | Case | Expected |
|---|---|---|
| `SERVICE-CONFIG-001` | Valid E0 configuration/component registry | accepted |
| `SERVICE-CONFIG-002` | Missing project/profile/reference/analyzer/rule identity | rejected |
| `SERVICE-CONFIG-003` | Inactive later component enabled | rejected |
| `SERVICE-CONFIG-004` | Deferred operation marked available | rejected |
| `SERVICE-CONFIG-005` | Invalid budget/output schema | rejected |
| `SERVICE-CONFIG-006` | Rule registry identity differs from rules contract | rejected |
| `SERVICE-CONFIG-007` | Configuration insertion order shuffled | same configuration ID/bytes |
| `SERVICE-CONFIG-008` | Temp path/time/process ID included | excluded or rejected |
| `SERVICE-CONFIG-009` | Implementation starts with null freeze fields | fail |

## 2. Context acquisition

| ID | Case | Expected |
|---|---|---|
| `SERVICE-CTX-001` | Exact generation selector | exact coherent lease |
| `SERVICE-CTX-002` | `CurrentPublished(project)` | resolve once, exact selected ID recorded |
| `SERVICE-CTX-003` | Current pointer changes after acquisition | request remains on acquired immutable snapshot |
| `SERVICE-CTX-004` | Exact generation unavailable | failure; no fallback |
| `SERVICE-CTX-005` | Profile mismatch | failure |
| `SERVICE-CTX-006` | Reference generation mismatch | failure |
| `SERVICE-CTX-007` | Project/analyzer generation mismatch | failure |
| `SERVICE-CTX-008` | Analyzer pin/config mismatch | failure |
| `SERVICE-CTX-009` | Rule registry/fixture policy mismatch | failure |
| `SERVICE-CTX-010` | Last-known-good substituted for required target | rejected |
| `SERVICE-CTX-011` | Stale source/fact scope | rejected |
| `SERVICE-CTX-012` | Nonmandatory fact capability degraded | coherent lease with blockers |
| `SERVICE-CTX-013` | Partial mandatory lease exposed | rejected |
| `SERVICE-CTX-014` | Silent retry/switch to another generation | mutation fails |
| `SERVICE-CTX-015` | Scope order/temp root/worker order changed | same lease ID/bytes |

## 3. Status operation

| ID | Case | Expected |
|---|---|---|
| `STATUS-001` | Healthy configured fixture | `available`, exact identities |
| `STATUS-002` | Ready components | no check/test/runtime/clean claim |
| `STATUS-003` | Analyzer nonmandatory capability failed | component Degraded + blocker |
| `STATUS-004` | Failed target with retained old snapshot | IDs/states separate |
| `STATUS-005` | Last-known-good relabeled target | rejected |
| `STATUS-006` | Reference fixture `release_eligible=false` | visible |
| `STATUS-007` | All deferred operations/capabilities | complete canonical list |
| `STATUS-008` | Deferred operation marked available | rejected |
| `STATUS-009` | Optional detail unavailable | `partial`, no green summary |
| `STATUS-010` | Invalid configuration/registry | `failed`, no coherent status result |
| `STATUS-011` | Detailed capability output budget exceeded | explicit partial/truncation |
| `STATUS-012` | Cancellation | no late status result |
| `STATUS-013` | Component/capability return order shuffled | identical canonical bytes |
| `STATUS-014` | Timestamp/message/temp path change | same result ID/digest |
| `STATUS-015` | Status invokes diagnostics/rules/source reads | mutation rejected |

## 4. Generic finding collection

| ID | Case | Expected |
|---|---|---|
| `CHECK-GENERIC-001` | Complete selected scope | exact generic set |
| `CHECK-GENERIC-002` | Dedicated E0-C generic fixture error | preserved unchanged |
| `CHECK-GENERIC-003` | Generic capability failed | explicit blocker, not empty clean |
| `CHECK-GENERIC-004` | Generic finding from wrong generation | failure |
| `CHECK-GENERIC-005` | Service changes category/severity/message/evidence | rejected |
| `CHECK-GENERIC-006` | Service reruns analyzer diagnostic logic | architecture test fails |
| `CHECK-GENERIC-007` | Generic return order shuffled | canonical raw set unchanged |

## 5. Rule report aggregation

| ID | Case | Expected |
|---|---|---|
| `CHECK-RULES-001` | Valid complete E0-E report | accepted |
| `CHECK-RULES-002` | Report context mismatch | failure |
| `CHECK-RULES-003` | Rule finding dropped | rejected |
| `CHECK-RULES-004` | Clean record dropped | rejected |
| `CHECK-RULES-005` | NotEvaluated record dropped | rejected |
| `CHECK-RULES-006` | Provider failure hidden/downgraded | rejected/fatal E0 |
| `CHECK-RULES-007` | Service reruns blocked rule with weaker gate | rejected |
| `CHECK-RULES-008` | Service reimplements API/Secret rule | architecture test fails |
| `CHECK-RULES-009` | Rule report order shuffled | identical aggregate |

## 6. Raw findings

| ID | Case | Expected |
|---|---|---|
| `CHECK-RAW-001` | Union generic + rule findings | all IDs present |
| `CHECK-RAW-002` | Exact duplicate ID repeated | one canonical raw record/reference |
| `CHECK-RAW-003` | Same message, distinct spans | distinct raw findings |
| `CHECK-RAW-004` | Cross-generation finding | rejected |
| `CHECK-RAW-005` | Finding deleted during folding | mutation fails |
| `CHECK-RAW-006` | Raw finding mutated by service | rejected |
| `CHECK-RAW-007` | Raw finding count replaced with root count | rejected |
| `CHECK-RAW-008` | Raw/source/Secret payload leak | rejected |

## 7. Root-cause folding

| ID | Case | Expected |
|---|---|---|
| `FOLD-001` | API finding + exact same-source generic symptom hint | API root, generic child |
| `FOLD-002` | Same text but different span | independent roots |
| `FOLD-003` | Annotation library failure blocks dependent API/Secret NotEvaluated | blocker root/children |
| `FOLD-004` | Independent generic type and Secret findings | independent roots |
| `FOLD-005` | Exact duplicate structured records | canonical root + duplicate edge, raw preserved |
| `FOLD-006` | Distinct source spans considered duplicates | mutation fails |
| `FOLD-007` | Two valid parents | deterministic one primary, secondary retained |
| `FOLD-008` | First-returned/message-based parent selection | mutation fails |
| `FOLD-009` | Cross-generation edge | rejected |
| `FOLD-010` | Self edge | rejected |
| `FOLD-011` | Directed cycle | rejected |
| `FOLD-012` | Multiple primary parents | rejected |
| `FOLD-013` | Problem record omitted from graph | rejected |
| `FOLD-014` | Input/edge order shuffled | identical graph ID/bytes |
| `FOLD-015` | Message/UI expansion changes | graph identity unchanged |

## 8. Clean check

| ID | Case | Expected |
|---|---|---|
| `CHECK-CLEAN-001` | KnownApi exact found + guarded concat scope | semantic `clean` |
| `CHECK-CLEAN-002` | No generic findings, explicit rule clean/nonapplicable outcomes | clean proof present |
| `CHECK-CLEAN-003` | Empty findings but missing rule outcome | not clean; failure/partial |
| `CHECK-CLEAN-004` | Empty findings with NotEvaluated | partial |
| `CHECK-CLEAN-005` | Empty findings with truncation | partial |
| `CHECK-CLEAN-006` | Clean claim too broad (“project safe”) | rejected |
| `CHECK-CLEAN-007` | Budget/scope complete | required |

## 9. Findings check

| ID | Case | Expected |
|---|---|---|
| `CHECK-FIND-001` | Full closed baseline, complete capabilities | semantic `findings` |
| `CHECK-FIND-002` | Fixed rule counts: API 1, Secret findings 3, Secret clean 1 | exact |
| `CHECK-FIND-003` | Accepted generic fixture finding retained | exact frozen count/family |
| `CHECK-FIND-004` | Optional API generic symptom only if E0-C freezes it | no invention |
| `CHECK-FIND-005` | Advisory rollout findings | still semantic findings |
| `CHECK-FIND-006` | Presentation folding reduces roots | raw finding count unchanged |
| `CHECK-FIND-007` | No NotEvaluated/failure/truncation | findings, not partial |

## 10. Partial check

| ID | Case | Expected |
|---|---|---|
| `CHECK-PARTIAL-001` | Findings + one rule NotEvaluated | semantic `partial`, findings retained |
| `CHECK-PARTIAL-002` | No findings + blocked rule scope | partial, not clean |
| `CHECK-PARTIAL-003` | Broken annotation library blocks dependent rules | partial if coherent independent result exists |
| `CHECK-PARTIAL-004` | Secret control-flow capability failed | partial with exact blocker |
| `CHECK-PARTIAL-005` | Reference exact partition conflict/partial | partial |
| `CHECK-PARTIAL-006` | Explicit output/scope truncation | partial |
| `CHECK-PARTIAL-007` | Provider implementation failure downgraded to partial | rejected in E0 |
| `CHECK-PARTIAL-008` | Deferred unrelated capabilities alone | do not force partial |

## 11. Failed check

| ID | Case | Expected |
|---|---|---|
| `CHECK-FAIL-001` | Invalid request | ServiceFailureResult |
| `CHECK-FAIL-002` | Exact generation unavailable | failure, no fallback |
| `CHECK-FAIL-003` | Context mismatch | failure |
| `CHECK-FAIL-004` | Mandatory component/registry invalid | failure |
| `CHECK-FAIL-005` | Invalid rule execution report | failure |
| `CHECK-FAIL-006` | Invalid presentation graph | failure |
| `CHECK-FAIL-007` | Envelope reference/digest/status invalid | failure |
| `CHECK-FAIL-008` | Security/architecture violation | failure |
| `CHECK-FAIL-009` | Malformed partial check envelope returned instead | rejected |

## 12. Semantic status precedence

| ID | Inputs | Expected |
|---|---|---|
| `STATUS-PREC-001` | mandatory failure + findings | failed |
| `STATUS-PREC-002` | cancellation + partial work | cancelled/no check envelope |
| `STATUS-PREC-003` | NotEvaluated + findings | partial |
| `STATUS-PREC-004` | truncation + no findings | partial |
| `STATUS-PREC-005` | findings only/complete | findings |
| `STATUS-PREC-006` | no findings/blockers, explicit complete outcomes | clean |
| `STATUS-PREC-007` | no findings and no completion proof | status derivation failure |
| `STATUS-PREC-008` | advisory findings interpreted clean | mutation fails |

## 13. Envelopes and canonicalization

| ID | Case | Expected |
|---|---|---|
| `ENVELOPE-001` | Valid clean envelope | validates |
| `ENVELOPE-002` | Valid findings envelope | validates |
| `ENVELOPE-003` | Valid partial envelope | validates |
| `ENVELOPE-004` | Valid failure/cancelled result | validates |
| `ENVELOPE-005` | Unresolved/cross-context reference | rejected |
| `ENVELOPE-006` | Digest mismatch | rejected |
| `ENVELOPE-007` | Current selector without resolved exact generation | rejected |
| `ENVELOPE-008` | Timestamp/temp path/process/thread in canonical identity | mutation fails |
| `ENVELOPE-009` | Lower-layer return order shuffled | identical bytes/ID/digest |
| `ENVELOPE-010` | Message/prose/text render changes only | identity unchanged |
| `ENVELOPE-011` | Root counts substituted for raw counts | rejected |
| `ENVELOPE-012` | Deferred records missing/misordered | rejected |

## 14. Budgets/cancellation

| ID | Case | Expected |
|---|---|---|
| `SERVICE-BUDGET-001` | Context budget exceeded | failure |
| `SERVICE-BUDGET-002` | Scope too large | request failure |
| `SERVICE-BUDGET-003` | Generic/rule stage incomplete | partial/failure explicit, never clean |
| `SERVICE-BUDGET-004` | Presentation graph over budget | partial/failure per explicit policy, no silent edges dropped |
| `SERVICE-BUDGET-005` | Output envelope too large | failure/explicit partial policy; no malformed truncation |
| `SERVICE-CANCEL-001` | Cancel before acquisition | cancelled |
| `SERVICE-CANCEL-002` | Cancel during generic/rules/folding/envelope | no late envelope |
| `SERVICE-CANCEL-003` | Late result after cancel | mutation fails |
| `SERVICE-CANCEL-004` | Background continuation | prohibited |

## 15. Deferred operations

| ID | Request | Expected |
|---|---|---|
| `SERVICE-DEFER-001` | lookup | typed unavailable |
| `SERVICE-DEFER-002` | search | unavailable |
| `SERVICE-DEFER-003` | tree/skeleton/plan | unavailable |
| `SERVICE-DEFER-004` | patch impact/index/runtime review | unavailable |
| `SERVICE-DEFER-005` | LSP/MCP | unavailable |
| `SERVICE-DEFER-006` | release/pack publication | unavailable |
| `SERVICE-DEFER-007` | empty/default success | prohibited |

## 16. Security and architecture

| ID | Case | Expected |
|---|---|---|
| `SERVICE-SEC-001` | Service reads/parses source directly | rejected |
| `SERVICE-SEC-002` | Source/repository code executed | rejected |
| `SERVICE-SEC-003` | Arbitrary filesystem/network/process/editor/client access | rejected |
| `SERVICE-SEC-004` | Service applies remediation/edit | rejected |
| `SERVICE-SEC-005` | Service imports lower internals/raw upstream types | architecture test fails |
| `SERVICE-SEC-006` | Service invokes search/replacement/runtime lane | rejected |
| `SERVICE-SEC-007` | Source comment changes policy | no effect |
| `SERVICE-SEC-008` | Private/raw Secret/local path/token leak | rejected |

## 17. CLI integration

| ID | Case | Expected |
|---|---|---|
| `CLI-001` | `wow status --format json` | canonical status JSON stdout |
| `CLI-002` | `wow check --project ... --generation exact --format json` | canonical service JSON stdout |
| `CLI-003` | `--generation current` | service resolves exact generation; output records it |
| `CLI-004` | clean | exit code 0 |
| `CLI-005` | findings | exit code 1 |
| `CLI-006` | partial | exit code 2 |
| `CLI-007` | request/context/failure | exit code 3 or 4 per CLI contract exact mapping |
| `CLI-008` | cancelled | exit code 130 |
| `CLI-009` | text format | noncanonical projection of same service result |
| `CLI-010` | CLI imports lower crates/recomputes logic | architecture test fails |
| `CLI-011` | stdout contains canonical JSON only in JSON mode | pass |
| `CLI-012` | stderr leaks source/path/token | fail |
| `CLI-013` | deferred command | typed error/exit, no empty success |

## 18. Cross-crate seams

### `SERVICE-SEAM-001`
Reference + project/analyzer/rules exact identities acquire one lease.

### `SERVICE-SEAM-002`
Generic findings consumed unchanged from ProjectView.

### `SERVICE-SEAM-003`
Rule report consumed unchanged and generation-validated.

### `SERVICE-SEAM-004`
API causal hint folds only exact generic symptom; raw both retained.

### `SERVICE-SEAM-005`
Rule NotEvaluated becomes partial with exact blockers.

### `SERVICE-SEAM-006`
Core envelope validation/canonicalization closes all references.

### `SERVICE-SEAM-007`
CLI depends only on service result family.

## 19. Determinism gate

Vary:

```text
component/capability/finding/outcome/root-edge return order
scope input order
worker scheduling
temporary root
message/human render wording
current pointer publication after lease acquisition
```

Require byte-identical:

```text
context lease identity
status result
raw finding set
presentation graph
semantic status
check envelope
failure/cancelled results
CLI JSON stdout
```

## 20. Fixture/checksum freeze

| ID | Case | Expected |
|---|---|---|
| `SERVICE-FIX-001` | Documentation-only null prerequisite/result IDs | allowed only while implementation_state not-started |
| `SERVICE-FIX-002` | Implementation starts with null required value | fail |
| `SERVICE-FIX-003` | Example byte change without checksum update | fail |
| `SERVICE-FIX-004` | Prerequisite component vectors differ | fail |
| `SERVICE-FIX-005` | Status/check/graph/envelope/CLI vectors valid after freeze | pass |

## 21. Acceptance gate

E0-F implementation is incomplete until:

```text
all applicable non-deferred test IDs execute
status has exact state and no false validation claims
check has one coherent context and fixed lower-layer counts/outcomes
raw findings are complete and presentation graph valid/non-destructive
semantic statuses and precedence are exact
failure/cancellation never publish malformed/late envelopes
all deferred operations fail explicitly
CLI consumes service only and exit mappings are exact
security/no-execution/no-mutation/no-search tests pass
randomized-order canonical outputs are byte-identical
all prerequisite/result/checksum vectors are frozen and verified
```
