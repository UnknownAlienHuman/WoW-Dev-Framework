# `wow-annotations` E1-C test matrix

**Status:** normative executable acceptance matrix for semantic projection, type lowering, rendering, sanitization, source maps, loss, parity, consumer probes, and deterministic artifacts.

Tests inspect structured semantic/file/map/loss/parity/probe IDs and must fail under deliberate silent-loss, injection, cross-generation, oracle-overwrite, or configuration-mutation changes.

## 1. Configuration/dependency/context

| ID | Case | Expected |
|---|---|---|
| `ANN-CONFIG-001` | Valid E1-C build request | accepted |
| `ANN-CONFIG-002` | Framework dependency beyond core/reference | architecture test fails |
| `ANN-CONFIG-003` | Store/analyzer/project/search/service/app import | rejected |
| `ANN-CONFIG-004` | Implicit current/layout/consumer profile | rejected |
| `ANN-CONFIG-005` | ReferenceView/profile/reference generation mismatch | rejected |
| `ANN-CONFIG-006` | Mixed generations in model/files/maps/parity | rejected |
| `ANN-CONFIG-007` | Required reference capability unavailable | status/loss/NotEvaluated, no guess |
| `ANN-CONFIG-008` | Implementation starts with required null freeze values | fail |

## 2. Semantic model

| ID | Case | Expected |
|---|---|---|
| `SEM-001` | Every active module kind | stable module ID/ownership |
| `SEM-002` | Function/method declarations | exact owner/signature/member order |
| `SEM-003` | Structure/table/class/field declarations | exact kind/fields/types |
| `SEM-004` | Events/payloads | exact frozen semantic strategy |
| `SEM-005` | Enums/values/CVars | exact values/metadata |
| `SEM-006` | Widgets/script objects/methods | exact receiver ownership |
| `SEM-007` | Dialect/globals/nominal Secret declarations | exact profile-bound set |
| `SEM-008` | Same name different kind/system/owner/signature | distinct declarations |
| `SEM-009` | Exact duplicate source fact | one declaration + all reference links |
| `SEM-010` | Reference Conflict | no declaration selected; conflict status/loss |
| `SEM-011` | Found declaration with one partial member | parent exact/field partial preserved |
| `SEM-012` | Selected input lacks projection status | model validation fails |
| `SEM-013` | Semantic output lacks reference/derivation | validation fails |
| `SEM-014` | Invalid ownership/member order/type graph | rejected |
| `SEM-015` | Layout/profile changes | semantic IDs unchanged when semantics equal |
| `SEM-016` | Random ReferenceView row/query/worker order | same model/order/digests |

## 3. Type lowering

| ID | Case | Expected |
|---|---|---|
| `TYPE-001` | nil/boolean/integer/number/string | exact profile mapping |
| `TYPE-002` | literal types | exact or explicit base+loss |
| `TYPE-003` | named/alias/recursive named types | exact links/cycles policy |
| `TYPE-004` | arrays/maps/collections | exact source-driven form |
| `TYPE-005` | tuples/multiple returns | distinction/order preserved |
| `TYPE-006` | unions/nil variants | exact variants/order/canonical duplicates |
| `TYPE-007` | callbacks/functions/receiver/variadic | exact shape or loss |
| `TYPE-008` | optional vs nullable vs missing vs default | distinct semantic state |
| `TYPE-009` | enum strategies | exact values or explicit loss |
| `TYPE-010` | unresolved/ambiguous/cross-profile named target | Unsupported/Conflict/rejected, no any |
| `TYPE-011` | explicit source any | any allowed with source link |
| `TYPE-012` | unsupported input silently any/unknown/omitted | mutation fails |
| `TYPE-013` | contextual/conditional Secret | exact sidecar/union/NotEvaluated per profile |
| `TYPE-014` | unknown restriction/runtime gap | no ordinary exact safe type |
| `TYPE-015` | nominal Secret type runtime method/declassification | rejected |
| `TYPE-016` | invalid/reserved identifier target | exact safe form or loss/Unsupported |
| `TYPE-017` | union/type depth/alias budget | bounded explicit status |
| `TYPE-018` | EmmyLua/LuaLS type interpretation difference | consumer-specific gate/profile |
| `TYPE-019` | Documentation prose tries to refine type | ignored as semantics |
| `TYPE-020` | Render profile changes syntax only | structural type IDs unchanged |

## 4. Dialect/globals

| ID | Case | Expected |
|---|---|---|
| `DIALECT-001` | Allowed standard globals | exact declarations |
| `DIALECT-002` | Removed/unavailable standard globals | not emitted in active profile |
| `DIALECT-003` | Blizzard namespaces/globals | exact source-confirmed declarations |
| `DIALECT-004` | require-like/nonstandard symbols absent/present | exact profile behavior |
| `DIALECT-005` | Restricted/private/conditional globals | sidecar/loss, no blanket accessibility |
| `DIALECT-006` | Installed addon/project/editor global | not emitted |
| `DIALECT-007` | Retail/PTR/Classic/historical union | rejected |
| `DIALECT-008` | Nominal Secret module | no runtime constructors/methods/whitelist |
| `DIALECT-009` | Access predicate declaration | no guard/runtime dominance claim |
| `DIALECT-010` | Editor settings/globals/library mutation | none |

## 5. Layout/rendering

| ID | Case | Expected |
|---|---|---|
| `RENDER-001` | Every active declaration template | valid inert syntax |
| `RENDER-002` | Ketho-compatible layout profile | exact frozen topology |
| `RENDER-003` | Missing ReferenceView FrameXML capability | no placeholder complete FrameXML output |
| `RENDER-004` | Path traversal/absolute/device/reserved/case collision | rejected |
| `RENDER-005` | Distinct semantic names map same file/symbol | deterministic safe split or rejected/loss |
| `RENDER-006` | Keyword/invalid/arbitrary string names | exact safe form or Unsupported |
| `RENDER-007` | String/numeric/literal escaping | consumer parse round-trip |
| `RENDER-008` | Function/method stubs | no source-provided body/side effects |
| `RENDER-009` | Class/alias/field/enum/event/CVar/widget forms | exact consumer-tested syntax |
| `RENDER-010` | Forward/recursive type ordering | deterministic and consumer-valid |
| `RENDER-011` | Size-based file splitting | deterministic ID-derived boundaries |
| `RENDER-012` | UTF-8/LF/final newline/no trailing whitespace | pass |
| `RENDER-013` | Timestamp/host/temp/provider/source path | absent from canonical files |
| `RENDER-014` | External formatter changes output | no correctness dependency/fixture fails |
| `RENDER-015` | 1/2/N workers/input serialization shuffle | byte-identical files/manifests |
| `RENDER-016` | Full Blizzard implementation/source body or addon TOC packaging | prohibited |

## 6. Documentation/identifier/source injection security

| ID | Case | Expected |
|---|---|---|
| `SAN-001` | Source doc line `---@class`/`---@diagnostic`/`---@meta` | neutralized; no directive |
| `SAN-002` | Leading whitespace/control/Unicode directive variant | neutralized |
| `SAN-003` | Short/long comment/string terminators | escaped/transformed |
| `SAN-004` | Quotes/backslashes/newlines/NUL/invalid UTF-8 | safe/rejected with loss |
| `SAN-005` | Source code/load/require/os/io/debug/metatable text | documentation only, no code |
| `SAN-006` | Prompt/agent/tool instruction in docs | treated as source data only |
| `SAN-007` | Path separators/traversal/device strings in names | cannot create path |
| `SAN-008` | Unicode normalization/confusable/case collision | exact policy/collision detection |
| `SAN-009` | Huge docs/name/type graph | bounded explicit loss/rejection |
| `SAN-010` | Post-render generated code shape outside allow-list | rejected |
| `SAN-011` | Extra declaration/global/file/module created by source text | mutation fails |
| `SAN-012` | Private root/token/private URL/excessive raw source leak | rejected |

## 7. Source maps

| ID | Case | Expected |
|---|---|---|
| `MAP-001` | Module/declaration/member/type/doc/restriction fragments | exact generated/reference links |
| `MAP-002` | UTF-8 byte/line span convention | exact validated ranges |
| `MAP-003` | File bytes/digest changed after map | map invalid/stale |
| `MAP-004` | Material fragment lacks map/derivation | validation fails |
| `MAP-005` | Generated safe alias maps to exact source/logical identity | bidirectional exact map |
| `MAP-006` | Cross-artifact/profile/generation map entry | rejected |
| `MAP-007` | Overlap/nesting outside kind policy | rejected |
| `MAP-008` | Full source body required for map | false; handles suffice |
| `MAP-009` | Worker/order changes | same map IDs/spans/order/digest |

## 8. Projection status/loss/coverage

| ID | Case | Expected |
|---|---|---|
| `LOSS-001` | Exact type/declaration | Exact |
| `LOSS-002` | Exact declaration + sidecar restriction/provenance | ExactWithSidecar |
| `LOSS-003` | Consumer representation approximation | LossyDeclared |
| `LOSS-004` | No acceptable representation | Unsupported |
| `LOSS-005` | Reference capability unavailable | NotEvaluated |
| `LOSS-006` | Unknown/unsupported/conflict selected input | loss/coverage record, never disappearance |
| `LOSS-007` | Docs sanitized/truncated | exact loss/source-map record |
| `LOSS-008` | Consumer-specific limitation | consumer-scoped loss |
| `LOSS-009` | Reference Complete but projection lossy | separate statuses retained |
| `LOSS-010` | Projection Complete but Reference partial/conflicted | source gap retained; no authority upgrade |
| `LOSS-011` | Sidecar-only mandatory semantic with consumer ignoring sidecar | release gate fails |
| `LOSS-012` | Loss report truncates blocking record | artifact not complete/release-ready |
| `LOSS-013` | Budget truncation | explicit counts/blockers/eligibility |
| `LOSS-014` | Nonblocking layout difference | informational parity/loss classification |
| `LOSS-015` | Random input/worker order | same records/order/digests/eligibility |

## 9. Oracle semantic parity

| ID | Case | Expected |
|---|---|---|
| `PARITY-001` | Same exact source/profile/oracle baseline, equal semantics | Equal |
| `PARITY-002` | Different file/syntax, equivalent consumer semantics | SemanticallyEquivalent |
| `PARITY-003` | Reviewed consumer/security/layout difference | ExpectedProjectionDifference |
| `PARITY-004` | Our missing/wrong supported declaration/type/member | OurDefect |
| `PARITY-005` | Oracle stale/wrong source/loss against ReferenceView | OracleDefectOrStale with evidence |
| `PARITY-006` | Different source/profile/correction/config | InputMismatch |
| `PARITY-007` | Oracle semantic equal but consumers differ | ConsumerDisagreement |
| `PARITY-008` | Insufficient/conflicting evidence | Unresolved/blocking as policy |
| `PARITY-009` | Byte equal but semantic/reference wrong | fails correctness |
| `PARITY-010` | Byte/layout differs but semantic equal | semantic gate passes, byte gate separate |
| `PARITY-011` | Oracle output automatically overwrites artifact | mutation fails |
| `PARITY-012` | Floating oracle revision | rejected |
| `PARITY-013` | Malicious/huge oracle output | bounded/rejected |
| `PARITY-014` | Classification/order under shuffled diff input | deterministic |

## 10. Consumer probes

| ID | Case | Expected |
|---|---|---|
| `CONS-001` | Pinned EmmyLua profile positive symbols/types | pass |
| `CONS-002` | Pinned LuaLS profile positive symbols/types | pass |
| `CONS-003` | Unknown API/global negative fixture | expected diagnostics remain |
| `CONS-004` | Wrong param/member/type negative fixture | expected diagnostics remain |
| `CONS-005` | Optional/nil/multiple-return/union/callback behavior | exact assertions |
| `CONS-006` | Nominal Secret type behavior | recognized, no runtime methods/declassification |
| `CONS-007` | Source span points to generated fragment/map | pass |
| `CONS-008` | Positive-only probe | rejected |
| `CONS-009` | Diagnostic suppression/weak-union/global auto-add | detected/rejected |
| `CONS-010` | User/workspace/library/extension config mutation | detected/rejected |
| `CONS-011` | Artifact files rewritten by consumer | detected/rejected |
| `CONS-012` | Shared artifact consumers disagree | shared release gate fails/consumer-specific profile required |
| `CONS-013` | Consumer version update changes mandatory behavior | update blocked/last-known-good retained |
| `CONS-014` | Process/tool inside library crate | architecture test fails |
| `CONS-015` | Probe timeout/memory/output limit | bounded failure |
| `CONS-016` | Malicious diagnostics/log/path/payload | validated/sanitized/rejected |
| `CONS-017` | Probe order/timing changes | semantic assertion/result IDs stable; timing supplemental |

## 11. Build/artifact manifests

| ID | Case | Expected |
|---|---|---|
| `ART-001` | Valid fixture/candidate/release-ready artifact states | exact eligibility |
| `ART-002` | Semantic/file/map/loss/consumer/parity manifest closure | pass |
| `ART-003` | Count/digest/file/span/profile/reference mismatch | rejected |
| `ART-004` | Partial candidate exposed as release-ready | rejected |
| `ART-005` | Cancel each model/render/map/parity/probe stage | no complete artifact/background work |
| `ART-006` | Deferred capability requested | typed unavailable, no empty success |
| `ART-007` | Final Reference Pack/signing/distribution claimed | rejected E1-C |
| `ART-008` | 1/2/N workers and randomized ordering | identical semantic/file/map/loss/artifact digests |
| `ART-009` | Physical files differ while semantic profile same | only allowed through explicit layout/profile identity change |

## 12. Deferred capabilities

| ID | Case | Expected |
|---|---|---|
| `ANN-DEFER-001` | Complete FrameXML/UI graph/skeleton output | unavailable/deferred |
| `ANN-DEFER-002` | Fuzzy search/lineage/replacement | unavailable/deferred |
| `ANN-DEFER-003` | Runtime spell/hotfix probe | unavailable/deferred |
| `ANN-DEFER-004` | Output filesystem writing by library | unavailable/owned by higher tool |
| `ANN-DEFER-005` | Final pack release/signing/CI | unavailable/deferred |
| `ANN-DEFER-006` | Empty/default success | prohibited |

## 13. Fixture/checksum freeze

| ID | Case | Expected |
|---|---|---|
| `ANN-FIX-001` | Documentation-only null pins/IDs/digests | allowed only before implementation |
| `ANN-FIX-002` | Implementation starts with required null | fail |
| `ANN-FIX-003` | Example/file bytes change without checksum update | fail |
| `ANN-FIX-004` | Oracle/consumer/type/layout/sanitization/source-map/loss vector differs | fail |
| `ANN-FIX-005` | All member/bundle checksums and golden artifacts verify | pass |

## 14. Acceptance gate

E1-C implementation is incomplete until:

```text
exact ReferenceView/reference/profile/type/layout/dialect/docs/source-map/loss/consumer/oracle pins frozen
semantic model retains exact declarations/members/types/restrictions/reference links/statuses
no silent any/omission/optionality/restriction/unknown loss
deterministic inert sanitized files and safe paths render
all material fragments map to exact reference evidence and all losses are explicit
Ketho semantic parity differences are classified without source overwrite
EmmyLua and LuaLS positive+negative probes pass without config mutation or diagnostic suppression
fixture/candidate/release-ready artifact states are honest
1/2/N output bytes/manifests/maps/loss/parity results are deterministic
no source execution/storage/editor/project/search/runtime/release behavior exists
all checksums and applicable tests pass
```
