# E5-C acceptance and mutation matrix

**Status:** normative executable gate. IDs are unique within E5-C.

## Submission and artifact

| ID | Case | Expected |
|---|---|---|
| `S5C-SUB-001` | Exact retained unblocked submission | revalidation pass |
| `S5C-SUB-002` | Newest/best/sole submission selector | reject |
| `S5C-SUB-003` | Submission label trusted without referenced artifacts | reject |
| `S5C-SUB-004` | Candidate bytes changed after submission | reject |
| `S5C-SUB-005` | Required hard-gate failure omitted | invalid |
| `S5C-SUB-006` | Unauthorized review despite valid metrics | blocked |
| `S5C-SUB-007` | Consumed holdout incompatible with policy | blocked |
| `S5C-SUB-008` | License/privacy/notice conflict | blocked |
| `S5C-SUB-009` | Partial/NotEvaluated required evidence | blocked |
| `S5C-ART-001` | Build distinct core artifact | new immutable ID |
| `S5C-ART-002` | Relabel candidate artifact as core | reject |
| `S5C-ART-003` | Repository/addon/owner/path condition in production rule | reject |
| `S5C-ART-004` | Unregistered graph output/producer namespace | reject |
| `S5C-ART-005` | Missing deactivation/stale-closure plan | invalid |
| `S5C-ART-006` | 1/2/N worker build | identical semantic bytes |
| `S5C-ART-007` | Shuffled inputs | identical semantic bytes |
| `S5C-ART-008` | Artifact mutation under same ID | reject |

## Attestation and signing

| ID | Case | Expected |
|---|---|---|
| `S5C-SIGN-001` | Complete provenance/SBOM/license set | valid |
| `S5C-SIGN-002` | Unknown dependency/license | blocked |
| `S5C-SIGN-003` | Authorized detached signature | verify |
| `S5C-SIGN-004` | Wrong artifact/attestation digest | reject |
| `S5C-SIGN-005` | Expired/revoked/untrusted key | reject |
| `S5C-SIGN-006` | GitHub/OS/CLI identity authorizes signing | reject |
| `S5C-SIGN-007` | Private key/token in request/fixture/log | fail |
| `S5C-SIGN-008` | Signature treated as semantic proof | mutation fails |
| `S5C-SIGN-009` | Lost response after possible signature effect | OutcomeUnknown |
| `S5C-SIGN-010` | Key rotation mutates artifact | mutation fails |

## Publication and read-back

| ID | Case | Expected |
|---|---|---|
| `S5C-PUB-001` | Publish exact validated artifact | PublishedInactive |
| `S5C-PUB-002` | Publication also changes current/default | reject |
| `S5C-PUB-003` | Same operation/digest retry | same publication |
| `S5C-PUB-004` | Same operation/different digest | reject |
| `S5C-PUB-005` | Response lost after catalog commit | reconcile exact publication |
| `S5C-PUB-006` | Fresh read-back validates all closure | ValidatedInactive |
| `S5C-PUB-007` | Missing object/digest mismatch | quarantine/fail |
| `S5C-PUB-008` | Invalid signature or SBOM on read-back | fail |
| `S5C-PUB-009` | Mark validated without fresh read-back | reject |
| `S5C-PUB-010` | Same name/digest only treated NoChange | reject |
| `S5C-PUB-011` | Public release/download produced | forbidden |
| `S5C-PUB-012` | Catalog list auto-selects activation target | mutation fails |

## Canary

| ID | Case | Expected |
|---|---|---|
| `S5C-CAN-001` | Exact bounded cohort plan | valid |
| `S5C-CAN-002` | Percentage without population/membership | reject |
| `S5C-CAN-003` | Random seed without frozen population | reject |
| `S5C-CAN-004` | Repository popularity/owner defines cohort | reject |
| `S5C-CAN-005` | Authorized exact assignment | CanaryAssigned |
| `S5C-CAN-006` | Assignment response loss | OutcomeUnknown/reconcile |
| `S5C-CAN-007` | Registered typed observation | append receipt |
| `S5C-CAN-008` | Free-form anecdote/issue count/model summary | reject |
| `S5C-CAN-009` | Missing required signal | InsufficientEvidence |
| `S5C-CAN-010` | Aggregate hides mandatory failure | mutation fails |
| `S5C-CAN-011` | Partial/conflicted observation | not pass |
| `S5C-CAN-012` | Canary pass rendered globally safe | mutation fails |
| `S5C-CAN-013` | Private cohort membership in default output | fail |
| `S5C-CAN-014` | Changed artifact reuses prior canary | reject |

## Rollout, activation, LKG

| ID | Case | Expected |
|---|---|---|
| `S5C-ROLL-001` | Finite staged rollout plan | valid |
| `S5C-ROLL-002` | Unbounded/implicit stage plan | reject |
| `S5C-ROLL-003` | Advance with all exact gates | stage receipt |
| `S5C-ROLL-004` | Advance because time elapsed/no complaint | reject |
| `S5C-ROLL-005` | Required signal partial/missing | pause/block |
| `S5C-ROLL-006` | Unauthorized advance | reject |
| `S5C-ROLL-007` | Pause records scope/evidence | RolloutPaused |
| `S5C-ACT-001` | Exact eligible target + fresh current + authorization | Active |
| `S5C-ACT-002` | Stale current CAS | reject |
| `S5C-ACT-003` | Cross-profile activation | reject |
| `S5C-ACT-004` | Activate newest/sole publication | reject |
| `S5C-ACT-005` | Activation response loss | OutcomeUnknown/reconcile |
| `S5C-LKG-001` | Explicit qualifying LKG designation | pass |
| `S5C-LKG-002` | Previous/newest inferred LKG | reject |
| `S5C-LKG-003` | LKG target unretained/revoked | reject |
| `S5C-LKG-004` | Same publication LKG for incompatible profile | reject |

## Rollback, revocation, and closure

| ID | Case | Expected |
|---|---|---|
| `S5C-RB-001` | Exact retained qualified rollback target | guarded rollback |
| `S5C-RB-002` | Previous/newest target shortcut | reject |
| `S5C-RB-003` | Stale current CAS | reject |
| `S5C-RB-004` | Rollback rewrites history/relabels failed target | mutation fails |
| `S5C-RB-005` | Response lost after CAS | OutcomeUnknown/reconcile |
| `S5C-REV-001` | Authorized scoped revocation | immutable record |
| `S5C-REV-002` | Revocation mutates historical validation | mutation fails |
| `S5C-REV-003` | Revoked active publication without required action | blocked incident |
| `S5C-DEACT-001` | Exact profile deactivation | explicit state/coverage |
| `S5C-DEACT-002` | Silent substitute another pack | reject |
| `S5C-CLOSE-001` | New target project/graph generations close stale pack partitions | pass |
| `S5C-CLOSE-002` | Stale target partition remains | fail |
| `S5C-CLOSE-003` | Foreign/core-independent/calibration partition changed | fail |
| `S5C-CLOSE-004` | Historical generation mutated | fail |
| `S5C-CLOSE-005` | Partial fleet/project closure called complete | mutation fails |
| `S5C-CLOSE-006` | Coverage change omitted | invalid |

## Lifecycle, security, and determinism

| ID | Case | Expected |
|---|---|---|
| `S5C-IDEM-001` | Same operation/digest at each effect | same effect |
| `S5C-IDEM-002` | Blind repeat under OutcomeUnknown | reject |
| `S5C-IDEM-003` | Conflicting duplicate effect | quarantine |
| `S5C-LIFE-001` | Retention before durable handle | pass |
| `S5C-LIFE-002` | Public success before mandatory close | fail |
| `S5C-LIFE-003` | Cancellation at every effect boundary | exact state/no background work |
| `S5C-LIFE-004` | GC removes active/canary/LKG/rollback evidence | fail |
| `S5C-LIFE-005` | Startup recovery activates orphan publication | reject |
| `S5C-SEC-001` | Raw SQL/script/plugin/model input | reject |
| `S5C-SEC-002` | Arbitrary filesystem/network/process/editor/client access | absent |
| `S5C-SEC-003` | Credential/private key/vault secret exposure | fail |
| `S5C-SEC-004` | Untrusted text alters profile/authorization/tool | fail |
| `S5C-SEC-005` | Public distribution in E5-C | reject |
| `S5C-SEC-006` | Oversized/deep cohort/observation/SBOM/audit data | bounded failure |
| `S5C-DET-001` | Worker/order/cache/storage variations | same semantic artifacts/results |
| `S5C-DET-002` | Host/path/clock/process enters semantic ID | mutation fails |
| `S5C-FIX-001` | Null pins while implementation not started | allowed |
| `S5C-FIX-002` | First Rust commit with required nulls | fail |
| `S5C-FIX-003` | All profiles/vectors/checksums frozen | pass |

## Acceptance

E5-C is incomplete until every nondeferred case executes with real owner, signing/authorization, store, canary/observation, reindex/graph closure, response-loss, retention/audit, privacy/license, platform, and benchmark implementations. Documentation fixtures alone are not passing implementation evidence.