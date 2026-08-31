# E2-D ProjectStore test matrix

**Status:** normative executable acceptance and mutation matrix.

## Profile and schema

| ID | Case | Expected |
|---|---|---|
| `PSTORE-PROFILE-001` | Exact selected physical profile | accepted |
| `PSTORE-PROFILE-002` | Unprobed SQLite/binding/platform | reject |
| `PSTORE-PROFILE-003` | Effective WAL/PRAGMA differs | reject |
| `PSTORE-PROFILE-004` | ReferenceStore profile reused | reject |
| `PSTORE-PROFILE-005` | Second writer or external reader contract | reject |
| `PSTORE-PROFILE-006` | Unlimited busy/WAL/result budget | reject |
| `PSTORE-SCHEMA-001` | Complete store+project+graph schema set | accepted |
| `PSTORE-SCHEMA-002` | Missing/mismatched owner bundle/catalog | reject |
| `PSTORE-SCHEMA-003` | Raw SQL/table/PRAGMA input | reject |
| `PSTORE-SCHEMA-004` | Store interprets domain fields | architecture test fails |
| `PSTORE-SCHEMA-005` | Breaking schema/profile in same epoch | rebuild required |
| `PSTORE-SCHEMA-006` | Shuffled bundle/catalog declarations | same schema-set ID |

## Partition versions

| ID | Case | Expected |
|---|---|---|
| `PSTORE-PART-001` | Materialize valid new project partition | sealed |
| `PSTORE-PART-002` | Materialize valid graph assertion partition | sealed |
| `PSTORE-PART-003` | Equivalent existing partition | validated reuse |
| `PSTORE-PART-004` | Same ID/different content | corruption/reject |
| `PSTORE-PART-005` | Write after seal | reject |
| `PSTORE-PART-006` | Row/count/digest mismatch | reject |
| `PSTORE-PART-007` | Missing evidence/coverage/object reference | reject |
| `PSTORE-PART-008` | Input/insertion order shuffled | same ID/manifest |
| `PSTORE-PART-009` | Removed source retained in target membership | stale-removal failure |
| `PSTORE-PART-010` | Another producer's partition | unaffected |
| `PSTORE-PART-011` | Recursive delta/base lookup introduced | architecture failure |
| `PSTORE-PART-012` | Oversized/high-fanout partition | bounded failure |

## Membership and identity

| ID | Case | Expected |
|---|---|---|
| `PSTORE-MEMBER-001` | Complete baseline map | accepted |
| `PSTORE-MEMBER-002` | One changed + many reused partitions | full map, exact reuse |
| `PSTORE-MEMBER-003` | Missing/duplicate/conflicting key | reject |
| `PSTORE-MEMBER-004` | Unsealed/missing partition reference | reject |
| `PSTORE-MEMBER-005` | Base generation needed to read target | mutation fails |
| `PSTORE-MEMBER-006` | Same final map via different update order | same digest/generation |
| `PSTORE-ID-001` | Row/page/WAL/order changes | semantic IDs unchanged |
| `PSTORE-ID-002` | Semantic partition/snapshot changes | target IDs change |
| `PSTORE-ID-003` | Store generation in upstream semantic ID | cycle test fails |
| `PSTORE-ID-004` | Project/graph/analyzer mismatch | reject |

## Inactive build and validation

| ID | Case | Expected |
|---|---|---|
| `PSTORE-BUILD-001` | Initial no-base build | PublishedInactive |
| `PSTORE-BUILD-002` | Incremental exact-base build | PublishedInactive |
| `PSTORE-BUILD-003` | Stale base before write | reject/no mutation |
| `PSTORE-BUILD-004` | Transaction failure | no target/current change |
| `PSTORE-BUILD-005` | Cancellation before commit | rollback/no background work |
| `PSTORE-BUILD-006` | Object written then rollback | unreferenced object only |
| `PSTORE-BUILD-007` | Partial membership/domain write | no inactive commit |
| `PSTORE-BUILD-008` | Equivalent inactive target | idempotent revalidation |
| `PSTORE-BUILD-009` | Mismatched existing target ID | quarantine/reject |
| `PSTORE-BUILD-010` | Randomized write batches | same logical result |
| `PSTORE-VALID-001` | Full store/project/graph golden validation | ValidatedInactive |
| `PSTORE-VALID-002` | Cross-generation leakage sentinel | fail |
| `PSTORE-VALID-003` | Missing reverse/axis/project index | fail |
| `PSTORE-VALID-004` | Project/Graph snapshot binding mismatch | fail |
| `PSTORE-VALID-005` | Object/reference mismatch | fail |
| `PSTORE-VALID-006` | Partial source/recognizer coverage | exact partial retained |
| `PSTORE-VALID-007` | Storage upgrades domain coverage | mutation fails |
| `PSTORE-VALID-008` | Validator repairs rows/checksums | mutation fails |
| `PSTORE-VALID-009` | Cancellation/truncation | not validated |
| `PSTORE-VALID-010` | Read-back uses current instead of target | mutation fails |

## Activation

| ID | Case | Expected |
|---|---|---|
| `PSTORE-ACT-001` | Exact base + successful validation | CAS succeeds |
| `PSTORE-ACT-002` | No validation report | reject |
| `PSTORE-ACT-003` | Base changed after inactive build | CAS rejects |
| `PSTORE-ACT-004` | Activation transaction crash | old or new atomically |
| `PSTORE-ACT-005` | Current record contains mixed IDs | reject |
| `PSTORE-ACT-006` | Identical target already current | AlreadyCurrent |
| `PSTORE-ACT-007` | Prior marked superseded without pointer change | rollback |
| `PSTORE-ACT-008` | LKG relabeled target | mutation fails |
| `PSTORE-ACT-009` | Post-open confirmation exact | pass |
| `PSTORE-ACT-010` | Post-open defect auto-repairs/rolls back | forbidden |

## Readers, WAL, and concurrency

| ID | Case | Expected |
|---|---|---|
| `PSTORE-READ-001` | Current reader exact coherent IDs | pass |
| `PSTORE-READ-002` | Old reader across activation | stays old |
| `PSTORE-READ-003` | New reader after activation | sees new |
| `PSTORE-READ-004` | Exact retained generation | pass |
| `PSTORE-READ-005` | Missing/GC generation | reject |
| `PSTORE-READ-006` | Query omits membership scope | architecture failure |
| `PSTORE-READ-007` | Reader switches generation | reject |
| `PSTORE-READ-008` | Raw connection/cursor/row ID escapes | architecture failure |
| `PSTORE-READ-009` | Lease blocks GC | pass |
| `PSTORE-READ-010` | Cancel/error releases lease | pass |
| `PSTORE-READ-011` | External process reader | unsupported |
| `PSTORE-READ-012` | Partial/conflict becomes absence/clean | mutation fails |
| `PSTORE-WAL-001` | WAL effective and one writer | pass |
| `PSTORE-WAL-002` | Finite busy timeout | deterministic busy |
| `PSTORE-WAL-003` | Spin/unbounded retry | mutation fails |
| `PSTORE-WAL-004` | Passive checkpoint, no readers | bounded success |
| `PSTORE-WAL-005` | Long reader pins frames | explicit busy/remaining |
| `PSTORE-WAL-006` | Checkpoint changes logical state | fail |
| `PSTORE-WAL-007` | WAL exceeds admission ceiling | policy warning/block |
| `PSTORE-WAL-008` | Process crash recovery | exact classification |
| `PSTORE-WAL-009` | Power-loss claim from process kill | reject claim |
| `PSTORE-WAL-010` | Two writer owners | reject |

## Crash, backup, epoch, retention, GC

| ID | Case | Expected |
|---|---|---|
| `PSTORE-CRASH-001` | Mid-partition transaction | rollback/orphan-safe |
| `PSTORE-CRASH-002` | Mid-generation transaction | no target/current |
| `PSTORE-CRASH-003` | After inactive commit | recoverable inactive |
| `PSTORE-CRASH-004` | After validation before activation | retry if base exact |
| `PSTORE-CRASH-005` | During activation commit | old or new current |
| `PSTORE-CRASH-006` | Current corrupt | explicit recovery path |
| `PSTORE-CRASH-007` | Auto-select LKG | forbidden |
| `PSTORE-CRASH-008` | Inactive recovery after base changed | no activation |
| `PSTORE-CRASH-009` | Target content changed after validation | reject/quarantine |
| `PSTORE-CRASH-010` | Recovery writes without owner | reject |
| `PSTORE-BACKUP-001` | Online exact-snapshot backup | valid manifest |
| `PSTORE-BACKUP-002` | Copy main DB without live WAL | reject |
| `PSTORE-BACKUP-003` | Restore exact IDs and validate | candidate epoch |
| `PSTORE-BACKUP-004` | Restore relabels generation | reject |
| `PSTORE-EPOCH-001` | Breaking schema/profile builds new epoch | pass |
| `PSTORE-EPOCH-002` | New epoch validated then outer CAS | coherent old/new |
| `PSTORE-EPOCH-003` | Old reader continues old epoch | pass |
| `PSTORE-EPOCH-004` | Failed epoch build changes registry | mutation fails |
| `PSTORE-EPOCH-005` | In-place breaking migration | reject v1 |
| `PSTORE-GC-001` | Current/LKG/reader/evidence pin | retained |
| `PSTORE-GC-002` | Unreferenced old generation | eligible |
| `PSTORE-GC-003` | Generation removal exposes orphan partition | candidate |
| `PSTORE-GC-004` | Shared partition in retained generation | retained |
| `PSTORE-GC-005` | Object still referenced | retained |
| `PSTORE-GC-006` | Unknown lease/reference | no delete |
| `PSTORE-GC-007` | Age-only deletion | reject |
| `PSTORE-GC-008` | GC failure/cancel | current/integrity preserved |
| `PSTORE-GC-009` | Domain delete catalog missing | no partition delete |
| `PSTORE-GC-010` | Old epoch reader/rollback pin | retained |

## Security, benchmark, and freeze

| ID | Case | Expected |
|---|---|---|
| `PSTORE-SEC-001` | SQL/DDL/PRAGMA/table name from source/caller | reject |
| `PSTORE-SEC-002` | Extension/ATTACH/untrusted writable DB | reject |
| `PSTORE-SEC-003` | Traversal/UNC/device/URI/symlink path | reject |
| `PSTORE-SEC-004` | Source prompt/code/hook/build script | never execute |
| `PSTORE-SEC-005` | Private root/token/source body leak | fail |
| `PSTORE-SEC-006` | SavedVariables/runtime/secret data | reject |
| `PSTORE-SEC-007` | Huge schema/partition/map/query/WAL/backup/GC | bounded failure |
| `PSTORE-SEC-008` | Cancellation in every phase | no background/current corruption |
| `PSTORE-SEC-009` | Filesystem/network/process/editor/client escape | fail |
| `PSTORE-SEC-010` | Row absence used as domain authority | fail |
| `PSTORE-BENCH-001` | Frozen synthetic and roth-ui corpus | run |
| `PSTORE-BENCH-002` | One-file update rewrites full DB | selected profile fails |
| `PSTORE-BENCH-003` | Recursive read ancestry | selected profile fails |
| `PSTORE-BENCH-004` | WAL/checkpoint over frozen ceiling | fail/regression |
| `PSTORE-BENCH-005` | Missing/old benchmark reported pass | fail |
| `PSTORE-DET-001` | 1/2/N and shuffled plans | same logical outputs |
| `PSTORE-DET-002` | SQLite page/WAL bytes differ | physical classification only |
| `PSTORE-DET-003` | Physical bytes claimed canonical without proof | fail |
| `PSTORE-FIX-001` | Documentation null pins | allowed before implementation |
| `PSTORE-FIX-002` | First Rust commit with required nulls | fail |
| `PSTORE-FIX-003` | Fixture bytes changed without checksums | fail |

## Acceptance

E2-D is incomplete until all nondeferred cases execute, the selected profile passes probe/benchmark gates, and a baseline plus representative incremental update prove atomic inactive-build/validate/activate behavior, coherent readers, recovery, and safe reclamation.
