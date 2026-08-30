# Transactions, sealing, and generation publication

**Status:** normative E1-A transaction and crash-safe immutable-generation publication contract.

SQLite transaction commit and store-generation publication are separate boundaries. A committed staging database is not active until every validation/seal/publication/pointer step succeeds.

## 1. Publication state machine

```text
Candidate
-> StagingCreated
-> SchemaApplied
-> DataWritten
-> ObjectsVerified
-> SchemaValidated
-> DataValidated
-> IntegrityValidated
-> ManifestFinalized
-> Sealed
-> GenerationPublished
-> ActivePointerPublished
```

Terminal nonactive states:

```text
Cancelled
Failed
Quarantined
Cleaned
```

No state is skipped. State transitions are explicit, idempotence-aware, and recorded.

## 2. Candidate identity

```text
StoreBuildId
    request/configuration/schema/domain-input/object-plan identities
    nonpublished candidate identity
```

Candidate ID is not StoreGenerationId. StoreGenerationId derives only after canonical logical data/object/schema manifest is finalized.

## 3. Staging root

Requirements:

- under configured private store root;
- same filesystem/volume as final generation publication path when atomic rename/replace requires it;
- generated path component not public/canonical identity;
- root-confined, no symlink/traversal/device escape;
- not reachable through active pointer;
- permissions restrictive/explicit;
- cleanup/quarantine policy explicit.

## 4. Staging database transactions

- one writer owner for candidate;
- explicit transaction around schema/migration/record batches according to plan;
- no auto-commit assumption without probe;
- registered prepared operations only;
- write transaction state and operation IDs recorded;
- cancellation/failure rolls back current transaction;
- no target metadata/ledger advance outside matching committed edge/plan state;
- no reader sees candidate through published namespace.

## 5. Domain write batches

`wow-reference` supplies a deterministic operation invocation plan. Store:

- validates catalog/parameter/result schema;
- canonicalizes operation batch order when domain semantics say unordered;
- executes within declared transactions;
- records counts/digests without interpreting records;
- enforces row/result/byte budgets;
- rejects duplicate/unknown operation IDs;
- does not infer completeness from successful insertion.

## 6. Object write integration

During candidate build:

- logical objects written/verified through ObjectStore;
- candidate object reference set accumulated transactionally/logically;
- StoreGeneration cannot seal until every referenced ObjectId/payload validates;
- object published before generation manifest references it;
- failure leaves existing shared valid object intact and removes/quarantines only unreferenced candidate artifacts;
- no GC during incomplete candidate reference accounting unless conservative snapshot excludes it.

## 7. Validation gates

Before sealing require all configured mandatory checks:

```text
runtime profile/capabilities
schema registry/target schema/migration ledger
foreign keys
registered application/domain validation
SQLite quick/integrity policy
no unexpected schema objects
no active transaction
no unexpected journal/WAL/SHM/temp sidecar
logical data manifest/count/digest
file digest/length
object logical/payload/reference manifests
profile/reference/domain generation identity
budgets complete, not cancelled/truncated
```

Any failure blocks sealing.

## 8. Manifest finalization

Construct canonical StoreManifest after database/object content is final.

Procedure conceptually:

1. close/flush write transaction/connection according to runtime profile;
2. finalize/checkpoint/remove allowed staging sidecars;
3. reopen/inspect with validation profile if needed;
4. compute SQLite file digest/length;
5. compute logical data/schema/migration/object-reference digests;
6. create StoreGenerationId and manifest;
7. verify all manifest references/digests;
8. write manifest through atomic temp/flush/rename within staging generation directory;
9. no further SQLite/object mutation.

If manifest changes due to final validation metadata, recompute generation/manifest identities according to noncyclic contract. StoreGenerationId cannot directly hash a manifest field containing itself without domain-separated construction.

## 9. Sealing

Seal report proves:

```text
candidate state validation-complete
SQLite file closed and immutable by policy
no pending temp/journal/WAL/SHM sidecar
manifest finalized and matches files/objects
referenced objects present/verified
final generation directory content set complete
permissions/open-mode expectations ready
```

Sealed generation remains in staging namespace until publication.

No write after seal. Any required change creates a new candidate/build ID.

## 10. Generation publication

```text
publish_generation(sealed, final_generation_path)
```

Requirements:

- final path derives from validated StoreGenerationId/namespace, not untrusted text;
- final parent/root same configured publication volume;
- no overwrite of different existing generation;
- if same generation already exists, validate full equivalence and treat as idempotent success; mismatch is corruption/collision;
- use platform adapter atomic rename/no-replace/replace semantics as required;
- flush file/directory metadata according to requested/tested durability level;
- reopen/validate generation at final path before pointer update;
- record publication result/durability.

Readers can address published generation directly even before it becomes active.

## 11. Active pointer publication

```text
ActivePointerRecord
    store ID
    new active generation/relative path/manifest digest
    previous active generation
    pointer schema/digest
```

Procedure:

1. validate published generation at final path;
2. build canonical pointer bytes;
3. write pointer temp under same directory/volume;
4. flush according to durability policy;
5. atomically replace active pointer;
6. flush parent directory according to platform adapter;
7. reopen/validate pointer and active generation;
8. record publication/activation result.

Generation remains valid if pointer update fails. Previous pointer remains authoritative. A retry can activate the already published generation after revalidation.

## 12. Reader acquisition

Readers:

- read/validate active pointer atomically as one file;
- resolve normalized relative generation path within root;
- open exact generation read-only with sealed profile;
- validate manifest/store/profile/reference/schema/file/object identity;
- optionally acquire generation/object leases for retention/GC;
- retain exact generation even if active pointer later changes;
- never open staging.

## 13. Crash/failure matrix

Test interruption after/before every transition:

```text
before staging creation
mid schema/migration transaction
mid domain write batch
mid object temp write
object published before DB commit/reference finalization
DB committed before validation
mid integrity/manifest generation
manifest temp written, not renamed
sealed but not generation-published
generation renamed/published, not active
active pointer temp written, not replaced
pointer replaced, parent metadata not flushed
post-pointer validation interrupted
```

Expected invariant:

- prior active pointer/generation remains valid until a fully valid pointer replacement;
- no pointer references missing/partial generation;
- no active generation mutates;
- orphan sealed/published generation may remain and is recoverable/retained/GC-eligible only after explicit scan;
- temp/candidate files never interpreted as active;
- referenced valid shared objects not deleted.

Power-loss expectations depend on recorded durability level; process-crash simulation alone does not prove them.

## 14. Idempotency and retries

Safe retry requires exact build/sealed/generation identities.

- repeated validation is read-only/idempotent;
- repeated generation publication validates existing equivalent target;
- repeated pointer publication to same generation validates and succeeds idempotently;
- retry after pointer failure does not rebuild/mutate sealed generation;
- retry after candidate failure starts new candidate unless exact staging state has a formally supported recovery contract (none required in E1-A);
- never retry against another implicit generation/schema/profile.

## 15. Cancellation

Cancellation checkpoints:

```text
before/after migration edge or bounded operation batch
between object writes
before each validation class
before manifest finalization
before seal
before generation publication
before active pointer publication
```

After entering noninterruptible platform atomic replace/flush call, complete/observe the call then classify state safely. No partial result/pointer guessing.

Cancellation:

- rolls back open write transaction;
- does not seal/publish/activate candidate;
- leaves prior active unchanged;
- no background continuation;
- returns exact transition/candidate state and cleanup/quarantine status.

## 16. Durability adapter

```text
PublicationPlatformAdapter
    adapter ID/version/platform/filesystem assumptions
    same-volume check
    atomic rename/no-replace/replace operations
    file flush
    directory flush
    metadata/permission application
    tested durability capabilities
    error normalization
```

One pinned adapter per supported platform in implementation. Avoid a generic filesystem plugin system.

Durability record distinguishes:

```text
process_atomic
crash_atomic_same_volume
power_loss_best_effort
power_loss_tested
```

Use only levels proven by adapter test/evidence.

## 17. Last-known-good and retention

- active pointer identifies current published generation;
- previous active generation retained at least until new activation/open validation succeeds and retention policy permits;
- failed candidate/failed activation recorded separately;
- last-known-good keeps original StoreGenerationId/manifest;
- no relabel as requested failed target;
- retention deletion coordinated with reader/object leases/references.

## 18. Deterministic publication records

Canonical semantic record includes:

```text
candidate/sealed/generation/pointer IDs
previous active generation
transition states/results
schema/store/object/integrity report IDs
platform adapter/durability level
```

Exclude event timestamps, temp filenames, system error prose, retry count if semantically irrelevant, worker order. Operational logs can record those outside canonical digest.

## 19. Required operations

```text
create_store_build_candidate
create_staging_store
begin_store_write_transaction
execute_registered_operation_batch
commit_or_rollback_store_transaction
validate_candidate_store
finalize_store_manifest
seal_store_generation
publish_store_generation
publish_active_pointer
acquire_active_store_generation
open_published_reference_store
record_publication_transition
recover_orphan_published_generation
abort_clean_or_quarantine_candidate
validate_publication_state_machine
```

## 20. Required tests

- valid complete publication;
- every crash/failure/cancellation point;
- generation published/pointer failure/retry;
- existing equivalent generation idempotent;
- existing mismatched same generation rejected;
- pointer never references partial/missing generation;
- active reader remains old generation after pointer change;
- last-known-good preserved and not relabeled;
- staging path inaccessible to reader resolution;
- sidecars absent before seal;
- write after seal rejected;
- durability level not overstated;
- randomized insertion/temp/staging names do not alter generation/manifest IDs;
- no active pointer update on schema/data/integrity/object failure.

## 21. Hard stops

- no active pointer before final-path validation;
- no staging exposure;
- no write after seal;
- no overwrite mismatch;
- no silent fallback/relabel;
- no reader generation switch mid-lease;
- no cancellation/background publication;
- no power-loss claim without test;
- no in-place active ReferenceStore migration;
- no deletion/GC based solely on failed candidate cleanup.
