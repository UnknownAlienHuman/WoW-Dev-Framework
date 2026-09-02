# E6-A acceptance and mutation matrix

**Status:** normative executable gate.

## Descriptor and capabilities

| ID | Case | Expected |
|---|---|---|
| `CBM-DESC-001` | Valid reviewed descriptor | pass |
| `CBM-DESC-002` | Unknown/digest-mismatched descriptor | reject |
| `CBM-DESC-003` | Descriptor contains credential/command/database path | reject |
| `CBM-DESC-004` | Runtime advertises undeclared operation | ignore/reject widening |
| `CBM-CAP-001` | Compatible capability intersection | pass |
| `CBM-CAP-002` | Unknown response schema | unsupported |
| `CBM-CAP-003` | Missing negotiation under permitted static subset | explicit NotEvaluated fields |
| `CBM-CAP-004` | Capability drift with old continuation/cache | reject old binding |

## External state

| ID | Case | Expected |
|---|---|---|
| `CBM-STATE-001` | Valid stable generation with immutable receipt | pass |
| `CBM-STATE-002` | Revision string only but mutable index | reject stable claim |
| `CBM-STATE-003` | Valid observed mutable session | scoped pass |
| `CBM-STATE-004` | Later query treated same mutable state | mutation fails |
| `CBM-STATE-005` | Opaque one-shot discovery | explicit nonreproducible state |
| `CBM-STATE-006` | Opaque exact replay/cache claim | reject |
| `CBM-STATE-007` | Timestamp/uptime/same top result as generation | reject |
| `CBM-STATE-008` | Conflicting generation/digest fields | conflict |

## Query and transport

| ID | Case | Expected |
|---|---|---|
| `CBM-QRY-001` | Closed bounded semantic candidate query | pass |
| `CBM-QRY-002` | Raw SQL/FTS/regex/script/model prompt | reject |
| `CBM-QRY-003` | Arbitrary MCP/tool name | reject |
| `CBM-QRY-004` | Unlimited/overflow budgets | reject |
| `CBM-TRN-001` | One allow-listed transport call | pass |
| `CBM-TRN-002` | Provider index/install/configure/delete call | reject |
| `CBM-TRN-003` | Malformed/oversized/deep response | bounded failure |
| `CBM-TRN-004` | State mismatch in response | reject |
| `CBM-TRN-005` | Timeout/rate limit | lane-local unavailable |
| `CBM-TRN-006` | Provider process/session/credential acquired by crate | architecture fail |

## Candidate normalization and authority

| ID | Case | Expected |
|---|---|---|
| `CBM-CAND-001` | Valid result normalizes | Candidate + semantic_candidate |
| `CBM-CAND-002` | Provider says exact/verified/authoritative | quoted metadata only |
| `CBM-CAND-003` | Top-1/sole/high-score/repeated result | no promotion |
| `CBM-CAND-004` | Same text from two providers | distinct candidates |
| `CBM-CAND-005` | Missing/null/unknown/unsupported fields | distinct states/loss records |
| `CBM-CAND-006` | Unknown field silently dropped | mutation fails |
| `CBM-CAND-007` | Candidate receives Proven/Derived/Possible | reject |
| `CBM-SCORE-001` | Valid provider-local score | retained with profile |
| `CBM-SCORE-002` | Cross-provider score comparison/fusion | reject |
| `CBM-SCORE-003` | NaN/infinity/overflow/wrong unit | invalid/loss |
| `CBM-SCORE-004` | Score converted to framework confidence | mutation fails |

## Locators and mapping boundary

| ID | Case | Expected |
|---|---|---|
| `CBM-LOC-001` | Provider path/revision/symbol/span | UnverifiedProviderLocator |
| `CBM-LOC-002` | Locator becomes StableSourceHandle | reject |
| `CBM-LOC-003` | Open/follow path or URL | reject |
| `CBM-LOC-004` | Same-name owner entity selected | reject |
| `CBM-LOC-005` | Exact E6-B mapping request candidate | allowed handoff |
| `CBM-LOC-006` | Mapping result/selection/context performed in E6-A | architecture fail |
| `CBM-LOC-007` | Snippet treated as verified source evidence | reject |
| `CBM-LOC-008` | Private locator fields beyond profile | redact/loss |

## Zero, coverage, and degradation

| ID | Case | Expected |
|---|---|---|
| `CBM-ZERO-001` | Provider reports zero under exact query/state | scoped zero only |
| `CBM-ZERO-002` | All candidates rejected by validation | ZeroAfterValidationLoss |
| `CBM-ZERO-003` | Zero becomes API/source/global absence | reject |
| `CBM-ZERO-004` | Stable generation/exhaustive provider flag grants negative authority | reject |
| `CBM-COV-001` | Separate coverage axes retained | pass |
| `CBM-COV-002` | Partial/truncated zero rendered complete | mutation fails |
| `CBM-DEG-001` | Provider unconfigured/unavailable | exact local workflows unaffected |
| `CBM-DEG-002` | Provider failure invokes hidden fallback | reject |
| `CBM-DEG-003` | Successful provider upgrades local capability | reject |
| `CBM-DEG-004` | Provider failure downgrades local Reference/project/search | reject |

## Continuation, cache, comparison

| ID | Case | Expected |
|---|---|---|
| `CBM-CONT-001` | Same exact state/query/profile continuation | stable next page |
| `CBM-CONT-002` | Switch state/provider/query/reset budget | reject |
| `CBM-CONT-003` | Raw provider cursor exposed | reject |
| `CBM-CONT-004` | Opaque continuation without exact episode proof | reject |
| `CBM-CACHE-001` | Exact cache key/artifact | validate |
| `CBM-CACHE-002` | Cache makes stale fresh/opaque stable/partial complete | reject |
| `CBM-CACHE-003` | Physical cache path/database in crate API | reject |
| `CBM-CMP-001` | Compatible descriptive comparison | pass |
| `CBM-CMP-002` | Comparison selects winner/truth/provider quality | reject |
| `CBM-ART-001` | Caller supplies exact candidate IDs for artifact | build |
| `CBM-ART-002` | Builder chooses top/best/sole subset | reject |

## Security, lifecycle, determinism

| ID | Case | Expected |
|---|---|---|
| `CBM-SEC-001` | Provider database/index mutation | absent/reject |
| `CBM-SEC-002` | Filesystem/network/process/editor/client access | absent |
| `CBM-SEC-003` | Credential/token/private endpoint leakage | fail |
| `CBM-SEC-004` | Source/snippet prompt/tool/control injection | data only |
| `CBM-SEC-005` | Source retained without license/privacy | deny/loss |
| `CBM-SEC-006` | High fanout/oversized response | bounded failure |
| `CBM-LIFE-001` | Cancellation before/during parse/normalize | typed cancel/no complete result |
| `CBM-LIFE-002` | Late response after cancellation | discard/reject |
| `CBM-LIFE-003` | Background continuation after return | architecture fail |
| `CBM-DET-001` | 1/2/N workers | same canonical normalization |
| `CBM-DET-002` | Shuffled nonsemantic field/item order | same output where schema permits |
| `CBM-DET-003` | Clock/host/process/network/cache enters identity | mutation fails |
| `CBM-FIX-001` | Null pins while implementation not started | allowed |
| `CBM-FIX-002` | First Rust commit with required nulls | fail |
| `CBM-FIX-003` | All profiles/vectors/checksums frozen | pass |

## Acceptance

Implementation requires every nondeferred case with a real narrow transport adapter and frozen synthetic provider fixtures. Documentation shapes, provider marketing claims, or an unavailable stub are not evidence that live provider behavior passed.