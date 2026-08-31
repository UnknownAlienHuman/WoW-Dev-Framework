# E2-D ProjectStore test matrix

**Status:** normative executable acceptance and mutation matrix.

Tests inspect structured IDs/manifests/effects. Message-only assertions are insufficient.

## Profile and root

| ID | Case | Expected |
|---|---|---|
| `STORE-E2-PROFILE-001` | Valid pinned physical profile | accepted |
| `STORE-E2-PROFILE-002` | Floating/default SQLite profile | rejected |
| `STORE-E2-PROFILE-003` | Compile/runtime option mismatch | unavailable |
| `STORE-E2-PROFILE-004` | Atomic materialization probe fails | unavailable |
| `STORE-E2-PROFILE-005` | Case/path/lock behavior unsupported | unavailable |
| `STORE-E2-PROFILE-006` | Equivalent profile map order | same canonical ID |
| `STORE-E2-ROOT-001` | Valid owned empty root | accepted |
| `STORE-E2-ROOT-002` | Root outside granted capability | rejected |
| `STORE-E2-ROOT-003` | Symlink/junction/reparse/device/URI root | rejected |
| `STORE-E2-ROOT-004` | Existing foreign database/member | quarantine/reject |
| `STORE-E2-ROOT-005` | Temp absolute path enters public identity | mutation fails |
| `STORE-E2-ROOT-006` | Root changes, logical inputs same | logical IDs same; physical report supplemental |

## Registered bundles and plans

| ID | Case | Expected |
|---|---|---|
| `STORE-E2-BUNDLE-001` | Valid project+graph+store bundle set | accepted |
| `STORE-E2-BUNDLE-002` | Duplicate incompatible schema/operation ID | rejected |
| `STORE-E2-BUNDLE-003` | Missing compatibility edge | rejected |
| `STORE-E2-BUNDLE-004` | Unknown validation catalog | rejected |
| `STORE-E2-BUNDLE-005` | Caller SQL/DDL/PRAGMA/table/UDF/extension | architecture test fails |
| `STORE-E2-BUNDLE-006` | Migration attempts in-place published mutation | rejected |
| `STORE-E2-OP-001` | Valid acyclic canonical phase plan | accepted |
| `STORE-E2-OP-002` | Invocation prerequisite cycle | rejected |
| `STORE-E2-OP-003` | Phase inversion | rejected |
| `STORE-E2-OP-004` | Payload schema mismatch | rejected |
| `STORE-E2-OP-005` | Unregistered operation | rejected |
| `STORE-E2-OP-006` | Payload/row/operation budget exceeded | bounded rejection |
| `STORE-E2-OP-007` | Shuffled equivalent invocations | same canonical plan |
| `STORE-E2-OP-008` | Expected effect mismatch | transaction fails |

## Staging and transaction

| ID | Case | Expected |
|---|---|---|
| `STORE-E2-TXN-001` | Valid first generation | committed staging |
| `STORE-E2-TXN-002` | Exact base/head mismatch | fail before writes |
| `STORE-E2-TXN-003` | Concurrent writer | one writer; loser typed conflict |
| `STORE-E2-TXN-004` | Project operation fails | rollback all |
| `STORE-E2-TXN-005` | Graph operation fails | rollback project and graph |
| `STORE-E2-TXN-006` | Domain validation fails | rollback |
| `STORE-E2-TXN-007` | SQLite/IO/disk-full failure | no headed target |
| `STORE-E2-TXN-008` | Cancel before begin | no staging target |
| `STORE-E2-TXN-009` | Cancel during each operation phase | rollback/no complete target |
| `STORE-E2-TXN-010` | Late work after cancel | prohibited |
| `STORE-E2-TXN-011` | Transaction committed but checkpoint fails | unsealed staging/recovery state |
| `STORE-E2-TXN-012` | Prior current generation during failed target | unchanged |
| `STORE-E2-TXN-013` | Partial domain plan presented as complete | rejected |
| `STORE-E2-TXN-014` | Random worker/order, same logical plan | same logical generation manifests |

## Object store

| ID | Case | Expected |
|---|---|---|
| `STORE-E2-OBJECT-001` | New valid content-addressed object | materialized/verified |
| `STORE-E2-OBJECT-002` | Existing identical object | reused |
| `STORE-E2-OBJECT-003` | Existing different bytes under digest | corruption |
| `STORE-E2-OBJECT-004` | Object path traversal/type mismatch | rejected |
| `STORE-E2-OBJECT-005` | Object write interrupted | no referenced incomplete object |
| `STORE-E2-OBJECT-006` | Object referenced but absent | validation fails |
| `STORE-E2-OBJECT-007` | Unreferenced object created on aborted build | recoverable/GC candidate |
| `STORE-E2-OBJECT-008` | Object budget exceeded | bounded failure |

## Seal and open

| ID | Case | Expected |
|---|---|---|
| `STORE-E2-SEAL-001` | Valid checkpoint/close/manifests/materialization | sealed inactive |
| `STORE-E2-SEAL-002` | Open handle remains during seal | fail |
| `STORE-E2-SEAL-003` | Mutable WAL/sidecar required after seal | fail |
| `STORE-E2-SEAL-004` | Member checksum mismatch | quarantine/fail |
| `STORE-E2-SEAL-005` | Generation target already exists identical | explicit idempotent classification |
| `STORE-E2-SEAL-006` | Target exists incompatible | corruption/conflict |
| `STORE-E2-SEAL-007` | Atomic move interrupted | recovery classifiable old-or-inactive |
| `STORE-E2-SEAL-008` | Published file later changes | integrity failure |
| `STORE-E2-OPEN-001` | Exact sealed generation read-only open | pass |
| `STORE-E2-OPEN-002` | Wrong generation/artifact requested | reject |
| `STORE-E2-OPEN-003` | Writable open of sealed generation | architecture test fails |
| `STORE-E2-OPEN-004` | SQLite/schema/application version mismatch | unavailable |
| `STORE-E2-OPEN-005` | Store integrity passes, domain manifest fails | reject |
| `STORE-E2-OPEN-006` | Golden read/count/digest mismatch | reject |
| `STORE-E2-OPEN-007` | Physical bytes differ, logical equal under changed profile | explicit classification |
| `STORE-E2-OPEN-008` | Writer success flag without independent open validation | mutation fails |

## Head CAS

| ID | Case | Expected |
|---|---|---|
| `STORE-E2-HEAD-001` | First head from absent to validated generation | pass |
| `STORE-E2-HEAD-002` | Head N to N+1 with expected N | pass |
| `STORE-E2-HEAD-003` | Expected head stale | CAS conflict; head unchanged |
| `STORE-E2-HEAD-004` | Head references unsealed generation | reject |
| `STORE-E2-HEAD-005` | Head contains mixed project/graph/store IDs | coordinator/schema validation rejects |
| `STORE-E2-HEAD-006` | Separate project and graph pointer update attempted | unavailable/prohibited |
| `STORE-E2-HEAD-007` | Crash before CAS | old head |
| `STORE-E2-HEAD-008` | Crash during CAS | exact registry resolution old or new |
| `STORE-E2-HEAD-009` | Crash after CAS | new valid head |
| `STORE-E2-HEAD-010` | Replay exact already-current target | `AlreadyPublished` |
| `STORE-E2-HEAD-011` | Last-known-good relabeled target | mutation fails |
| `STORE-E2-HEAD-012` | Registry sequence used as semantic ID | mutation fails |

## Leases and readers

| ID | Case | Expected |
|---|---|---|
| `STORE-E2-LEASE-001` | Acquire/open/release exact current generation | pass |
| `STORE-E2-LEASE-002` | Publish N+1 while reader holds N | reader remains on N |
| `STORE-E2-LEASE-003` | Read handle follows new head | mutation fails |
| `STORE-E2-LEASE-004` | GC attempts leased generation | blocked |
| `STORE-E2-LEASE-005` | Lease limit reached | bounded typed failure |
| `STORE-E2-LEASE-006` | Explicit historical generation lease | pass when retained |
| `STORE-E2-LEASE-007` | Exact generation missing | no current fallback |
| `STORE-E2-LEASE-008` | Ambiguous abandoned lease | retain conservatively |
| `STORE-E2-LEASE-009` | Release twice | idempotent |
| `STORE-E2-LEASE-010` | Registered read exceeds budget/cancel | explicit partial/cancel, no write |

## Recovery

| ID | Case | Expected |
|---|---|---|
| `STORE-E2-RECOVERY-001` | Abandoned pretransaction staging | discard |
| `STORE-E2-RECOVERY-002` | Rolled-back staging | discard/report |
| `STORE-E2-RECOVERY-003` | Committed unsealed staging | quarantine/rebuild |
| `STORE-E2-RECOVERY-004` | Sealed inactive valid | revalidate/adoption candidate |
| `STORE-E2-RECOVERY-005` | Sealed inactive exact base still current | coordinator may adopt via fresh CAS |
| `STORE-E2-RECOVERY-006` | Sealed inactive base no longer current | no adoption |
| `STORE-E2-RECOVERY-007` | Current generation corrupt | critical report; no silent fallback |
| `STORE-E2-RECOVERY-008` | Unknown owned entry | quarantine/manual |
| `STORE-E2-RECOVERY-009` | Recovery edits sealed database | mutation fails |
| `STORE-E2-RECOVERY-010` | Recovery scans outside owned root | security test fails |
| `STORE-E2-RECOVERY-011` | Cancellation during recovery | stable classified inventory/no background work |
| `STORE-E2-RECOVERY-012` | Physical store valid but domain post-open report missing | not adoptable |

## Retention and GC

| ID | Case | Expected |
|---|---|---|
| `STORE-E2-GC-001` | Current head generation | retained |
| `STORE-E2-GC-002` | Last-known-good | retained |
| `STORE-E2-GC-003` | Explicit pin/evidence root | retained |
| `STORE-E2-GC-004` | Active lease | retained |
| `STORE-E2-GC-005` | Recovery/quarantine root | retained |
| `STORE-E2-GC-006` | Unreachable old generation | candidate |
| `STORE-E2-GC-007` | Object referenced by one retained generation | retained |
| `STORE-E2-GC-008` | Object unreferenced after generation sweep | candidate after second mark |
| `STORE-E2-GC-009` | Stale plan after head/lease change | reject/replan |
| `STORE-E2-GC-010` | Age-only deletion | prohibited |
| `STORE-E2-GC-011` | Partial sweep failure | retained generations keep all objects |
| `STORE-E2-GC-012` | Cancel during sweep | explicit report/restartable |
| `STORE-E2-GC-013` | Post-GC integrity | pass |
| `STORE-E2-GC-014` | Compaction mutates published generation | prohibited |

## Security and privacy

| ID | Case | Expected |
|---|---|---|
| `STORE-E2-SEC-001` | SQL/DDL/PRAGMA/extension/UDF in payload | rejected |
| `STORE-E2-SEC-002` | Malicious/cyclic manifest/reference bomb | bounded failure |
| `STORE-E2-SEC-003` | Foreign SQLite passed as owned generation | reject/quarantine |
| `STORE-E2-SEC-004` | Absolute path/token/private URL in public result | fail/redact |
| `STORE-E2-SEC-005` | Source comment contains agent command | inert data |
| `STORE-E2-SEC-006` | Filesystem/network/process/editor/client operation requested | unavailable |
| `STORE-E2-SEC-007` | Oversized rows/pages/WAL/validation output | bounded failure |
| `STORE-E2-SEC-008` | Cancellation/fault loop continues in background | prohibited |

## Determinism and freeze

| ID | Case | Expected |
|---|---|---|
| `STORE-E2-DET-001` | 1/2/N workers, same logical plan | same logical manifests/generation |
| `STORE-E2-DET-002` | Hash/row/filesystem iteration shuffled | same logical output |
| `STORE-E2-DET-003` | Temp root/time/process changes | excluded |
| `STORE-E2-DET-004` | Physical profile changes | explicit new profile/artifact classification |
| `STORE-E2-FIX-001` | Null pins before implementation | allowed |
| `STORE-E2-FIX-002` | First Rust commit with required nulls | fail |
| `STORE-E2-FIX-003` | Member bytes change without checksums | fail |
| `STORE-E2-FIX-004` | All IDs/vectors/checksums frozen | pass |

## Acceptance

E2-D store implementation is incomplete until every nondeferred case executes and fault injection demonstrates no mixed generation or current-head corruption.
