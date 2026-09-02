# E6-B service acceptance and mutation matrix

**Status:** normative executable gate. IDs are unique within E6-B.

## Configuration and dependency boundary

| ID | Case | Expected |
|---|---|---|
| `S6B-CONF-001` | Valid exact E6-B configuration/profile bundle | pass |
| `S6B-CONF-002` | Unknown field/profile/version | reject |
| `S6B-CONF-003` | Unlimited/negative/overflow budget | reject |
| `S6B-CONF-004` | E6-B activates undeclared direct dependency | architecture failure |
| `S6B-CONF-005` | Service reimplements E6-A query/normalization | mutation fails |
| `S6B-CONF-006` | Service reimplements project/reference mapping | mutation fails |
| `S6B-CONF-007` | Service builds context directly | mutation fails |
| `S6B-CONF-008` | Patch-sensitive WoW constant embedded in orchestration | mutation fails |
| `S6B-CONF-009` | Missing implementation/profile reported pass | mutation fails |
| `S6B-CONF-010` | Cargo/Rust/workflow added during docs package | fail |

## Provider descriptor, authorization, and session

| ID | Case | Expected |
|---|---|---|
| `S6B-PROV-001` | Exact configured descriptor and adapter | pass |
| `S6B-PROV-002` | Provider not configured | scoped unavailable |
| `S6B-PROV-003` | Descriptor digest/schema mismatch | reject |
| `S6B-PROV-004` | Session capability set matches request | pass |
| `S6B-PROV-005` | Required capability missing/changed | NotEvaluated/fail |
| `S6B-PROV-006` | Arbitrary provider method/tool requested | reject |
| `S6B-PROV-007` | Generic MCP payload/tool schema | reject |
| `S6B-PROV-008` | Valid scoped credential authorization | pass |
| `S6B-PROV-009` | Unauthorized principal/scope | deny |
| `S6B-PROV-010` | Expired authorization | deny |
| `S6B-PROV-011` | Revoked authorization | deny |
| `S6B-PROV-012` | Replayed authorization | deny |
| `S6B-PROV-013` | GitHub/OS/CLI/file identity used as authorization | mutation fails |
| `S6B-PROV-014` | Raw token/key/cookie/private endpoint in request | reject/redact |
| `S6B-PROV-015` | Opaque live session handle serialized publicly | fail |
| `S6B-PROV-016` | Session receipt from another provider/operation | reject |
| `S6B-PROV-017` | Session close succeeds after query | pass |
| `S6B-PROV-018` | Session close fails after useful work | no public success |

## External-state binding

| ID | Case | Expected |
|---|---|---|
| `S6B-STATE-001` | Valid stable external generation | exact binding |
| `S6B-STATE-002` | Stable generation lacks verification | reject/NotEvaluated |
| `S6B-STATE-003` | Valid observed mutable receipt | exact observation binding |
| `S6B-STATE-004` | Later observation substituted mid-request | reject |
| `S6B-STATE-005` | Opaque external state used within allowed profile | Candidate-only explicit nonreproducible |
| `S6B-STATE-006` | Opaque state called stable/reproducible | mutation fails |
| `S6B-STATE-007` | Same results used to prove same generation | mutation fails |
| `S6B-STATE-008` | Provider current/latest re-resolved on continuation | reject |
| `S6B-STATE-009` | State receipt from another session/descriptor | reject |
| `S6B-STATE-010` | State coverage partial | preserve Partial/NotEvaluated |

## Durable operations and idempotency

| ID | Case | Expected |
|---|---|---|
| `S6B-IDEM-001` | Register operation before effect | pass |
| `S6B-IDEM-002` | Effect before durable registration | mutation fails |
| `S6B-IDEM-003` | Same OperationId/same digest completed | same exact result, no repeat |
| `S6B-IDEM-004` | Same OperationId/different digest | reject |
| `S6B-IDEM-005` | Operation ID reused across kind | reject |
| `S6B-IDEM-006` | Lost response before dispatch | no effect; safe retry |
| `S6B-IDEM-007` | Lost response after dispatch | OutcomeUnknown until reconcile |
| `S6B-IDEM-008` | Blind repeat while OutcomeUnknown | mutation fails |
| `S6B-IDEM-009` | Provider proves no effect | retry per exact profile |
| `S6B-IDEM-010` | Provider returns committed receipt | recover exact result |
| `S6B-IDEM-011` | Provider reports effect in progress | remain blocked/in progress |
| `S6B-IDEM-012` | Conflicting duplicate effects | quarantine |
| `S6B-IDEM-013` | Reconciliation unsupported after possible effect | OutcomeUnknown |
| `S6B-IDEM-014` | Loss after local artifact commit | recover exact artifact |
| `S6B-IDEM-015` | Loss after retention admission | return recorded result after reconcile |
| `S6B-IDEM-016` | Empty output treated NoChange | mutation fails |
| `S6B-IDEM-017` | Exact artifact already exists for full request | NoChange with proof |
| `S6B-IDEM-018` | Retry changes provider/query/profile/budget | reject |

## E6-A query/result/artifact orchestration

| ID | Case | Expected |
|---|---|---|
| `S6B-QUERY-001` | Valid exact E6-A query | CandidateOnly result |
| `S6B-QUERY-002` | E6-A owner unavailable | scoped NotEvaluated/failure |
| `S6B-QUERY-003` | Owner result schema/digest mismatch | reject |
| `S6B-QUERY-004` | Provider-local rank/score preserved | labelled metadata only |
| `S6B-QUERY-005` | Rank/score promoted to confidence | mutation fails |
| `S6B-QUERY-006` | Provider exact/verified label promoted | mutation fails |
| `S6B-QUERY-007` | Zero result | request/state-scoped zero only |
| `S6B-QUERY-008` | Zero result becomes source/platform absence | mutation fails |
| `S6B-QUERY-009` | Partial/truncated result called complete | mutation fails |
| `S6B-QUERY-010` | Immutable result persisted/cataloged/retained | pass |
| `S6B-QUERY-011` | Artifact not retained before handle return | fail |
| `S6B-QUERY-012` | Explain exact candidate | preserve Candidate/nonclaims |
| `S6B-QUERY-013` | Artifact build changes candidate bytes | reject |
| `S6B-QUERY-014` | Cross-provider score comparison/fusion | reject |

## Continuation and catalogs

| ID | Case | Expected |
|---|---|---|
| `S6B-CONT-001` | Exact continuation same state/query/profile | stable next page |
| `S6B-CONT-002` | Continuation changes provider | reject |
| `S6B-CONT-003` | Continuation changes external state | reject |
| `S6B-CONT-004` | Continuation changes query/profile/privacy | reject |
| `S6B-CONT-005` | Continuation resets cumulative budget | reject |
| `S6B-CONT-006` | Provider cursor exposed publicly | fail |
| `S6B-CONT-007` | Opaque-state continuation beyond profile | reject/NotEvaluated |
| `S6B-CONT-008` | Catalog list exact snapshot/pagination | pass |
| `S6B-CONT-009` | Catalog changes mid-page | original snapshot retained or stale reject |
| `S6B-CONT-010` | List sorted newest/best by app/service shortcut | mutation fails |
| `S6B-CONT-011` | Multiple exact catalog matches | conflict; no selection |
| `S6B-CONT-012` | No exact catalog match | unavailable; no fallback |

## Mapping

| ID | Case | Expected |
|---|---|---|
| `S6B-MAP-001` | Exact project owner mapping with evidence | ExactMapped |
| `S6B-MAP-002` | Exact Reference owner mapping with evidence | ExactMapped |
| `S6B-MAP-003` | Same candidate mapped to two owners in one request | reject; separate requests required |
| `S6B-MAP-004` | Multiple owner records satisfy profile | MultipleMappings |
| `S6B-MAP-005` | No mapping under partial coverage | NoMappingPartial |
| `S6B-MAP-006` | Owner explicit scoped negative authority | NoMappingWithOwnerAuthority |
| `S6B-MAP-007` | Empty lookup alone used as authoritative no mapping | mutation fails |
| `S6B-MAP-008` | Owner conflict | Conflict preserved |
| `S6B-MAP-009` | Owner capability unavailable | NotEvaluated |
| `S6B-MAP-010` | Mapping receipt for another result/candidate | reject |
| `S6B-MAP-011` | Mapping receipt for another generation/profile | reject |
| `S6B-MAP-012` | Provider path/URI followed by service | mutation fails |
| `S6B-MAP-013` | Same name/filename/path suffix heuristic | mutation fails |
| `S6B-MAP-014` | Snippet/fuzzy/FTS/embedding mapping | mutation fails |
| `S6B-MAP-015` | Provider-labelled exact accepted without owner proof | reject |
| `S6B-MAP-016` | Exact digest/source-object mapping | pass |
| `S6B-MAP-017` | Line number alone maps source | reject |
| `S6B-MAP-018` | ExactMapped validates provider summary/relation | mutation fails |
| `S6B-MAP-019` | Same mapping receipt reused for later current generation | reject |
| `S6B-MAP-020` | Mapping output leaks private provider path/source | privacy failure |

## Explicit selection

| ID | Case | Expected |
|---|---|---|
| `S6B-SEL-001` | Exact candidate + ExactMapped receipt + explicit origin | Selected receipt |
| `S6B-SEL-002` | Top-1 selection | reject |
| `S6B-SEL-003` | First/last selection | reject |
| `S6B-SEL-004` | Highest-score/best selection | reject |
| `S6B-SEL-005` | Sole candidate auto-selected | reject |
| `S6B-SEL-006` | Same-name/path/snippet selection | reject |
| `S6B-SEL-007` | Provider exact label selection shortcut | reject |
| `S6B-SEL-008` | Candidate not in exact result set | reject |
| `S6B-SEL-009` | Mapping is Multiple/Partial/Conflict/NotEvaluated | block Selected |
| `S6B-SEL-010` | Mapped root differs from receipt | reject |
| `S6B-SEL-011` | Selection upgrades confidence/provenance | mutation fails |
| `S6B-SEL-012` | Selection grants edit/tool/runtime authority | mutation fails |
| `S6B-SEL-013` | Rejected candidate rendered false | mutation fails |
| `S6B-SEL-014` | Response loss after receipt commit | recover same receipt |
| `S6B-SEL-015` | Supersession without prior link | reject |
| `S6B-SEL-016` | Presentation order changes but exact choice same | same receipt identity |

## Context handoff

| ID | Case | Expected |
|---|---|---|
| `S6B-CTX-001` | Valid exact project root handoff | context result + external attachment |
| `S6B-CTX-002` | Valid exact Reference root handoff | context result + external attachment |
| `S6B-CTX-003` | Provider prose inserted as framework fact | mutation fails |
| `S6B-CTX-004` | Provider rank/score inserted into context authority | mutation fails |
| `S6B-CTX-005` | Mapping/current generation mismatch | reject |
| `S6B-CTX-006` | Selection for another root | reject |
| `S6B-CTX-007` | Context owner unavailable | NotEvaluated/failure; no snippet fallback |
| `S6B-CTX-008` | Provider snippet denied but local context allowed | context + explicit omission |
| `S6B-CTX-009` | Context partial/conflict/truncated | preserve exact state |
| `S6B-CTX-010` | Response loss after context effect | reconcile exact operation |
| `S6B-CTX-011` | Retry selects another candidate/root | reject |
| `S6B-CTX-012` | Context success validates provider relation | mutation fails |

## Cache, retention, degradation, and closure

| ID | Case | Expected |
|---|---|---|
| `S6B-RET-001` | Complete retention closure | pass |
| `S6B-RET-002` | Artifact disappears before lease admission | unavailable; no substitute |
| `S6B-RET-003` | Valid exact cache entry | ValidExactHit |
| `S6B-RET-004` | Cache entry another mutable observation | reject |
| `S6B-RET-005` | Cache privacy/license profile wider | reject |
| `S6B-RET-006` | Cache hit upgrades freshness/authority | mutation fails |
| `S6B-RET-007` | Provider unavailable | scoped degradation; local workflows unaffected |
| `S6B-RET-008` | Hidden fallback to another provider/state/cache/model/search | mutation fails |
| `S6B-RET-009` | Public success before mandatory close | mutation fails |
| `S6B-RET-010` | Close failure after useful work | failure with recovery refs |
| `S6B-RET-011` | Background cleanup/continuation after return | mutation fails |
| `S6B-RET-012` | Provider failure marks framework globally failed | mutation fails |

## Security and privacy

| ID | Case | Expected |
|---|---|---|
| `S6B-SEC-001` | Raw token/key/cookie/private endpoint in request/config | reject |
| `S6B-SEC-002` | Credential/session/cursor in result/error/log | fail |
| `S6B-SEC-003` | Arbitrary MCP/tool/SQL/script/plugin/model input | reject |
| `S6B-SEC-004` | Provider install/start/configure/index/delete call | absent |
| `S6B-SEC-005` | Direct provider DB/file access | absent |
| `S6B-SEC-006` | Arbitrary filesystem/network/process/editor/client access | absent |
| `S6B-SEC-007` | Malicious path/URI/snippet/summary controls request | data only/reject |
| `S6B-SEC-008` | Cross-consumer privacy widening | reject |
| `S6B-SEC-009` | Oversized/deep/high-fanout result/audit bomb | bounded failure/truncation |
| `S6B-SEC-010` | Private path/source leak in mapping/context attachment | fail |
| `S6B-SEC-011` | App directly opens session/owner/store | architecture failure |
| `S6B-SEC-012` | External result grants tool/edit permission | mutation fails |

## Cancellation, determinism, and freeze

| ID | Case | Expected |
|---|---|---|
| `S6B-LIFE-001` | Cancel before authorization/session | Cancelled, no effect |
| `S6B-LIFE-002` | Cancel after provider dispatch | reconcile/OutcomeUnknown as required |
| `S6B-LIFE-003` | Cancel during mapping | exact partial state, close all |
| `S6B-LIFE-004` | Cancel during context | exact context reconciliation |
| `S6B-LIFE-005` | Broken serialization after result | no duplicate provider call |
| `S6B-DET-001` | 1/2/N workers | identical semantic artifacts/envelopes |
| `S6B-DET-002` | Shuffled provider/catalog/owner result order | stable output |
| `S6B-DET-003` | Cold/warm cache and different store layout | same semantic result |
| `S6B-DET-004` | Host/path/clock/process/session-handle changes | no semantic ID change |
| `S6B-DET-005` | Independent histories same exact final inputs | same logical output |
| `S6B-FIX-001` | Null pins while implementation not started | allowed |
| `S6B-FIX-002` | First Rust commit with required null pins | fail |
| `S6B-FIX-003` | All profiles/vectors/checksums frozen | pass |
| `S6B-FIX-004` | Test rewrites fixture automatically | fail |

## Acceptance

E6-B cannot be marked implemented until every nondeferred case executes against synthetic and configured real-provider fixtures; exact E6-A authority remains Candidate; provider credentials/session state stay private; response loss is reconcilable; mapping and selection shortcuts fail; context facts exclude provider prose/rank/score; retention and closure complete; optional provider failure leaves local workflows intact; and all canonical bytes/checksums freeze.
