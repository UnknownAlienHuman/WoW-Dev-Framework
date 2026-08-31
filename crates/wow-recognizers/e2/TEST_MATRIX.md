# E2-B recognizer test matrix

**Status:** normative executable acceptance and mutation matrix.

## Configuration and dependency boundaries

| ID | Case | Expected |
|---|---|---|
| `RECOG-CONFIG-001` | Valid core pack/fact/graph profiles | accepted |
| `RECOG-CONFIG-002` | Dependency beyond core/emmy/graph | architecture test fails |
| `RECOG-CONFIG-003` | Direct project/reference/store/rules/search/service/app import | rejected |
| `RECOG-CONFIG-004` | Implicit current/latest profile or pack | rejected |
| `RECOG-CONFIG-005` | Fact/graph/profile/generation mismatch | rejected |
| `RECOG-CONFIG-006` | Calibration/experimental auto-enabled | rejected |
| `RECOG-CONFIG-007` | Repository/addon identity in condition | rejected |
| `RECOG-CONFIG-008` | Raw source/parser fallback | rejected |
| `RECOG-CONFIG-009` | LLM/model output in correctness input | rejected |
| `RECOG-CONFIG-010` | Implementation begins with required null freeze values | fail |

## Pack/schema/compiler

| ID | Case | Expected |
|---|---|---|
| `RECOG-SCHEMA-001` | Valid core pack | parse/validate |
| `RECOG-SCHEMA-002` | Duplicate incompatible pack/rule/capture/output ID | reject |
| `RECOG-SCHEMA-003` | Clause/capture/derivation cycle | reject |
| `RECOG-SCHEMA-004` | Unknown fact or graph kind/attribute | reject |
| `RECOG-SCHEMA-005` | Invalid relation endpoint/direction | reject |
| `RECOG-SCHEMA-006` | Executable Lua/JS/Wasm/native payload | reject |
| `RECOG-SCHEMA-007` | Include/URL/environment/template/plugin | reject |
| `RECOG-SCHEMA-008` | Regex/glob/expression/SQL/shell operator | reject E2 |
| `RECOG-SCHEMA-009` | Undeclared literal convention | reject |
| `RECOG-SCHEMA-010` | Negative clause lacks closed complete scope | reject |
| `RECOG-SCHEMA-011` | Unbounded nesting/join/output cardinality | reject |
| `RECOG-SCHEMA-012` | Missing positive/negative/partial/mutation fixtures | reject default rollout |
| `RECOG-SCHEMA-013` | Shuffled equivalent pack definitions | same semantic plan digest |
| `RECOG-SCHEMA-014` | Physical join/index plan differs, semantics same | allowed; outputs equal |
| `RECOG-SCHEMA-015` | Breaking rule change without version | reject |

## Fact input/adapters

| ID | Case | Expected |
|---|---|---|
| `RECOG-INPUT-001` | Valid normalized Emmy fact bundle | accepted |
| `RECOG-INPUT-002` | Valid normalized TOC/XML/project bundle | accepted |
| `RECOG-INPUT-003` | Stale file/analyzer/project generation | reject |
| `RECOG-INPUT-004` | Invalid source/evidence/coverage reference | reject |
| `RECOG-INPUT-005` | Mixed project/reference generations | reject |
| `RECOG-INPUT-006` | Undeclared cross-partition fact | invisible/reject join |
| `RECOG-INPUT-007` | Unsupported adapter field silently dropped | fail |
| `RECOG-INPUT-008` | Raw AST/source/documentation payload | reject |
| `RECOG-INPUT-009` | Exact duplicate facts | one semantic match; all evidence retained |
| `RECOG-INPUT-010` | Same names in different scopes | distinct fact identities |
| `RECOG-INPUT-011` | Unresolved symbol | no exact-symbol fallback |
| `RECOG-INPUT-012` | Partial capability with empty list | NotEvaluated/Partial, not no-match |
| `RECOG-INPUT-013` | Fact order shuffled | same bundle/matches |
| `RECOG-INPUT-014` | Host path/token/private URL in public fact | reject/redact |
| `RECOG-INPUT-015` | Project adapter infers role from path/repo name | mutation fails |

## Core matcher semantics

| ID | Case | Expected |
|---|---|---|
| `RECOG-MATCH-001` | Exact fact selector/join/predicate | match |
| `RECOG-MATCH-002` | Typed join mismatch | no match/reject plan |
| `RECOG-MATCH-003` | `exists` witness | exact explanation facts retained |
| `RECOG-MATCH-004` | `not_exists` complete closed scope | may evaluate |
| `RECOG-MATCH-005` | `not_exists` partial/conflict/truncated scope | NotEvaluated/Partial |
| `RECOG-MATCH-006` | Source ordering fact | exact ordered match |
| `RECOG-MATCH-007` | Missing ordering fact | no guessed order |
| `RECOG-MATCH-008` | Supplied dominance relation | usable |
| `RECOG-MATCH-009` | No dominance relation | matcher does not build CFG |
| `RECOG-MATCH-010` | Multiple independent valid matches | all retained |
| `RECOG-MATCH-011` | Mutually competing bindings | ambiguity + Possible |
| `RECOG-MATCH-012` | First/last match implementation mutation | fails |
| `RECOG-MATCH-013` | Possible decisive input | output Possible |
| `RECOG-MATCH-014` | Recognizer output marked Proven/Candidate | reject |
| `RECOG-MATCH-015` | Join/output amplification at limit | deterministic bounded result |
| `RECOG-MATCH-016` | Join/output amplification over limit | explicit Partial/truncated |
| `RECOG-MATCH-017` | Cancellation during index/join/output | no complete partition/background work |
| `RECOG-MATCH-018` | Rule reads another unpublished rule output | reject E2 |

## TOC and XML rules

| ID | Case | Expected |
|---|---|---|
| `RECOG-TOC-001` | Exact package/manifest/variant | package graph proposals |
| `RECOG-TOC-002` | Ordered files | loads/loads_before exact |
| `RECOG-TOC-003` | Required dependency | depends_on |
| `RECOG-TOC-004` | Optional dependency | optional_depends_on |
| `RECOG-TOC-005` | Unresolved dependency target | Possible/blocker |
| `RECOG-TOC-006` | LoadOnDemand/bootstrap fact | exact metadata role only |
| `RECOG-TOC-007` | LOD used to infer frame existence | mutation fails |
| `RECOG-TOC-008` | SavedVariables declaration | exact state_root |
| `RECOG-TOC-009` | Same Lua global without TOC declaration | no state_root |
| `RECOG-XML-001` | Template declaration | xml_template proposal |
| `RECOG-XML-002` | Object/frame/region declaration | exact object proposal |
| `RECOG-XML-003` | Exact parent | object parent relation |
| `RECOG-XML-004` | Exact inherits/template | inherits/references_template |
| `RECOG-XML-005` | Unresolved/multiple inherits | Possible/ambiguity |
| `RECOG-XML-006` | Script handler | sets_script relation |
| `RECOG-XML-007` | Embedded XML/Lua code execution attempt | never executed |
| `RECOG-XML-008` | Generic parent relation outside object axis | graph proposal rejected |

## Frame/mixin rules

| ID | Case | Expected |
|---|---|---|
| `RECOG-FRAME-001` | Exact CreateFrame type/name/parent/template | exact proposals |
| `RECOG-FRAME-002` | Dynamic frame name | Possible/no exact name identity |
| `RECOG-FRAME-003` | Dynamic parent/template | Possible/ambiguity |
| `RECOG-FRAME-004` | Variable name used as frame identity | mutation fails |
| `RECOG-FRAME-005` | CreateFromMixins exact mixins | instantiates/mixes_in |
| `RECOG-FRAME-006` | Dynamic mixin list | Possible |
| `RECOG-FRAME-007` | Mixin(target, ...) exact | mixes_in |
| `RECOG-FRAME-008` | Mixin relation treated as inheritance | mutation fails |

## Event/callback rules

| ID | Case | Expected |
|---|---|---|
| `RECOG-EVENT-001` | Frame:RegisterEvent exact literal | native event registration |
| `RECOG-EVENT-002` | RegisterUnitEvent exact varargs | ordered units retained |
| `RECOG-EVENT-003` | Assumed table overload | no invented semantics |
| `RECOG-EVENT-004` | EventRegistry frame-event bridge | distinct native bridge relation |
| `RECOG-EVENT-005` | TriggerEvent + RegisterCallback same exact key | Derived custom producer/subscriber |
| `RECOG-EVENT-006` | RegisterCallback no producer, complete local scope | unresolved registration; no custom/native proof |
| `RECOG-EVENT-007` | RegisterCallback producer scope partial | Possible + blocker |
| `RECOG-EVENT-008` | Multiple producer targets | ambiguity + Possible |
| `RECOG-EVENT-009` | Plain RegisterCallback("PLAYER_LOGIN") | never native event proof |
| `RECOG-EVENT-010` | Event name exists but payload/accessibility unknown | no readability/safety claim |
| `RECOG-EVENT-011` | CVar callback registry | distinct CVar subscription |
| `RECOG-EVENT-012` | Event/callback key dynamic | Possible/no exact event identity |
| `RECOG-EVENT-013` | Producer outside declared scope | not silently joined |
| `RECOG-EVENT-014` | Native/custom conflation mutation | fails |

## Hook/script rules

| ID | Case | Expected |
|---|---|---|
| `RECOG-HOOK-001` | SetScript exact object/name/handler | sets_script |
| `RECOG-HOOK-002` | HookScript exact | hooks with script-posthook kind |
| `RECOG-HOOK-003` | hooksecurefunc global form | hooks exact target |
| `RECOG-HOOK-004` | hooksecurefunc table/method form | hooks exact target |
| `RECOG-HOOK-005` | Dynamic hook target | Possible |
| `RECOG-HOOK-006` | Blizzard global override assignment | not recognized as safe hook |
| `RECOG-HOOK-007` | Hook relation claims taint/combat/protected safety | reject |
| `RECOG-HOOK-008` | Managed/private target inferred safe | reject/no claim |
| `RECOG-HOOK-009` | Handler ownership unresolved | Possible/ambiguity |

## Library and state rules

| ID | Case | Expected |
|---|---|---|
| `RECOG-LIB-001` | LibStub exact library key | requires_library |
| `RECOG-LIB-002` | NewLibrary exact key/version | library declaration relation |
| `RECOG-LIB-003` | Exact reviewed embed structure | embeds_library |
| `RECOG-LIB-004` | `Libs/` path only | no library/embed inference |
| `RECOG-LIB-005` | Dynamic library key | Possible/no exact identity |
| `RECOG-LIB-006` | Upstream repo/license inferred | prohibited |
| `RECOG-STATE-001` | TOC root + resolved Lua root | exact state root link |
| `RECOG-STATE-002` | Literal read chain | state_path + reads_state |
| `RECOG-STATE-003` | Literal write chain | state_path + writes_state |
| `RECOG-STATE-004` | Dynamic suffix | root/exact prefix + Possible suffix |
| `RECOG-STATE-005` | Same name shadowed local | no root relation |
| `RECOG-STATE-006` | SavedVariables contents/runtime value read | unavailable/prohibited |

## Graph outputs and partitions

| ID | Case | Expected |
|---|---|---|
| `RECOG-OUT-001` | Valid entity/relation proposal | graph accepts |
| `RECOG-OUT-002` | Undeclared kind/relation/attribute | reject |
| `RECOG-OUT-003` | Invalid endpoint/direction/key ingredients | reject |
| `RECOG-OUT-004` | Missing input/evidence/coverage derivation | reject |
| `RECOG-OUT-005` | Recognizer constructs final graph ID/generation | architecture fail |
| `RECOG-OUT-006` | Graph validation rejects proposal | explicit report/evaluation defect |
| `RECOG-OUT-007` | Empty complete no-match partition | allowed only full coverage |
| `RECOG-OUT-008` | Partial/truncated output marked complete | reject |
| `RECOG-PART-001` | First producer partition | valid |
| `RECOG-PART-002` | Rule version replacement | stale outputs removed |
| `RECOG-PART-003` | Other producer outputs | unchanged |
| `RECOG-PART-004` | Disable pack/rule | empty replacement + coverage downgrade |
| `RECOG-PART-005` | Failed/cancelled target | prior identity retained, no relabel |
| `RECOG-PART-006` | Stale base/project generation | reject |
| `RECOG-PART-007` | Same final input from different update order | same output partition |

## Coverage/ambiguity/no-match

| ID | Case | Expected |
|---|---|---|
| `RECOG-COV-001` | Complete positive | Matched |
| `RECOG-COV-002` | Complete closed no-match | EvaluatedNoMatch |
| `RECOG-COV-003` | Partial input no-match | NotEvaluated/Partial |
| `RECOG-COV-004` | Failed required capability | NotEvaluated |
| `RECOG-COV-005` | Unsupported fact shape | explicit adapter/rule gap |
| `RECOG-COV-006` | Ambiguous target | Possible + ambiguity |
| `RECOG-COV-007` | Matcher complete, source partial | source gap preserved |
| `RECOG-COV-008` | Truncated report hides blockers | output not complete/default-eligible |
| `RECOG-COV-009` | No new match/evidence | distinct from project-wide absence |
| `RECOG-COV-010` | Pack removal | coverage loss only |

## Mutation/evaluation

| ID | Case | Expected |
|---|---|---|
| `RECOG-MUT-001` | Rename repository/owner/addon metadata | identical outputs |
| `RECOG-MUT-002` | Rename/move paths preserving semantic facts | identical outputs |
| `RECOG-MUT-003` | Rename local identifiers irrelevant to rule | identical outputs |
| `RECOG-MUT-004` | Change decisive public convention literal | precise expected change |
| `RECOG-MUT-005` | Remove decisive structural edge | expected match disappears/changes |
| `RECOG-MUT-006` | Exact target -> dynamic | Derived -> Possible/NotEvaluated |
| `RECOG-MUT-007` | Complete -> Partial coverage | no false no-match/negative |
| `RECOG-MUT-008` | Duplicate/shuffle facts | stable results/evidence retained |
| `RECOG-MUT-009` | Positive-only corpus | promotion rejected |
| `RECOG-MUT-010` | Structurally similar near-negative | no false positive |
| `RECOG-MUT-011` | Hidden repository condition | detected/fails |
| `RECOG-MUT-012` | Unknown/unlabeled omitted from metrics | report invalid |
| `RECOG-MUT-013` | Graph proposal rejection omitted | report invalid |
| `RECOG-MUT-014` | Corpus label changed to make green | review/version required |
| `RECOG-MUT-015` | Frozen thresholds pass/fail | exact promotion outcome |

## Security and determinism

| ID | Case | Expected |
|---|---|---|
| `RECOG-SEC-001` | Huge/deep pack | bounded reject |
| `RECOG-SEC-002` | Cartesian join/fanout bomb | deterministic limit |
| `RECOG-SEC-003` | Oversized strings/evidence/output | bounded reject/Partial |
| `RECOG-SEC-004` | Prompt/source docs as rule instructions | data only/reject |
| `RECOG-SEC-005` | Private path/token/raw source leak | reject/redact |
| `RECOG-SEC-006` | Cross-universe candidate identity collision | reject |
| `RECOG-SEC-007` | Filesystem/network/process/editor/database access | absent |
| `RECOG-SEC-008` | Source/generated code execution | absent |
| `RECOG-SEC-009` | Unlimited/overflow budget | reject |
| `RECOG-SEC-010` | Cancellation each phase | bounded stop/no complete partition |
| `RECOG-DET-001` | 1/2/N workers | byte-identical outputs |
| `RECOG-DET-002` | Hash/input/rule serialization shuffle | byte-identical outputs |
| `RECOG-DET-003` | Clock/temp/row/process IDs | excluded |
| `RECOG-DET-004` | Repeated corpus evaluation | same semantic report |
| `RECOG-FIX-001` | Null pins while not-started | allowed |
| `RECOG-FIX-002` | First Rust commit with required nulls | fail |
| `RECOG-FIX-003` | Member bytes changed without checksum update | fail |
| `RECOG-FIX-004` | All pins/vectors/checksums frozen | pass |

## Deferred boundaries

| ID | Case | Expected |
|---|---|---|
| `RECOG-DEFER-001` | Named framework calibration pack | unavailable E2-B |
| `RECOG-DEFER-002` | Lifecycle/module/plugin/style/element heuristics | unavailable E2-B |
| `RECOG-DEFER-003` | Secret guard/sink diagnostics | owned by later recognizer/rules scope |
| `RECOG-DEFER-004` | Runtime observations | unavailable |
| `RECOG-DEFER-005` | Search/semantic/Codebase Memory candidates | unavailable |
| `RECOG-DEFER-006` | Empty/default success | prohibited |

## Acceptance gate

E2-B is incomplete until all nondeferred cases execute, graph proposals validate, every rule has positive/near-negative/partial/dynamic/mutation coverage, repository/path/name overfitting is absent, native/custom signal systems remain distinct, no structural hook is labeled safe, SavedVariables roots remain TOC-authoritative, producer replacement is atomic, and all canonical outputs are deterministic and bounded.
