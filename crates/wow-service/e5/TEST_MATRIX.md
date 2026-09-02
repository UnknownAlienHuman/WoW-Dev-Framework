# E5-B service acceptance and mutation matrix

**Status:** normative executable gate. IDs are unique within E5-B.

## Configuration and routing

| ID | Case | Expected |
|---|---|---|
| `S5B-CONF-001` | Valid exact configuration/profile bundle | pass |
| `S5B-CONF-002` | Unknown field/profile/version | reject |
| `S5B-CONF-003` | Unlimited/negative/overflow budget | reject |
| `S5B-CONF-004` | Active dependency outside E5-B slice | architecture failure |
| `S5B-CONF-005` | E5-A algorithm duplicated in service | mutation fails |
| `S5B-CONF-006` | E5-C publication operation exposed | architecture failure |
| `S5B-CONF-007` | Patch-sensitive WoW constant in orchestration | mutation fails |
| `S5B-CONF-008` | Missing implementation/probe reported pass | mutation fails |

## Selectors and catalogs

| ID | Case | Expected |
|---|---|---|
| `S5B-SEL-001` | Exact artifact/digest selector | pass |
| `S5B-SEL-002` | Exact binding has unique eligible artifact | select exact ID |
| `S5B-SEL-003` | Exact binding has none | typed unavailable |
| `S5B-SEL-004` | Exact binding has multiple | conflict; no selection |
| `S5B-SEL-005` | Latest/best/highest/newest selector | reject |
| `S5B-SEL-006` | Single catalog row assumed eligible without validation | mutation fails |
| `S5B-SEL-007` | Same name, different generation/digest | remain distinct |
| `S5B-SEL-008` | Superseded artifact selected as active | reject |
| `S5B-SEL-009` | Symbolic project current resolved once | exact receipt |
| `S5B-SEL-010` | Current reread after owner operation | mutation fails |
| `S5B-SEL-011` | Artifact disappears before retention | unavailable; no handle |
| `S5B-SEL-012` | Acquisition order reversed | fail and close |

## Corpus/source/split orchestration

| ID | Case | Expected |
|---|---|---|
| `S5B-CORP-001` | Delegate source validation to E5-A | exact owner result |
| `S5B-CORP-002` | Commit pin alone admitted | mutation fails |
| `S5B-CORP-003` | Corpus validation exact artifacts | pass |
| `S5B-CORP-004` | Corpus admit with missing provenance/license/labels | blocked |
| `S5B-CORP-005` | Copied/forked group crosses ordinary splits | invalid |
| `S5B-CORP-006` | Unknown provenance treated independent | mutation fails |
| `S5B-CORP-007` | Split validation exposes sealed labels | privacy failure |
| `S5B-CORP-008` | Owner partial result folded complete | mutation fails |

## Durable run submission

| ID | Case | Expected |
|---|---|---|
| `S5B-RUN-001` | Exact visible-split run submission | durable receipt |
| `S5B-RUN-002` | Same operation ID/same digest retry | same result/effect |
| `S5B-RUN-003` | Same operation ID/different digest | reject |
| `S5B-RUN-004` | Service changes pack/corpus/split/profile | mutation fails |
| `S5B-RUN-005` | Hidden holdout use in visible run | reject |
| `S5B-RUN-006` | E5-A owner unavailable | NotEvaluated/failure |
| `S5B-RUN-007` | Graph validation unavailable | blocker, not pass |
| `S5B-RUN-008` | Per-case result missing but aggregate exists | invalid |
| `S5B-RUN-009` | Hard gate hidden by weighted metric | mutation fails |
| `S5B-RUN-010` | Candidate/possible/unknown coerced negative | mutation fails |
| `S5B-RUN-011` | Complete owner run and retention/close | Complete |
| `S5B-RUN-012` | Successful work then mandatory close failure | Failed; artifact refs retained |

## Idempotency and response loss

| ID | Case | Expected |
|---|---|---|
| `S5B-IDEM-001` | Loss before owner dispatch | no effect; safe retry |
| `S5B-IDEM-002` | Loss after owner dispatch before response | OutcomeUnknown until reconcile |
| `S5B-IDEM-003` | Loss after owner artifact commit | recover exact receipt |
| `S5B-IDEM-004` | Blind redispatch while OutcomeUnknown | mutation fails |
| `S5B-IDEM-005` | Owner proves no effect | retry allowed by profile |
| `S5B-IDEM-006` | Conflicting duplicate effects | quarantine |
| `S5B-IDEM-007` | Loss after retention admission | return recorded result |
| `S5B-IDEM-008` | No owner reconciliation capability | blocked/NotEvaluated |
| `S5B-IDEM-009` | Empty output treated NoChange | mutation fails |
| `S5B-IDEM-010` | Exact artifact already exists | NoChange with proof |

## Cancellation and lifecycle

| ID | Case | Expected |
|---|---|---|
| `S5B-LIFE-001` | Cancel before acquisition | Cancelled, no effects |
| `S5B-LIFE-002` | Cancel during owner run | owner safe stop, exact state retained |
| `S5B-LIFE-003` | Cancel after effect before response | reconcile; no blind retry |
| `S5B-LIFE-004` | Partial acquisition failure | reverse close all |
| `S5B-LIFE-005` | Background continuation after return | architecture failure |
| `S5B-LIFE-006` | Public success before closure | mutation fails |
| `S5B-LIFE-007` | Retention unavailable | no continuation/handle |
| `S5B-LIFE-008` | GC race during retention | explicit owner resolution/failure |

## Candidate and deactivation

| ID | Case | Expected |
|---|---|---|
| `S5B-CAND-001` | Build candidate through E5-A | exact artifact |
| `S5B-CAND-002` | Candidate bytes change after run | new identity; prior reviews stale |
| `S5B-CAND-003` | Validate exact candidate closure | pass/invalid payload |
| `S5B-CAND-004` | Metric eligibility treated authorization | mutation fails |
| `S5B-CAND-005` | Deactivation plan touches candidate partitions only | pass |
| `S5B-CAND-006` | Deactivation touches core/foreign partition | reject |
| `S5B-CAND-007` | Coverage downgrade omitted | invalid |
| `S5B-CAND-008` | Candidate artifact used as published core pack | mutation fails |

## Review authorization

| ID | Case | Expected |
|---|---|---|
| `S5B-REV-001` | Authorized exact review envelope | authorization valid |
| `S5B-REV-002` | GitHub login/repo owner authorizes review | reject |
| `S5B-REV-003` | OS/CLI/file owner authorizes review | reject |
| `S5B-REV-004` | Plain prose approval | reject |
| `S5B-REV-005` | Expired/revoked/replayed attestation | reject |
| `S5B-REV-006` | Role/scope mismatch | reject |
| `S5B-REV-007` | Authorized but candidate invalid | no approval record |
| `S5B-REV-008` | Candidate valid but unauthorized | no approval record |
| `S5B-REV-009` | Review note used as evidence/matcher input | mutation fails |
| `S5B-REV-010` | Exact immutable review record | pass |
| `S5B-REV-011` | Response loss after review append | recover same record |
| `S5B-REV-012` | Duplicate review counts quorum twice | mutation fails |
| `S5B-REV-013` | Conflicting reviews under policy | explicit conflict |
| `S5B-REV-014` | Supersede without exact prior record | reject |

## Holdout request and authorization

| ID | Case | Expected |
|---|---|---|
| `S5B-HOLD-001` | Candidate/run/profile frozen before request | pass |
| `S5B-HOLD-002` | Pack bytes change after grant | grant invalid |
| `S5B-HOLD-003` | Review authorization reused as holdout grant | reject |
| `S5B-HOLD-004` | Authorized exact holdout request | grant receipt |
| `S5B-HOLD-005` | Expired/revoked/replayed grant | reject |
| `S5B-HOLD-006` | Wrong holdout generation/digest | reject |
| `S5B-HOLD-007` | Caller-provided evaluator code | reject |
| `S5B-HOLD-008` | Raw vault credential in request/log | security failure |
| `S5B-HOLD-009` | Denied attempt omitted from audit | mutation fails |
| `S5B-HOLD-010` | Multiple grants chosen by newest | mutation fails |

## Holdout execution and consumption

| ID | Case | Expected |
|---|---|---|
| `S5B-HEX-001` | Exact authorized execution | immutable receipt |
| `S5B-HEX-002` | Default aggregate-only disclosure | hidden inputs remain sealed |
| `S5B-HEX-003` | Broader disclosure without grant | reject |
| `S5B-HEX-004` | Per-case labels disclosed | consumption per profile |
| `S5B-HEX-005` | Adaptive use changes descendant candidate | consumed lineage state |
| `S5B-HEX-006` | Consumed holdout called untouched | mutation fails |
| `S5B-HEX-007` | Access may have occurred; response lost | OutcomeUnknown/ContaminationUnknown |
| `S5B-HEX-008` | Blind second vault execution | mutation fails |
| `S5B-HEX-009` | Cancel after vault open | audit + conservative consumption |
| `S5B-HEX-010` | Audit chain missing event/digest | invalid |
| `S5B-HEX-011` | Local clock proves required trusted order | reject when insufficient |
| `S5B-HEX-012` | Vault close failure after result | service failure; no false success |

## Promotion submission

| ID | Case | Expected |
|---|---|---|
| `S5B-PROM-001` | Complete exact evidence and authorized reviews | Prepared/Validated |
| `S5B-PROM-002` | Missing hard-gate report | Blocked |
| `S5B-PROM-003` | Failed gate hidden by aggregate metrics | mutation fails |
| `S5B-PROM-004` | Missing/unauthorized review | Blocked |
| `S5B-PROM-005` | Incompatible holdout generation/consumption | Blocked |
| `S5B-PROM-006` | License/privacy/notice conflict | Blocked |
| `S5B-PROM-007` | Generalization claim exceeds provenance evidence | invalid |
| `S5B-PROM-008` | Submission mutates candidate bytes | reject |
| `S5B-PROM-009` | Prepared called published/promoted/active | mutation fails |
| `S5B-PROM-010` | E5-B changes core/default/current pointer | architecture failure |
| `S5B-PROM-011` | Exact submission validation read-only | pass/invalid |
| `S5B-PROM-012` | Response loss after submission commit | recover same submission |
| `S5B-PROM-013` | Input/profile changes | new submission identity |
| `S5B-PROM-014` | Required E5-C nonclaims absent | invalid |

## Status, result, and output

| ID | Case | Expected |
|---|---|---|
| `S5B-RES-001` | Status precedence | exact expected state |
| `S5B-RES-002` | Completed invalid validation | Complete + Invalid where specified |
| `S5B-RES-003` | OutcomeUnknown folded Failed/Cancelled | mutation fails |
| `S5B-RES-004` | Warning hides blocker | mutation fails |
| `S5B-RES-005` | Owner result rewritten | mutation fails |
| `S5B-RES-006` | Mandatory nonclaims omitted | validation fails |
| `S5B-RES-007` | Hidden holdout data in envelope | privacy failure |
| `S5B-RES-008` | Canonical repeated result | byte identical |
| `S5B-RES-009` | Timing/host/retry enters semantic digest | mutation fails |
| `S5B-RES-010` | Error contains credentials/private labels | redaction failure |

## Security and privacy

| ID | Case | Expected |
|---|---|---|
| `S5B-SEC-001` | Raw SQL/script/plugin/callback input | reject |
| `S5B-SEC-002` | Arbitrary filesystem/network/process/editor/client access | absent |
| `S5B-SEC-003` | Model/embedding/CBM call | absent |
| `S5B-SEC-004` | Source/label/review prose alters control flow | mutation fails |
| `S5B-SEC-005` | Cross-consumer privacy widening | reject |
| `S5B-SEC-006` | Raw key/token/signature/vault secret exposed | fail |
| `S5B-SEC-007` | Oversized/deep artifact or audit bomb | bounded failure |
| `S5B-SEC-008` | Cross-candidate/run/holdout substitution | reject |
| `S5B-SEC-009` | Application bypasses service | architecture failure |
| `S5B-SEC-010` | Holdout labels sent to default CLI text | fail |
| `S5B-SEC-011` | Unknown privacy/license | safest explicit state |
| `S5B-SEC-012` | Context/artifact treated as tool authorization | mutation fails |

## Determinism and freeze

| ID | Case | Expected |
|---|---|---|
| `S5B-DET-001` | 1/2/N workers | identical canonical artifacts/results |
| `S5B-DET-002` | Shuffled catalog/owner/result order | stable output |
| `S5B-DET-003` | Cold/warm cache/storage layout | same semantic output |
| `S5B-DET-004` | Host/path/clock/process differences | no identity change |
| `S5B-DET-005` | Independent histories same exact final inputs | same logical artifacts |
| `S5B-FIX-001` | Null pins while implementation not started | allowed |
| `S5B-FIX-002` | First E5-B Rust commit with required nulls | fail |
| `S5B-FIX-003` | All profiles/vectors/checksums frozen | pass |
| `S5B-FIX-004` | Cargo/.rs/workflow added in docs package | fail |

## Acceptance

E5-B cannot be marked implemented until every nondeferred test executes; exact artifact acquisition, owner delegation, idempotency, response-loss recovery, reviewer authorization, sealed-holdout audit/consumption, submission construction, retention, closure, privacy, CLI transport, determinism, and checksum gates pass; and no operation publishes or activates a core pack.
