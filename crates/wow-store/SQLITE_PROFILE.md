# SQLite runtime and open-mode profile

**Status:** normative E1-A capability probe and configuration contract.

This document defines required semantics. The implementation must select and pin an actual Rust SQLite binding/version only after a compatibility/security/behavior probe. Do not infer binding behavior from SQLite documentation alone.

## 1. Runtime profile purpose

SQLite behavior depends on:

```text
SQLite library version
Rust binding/version/features
compile options/extensions
open flags and URI handling
PRAGMA defaults/effects
journal/synchronous/locking/busy policy
filesystem/platform adapter
```

The accepted profile is part of store compatibility and publication evidence.

## 2. Probe report

```text
SqliteRuntimeProbeReport
    report_id
    SQLite version/source ID
    Rust binding/version/features
    compile option set/digest
    tested platform/filesystem adapter
    tested capabilities and observed behavior
    selected open/PRAGMA profiles by store kind/state
    unsupported/degraded capabilities
    fixture/test IDs
    canonical semantic digest
```

Wall-clock/host path are supplemental, not canonical.

## 3. Required capability probes

At minimum test:

- URI filename support and read-only open behavior;
- immutable/query-only behavior where used;
- foreign-key enablement/enforcement/check;
- transactions/savepoints/rollback behavior;
- journal modes required by staging/project paths;
- WAL support/sidecar/checkpoint behavior (for future ProjectStore; may be probe-only/deferred);
- synchronous/durability configuration effects exposed by binding;
- busy timeout/locking/cancellation behavior;
- `quick_check`/`integrity_check`/foreign-key check execution;
- normalized schema inspection;
- trusted-schema/defensive/extension-loading controls exposed by binding;
- database/application/user version read/write;
- backup/export APIs if selected;
- statement parameter/result type behavior and limits;
- file/page/length/variable/column/compound/select/trigger recursion limits as configurable/observable;
- read-only ReferenceStore mutation attempts fail and create no sidecar;
- power-loss/crash claims separately tested by publication adapter, not assumed.

## 4. Open profiles

### Staging ReferenceStore writer

Conceptual requirements:

```text
read-write/create under private staging root
foreign keys enabled
extension loading disabled
trusted schema disabled/defensive behavior where supported
explicit journal/synchronous policy
explicit busy policy
no arbitrary ATTACH
registered schema/operation catalogs only
one owning build writer
```

Exact journal mode may be rollback journal or WAL during staging only after measurement. Before sealing, all transactions close, sidecars are checkpointed/removed, and store revalidates.

### Sealed ReferenceStore reader

Conceptual requirements:

```text
read-only open
query-only
immutable hint only when file truly immutable and binding/SQLite semantics verified
foreign keys/schema/integrity checks as permitted
extension loading disabled
trusted schema disabled/defensive behavior where supported
no journal/WAL sidecar creation
no write/DDL/temp persistent mutation
bounded read/query operations
```

If `immutable=1` bypasses change detection/locking semantics that are unsafe for the environment, do not use it. The profile records the exact selected mode.

### ProjectStore writer (deferred)

Conceptual future profile:

```text
read-write existing owned store
WAL enabled/verified
one writer actor
foreign keys enabled
explicit synchronous/busy/checkpoint policy
read snapshot behavior verified
```

Not activated in E1-A.

### ProjectStore reader (deferred)

Read transaction/snapshot profile against WAL store; exact snapshot/lease semantics measured in E2.

## 5. Required PRAGMA policy

Record every behavior-affecting PRAGMA explicitly. Candidates to evaluate/include:

```text
foreign_keys
query_only
trusted_schema
journal_mode
synchronous
busy_timeout
wal_autocheckpoint (ProjectStore only)
cache_size / mmap_size / temp_store where used
page_size / auto_vacuum before schema creation if selected
application_id / user_version convenience markers
recursive_triggers if needed
secure_delete only if security model requires it
```

Rules:

- defaults are not a contract;
- read actual effective value after setting;
- unsupported/ignored value is capability failure/degradation;
- runtime profile/manifest records selected values/digest;
- security/consistency-affecting PRAGMA cannot be changed by user/source/transport;
- application/user version are supplementary, not schema authority;
- `integrity_check`/`quick_check` are validation operations, not persistent profile settings.

## 6. Compile options and extensions

- record compile options/digest;
- extension loading disabled and no API surface to enable it;
- required built-in virtual tables/features declared by schema bundle and probed;
- FTS5 not required for E1-A unless the reference schema contract explicitly activates it later;
- JSON/other optional functions cannot become correctness dependencies without capability/fixture contract;
- no loadable extension path in application/service/source input.

## 7. Limits and budgets

Set/query explicit bounds where binding permits:

```text
max database/page/file size
max SQL length
max variables/columns/compound selects
max expression/trigger depth
max result rows/bytes per operation
max schema objects
max attached databases = zero or strict internal minimum
```

If SQLite runtime limit cannot be lowered through selected binding, enforce at schema/operation input layer and report the gap.

## 8. Foreign keys

- enable for every connection where schema uses them;
- verify effective state;
- schema bundle must declare required FK relations;
- run foreign-key check before seal/publication and after migration;
- reject violations;
- do not depend on implicit cascade/deferred behavior without fixture tests;
- no connection path with foreign keys accidentally off.

## 9. Read-only/immutable validation

Tests for sealed ReferenceStore:

- insert/update/delete fail;
- schema create/drop/alter fail;
- writable PRAGMA/state change fail or have no persistence/effect according to selected profile;
- no `-journal`, `-wal`, `-shm` appears;
- store file/payload digest unchanged;
- manifest/profile/generation validate;
- concurrent readers observe same immutable data;
- replacement of active pointer does not mutate already opened generation.

## 10. Busy/locking policy

E1 staging build has one writer; unexpected busy/lock usually indicates a lifecycle bug.

- explicit finite busy timeout or deterministic immediate failure selected/probed;
- cancellation does not become infinite busy retry;
- no spin loop;
- error includes operation/store/generation/transaction IDs, not private path;
- ReferenceStore readers should not require writer locks.

ProjectStore future policy separately measured.

## 11. Integrity operations

Probe and classify:

```text
schema metadata comparison
foreign_key_check
quick_check
integrity_check
application/domain validation catalog
file/object checksum validation
```

E1 publication can choose quick vs full integrity based on artifact size/performance only through a documented policy; release validation must include the required full check at a defined stage. Missing/unsupported check is never pass.

## 12. Durability and filesystem boundary

SQLite transaction atomicity is not the complete generation-publication durability contract.

Separate:

```text
SQLite transaction commit
SQLite file/journal sidecar finalization
file data flush
manifest/pointer file flush
generation directory/rename/replace atomicity
directory metadata flush
power-loss behavior
```

Platform adapter tests decide achieved durability level. Do not infer `power_loss_tested` from successful process-crash tests.

## 13. Binding abstraction

`wow-store` may isolate binding-specific calls behind one adapter, but avoid a generic pluggable database abstraction.

Adapter owns:

```text
open flags/URI construction
PRAGMA/effective-value queries
transaction/savepoint primitives
prepared statement execution
schema inspection
integrity checks
backup/export if selected
SQLite limit/capability access
error normalization
```

The framework pins one implementation; abstraction exists for upgrade isolation/testing, not multiple database engines.

## 14. Upgrade probe

Every candidate SQLite/binding update reruns:

- compile options/capability profile;
- open/read-only/immutable behavior;
- transaction/rollback/sidecar behavior;
- schema normalization/digest compatibility;
- prepared parameter/result behavior;
- integrity/foreign-key checks;
- object/publication crash fixtures where relevant;
- determinism/performance corpus;
- malformed/untrusted DB security tests.

Lost mandatory capability blocks activation; retain last-known-good pin.

## 15. Required operations

```text
probe_sqlite_runtime
validate_sqlite_runtime_probe
build_sqlite_runtime_profile
validate_sqlite_runtime_profile
open_staging_reference_connection
open_sealed_reference_connection
apply_and_verify_connection_profile
inspect_effective_pragmas
inspect_compile_options_and_capabilities
apply_runtime_limits
validate_read_only_reference_behavior
normalize_sqlite_error
```

## 16. Required tests

- exact selected SQLite/binding pin/profile;
- effective PRAGMA values;
- extension loading unavailable;
- arbitrary attach unavailable;
- foreign keys enforced/check violations rejected;
- transaction rollback/cancellation;
- read-only/query-only/immutable mutation and sidecar tests;
- runtime limit enforcement/gap reporting;
- unsupported PRAGMA/capability rejection;
- staging sidecar finalization before seal;
- profile digest changes on relevant capability/config change;
- temp root/platform differences excluded from canonical profile identity where semantically equivalent;
- upgrade last-known-good rollback.

## 17. Hard stops

- no unprobed binding/version;
- no defaults-as-contract;
- no extension-loading/attach path;
- no foreign-keys-off connection;
- no sealed-reference writable open;
- no immutable hint on mutable/unverified file;
- no WAL sidecars in sealed artifact;
- no power-loss guarantee from transaction/process-crash alone;
- no ProjectStore profile activation in E1-A.
