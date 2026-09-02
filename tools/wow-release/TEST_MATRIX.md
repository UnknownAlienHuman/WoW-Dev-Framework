# `wow-release` E7-B acceptance matrix

**Status:** normative.

## Dependency and routing

| ID | Case | Expected |
|---|---|---|
| `R7B-ROUTE-001` | Tool imports only `wow-service` among framework crates | pass |
| `R7B-ROUTE-002` | Every command maps to exactly one service operation | pass |
| `R7B-ROUTE-003` | Tool composes build/evidence/sign/bundle/publish | fail |
| `R7B-ROUTE-004` | Tool accesses Git/Cargo/process/signing/provider/installer API | fail |
| `R7B-ROUTE-005` | Unknown command/field | pre-service reject |

## Source, build, evidence, signing, and bundle

| ID | Case | Expected |
|---|---|---|
| `R7B-SRC-001` | Exact source request -> source validate | one service call |
| `R7B-SRC-002` | cwd/branch/tag convenience used as source proof | reject |
| `R7B-PLAN-001` | Exact release plan validate | one service call |
| `R7B-BLD-001` | Build submit/get exact IDs | one service call each |
| `R7B-BLD-002` | Cargo/rustc/linker/shell/env options | reject |
| `R7B-REP-001` | Two exact build IDs compare | one service call |
| `R7B-REP-002` | Tool chooses newest/majority output | fail |
| `R7B-EVD-001` | Artifact/SBOM/provenance commands | one service call each |
| `R7B-EVD-002` | Tool invents missing test/evidence fields | fail |
| `R7B-SIGN-001` | Nonsecret exact signing request | one service call |
| `R7B-SIGN-002` | Private key/token/passphrase/socket/environment secret | reject |
| `R7B-SIGN-003` | Tool signs/verifies locally | fail |
| `R7B-BND-001` | Bundle build/validate exact request | one service call |
| `R7B-BND-002` | Tool archives/extracts/repacks | fail |
| `R7B-SUP-001` | Support/candidate validation | one service call |
| `R7B-SUP-002` | Force/skip/assume-supported option | reject |

## Channel, update manifest, revocation, retirement

| ID | Case | Expected |
|---|---|---|
| `R7B-CH-001` | Channel prepare exact candidate/channel | one service call |
| `R7B-CH-002` | Channel publish exact frozen plan/current guard | one service call |
| `R7B-CH-003` | Tool supplies GitHub tag/release/assets/API payload | reject |
| `R7B-CH-004` | Tool selects latest/newest/tag candidate | reject |
| `R7B-CH-005` | Tool treats ambient GitHub/CI identity as authorization | fail |
| `R7B-CH-006` | Upload response loss triggers direct second publish | reject |
| `R7B-UPM-001` | Update manifest build/validate | one service call |
| `R7B-REV-001` | Exact revoke request | one service call |
| `R7B-REV-002` | Provider asset deletion substitutes revocation | reject |
| `R7B-RET-001` | Exact retire request | one service call |
| `R7B-REC-001` | Exact operation reconcile | one service call, no redispatch |
| `R7B-REC-002` | `--retry-unknown` | reject |

## Inputs, security, output, and lifecycle

| ID | Case | Expected |
|---|---|---|
| `R7B-IN-001` | Strict bounded JSON/file/stdin input | pass |
| `R7B-IN-002` | Unknown/deep/oversized/duplicate-key/polyglot | reject |
| `R7B-IN-003` | Two stdin consumers | reject |
| `R7B-IN-004` | Include/interpolation/env expansion/plugin/script | reject |
| `R7B-SEC-001` | Token/key/password/private endpoint/URL/header in input | reject/redact |
| `R7B-SEC-002` | Generic shell/process/HTTP/GitHub/provider/SQL callback | absent |
| `R7B-SEC-003` | Path traversal/reparse/device/UNC/ADS | reject by profile |
| `R7B-SEC-004` | Source/release text creates command/path/profile | data only |
| `R7B-SEC-005` | Sensitive data/raw payload/private path in logs/errors | fail |
| `R7B-OUT-001` | JSON | exact service bytes + LF |
| `R7B-OUT-002` | Artifact | exact approved bytes |
| `R7B-OUT-003` | Text preserves gates/NotEvaluated/OutcomeUnknown/nonclaims | pass |
| `R7B-OUT-004` | Banner/progress on machine stdout | fail |
| `R7B-LIFE-001` | Signal cancellation | one typed cancellation/no retry |
| `R7B-LIFE-002` | Broken pipe/output failure | no second service call |
| `R7B-LIFE-003` | Daemon disconnect falls back to direct mode | reject |
| `R7B-LIFE-004` | Background build/sign/upload/poll/cleanup | fail |
| `R7B-CI-001` | CI invokes exact implemented tool commands | allowed after freeze |
| `R7B-CI-002` | Workflow contains release logic/automatic publish | reject |
| `R7B-DET-001` | Transport/locale/terminal/timing changes | same semantic request/result |
| `R7B-FIX-001` | Null pins before implementation | allowed |
| `R7B-FIX-002` | First Rust commit with required nulls | fail |

## Acceptance

The release tool is incomplete until all nondeferred tests execute against the real E7-B service, including direct and explicitly supported daemon transport. A locally successful Cargo command or GitHub upload is not release-tool conformance.