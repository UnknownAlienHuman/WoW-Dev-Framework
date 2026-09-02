# E5-C CLI acceptance matrix

**Status:** normative.

| ID | Case | Expected |
|---|---|---|
| `A5C-ROUTE-001` | Every command maps 1:1 to service operation | pass |
| `A5C-ROUTE-002` | App imports only wow-service | pass |
| `A5C-ROUTE-003` | Valid command invokes service once | pass |
| `A5C-ROUTE-004` | App composes multiple service calls | fail |
| `A5C-IN-001` | Exact IDs and digest/CAS guards | pass through |
| `A5C-IN-002` | latest/best/previous/default selector | exit 64 |
| `A5C-IN-003` | unknown/deep/oversized/polyglot JSON | reject |
| `A5C-IN-004` | two stdin consumers | reject |
| `A5C-IN-005` | cwd/home/env/Git/editor/WoW discovery | fail |
| `A5C-IN-006` | include/interpolation/script/plugin/archive | reject |
| `A5C-SIGN-001` | nonsecret exact signing request envelope | one service call |
| `A5C-SIGN-002` | private key/KMS token/vault credential flag | exit 64 |
| `A5C-SIGN-003` | app signs or verifies locally | fail |
| `A5C-PUB-001` | publish command | one service call |
| `A5C-PUB-002` | publish text says active/default | fail |
| `A5C-PUB-003` | public release/download command | absent |
| `A5C-CAN-001` | canary exact plan/start/status/observe/evaluate | mapped |
| `A5C-CAN-002` | app constructs random/popularity cohort | fail |
| `A5C-CAN-003` | default output leaks private cohort/observation | fail |
| `A5C-CAN-004` | canary pass text says globally safe | fail |
| `A5C-ROLL-001` | rollout plan/advance/pause | mapped |
| `A5C-ROLL-002` | auto-advance/time-only option | reject |
| `A5C-ACT-001` | activate with exact current guard | mapped |
| `A5C-ACT-002` | force without CAS/current digest | reject |
| `A5C-LKG-001` | exact LKG get/designate | mapped |
| `A5C-LKG-002` | previous/newest inferred LKG | reject |
| `A5C-RB-001` | exact rollback target/current guard | mapped |
| `A5C-RB-002` | app chooses previous target | fail |
| `A5C-REV-001` | revoke/deactivate/closure validate | mapped |
| `A5C-REV-002` | app edits store/project/graph directly | fail |
| `A5C-OUT-001` | envelope JSON | exact bytes + LF |
| `A5C-OUT-002` | artifact output | exact eligible bytes |
| `A5C-OUT-003` | text preserves scoped states/nonclaims | pass |
| `A5C-OUT-004` | text hides OutcomeUnknown/partial closure | fail |
| `A5C-OUT-005` | status/exit mapping | exact |
| `A5C-SEC-001` | GitHub/OS/CLI identity authorizes effect | reject |
| `A5C-SEC-002` | raw SQL/script/model/tool execution | absent |
| `A5C-SEC-003` | path/link/device/UNC/ADS attack | reject by profile |
| `A5C-SEC-004` | credentials/private data in log/error | fail |
| `A5C-LIFE-001` | cancellation | one typed service cancellation, exit 130 |
| `A5C-LIFE-002` | broken pipe/output failure | no second service call, exit 4 |
| `A5C-DET-001` | locale/terminal/timing changes | same machine bytes/exit |
| `A5C-FIX-001` | null pins before implementation | allowed |
| `A5C-FIX-002` | first Rust commit with required nulls | fail |

Implementation requires all nondeferred cases plus platform file/signal/broken-pipe tests and exact frozen bytes.