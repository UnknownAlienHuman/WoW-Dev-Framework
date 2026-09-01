# E4-C service acceptance and mutation matrix

**Status:** normative executable gate. All IDs are unique within the service E4-C package.

## Configuration and routing

| ID | Case | Expected |
|---|---|---|
| `S4-CONF-001` | Valid E4-C configuration/profile bundle | pass |
| `S4-CONF-002` | Unknown profile alias/version | reject |
| `S4-CONF-003` | Alias resolves to multiple profiles | reject |
| `S4-CONF-004` | Unlimited/negative/overflow budget | reject |
| `S4-CONF-005` | E4-C direct dependency outside active slice | architecture failure |
| `S4-CONF-006` | App/lower owner algorithm registered in service | architecture failure |
| `S4-CONF-007` | Patch-sensitive constant embedded in orchestration | mutation fails |
| `S4-CONF-008` | Missing implementation/probe reported pass | mutation fails |

## Selectors and exact acquisition

| ID | Case | Expected |
|---|---|---|
| `S4-SEL-001` | Exact project/reference/shard selection | pass |
| `S4-SEL-002` | Current resolves once to exact IDs | pass with observation receipt |
| `S4-SEL-003` | Current reread after owner call | mutation fails |
| `S4-SEL-004` | Latest/nearest/LKG fallback | reject |
| `S4-SEL-005` | Exact owner generation has no shard | typed unavailable; no build |
| `S4-SEL-006` | Multiple shards for exact generation/profile | conflict; no selection |
| `S4-SEL-007` | Exact lineage comparison has no snapshot | typed unavailable |
| `S4-SEL-008` | Multiple lineage snapshots for exact comparison/profile | conflict |
| `S4-SEL-009` | Same display name, different generation/universe | remain distinct |
| `S4-SEL-010` | Comparison crosses project/reference universe classes | reject |
| `S4-SEL-011` | Shard owner binding/profile mismatch | reject |
| `S4-SEL-012` | Lineage snapshot comparison/profile mismatch | reject |

## Stable current and acquisition order

| ID | Case | Expected |
|---|---|---|
| `S4-ACQ-001` | Stable double collect unchanged | accept exact set |
| `S4-ACQ-002` | Current changes between collects | close and retry whole attempt |
| `S4-ACQ-003` | Current churn exceeds limit | `AcquisitionUnstable` |
| `S4-ACQ-004` | Exact request uses current collect | mutation fails |
| `S4-ACQ-005` | Fixed acquisition order | pass |
| `S4-ACQ-006` | Out-of-order owner/shard/lineage acquisition | fail |
| `S4-ACQ-007` | Acquisition failure after N handles | reverse close all prior |
| `S4-ACQ-008` | Partial view set enters owner operation | mutation fails |
| `S4-ACQ-009` | Distributed atomic-current claim | mutation fails |
| `S4-ACQ-010` | New current after accepted collect | exact retained views remain valid; no global-current claim |

## Search shard operations

| ID | Case | Expected |
|---|---|---|
| `S4-SIDX-001` | Build exact owner shard | validated/cataloged artifact |
| `S4-SIDX-002` | Build current alias changes mid-acquisition | bounded reacquire before build |
| `S4-SIDX-003` | Query missing shard | unavailable; no implicit build |
| `S4-SIDX-004` | Built but unvalidated shard | not query eligible |
| `S4-SIDX-005` | Invalid shard validation | `Invalid` payload, no repair |
| `S4-SIDX-006` | Shard build failure/cancel | prior catalog unchanged |
| `S4-SIDX-007` | In-place shard mutation | mutation fails |
| `S4-SIDX-008` | Same build operation ID/digest retry | same receipt/artifact |
| `S4-SIDX-009` | Same operation ID/different digest | reject |
| `S4-SIDX-010` | Response lost after catalog | retry returns recorded receipt |

## Search query and explanation

| ID | Case | Expected |
|---|---|---|
| `S4-SEARCH-001` | Exact multi-shard query | owner result preserved |
| `S4-SEARCH-002` | Owner exact authoritative miss | preserve exact miss class |
| `S4-SEARCH-003` | Empty FTS/fuzzy lanes | no authoritative-miss upgrade |
| `S4-SEARCH-004` | Lane partial/failed/skipped | visible in envelope/status |
| `S4-SEARCH-005` | CandidateOnly owner result | `CandidateOnly` preserved |
| `S4-SEARCH-006` | Service recalculates rank/weights | mutation fails |
| `S4-SEARCH-007` | Raw BM25 exposed as cross-shard authority | mutation fails |
| `S4-SEARCH-008` | Query text drives profile/tool behavior | reject |
| `S4-SEARCH-009` | Explain exact candidate | complete owner arithmetic/evidence |
| `S4-SEARCH-010` | Service prose implies replacement/intent | mutation fails |
| `S4-SEARCH-011` | Result validation failure | service failure with owner refs |
| `S4-SEARCH-012` | Private/local field under external consumer | omitted/denied, not leaked |

## Explicit candidate selection

| ID | Case | Expected |
|---|---|---|
| `S4-SELECT-001` | Exact result/candidate guards | selection receipt |
| `S4-SELECT-002` | Select by rank only | reject |
| `S4-SELECT-003` | Select by display name only | reject |
| `S4-SELECT-004` | Auto-select top-1 | mutation fails |
| `S4-SELECT-005` | Auto-select sole candidate | mutation fails |
| `S4-SELECT-006` | Candidate from another result/shard | reject |
| `S4-SELECT-007` | Result/candidate digest mismatch | reject |
| `S4-SELECT-008` | Selection raises entity confidence | mutation fails |
| `S4-SELECT-009` | Selection receipt retains rank/signals as provenance | pass |
| `S4-SELECT-010` | Selection origin absent where required | reject/NotEvaluated |

## Search-to-context

| ID | Case | Expected |
|---|---|---|
| `S4-CTX-001` | Explicit selected exact entity to E3-C context | pass |
| `S4-CTX-002` | Search query text used as context root | reject |
| `S4-CTX-003` | Context owner generation differs from result | reject |
| `S4-CTX-004` | Current advanced after search | use retained exact result generation or require new search |
| `S4-CTX-005` | Search score injected as context fact/confidence | mutation fails |
| `S4-CTX-006` | Search explanation hidden inside context truth | mutation fails |
| `S4-CTX-007` | Context partial with complete selection | combined status partial |
| `S4-CTX-008` | Context source policy broader than search/owner | reject/omit |
| `S4-CTX-009` | Context continuation changes candidate | reject |
| `S4-CTX-010` | No edit/tool authorization in context outcome | pass |

## Lineage producer/build/query

| ID | Case | Expected |
|---|---|---|
| `S4-LIN-001` | Exact project before/after producer set | build input valid |
| `S4-LIN-002` | Exact Reference before/after producer set | build input valid |
| `S4-LIN-003` | Optional search candidates | remain Candidate |
| `S4-LIN-004` | Search candidate promoted without independent evidence/review | reject |
| `S4-LIN-005` | Producer partitions merged/overwritten | mutation fails |
| `S4-LIN-006` | Ambiguous one-to-many component | retained unresolved |
| `S4-LIN-007` | Greedy highest-rank assignment | mutation fails |
| `S4-LIN-008` | Complete Candidate-only build | `CandidateOnly`, not failed |
| `S4-LIN-009` | Removed/Introduced under partial coverage | blocked/NotEvaluated |
| `S4-LIN-010` | Build publication/read-back valid | catalog exact snapshot |
| `S4-LIN-011` | Snapshot invalid | no eligible catalog admission |
| `S4-LIN-012` | In-place lineage snapshot mutation | mutation fails |
| `S4-LIN-013` | Compare exact entities | owner changes/assertions preserved |
| `S4-LIN-014` | Trace Candidate/Possible opt-in | labeled paths |
| `S4-LIN-015` | Path flattened to direct relation | mutation fails |
| `S4-LIN-016` | Lineage explain | full producer/proof/review/coverage closure |

## Review authorization and apply

| ID | Case | Expected |
|---|---|---|
| `S4-REV-001` | Authorized exact review + semantic validity | valid |
| `S4-REV-002` | GitHub/OS/CLI identity used as authorization | reject |
| `S4-REV-003` | Plain prose acceptance | reject |
| `S4-REV-004` | Invalid signature/attestation | unauthorized |
| `S4-REV-005` | Expired/revoked/replayed envelope | reject |
| `S4-REV-006` | Principal role/scope mismatch | reject |
| `S4-REV-007` | Valid authorization but invalid graph target | semantic invalid |
| `S4-REV-008` | Requested confidence above producer/profile ceiling | reject/cap only by explicit profile rule |
| `S4-REV-009` | Review note treated as proof | mutation fails |
| `S4-REV-010` | Apply exact review to exact base | new immutable snapshot |
| `S4-REV-011` | Apply against stale base/proposal revision | reject |
| `S4-REV-012` | Rejected decision deletes proposal | mutation fails |
| `S4-REV-013` | Supersede without prior exact decision | reject |
| `S4-REV-014` | Response loss after review publication | exact receipt returned on retry |

## Migration

| ID | Case | Expected |
|---|---|---|
| `S4-MIG-001` | Explicit replacement supports candidate | preserve exact relation |
| `S4-MIG-002` | Same lineage without replacement | `LineageOnlyNoReplacement` |
| `S4-MIG-003` | Deprecated with no target | explicit no-target result |
| `S4-MIG-004` | Top search result treated as replacement | mutation fails |
| `S4-MIG-005` | Valid recipe static closure | `ValidatedRecipe`, not applied |
| `S4-MIG-006` | Missing precondition/step/postcondition | invalid |
| `S4-MIG-007` | Recipe generation/profile mismatch | invalid |
| `S4-MIG-008` | Validation edits source/runs tool/client | architecture failure |
| `S4-MIG-009` | Validated recipe claims runtime success | mutation fails |
| `S4-MIG-010` | Privacy/license blocks recipe detail | explicit denied/NotEvaluated |

## Static impact

| ID | Case | Expected |
|---|---|---|
| `S4-IMP-001` | Exact accepted change root | valid plan |
| `S4-IMP-002` | Candidate/unresolved root under strict profile | reject/NotEvaluated |
| `S4-IMP-003` | Bounded direct/transitive paths | preserve paths |
| `S4-IMP-004` | Possible edge | remains possible |
| `S4-IMP-005` | High-fanout/cycle | deterministic truncation |
| `S4-IMP-006` | Impact path labeled runtime breakage/severity | mutation fails |
| `S4-IMP-007` | Path flattened to direct dependency | mutation fails |
| `S4-IMP-008` | Graph generation mismatch | reject |
| `S4-IMP-009` | Incomplete traversal coverage | partial/NotEvaluated |
| `S4-IMP-010` | Explain path | exact root/edge/evidence closure |
| `S4-IMP-011` | Continuation same exact artifacts/budget | stable page |
| `S4-IMP-012` | Continuation resets budget/switches graph | reject |

## Lifecycle and closure

| ID | Case | Expected |
|---|---|---|
| `S4-LIFE-001` | Success then all closes succeed | publish success |
| `S4-LIFE-002` | Success then mandatory close fails | service failure; no false success |
| `S4-LIFE-003` | Failure after partial acquisition | reverse close all |
| `S4-LIFE-004` | Cancellation during owner call | synchronous cleanup, cancelled |
| `S4-LIFE-005` | Background continuation/cleanup | architecture failure |
| `S4-LIFE-006` | Continuation advertised before retention | mutation fails |
| `S4-LIFE-007` | Retention admission fails | no cursor advertised |
| `S4-LIFE-008` | GC race during retention admission | owner port resolves or fails explicitly |
| `S4-LIFE-009` | Same exact read retry | same semantic bytes |
| `S4-LIFE-010` | Current retry after changed current reuses old ID | mutation fails |
| `S4-LIFE-011` | Build response loss at each durable phase | no duplicate artifact |
| `S4-LIFE-012` | Operation ID digest conflict | reject |

## Result/status/serialization

| ID | Case | Expected |
|---|---|---|
| `S4-RES-001` | Status precedence table | exact expected status |
| `S4-RES-002` | Invalid validation completed | Complete + Invalid payload where specified |
| `S4-RES-003` | Warning hides blocker | mutation fails |
| `S4-RES-004` | CandidateOnly folded to Complete/Proven | mutation fails |
| `S4-RES-005` | Empty output called NoChange | mutation fails |
| `S4-RES-006` | Owner payload rewritten by service | mutation fails |
| `S4-RES-007` | Mandatory nonclaims missing | validation fails |
| `S4-RES-008` | Canonical envelope repeated run | byte identical |
| `S4-RES-009` | Timing/host/terminal enters semantic digest | mutation fails |
| `S4-RES-010` | Private raw review/source data in error | redaction failure |

## Security and privacy

| ID | Case | Expected |
|---|---|---|
| `S4-SEC-001` | Raw SQL/FTS/regex/executable input | reject |
| `S4-SEC-002` | Filesystem/network/process/editor/client access | absent |
| `S4-SEC-003` | Model/embedding/CBM call | absent |
| `S4-SEC-004` | Source/query/review prose alters profile/proof | mutation fails |
| `S4-SEC-005` | Cross-consumer privacy widening | reject |
| `S4-SEC-006` | Raw key/token/signature logged | fail |
| `S4-SEC-007` | Malformed/tampered artifact/cursor | reject |
| `S4-SEC-008` | Oversized request/output/resource bomb | bounded failure |
| `S4-SEC-009` | Cross-universe entity substitution | reject |
| `S4-SEC-010` | Context result treated as tool authorization | mutation fails |
| `S4-SEC-011` | Review authorization port unavailable | NotEvaluated; no apply |
| `S4-SEC-012` | Unknown privacy/license state | safest explicit behavior |

## Determinism and freeze

| ID | Case | Expected |
|---|---|---|
| `S4-DET-001` | 1/2/N workers | identical canonical results |
| `S4-DET-002` | Shuffled owner/lane/proposal/path results | stable output |
| `S4-DET-003` | Cold/warm cache or storage layout | same semantic output |
| `S4-DET-004` | Host/path/clock/process differences | no identity change |
| `S4-DET-005` | Independent build histories same final inputs | same logical artifacts |
| `S4-FIX-001` | Null pins while implementation not-started | allowed |
| `S4-FIX-002` | First E4-C Rust commit with required null pins | fail |
| `S4-FIX-003` | All prerequisite/profile/vector/checksum pins frozen | pass |
| `S4-FIX-004` | Cargo/.rs/workflow introduced in documentation package | fail |

## Acceptance

E4-C cannot be marked implemented until all nondeferred tests execute, all current-alias and exact-input lifecycles are covered, search selection is explicitly receipted, review authorization and graph semantics are independently validated, no owner confidence/authority is upgraded, migration remains nonexecuting, static impact remains path-based and static-only, and all closure/retention/idempotency/security/determinism gates pass.
