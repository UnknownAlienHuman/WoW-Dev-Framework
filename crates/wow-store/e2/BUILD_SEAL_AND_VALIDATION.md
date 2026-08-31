# E2-D generation build, seal, and validation

**Status:** normative write pipeline before activation.

## State machine

```text
Requested
-> PreflightValidated
-> Planned
-> StagingPrepared
-> ObjectsVerified
-> DatabaseCreated
-> SchemaApplied
-> OperationsExecuted
-> TransactionValidated
-> TransactionCommitted
-> CheckpointedAndClosed
-> ImageChecksummed
-> GenerationDirectoryPublished
-> ReopenedReadOnly
-> StoreValidated
-> AwaitingDomainAttestations
-> DomainValidated
-> EligibleForActivation
```

Terminal/noncurrent states:

```text
Cancelled
Failed
SealedInactive
Quarantined
CleanupPending
```

No state is inferred from file presence alone.

## Request preflight

Validate before expensive work:

- store root/registry/profile compatibility;
- exact publication key and expected current identity;
- operation ID/idempotency request digest;
- project/graph candidate and generation IDs present;
- selected schema/operation/query/validation bundles registered and compatible;
- invocation dependency graph/phases/budgets valid;
- object declarations and supplied streams bounded;
- expected logical manifests/counts/digests complete;
- cancellation not already requested;
- no active incompatible recovery/GC operation;
- sufficient configured storage quota under policy.

Preflight does not change current.

## Idempotency lookup

Before staging:

```text
no record
    insert Planned record in registry transaction

same operation ID + same request digest
    return/resume exact recorded state according to recovery policy

same operation ID + different request digest
    reject

already Activated
    return existing receipt after closure validation
```

No second independent build for an already activated identical request.

## Generation plan

Canonical plan construction:

1. validate selected bundles;
2. canonicalize dependency graph and phases;
3. reject ambiguous/order-dependent duplicates;
4. bind exact object set;
5. derive `StoreGenerationId` from logical state;
6. derive expected generation layout;
7. build validation catalog;
8. persist plan digest/idempotency transition.

A derived generation ID is not publication success.

## Object staging

- stream each object into operation-local temp file;
- hash/length-check while writing;
- validate declared type/license/security policy;
- canonical duplicate digest merges references only after metadata compatibility;
- place object atomically at final digest path or validate existing object;
- record staged/verified object result;
- do not count an object as retained authority until a committed generation references it.

Failure leaves no generation current; orphan objects enter later GC consideration.

## Database creation

Create a new empty database at operation staging path under exact runtime profile. Reject preexisting unexpected database/sidecars. Apply fixed store metadata and exact selected domain schemas.

Never:

- copy an untrusted database as baseline;
- attach a prior generation for writable migration;
- inherit row IDs/pages as semantic identity;
- allow caller PRAGMA or schema SQL.

## Operation execution

```text
begin one generation transaction
-> apply schema/create-from-empty steps
-> execute canonical project write invocations
-> execute canonical graph write/replacement invocations
-> build deterministic domain indexes/views
-> insert exact logical and object-reference manifests
-> run in-transaction generic/domain-registered validation operations
-> compare all expected counts/digests/results
-> commit once
```

Any failure/cancellation before commit rolls back the complete generation database transaction.

## In-transaction validation

At minimum:

- selected schema/user/application versions;
- required tables/indexes/constraints created by reviewed bundles;
- foreign-key and uniqueness closure;
- project and graph logical manifest rows present;
- operation result counts/digests equal plan;
- object reference set equals verified plan;
- no unexpected owner/bundle/operation records;
- no dangling internal store references visible through registered checks;
- coverage/conflict/truncation manifests retained exactly;
- cancellation/budget state permits commit.

## Commit and checkpoint

After successful commit:

- execute frozen checkpoint/journal cleanup sequence;
- run required SQLite quick/integrity/foreign-key checks under build profile;
- ensure no writable transaction/statement/connection remains;
- close writer and auxiliary handles;
- verify declared sidecar policy;
- fsync according to durability profile.

A committed staging database is not yet a sealed generation.

## Image and member checksums

Compute exact:

```text
project.sqlite byte length and SHA-256 = StoreImageId
generation-manifest.json digest
image-checksums.json digest/object member set
optional declared sidecars: normally none after seal
```

Manifests bind exact `StoreGenerationId`, profile IDs, schema/operation bundles, logical manifest, object set, and build/store validation reports.

Avoid manifest self-hash cycles: identity construction excludes fields containing the final identity and uses explicit staged manifest layers.

## Atomic generation-directory publication

- target path derived solely from validated generation ID;
- if absent, atomically rename complete staging generation directory;
- if present, validate exact logical generation and accepted image/idempotency policy;
- never overwrite or merge directory members;
- fsync parent per profile;
- update registry generation lifecycle to `SealedInactive` only after final path checks.

A directory that appears after crash but lacks a valid registry/manifests is recovery input, not current.

## Read-only reopen

Open the exact final `project.sqlite`:

```text
read-only/query-only/defensive
selected runtime/profile compatibility
no writable sidecar creation
exact StoreImageId and member manifest
exact application/schema/logical generation metadata
```

Run registered read/golden queries and compare canonical results to plan. Do not validate only through the writer connection.

## Store validation report

Store validation includes:

```text
final member/checksum closure
SQLite open/quick/integrity/foreign-key/schema checks
logical manifest/count/digest checks
operation bundle and object-reference closure
read-only query result closure
private path/payload leakage checks
budget/cancellation state
```

`StoreValidated` does not mean project or graph semantics accepted.

## Domain attestations

The caller opens the sealed exact generation through domain-owned read views/query bundles:

```text
wow-project validator -> Project attestation
wow-graph validator   -> Graph attestation
```

Attestations must:

- bind the same generation/image;
- bind exact project/graph candidates/generations/manifests;
- report golden queries and blockers;
- retain partial/conflict/coverage state;
- be accepted under the target publication policy.

Mixed/stale/image-mismatched attestations make the generation ineligible for activation.

## Cancellation points

Check during:

```text
preflight/plan
object streams
schema/operation batches
validation loops
checkpoint/checksum
read-only validation
waiting/recording attestations
before activation CAS
```

If cancellation arrives after an irreversible durable state:

- report exact state;
- create/update recovery/idempotency record;
- never hide sealed or active outcome;
- never continue in background.

## Failure isolation

| Failure point | Current pointer | Target state |
|---|---|---|
| preflight/plan | unchanged | none/failed record |
| object stage | unchanged | cleanup/orphans |
| DB transaction | unchanged | rolled back/cleanup |
| after commit before seal | unchanged | recovery staging |
| after final rename before reopen | unchanged | sealed candidate/recovery |
| store validation | unchanged | quarantined or sealed invalid |
| domain attestation | unchanged | sealed inactive/rejected |
| activation CAS | unchanged on stale/failure | sealed inactive |
| activation committed, response lost | new exact set active | idempotent receipt recovery |

## Hard stops

- no current pointer before final read-only/domain validation;
- no partial generation directory merge;
- no writable open after seal;
- no sidecar ignored by checksum/security policy;
- no store validation used as domain authority;
- no cancellation translated into successful complete publication;
- no cleanup that deletes an activated/leased generation.
