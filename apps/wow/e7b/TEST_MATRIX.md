# E7-B public release/update client acceptance matrix

**Status:** normative.

## Routing and dependency

| ID | Case | Expected |
|---|---|---|
| `A7B-ROUTE-001` | Every documented command maps to one service operation | pass |
| `A7B-ROUTE-002` | App imports only `wow-service` | pass |
| `A7B-ROUTE-003` | Valid command invokes service exactly once | pass |
| `A7B-ROUTE-004` | App composes check/plan/apply/rollback locally | fail |
| `A7B-ROUTE-005` | App opens build/sign/publisher/installer/store APIs | fail |

## Local identity and verification

| ID | Case | Expected |
|---|---|---|
| `A7B-VER-001` | `wow version` reports exact build/release/registry IDs | pass, no network |
| `A7B-VER-002` | Version inferred from filename/directory/tag | reject |
| `A7B-VER-003` | `release status` preserves partial/revoked/retired state | pass |
| `A7B-VER-004` | Exact local bundle verify | one service call |
| `A7B-VER-005` | App extracts/executes bundle before service validation | fail |
| `A7B-VER-006` | Extension/MIME/name treated as bundle identity | fail |
| `A7B-INS-001` | Installation validate with exact selector/resolution | pass |
| `A7B-INS-002` | PATH/registry/cwd/executable directory inferred install | reject |

## Update check and plan

| ID | Case | Expected |
|---|---|---|
| `A7B-CHK-001` | Explicit update check | one service call and network policy visible |
| `A7B-CHK-002` | Startup/background check | absent |
| `A7B-CHK-003` | Check downloads or installs | fail |
| `A7B-CHK-004` | No-update versus revoked/retired/unsupported preserved | pass |
| `A7B-CHK-005` | Update available called installed/safe for all targets | fail |
| `A7B-PLAN-001` | Exact manifest/current guard plan | one service call |
| `A7B-PLAN-002` | Plan mutates current installation | fail |
| `A7B-PLAN-003` | latest/newest/tag/channel name substituted exact manifest | reject |
| `A7B-PLAN-004` | App chooses helper/migration/rollback target | fail |

## Apply and rollback

| ID | Case | Expected |
|---|---|---|
| `A7B-APPLY-001` | Exact frozen update plan/current digest | one service call |
| `A7B-APPLY-002` | Replacement fields accepted by apply | reject |
| `A7B-APPLY-003` | App overwrites running executable | fail |
| `A7B-APPLY-004` | App runs shell/helper command or bundle script | fail |
| `A7B-APPLY-005` | Active daemon/LSP/MCP not drained but update proceeds | blocked |
| `A7B-APPLY-006` | Handoff initiation rendered final Updated | fail |
| `A7B-APPLY-007` | Partial migration/self-check hidden | fail |
| `A7B-RB-001` | Exact retained rollback target/current digest | one service call |
| `A7B-RB-002` | previous/newest/version/directory target | reject |
| `A7B-RB-003` | App restores/deletes store/config/files directly | fail |
| `A7B-RB-004` | Rollback history rewrite hidden in text | fail |

## Reconciliation and lifecycle

| ID | Case | Expected |
|---|---|---|
| `A7B-REC-001` | Exact operation/request reconciliation | one read/reconcile service call |
| `A7B-REC-002` | Reconcile accepts replacement target | reject |
| `A7B-REC-003` | `--retry-unknown` or automatic redispatch | reject |
| `A7B-REC-004` | OutcomeUnknown rendered failed/succeeded | fail |
| `A7B-REC-005` | Broken pipe/output failure repeats service | fail |
| `A7B-REC-006` | Cancellation deletes staging/backup/evidence | fail |
| `A7B-REC-007` | Abrupt process loss later reconciles exact state | pass |
| `A7B-REC-008` | Background cleanup/update continues after return | fail |

## Inputs, paths, secrets, and network

| ID | Case | Expected |
|---|---|---|
| `A7B-IN-001` | Strict known options/config fields | pass |
| `A7B-IN-002` | Unknown/deep/oversized/duplicate-key input | reject |
| `A7B-IN-003` | Include/interpolation/env expansion/script/plugin | reject |
| `A7B-IN-004` | Two stdin consumers | reject |
| `A7B-IN-005` | `--force`, skip/ignore/downgrade flags | reject |
| `A7B-SEC-001` | Token/key/password/private endpoint in argv/config | reject/redact |
| `A7B-SEC-002` | Arbitrary URL/header/provider API/helper path | reject |
| `A7B-SEC-003` | Path traversal/reparse/device/UNC/ADS/collision | reject by profile |
| `A7B-SEC-004` | Manifest/release text creates command/path/option | data only |
| `A7B-SEC-005` | Hidden telemetry/crash upload/remote config | absent |
| `A7B-SEC-006` | Local-only commands perform network access | fail |
| `A7B-SEC-007` | Source/user/private path/secret in logs/errors | fail |

## Output and platform

| ID | Case | Expected |
|---|---|---|
| `A7B-OUT-001` | JSON output | exact service bytes + LF |
| `A7B-OUT-002` | Artifact output | exact approved public bytes |
| `A7B-OUT-003` | Text preserves all stages/IDs/nonclaims | pass |
| `A7B-OUT-004` | Signature/channel/update/install conflated | fail |
| `A7B-OUT-005` | UpdateAvailable/no-change/action-required exit mapping | exact |
| `A7B-OUT-006` | OutcomeUnknown/handoff/partial status nonzero and visible | pass |
| `A7B-WIN-001` | Console/PowerShell/cmd/Unicode/space/long-path behavior | pass profile |
| `A7B-WIN-002` | Running executable/helper/lock/antivirus scenario | exact result |
| `A7B-WIN-003` | Daemon/LSP/MCP drain before handoff | pass |
| `A7B-WIN-004` | UAC/elevation/reboot behavior not implemented but advertised | reject |
| `A7B-DET-001` | Locale/terminal/cwd/timing changes | same semantic request/output |
| `A7B-FIX-001` | Null pins before implementation | allowed |
| `A7B-FIX-002` | First Rust commit with required nulls | fail |

## Acceptance

The E7-B update client is incomplete until every nondeferred case runs against the real E7-B service and Windows installation/helper adapters. CLI parsing examples or a locally copied executable are not update/install evidence.