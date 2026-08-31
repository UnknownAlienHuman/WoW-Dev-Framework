# E2-A test matrix

**Status:** normative executable acceptance and mutation matrix.

## Registry

| ID | Case | Expected |
|---|---|---|
| `GRAPH-REG-001` | Valid initial bundle | pass |
| `GRAPH-REG-002` | Duplicate incompatible entity kind | reject |
| `GRAPH-REG-003` | Invalid relation endpoints/direction | reject |
| `GRAPH-REG-004` | Generic parent semantics | reject |
| `GRAPH-REG-005` | Unbounded/arbitrary attribute schema | reject |
| `GRAPH-REG-006` | Breaking identity change without version | reject |
| `GRAPH-REG-007` | Unknown bundle in producer batch | NotEvaluated/reject |
| `GRAPH-REG-008` | Shuffled definition order | same bundle ID/bytes |

## Identity and assertions

| ID | Case | Expected |
|---|---|---|
| `GRAPH-ID-001` | Same semantic entity/input reorder | same EntityKey |
| `GRAPH-ID-002` | Same name different universe/generation | distinct keys |
| `GRAPH-ID-003` | Two producers same entity | same key, distinct assertions |
| `GRAPH-ID-004` | Relation producer/evidence changes | same RelationKey, distinct assertion |
| `GRAPH-ID-005` | Row/insertion/display name used as identity | mutation fails |
| `GRAPH-ID-006` | Endpoint same-batch/existing base | resolves |
| `GRAPH-ID-007` | Dangling/cross-scope endpoint | reject |
| `GRAPH-ID-008` | Evidence/coverage/derivation missing | reject |
| `GRAPH-ID-009` | Derivation cycle | reject |
| `GRAPH-ID-010` | Candidate assertion merged into project key | reject |

## Assertions/conflicts

| ID | Case | Expected |
|---|---|---|
| `GRAPH-ASSERT-001` | Compatible repeated assertions | all evidence retained |
| `GRAPH-ASSERT-002` | Exclusive attribute disagreement | conflict |
| `GRAPH-ASSERT-003` | Multiplicity violation | conflict/fail by schema |
| `GRAPH-ASSERT-004` | Possible + proven | no confidence upgrade beyond valid view |
| `GRAPH-ASSERT-005` | Last-write-wins mutation | fails |
| `GRAPH-ASSERT-006` | Missing assertion under partial producer coverage | no authoritative absence |
| `GRAPH-ASSERT-007` | Conflict affects axis/capability | explicit blocker |
| `GRAPH-ASSERT-008` | Producer removed | only its assertions disappear |

## Partitions/publication

| ID | Case | Expected |
|---|---|---|
| `GRAPH-PART-001` | First partition publish | snapshot valid |
| `GRAPH-PART-002` | Replace partition removes stale assertions | pass |
| `GRAPH-PART-003` | Other producer partition unchanged | pass |
| `GRAPH-PART-004` | Stale base generation | reject |
| `GRAPH-PART-005` | Partial store write/failure | old snapshot only |
| `GRAPH-PART-006` | Cancel before/after store handoff | no partial target |
| `GRAPH-PART-007` | Multi-partition one publication | all or none |
| `GRAPH-PART-008` | Disable producer to empty | coverage downgraded |
| `GRAPH-PART-009` | Post-open golden mismatch | target inactive/quarantined |
| `GRAPH-PART-010` | Random partition order same final set | same snapshot ID/manifest |

## Axes

| ID | Case | Expected |
|---|---|---|
| `GRAPH-AXIS-001` | One entity in ownership/load/call axes | separate valid views |
| `GRAPH-AXIS-002` | Object `parent_of` | object axis only |
| `GRAPH-AXIS-003` | Single-parent request on multi-parent axis | typed policy required |
| `GRAPH-AXIS-004` | Forbidden load/hierarchy cycle | conflict/reject |
| `GRAPH-AXIS-005` | Call/state cycle | safe traversal |
| `GRAPH-AXIS-006` | Axis uses undeclared relation | reject |
| `GRAPH-AXIS-007` | Axis query returns supporting assertions | pass |
| `GRAPH-AXIS-008` | View materializes generic parent edge | mutation fails |

## Queries

| ID | Case | Expected |
|---|---|---|
| `GRAPH-QUERY-001` | Exact entity found | evidence-bearing view |
| `GRAPH-QUERY-002` | Exact absent complete coverage | authoritative absence |
| `GRAPH-QUERY-003` | Exact absent partial/conflict | nonauthoritative/NotEvaluated |
| `GRAPH-QUERY-004` | Deterministic neighbors | stable order |
| `GRAPH-QUERY-005` | Axis traversal bounded cycle-safe | pass |
| `GRAPH-QUERY-006` | Bounded paths max paths/depth | exact truncation |
| `GRAPH-QUERY-007` | Candidate excluded by default | pass |
| `GRAPH-QUERY-008` | Possible included explicitly | labeled |
| `GRAPH-QUERY-009` | Path persisted as direct edge | mutation fails |
| `GRAPH-QUERY-010` | Project subgraph exceeds bytes/nodes | deterministic truncation |
| `GRAPH-QUERY-011` | Explain relation | exact assertions/evidence |
| `GRAPH-QUERY-012` | Cursor same snapshot/request | continuation stable |
| `GRAPH-QUERY-013` | Cursor another snapshot/tampered | reject |
| `GRAPH-QUERY-014` | Cancellation | no complete claim/background work |
| `GRAPH-QUERY-015` | Whole graph public dump | unavailable |
| `GRAPH-QUERY-016` | No new evidence | distinct from authoritative absence |

## Store/persistence

| ID | Case | Expected |
|---|---|---|
| `GRAPH-STORE-001` | Registered replacement plan | pass |
| `GRAPH-STORE-002` | Raw SQL/connection/PRAGMA | architecture test fails |
| `GRAPH-STORE-003` | Read snapshot stays exact while writer publishes | pass |
| `GRAPH-STORE-004` | Cross-generation row leakage | mutation fails |
| `GRAPH-STORE-005` | Reverse index missing/wrong | validation fails |
| `GRAPH-STORE-006` | Logical round trip | same manifests/query results |
| `GRAPH-STORE-007` | Physical model changes, logical same | allowed/profile-classified |
| `GRAPH-STORE-008` | GC removes leased/referenced generation | mutation fails |

## Security/budgets

| ID | Case | Expected |
|---|---|---|
| `GRAPH-SEC-001` | Huge registry/assertion batch | bounded failure |
| `GRAPH-SEC-002` | High-fanout/cyclic path bomb | deterministic truncation |
| `GRAPH-SEC-003` | Oversized/raw source/private path/token attribute | reject/redact |
| `GRAPH-SEC-004` | Source comment/prompt as schema/instruction | data only/reject |
| `GRAPH-SEC-005` | Executable callback/query expression | reject |
| `GRAPH-SEC-006` | Filesystem/network/process/editor access | absent |
| `GRAPH-SEC-007` | Invalid max/unlimited budget | reject |
| `GRAPH-SEC-008` | Cancellation inside validation/traversal/serialization | bounded stop |

## Determinism/freeze

| ID | Case | Expected |
|---|---|---|
| `GRAPH-DET-001` | 1/2/N workers | same IDs/manifests/results |
| `GRAPH-DET-002` | Hash/store traversal order | no output change |
| `GRAPH-DET-003` | Clock/temp/row IDs | excluded from identity |
| `GRAPH-DET-004` | Canonical fixture bytes changed without checksums | fail |
| `GRAPH-FIX-001` | Null pins before implementation | allowed |
| `GRAPH-FIX-002` | First Rust commit with required nulls | fail |
| `GRAPH-FIX-003` | Registry/partition/query vectors frozen | pass |

## Acceptance

E2-A is incomplete until all nondeferred cases execute and the implementation proves producer-independent semantic identity, assertion preservation, atomic partition replacement, immutable snapshots, explicit axes, bounded deterministic queries, store boundary isolation, and no authority upgrade.
