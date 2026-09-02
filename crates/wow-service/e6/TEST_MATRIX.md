# E6-B acceptance and mutation matrix

**Status:** normative executable gate. IDs are unique within E6-B.

## Configuration, authorization, and session

| ID | Case | Expected |
|---|---|---|
| `S6B-CFG-001` | Exact enabled provider configuration | pass |
| `S6B-CFG-002` | Floating default/environment/tool discovery | reject |
| `S6B-CFG-003` | Disabled/revoked/superseded configuration | blocked |
| `S6B-CFG-004` | Descriptor/profile/digest mismatch | reject |
| `S6B-AUTH-001` | Exact credential-use authorization | pass |
| `S6B-AUTH-002` | Expired/revoked/replayed authorization | reject |
| `S6B-AUTH-003` | GitHub/OS/CLI identity used as authorization | reject |
| `S6B-AUTH-004` | Raw token/key/cookie/private endpoint in request | reject |
| `S6B-SES-001` | Acquire narrow reviewed session/transport | pass |
| `S6B-SES-002` | Session exposes undeclared arbitrary tool | reject |
| `S6B-SES-003` | Service spawns/configures/indexes provider | architecture fail |
| `S6B-SES-004` | Cross-consumer/credential/profile session reuse | reject |
| `S6B-SES-005` | Mutable state recreated through new session | reject |
| `S6B-SES-006` | Mandatory session close fails | no public success |

## Durable query and result catalog

| ID | Case | Expected |
|---|---|---|
| `S6B-QRY-001` | Register exact operation/request before dispatch | pass |
| `S6B-QRY-002` | Same operation/same digest | same operation/result |
| `S6B-QRY-003` | Same operation/different digest | reject |
| `S6B-QRY-004` | Query dispatch through one E6-A allow-listed operation | pass |
| `S6B-QRY-005` | Provider timeout after possible dispatch | OutcomeUnknown |
| `S6B-QRY-006` | Blind redispatch while unknown | reject |
| `S6B-QRY-007` | Unreconcilable old query followed by explicit new operation | allowed, distinct observation |
| `S6B-QRY-008` | Provider failure invokes another provider/local search | reject |
| `S6B-QRY-009` | Provider failure lowers local exact capability | reject |
| `S6B-QRY-010` | Cancellation before dispatch | cancelled/no dispatch |
| `S6B-QRY-011` | Cancellation after possible dispatch | cancelled + reconcile |
| `S6B-RES-001` | Publish normalized immutable result | fresh read-back valid |
| `S6B-RES-002` | Response received but result publication lost | OutcomeUnknown/reconcile |
| `S6B-RES-003` | Result bytes/digest/schema/state mismatch | reject/quarantine |
| `S6B-RES-004` | Catalog list selects latest/best result | mutation fails |
| `S6B-RES-005` | Same query text/count treated NoChange | reject |
| `S6B-RES-006` | Zero provider result | scoped zero, no negative authority |
| `S6B-RES-007` | All returned items rejected | ZeroAfterValidationLoss |
| `S6B-RES-008` | Service upgrades Candidate authority | reject |
| `S6B-ART-001` | Explicit candidate-ID subset artifact | build |
| `S6B-ART-002` | Artifact builder selects top/best/sole | reject |
| `S6B-CONT-001` | Continue same exact state/query/profile/budgets | linked new page |
| `S6B-CONT-002` | Switch provider/state/query or reset budget | reject |
| `S6B-CONT-003` | Prior truncation/loss omitted | reject |
| `S6B-CONT-004` | Raw provider cursor exposed | reject |
| `S6B-CACHE-001` | Exact retained cache entry validated by E6-A | pass |
| `S6B-CACHE-002` | Cache makes stale fresh/opaque stable/partial complete | reject |

## Project/reference owner mapping

| ID | Case | Expected |
|---|---|---|
| `S6B-MAP-001` | Exact project generation + locator maps once | ExactMapped |
| `S6B-MAP-002` | Exact reference generation + locator maps once | ExactMapped |
| `S6B-MAP-003` | Two valid owner records | MultipleMappings |
| `S6B-MAP-004` | Service chooses first/nearest/top mapping | reject |
| `S6B-MAP-005` | No match with complete owner coverage | NoMappingWithOwnerAuthority |
| `S6B-MAP-006` | No match with partial coverage | NoMappingPartial |
| `S6B-MAP-007` | Clean no-mapping without owner negative authority | reject |
| `S6B-MAP-008` | Provider path/digest trusted without owner validation | reject |
| `S6B-MAP-009` | Conflicting revision/path/span/digest | Conflict |
| `S6B-MAP-010` | Missing owner mapping capability | NotEvaluated |
| `S6B-MAP-011` | Service opens provider path/URL/source | architecture fail |
| `S6B-MAP-012` | Search/same-name result substituted for mapping | reject |
| `S6B-MAP-013` | Exact mapping called provider semantic verification | mutation fails |
| `S6B-MAP-014` | Mapping publication response lost | OutcomeUnknown/reconcile |
| `S6B-MAP-015` | Mapping against floating current refreshed mid-operation | reject |

## Explicit selection

| ID | Case | Expected |
|---|---|---|
| `S6B-SEL-001` | Caller supplies exact result/candidate/mapping and Selected | immutable receipt |
| `S6B-SEL-002` | Caller records Rejected | immutable receipt, candidate retained |
| `S6B-SEL-003` | Caller records Deferred | immutable receipt |
| `S6B-SEL-004` | Service selects top-1/sole/highest score | reject |
| `S6B-SEL-005` | Selected with MultipleMappings/partial/conflict | reject |
| `S6B-SEL-006` | Candidate/result/mapping identity mismatch | reject |
| `S6B-SEL-007` | Selection called verification/acceptance/edit authorization | mutation fails |
| `S6B-SEL-008` | Later decision mutates prior receipt | reject; new superseding receipt |
| `S6B-SEL-009` | Selection publication response lost | OutcomeUnknown/reconcile |

## Context handoff

| ID | Case | Expected |
|---|---|---|
| `S6B-CTX-001` | Exact mapping + Selected + compatible owner views | context build |
| `S6B-CTX-002` | Missing mapping or selection | blocked |
| `S6B-CTX-003` | Mapped owner generation unavailable/mismatched | blocked/NotEvaluated |
| `S6B-CTX-004` | Service substitutes parent/nearest/current root | reject |
| `S6B-CTX-005` | Mapped root unsupported by requested E3 profile | NotEvaluated |
| `S6B-CTX-006` | Provider snippet/summary enters ContextSemanticPack facts | reject |
| `S6B-CTX-007` | External Candidate sidecar remains separate | pass |
| `S6B-CTX-008` | Service recursively calls public E3 service API | architecture fail |
| `S6B-CTX-009` | Exact context succeeds, sidecar redacted by policy | context complete + explicit sidecar omission |
| `S6B-CTX-010` | Sidecar success hides partial/failed context | reject |
| `S6B-CTX-011` | Context continuation changes candidate/mapping/selection | reject |
| `S6B-CTX-012` | Context publication response lost | OutcomeUnknown/reconcile |
| `S6B-CTX-013` | Combined result claims provider interpretation verified | mutation fails |

## Retention, security, lifecycle, and determinism

| ID | Case | Expected |
|---|---|---|
| `S6B-LIFE-001` | Retain all referenced query/mapping/selection/context evidence | pass |
| `S6B-LIFE-002` | GC removes unresolved/selected/context evidence | fail |
| `S6B-LIFE-003` | Public success before mandatory reverse close | fail |
| `S6B-LIFE-004` | Startup recovery dispatches a new provider query | reject |
| `S6B-LIFE-005` | Conflicting duplicate effect | quarantine |
| `S6B-LIFE-006` | Cancellation at every stage | exact state/no background work |
| `S6B-SEC-001` | Raw SQL/MCP tool/script/plugin/model/shell input | reject |
| `S6B-SEC-002` | Provider database/index mutation | absent/reject |
| `S6B-SEC-003` | Arbitrary filesystem/network/process/editor/client access | absent |
| `S6B-SEC-004` | Credential/private endpoint/cursor leakage | fail |
| `S6B-SEC-005` | Provider text changes selector/tool/authorization | fail |
| `S6B-SEC-006` | Private source/snippet redistributed without policy | reject/redact |
| `S6B-SEC-007` | Unlimited response/mapping/context/reconciliation | bounded failure |
| `S6B-DET-001` | 1/2/N workers and allowed shuffled inputs | same canonical records |
| `S6B-DET-002` | Host/clock/network/cache/process enters semantic ID | mutation fails |
| `S6B-DET-003` | Mapping owner order changes output | stable status/ordering |
| `S6B-FIX-001` | Null pins while implementation not started | allowed |
| `S6B-FIX-002` | First Rust commit with required nulls | fail |
| `S6B-FIX-003` | All profiles/vectors/checksums frozen | pass |

## Acceptance

E6-B is incomplete until every nondeferred case executes with real E6-A, provider configuration/credential/session adapters, store catalogs, project/reference mapping owners, exact context owners, response-loss reconciliation, cancellation, security/privacy/license, platform behavior, and measured limits. Documentation fixtures or a fake provider stub are not passing implementation evidence.