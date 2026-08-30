# `wow-store` E1-A test matrix

**Status:** normative executable acceptance matrix for SQLite runtime, schema/migrations, immutable ReferenceStore publication, objects, integrity, security, determinism, and deferred ProjectStore behavior.

Tests compare structured IDs/states/manifests/reports/digests and prove crash/failure/cancellation invariants. A committed candidate is not accepted unless the exact seal/publication/pointer path executes.

## 1. Configuration and dependency boundary

| ID | Case | Expected |
|---|---|---|
| `STORE-CONFIG-001` | Valid E1-A configuration | accepted |
| `STORE-CONFIG-002` | Framework dependency beyond `wow-core` | rejected |
| `STORE-CONFIG-003` | Domain entity/restriction/project/rule type imported into store | architecture test fails |
| `STORE-CONFIG-004` | Invalid root/publication volume/platform adapter | rejected |
| `STORE-CONFIG-005` | Invalid budgets/retention/object profile | rejected |
| `STORE-CONFIG-006` | Configuration order/temp absolute root changed | canonical config identity unchanged when semantics equivalent |
| `STORE-CONFIG-007` | Implementation starts with null freeze values | fail |

## 2. SQLite runtime profile

| ID | Case | Expected |
|---|---|---|
| `SQLITE-001` | Exact binding/SQLite pin and capability probe | accepted |
| `SQLITE-002` | Unprobed/different pin | rejected |
| `SQLITE-003` | Effective PRAGMA differs from requested | rejected/degraded as declared |
| `SQLITE-004` | Foreign keys disabled | rejected |
| `SQLITE-005` | Extension loading route available | rejected |
| `SQLITE-006` | ATTACH/external writable DB route | rejected |
| `SQLITE-007` | Read-only ReferenceStore insert/update/delete/DDL | fails, no file/sidecar change |
| `SQLITE-008` | Immutable hint used on mutable/unverified file | rejected |
| `SQLITE-009` | Unsupported required runtime limit/capability | fail before build |
| `SQLITE-010` | Busy/lock spins/unbounded retry | mutation fails |
| `SQLITE-011` | Staging sidecars finalized before seal | pass |
| `SQLITE-012` | Sealed artifact creates journal/WAL/SHM | fail |
| `SQLITE-013` | Runtime profile relevant field changes | profile ID/digest changes |
| `SQLITE-014` | Power-loss level claimed from process-crash only | rejected |
| `SQLITE-015` | Binding upgrade loses capability | activation blocked; last pin retained |

## 3. Schema bundles and registry

| ID | Case | Expected |
|---|---|---|
| `SCHEMA-001` | Standard metadata + valid fixture domain bundle | registry valid |
| `SCHEMA-002` | Duplicate namespace/version/object/operation | rejected |
| `SCHEMA-003` | Reserved/internal/temp/attached object | rejected |
| `SCHEMA-004` | Bundle digest tampered | rejected |
| `SCHEMA-005` | Required SQLite capability absent | rejected |
| `SCHEMA-006` | Dynamic SQL from source/user/transport | no execution surface/rejected |
| `SCHEMA-007` | Raw connection/SQL exposed to service/application | architecture test fails |
| `SCHEMA-008` | Schema object declaration order shuffled | same canonical registry/schema digest |
| `SCHEMA-009` | Unexpected object in built DB | validation fails |
| `SCHEMA-010` | Missing expected index/constraint/trigger | validation fails |

## 4. Migration graph and ledger

| ID | Case | Expected |
|---|---|---|
| `MIG-001` | Empty -> metadata v1 -> domain target exact path | pass |
| `MIG-002` | Graph cycle | rejected |
| `MIG-003` | Missing endpoint/parent | rejected |
| `MIG-004` | Ambiguous path without policy | rejected |
| `MIG-005` | Unknown/skipped edge | rejected |
| `MIG-006` | Migration edge/catalog digest changed | rejected |
| `MIG-007` | Target metadata advanced before operation | mutation fails/rollback |
| `MIG-008` | Ledger missing/duplicate/out-of-order edge | rejected |
| `MIG-009` | Ledger/schema object mismatch | rejected |
| `MIG-010` | Nontransactional/auto-commit operation unprobed | rejected |
| `MIG-011` | Edge failure/cancel | transaction rollback; candidate not advanced |
| `MIG-012` | Sealed/published ReferenceStore migration | rejected |
| `MIG-013` | `user_version` changed without ledger/schema | rejected |
| `MIG-014` | Force/repair/skip option | absent/rejected |
| `MIG-015` | Migration input/order/temp root randomized | same plan/ledger/target schema digest |

## 5. Candidate build and transactions

| ID | Case | Expected |
|---|---|---|
| `BUILD-001` | Valid candidate/staging creation | pass |
| `BUILD-002` | Staging outside root/wrong volume/link escape | rejected |
| `BUILD-003` | Second writer or wrong owner/state | rejected |
| `BUILD-004` | Unknown/over-budget operation batch | rejected/rollback |
| `BUILD-005` | Domain operation order shuffled where unordered | same logical manifest |
| `BUILD-006` | Transaction commit fails/uncertain | candidate not sealed; quarantine as needed |
| `BUILD-007` | Rollback fails/uncertain | quarantine; no publication |
| `BUILD-008` | Candidate visible through active reader path | mutation fails |
| `BUILD-009` | Write after seal | rejected |
| `BUILD-010` | Cancellation | rollback/no seal/publication/background work |

## 6. Validation and sealing

| ID | Case | Expected |
|---|---|---|
| `VALIDATE-001` | All mandatory schema/FK/integrity/domain/file/object checks pass | seal eligible |
| `VALIDATE-002` | Mandatory check unavailable/skipped | fail, never pass |
| `VALIDATE-003` | Foreign-key violation | fail |
| `VALIDATE-004` | quick/integrity corruption | fail |
| `VALIDATE-005` | Domain validation failure | fail |
| `VALIDATE-006` | Schema/ledger mismatch | fail |
| `VALIDATE-007` | Object missing/corrupt/reference-set mismatch | fail |
| `VALIDATE-008` | Unexpected sidecar/schema object | fail |
| `VALIDATE-009` | Manifest/file digest mismatch | fail |
| `VALIDATE-010` | Active transaction/cancel/truncation/budget blocker | no seal |
| `VALIDATE-011` | Seal complete then mutation attempt | rejected |
| `VALIDATE-012` | Corruption auto-repair attempt | rejected |

## 7. Generation publication and active pointer

| ID | Crash/failure point | Expected |
|---|---|---|
| `PUB-001` | Complete build/seal/publish/open/pointer | new active exact generation |
| `PUB-002` | Before staging creation | old active unchanged |
| `PUB-003` | Mid migration/domain write | old active unchanged; candidate rollback/quarantine |
| `PUB-004` | Mid object write | old active unchanged; no retained candidate refs |
| `PUB-005` | DB committed before validation | no seal/publish |
| `PUB-006` | Mid manifest finalization | no seal/publish |
| `PUB-007` | Sealed but not final-path published | old active unchanged; recoverable candidate policy |
| `PUB-008` | Generation published, pointer not updated | old active unchanged; orphan published generation recoverable |
| `PUB-009` | Pointer temp written, not replaced | old active unchanged |
| `PUB-010` | Pointer replaced/flush uncertainty | exact adapter-classified state, never guess |
| `PUB-011` | Final-path reopen/validation fails | no pointer update |
| `PUB-012` | Existing exact equivalent generation | idempotent success |
| `PUB-013` | Existing same ID/path mismatch | corruption/collision; no overwrite |
| `PUB-014` | Retry pointer activation same validated generation | idempotent success |
| `PUB-015` | Active reader opened old generation, pointer changes | reader remains old exact generation |
| `PUB-016` | Last-known-good relabeled new failed target | mutation fails |
| `PUB-017` | Generation/pointer/temp names/order changed | same canonical generation/manifest/pointer semantics |
| `PUB-018` | Durability level overstated | rejected |

## 8. ReferenceStore open/read

| ID | Case | Expected |
|---|---|---|
| `REFSTORE-001` | Exact sealed generation read-only open | pass |
| `REFSTORE-002` | Profile/reference/generation mismatch | rejected |
| `REFSTORE-003` | Unsealed/staging generation | rejected |
| `REFSTORE-004` | Writable mode or file mutation | rejected/no sidecar/change |
| `REFSTORE-005` | In-place row/schema/metadata update | rejected |
| `REFSTORE-006` | Missing/corrupt manifest/file/object | rejected |
| `REFSTORE-007` | Unsupported SQLite runtime profile | rejected |
| `REFSTORE-008` | Active pointer references missing/partial generation | rejected; prior valid pointer/generation preserved |
| `REFSTORE-009` | Missing row used as negative authority | mutation test fails |
| `REFSTORE-010` | Read operation unknown/wrong catalog/state/over budget | rejected |
| `REFSTORE-011` | Same immutable read across concurrent readers | identical results/context |
| `REFSTORE-012` | Open validation level not met but claimed pass | rejected |

## 9. Logical objects

| ID | Case | Expected |
|---|---|---|
| `OBJECT-001` | Known canonical bytes -> ObjectId vector | exact |
| `OBJECT-002` | Same bytes different chunking | same ObjectId |
| `OBJECT-003` | Different codec/parameters | same ObjectId, different payload ID/digest as applicable |
| `OBJECT-004` | Write/read/decode/logical+payload verify | pass |
| `OBJECT-005` | Existing valid object | dedup, no overwrite |
| `OBJECT-006` | Existing same ID but mismatch | corruption/quarantine, no overwrite |
| `OBJECT-007` | Malformed digest/path/traversal/link preplacement | rejected |
| `OBJECT-008` | Temp/flush/rename interruption | no invalid final object/reference |
| `OBJECT-009` | Unknown codec | rejected |
| `OBJECT-010` | Decompression bomb/ratio/memory/size limit | rejected bounded |
| `OBJECT-011` | Source filename/path changes | ObjectId/path identity unchanged |
| `OBJECT-012` | Candidate references object before verify/publication | rejected |
| `OBJECT-013` | Candidate fails after shared object dedup | existing object intact; no retained candidate ref |
| `OBJECT-014` | Object path/error leaks private source/root | fail |

## 10. Object references and GC

| ID | Case | Expected |
|---|---|---|
| `GC-001` | Reference set sorted/digested | deterministic |
| `GC-002` | Active generation reference | no delete |
| `GC-003` | Last-known-good/configured historical reference | no delete |
| `GC-004` | Published-protected recovery generation reference | no delete |
| `GC-005` | Active reader/object lease | no delete |
| `GC-006` | Unknown/incomplete reference or lease scan | eligible unknown, no delete |
| `GC-007` | Age/name only suggests orphan | no delete |
| `GC-008` | Proven unreferenced/unleased valid object | delete/report |
| `GC-009` | TOCTOU path/replacement before delete | revalidation rejects wrong target |
| `GC-010` | Cancellation mid GC | completed deletions reported, future deletes stop |
| `GC-011` | Broad recursive delete/untrusted path | rejected |
| `GC-012` | Referenced object deletion mutation | test fails |

## 11. Security

| ID | Case | Expected |
|---|---|---|
| `STORE-SEC-001` | Absolute/traversal/device/UNC/reserved path | rejected |
| `STORE-SEC-002` | Symlink/reparse/hardlink escape/race | rejected |
| `STORE-SEC-003` | Malicious SQLite trigger/view/virtual table/schema | rejected/import isolation |
| `STORE-SEC-004` | Extension loading/attach attempt | rejected |
| `STORE-SEC-005` | SQL/source instruction injection | no policy/execution effect |
| `STORE-SEC-006` | Oversized DB/schema/row/blob/object | bounded rejection |
| `STORE-SEC-007` | Untrusted SQLite opened writable as owned store | rejected |
| `STORE-SEC-008` | Network/process/shell/editor/client/source execution | rejected |
| `STORE-SEC-009` | Manifest/error leaks absolute path/token/raw payload | rejected |
| `STORE-SEC-010` | Corruption silently repaired/activated | mutation fails |

## 12. Determinism

| ID | Case | Expected |
|---|---|---|
| `STORE-DET-001` | Random schema/operation/row/object input order | same logical manifests/IDs |
| `STORE-DET-002` | Temp root/host/time/worker order changes | canonical IDs/digests unchanged |
| `STORE-DET-003` | Compression metadata normalized | expected payload reproducibility or explicit nonreproducible physical status |
| `STORE-DET-004` | Raw SQLite bytes differ while logical manifest equal | do not falsely fail/claim byte identity; report physical status |
| `STORE-DET-005` | Raw file byte reproducibility claimed without proof | rejected |
| `STORE-DET-006` | Publication state record order shuffled | same canonical report |

## 13. Deferred ProjectStore

| ID | Case | Expected |
|---|---|---|
| `PROJECTSTORE-DEFER-001` | Open/create/write ProjectStore | typed unavailable |
| `PROJECTSTORE-DEFER-002` | WAL/checkpoint/read snapshot operation | unavailable |
| `PROJECTSTORE-DEFER-003` | Physical generation model assumed/created | rejected |
| `PROJECTSTORE-DEFER-004` | Project/graph tables placed in standard metadata schema | architecture test fails |
| `PROJECTSTORE-DEFER-005` | Empty/default ProjectStore success | prohibited |

## 14. Checksum/freeze

| ID | Case | Expected |
|---|---|---|
| `STORE-FIX-001` | Documentation-only null runtime/schema/generation/object IDs | allowed only while implementation not-started |
| `STORE-FIX-002` | Implementation starts with required null | fail |
| `STORE-FIX-003` | Example bytes change without checksum update | fail |
| `STORE-FIX-004` | Selected SQLite/binding profile differs from frozen probe | fail |
| `STORE-FIX-005` | Schema/migration/store/object/publication vectors verify | pass |

## 15. Acceptance gate

E1-A implementation is incomplete until:

```text
only wow-core dependency and no domain semantics
exact SQLite/binding/profile probe passes
registered metadata/domain schema and migration graph apply deterministically
immutable ReferenceStore publishes crash-safely and opens strictly read-only
all corruption/integrity/schema/file/object mismatches reject activation
all object write/dedup/reference/GC safety cases pass
all publication crash/cancellation points retain previous active generation
no raw SQL/extension/attach/source execution/private-path leak
ProjectStore remains typed Deferred
logical/manifests/reports deterministic and physical byte claims honest
all required fixture IDs/digests/checksums frozen
```
