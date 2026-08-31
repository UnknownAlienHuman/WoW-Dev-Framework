# E2-D idempotency and architecture-consolidation test supplement

**Status:** normative; these cases supplement [`TEST_MATRIX.md`](TEST_MATRIX.md).

## Operation idempotency

| ID | Case | Expected |
|---|---|---|
| `PSTORE-IDEMP-001` | Same operation ID and same digest before work | one operation begins |
| `PSTORE-IDEMP-002` | Same operation ID and different digest | `project_store_idempotency_key_conflict` |
| `PSTORE-IDEMP-003` | Retry after sealed unreferenced partitions | validate/reuse exact partitions |
| `PSTORE-IDEMP-004` | Retry after inactive generation commit | revalidate exact target |
| `PSTORE-IDEMP-005` | Retry after validation before activation | exact-base CAS only |
| `PSTORE-IDEMP-006` | Retry after activation response loss | return existing receipt, no second activation |
| `PSTORE-IDEMP-007` | Same target under different operation ID | classify no-change/reuse/stale/collision |
| `PSTORE-IDEMP-008` | Existing ID with different content | quarantine/reject |
| `PSTORE-IDEMP-009` | Cancelled operation resumed without permitted recovery class | reject |
| `PSTORE-IDEMP-010` | Quarantined target resumed as authoritative | mutation fails |

## Crash and recovery classification

| ID | Case | Expected |
|---|---|---|
| `PSTORE-RECOVER-001` | Caller error while activation commit outcome unknown | read durable state first |
| `PSTORE-RECOVER-002` | Process kill after partition commit | old current, inert sealed partition |
| `PSTORE-RECOVER-003` | Process kill after inactive generation commit | recoverable inactive target |
| `PSTORE-RECOVER-004` | Process kill after validation report commit | validated inactive target |
| `PSTORE-RECOVER-005` | Process kill after activation commit before response | new current, existing receipt returned |
| `PSTORE-RECOVER-006` | Current advanced before inactive recovery | target remains stale inactive |
| `PSTORE-RECOVER-007` | WAL/SHM present with no classification | no outcome inference |
| `PSTORE-RECOVER-008` | Recovery budget exhausted | partial/NotEvaluated; no destructive action |
| `PSTORE-RECOVER-009` | Current closure corrupt | block normal open; explicit recovery |
| `PSTORE-RECOVER-010` | Recovery auto-selects last-known-good | mutation fails |

## Reader, cursor, and GC races

| ID | Case | Expected |
|---|---|---|
| `PSTORE-CURSOR-001` | Continue exact retained generation | stable semantic continuation |
| `PSTORE-CURSOR-002` | Continue after current advances | original retained generation only |
| `PSTORE-CURSOR-003` | Cursor changed query/parameters/order | reject |
| `PSTORE-CURSOR-004` | Cursor changed publication/generation | reject |
| `PSTORE-CURSOR-005` | Cursor uses row ID/page/scan position | architecture failure |
| `PSTORE-RACE-001` | GC begins while current reader acquires lease | guard yields retained or clean retry, never deletion race |
| `PSTORE-RACE-002` | Lease acquired after stale GC plan | pre-delete recheck rejects plan |
| `PSTORE-RACE-003` | Current changes after GC dry run | stale plan rejected |
| `PSTORE-RACE-004` | Operation adopts inactive target after GC dry run | target retained |
| `PSTORE-RACE-005` | Long reader pins WAL frames | checkpoint reports pressure, reader remains exact |

## Windows and physical operations

| ID | Case | Expected |
|---|---|---|
| `PSTORE-WIN-001` | Old epoch delete blocked by open handle | finite retryable sharing state |
| `PSTORE-WIN-002` | Object delete blocked by open handle | preserve intent, recheck roots on retry |
| `PSTORE-WIN-003` | Sharing violation classified as corruption | mutation fails |
| `PSTORE-WIN-004` | Valid reader force-closed for GC | mutation fails |
| `PSTORE-WIN-005` | Rename/delete spin loop | mutation fails |

## Architecture consolidation

| ID | Case | Expected |
|---|---|---|
| `PSTORE-ARCH-001` | Current docs select one WAL/partition model | pass |
| `PSTORE-ARCH-002` | Normal update creates full SQLite generation image | fail |
| `PSTORE-ARCH-003` | Recursive generation delta chain introduced | fail |
| `PSTORE-ARCH-004` | Full duplicated rows per generation introduced | fail |
| `PSTORE-ARCH-005` | Project and graph use separate current pointers | fail |
| `PSTORE-ARCH-006` | Superseded image-design file enters normative routes | fail |
| `PSTORE-ARCH-007` | Benchmark switches physical model without contract revision | fail |
| `PSTORE-ARCH-008` | Rejected alternative remains available in Git history/rationale | pass |

## Acceptance

The first E2-D Rust implementation must execute all applicable cases here in addition to the main matrix. Documentation-only validation must prove that current routing, machine manifests, and checksum member lists contain one physical architecture and no deleted generation-image fixture.
