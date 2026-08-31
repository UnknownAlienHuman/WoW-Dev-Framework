# E1-D test matrix

**Status:** normative executable acceptance and mutation matrix.

## Configuration and prerequisites

| ID | Case | Expected |
|---|---|---|
| `PACK-CONFIG-001` | Exact valid build request | accepted |
| `PACK-CONFIG-002` | Implicit current/source/component/layout | rejected |
| `PACK-CONFIG-003` | Mixed profile/reference generation | rejected |
| `PACK-CONFIG-004` | Unfrozen prerequisite implementation/fixture | blocked |
| `PACK-CONFIG-005` | Incompatible store/reference/annotation contracts | rejected |
| `PACK-CONFIG-006` | Invalid budgets/eligibility target | rejected |

## Build state machine

| ID | Case | Expected |
|---|---|---|
| `PACK-BUILD-001` | Complete fixture build | fixture candidate |
| `PACK-BUILD-002` | Complete real candidate build | candidate |
| `PACK-BUILD-003` | All mandatory E1 gates pass | validated-local |
| `PACK-BUILD-004` | State skipped/reordered | fail |
| `PACK-BUILD-005` | ReferenceData build/store/open failure | no pack completion |
| `PACK-BUILD-006` | Reference partial/conflict blocker | candidate/blocked; exact gates |
| `PACK-BUILD-007` | Annotation build/source-map/loss failure | candidate/blocked; no validated-local |
| `PACK-BUILD-008` | Parity/consumer blocker | exact profile-scoped block |
| `PACK-BUILD-009` | Component result generation mismatch | fail |
| `PACK-BUILD-010` | Silent weaker component/profile fallback | mutation fails |

## Assembly and layout

| ID | Case | Expected |
|---|---|---|
| `PACK-ASSEMBLY-001` | Exact required member set | pass |
| `PACK-ASSEMBLY-002` | Missing required member | rejected |
| `PACK-ASSEMBLY-003` | Undeclared extra member | rejected |
| `PACK-ASSEMBLY-004` | Duplicate/case-colliding path | rejected |
| `PACK-ASSEMBLY-005` | Traversal/absolute/device/symlink escape | rejected |
| `PACK-ASSEMBLY-006` | Member profile/generation mismatch | rejected |
| `PACK-ASSEMBLY-007` | Written bytes differ from plan | rejected |
| `PACK-ASSEMBLY-008` | Manifest/checksum identity cycle | rejected |
| `PACK-ASSEMBLY-009` | Full Blizzard source/runtime addon file included by default | rejected |
| `PACK-ASSEMBLY-010` | Executable bit/source script | rejected |

## Validation

| ID | Case | Expected |
|---|---|---|
| `PACK-VALIDATE-001` | Independent validation of valid candidate | same eligibility |
| `PACK-VALIDATE-002` | Validator repairs missing checksum/member | mutation fails |
| `PACK-VALIDATE-003` | Altered member length/digest | fail |
| `PACK-VALIDATE-004` | Store mutable/corrupt/wrong schema | fail |
| `PACK-VALIDATE-005` | ReferenceView golden mismatch | fail |
| `PACK-VALIDATE-006` | Annotation syntax/file/map span mismatch | fail |
| `PACK-VALIDATE-007` | Blocking loss omitted or report truncated | fail |
| `PACK-VALIDATE-008` | Oracle/consumer result from another pin | fail |
| `PACK-VALIDATE-009` | Unknown mandatory license/redistribution | block |
| `PACK-VALIDATE-010` | Mandatory check unavailable | NotEvaluated and block |
| `PACK-VALIDATE-011` | Manifest says validated but derived gates fail | reject claim |

## Determinism and rebuild

| ID | Case | Expected |
|---|---|---|
| `PACK-DET-001` | 1/2/N workers | equal required semantic/canonical bytes |
| `PACK-DET-002` | Shuffled source/member/store row order | equal required outputs |
| `PACK-DET-003` | Clock/temp/root/host changes | no canonical identity change |
| `PACK-DET-004` | Annotation split/order follows worker completion | mutation fails |
| `PACK-DET-005` | ReferenceData logical manifest differs | fail |
| `PACK-DET-006` | Canonical JSON or annotation bytes differ | fail |
| `PACK-DET-007` | Object encoding differs under same profile | fail |
| `PACK-DET-008` | SQLite bytes differ but logical profile allows | classified, not false failure |
| `PACK-DET-009` | SQLite byte equality claimed without store guarantee | fail |
| `PACK-DET-010` | Archive equality claimed without container profile | NotEvaluated/block claim |

## Cancellation, failure, recovery

| ID | Case | Expected |
|---|---|---|
| `PACK-CANCEL-001` | Cancel before component build | no staging finalization |
| `PACK-CANCEL-002` | Cancel after immutable ReferenceStore publication | store retains identity; no pack claim |
| `PACK-CANCEL-003` | Cancel during annotation/materialization | partial quarantined/cleaned |
| `PACK-CANCEL-004` | Cancel after candidate validation before final rename | prior destination unchanged |
| `PACK-CANCEL-005` | Late work after cancel | mutation fails |
| `PACK-RECOVER-001` | Failed build with prior destination | prior bytes/digest unchanged |
| `PACK-RECOVER-002` | Resume without exact identities | rejected |
| `PACK-RECOVER-003` | Reuse exact content-addressed objects | allowed with validation |

## Application and CLI

| ID | Case | Expected |
|---|---|---|
| `PACK-APP-001` | `build` exact arguments | service request + plan materialization |
| `PACK-APP-002` | `validate` exact pack root | read-only service validation |
| `PACK-APP-003` | `rebuild-compare` exact scratch root | repeated isolated builds |
| `PACK-APP-004` | Missing source/output/current defaults | usage error |
| `PACK-APP-005` | JSON and text projections | same semantics |
| `PACK-APP-006` | Frozen exit-code classes | exact mapping |
| `PACK-APP-007` | Direct lower-crate import/domain orchestration | architecture test fails |
| `PACK-APP-008` | Generic shell/network/repository script | prohibited |
| `PACK-APP-009` | Atomic finalization failure | prior destination unchanged |

## Security and privacy

| ID | Case | Expected |
|---|---|---|
| `PACK-SEC-001` | Malicious source comment/instruction | data only; never executed/followed |
| `PACK-SEC-002` | Path/symlink/device escape | rejected |
| `PACK-SEC-003` | Huge/deep/member/decompression payload | bounded failure |
| `PACK-SEC-004` | Token/private root/private URL in output | rejected/redacted |
| `PACK-SEC-005` | Generated annotation/source execution | prohibited |
| `PACK-SEC-006` | User/editor settings mutation | none |
| `PACK-SEC-007` | Upload/sign/release/CI attempt | unavailable E1 |

## Freeze gate

| ID | Case | Expected |
|---|---|---|
| `PACK-FIX-001` | Documentation-only null pins/checksums | allowed before implementation |
| `PACK-FIX-002` | First Rust commit with required nulls | fail |
| `PACK-FIX-003` | Example bytes changed without checksum update | fail |
| `PACK-FIX-004` | Component identity/profile changed without pack vectors | fail |
| `PACK-FIX-005` | All member/bundle/checksums frozen | pass |

## Acceptance

E1-D is incomplete until all nondeferred cases execute, every build/validation/rebuild result is structured and deterministic, no component responsibility leaks into service/app, and failure/cancellation/security tests preserve prior outputs.
