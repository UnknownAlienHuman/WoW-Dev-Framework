# ProjectStore publication transaction

**Status:** normative physical transaction protocol.

## Inputs

```text
ProjectStoreGenerationRequest
expected current PublicationHeadRecord
exact ProjectPublicationBundle ID/digest
registered bundle set
ordered registered operation plan
object write/reference plan
expected logical manifests/counts/digests
validation catalog
physical/durability/budget/cancellation profiles
```

## Protocol

### 1. Preflight

- validate IDs, profile, bundles, plan DAG, payload schemas, budgets;
- read exact current head and compare expected base;
- verify target generation does not conflict with existing artifact;
- validate owned root/filesystem capability;
- reserve one writer/staging ID;
- perform no domain write before cheap failures are resolved.

### 2. Stage

- create isolated staging location;
- initialize database through selected profile;
- create exact schemas;
- record request/bundle/plan identities;
- stage and verify content-addressed objects.

### 3. Execute one transaction

```text
BEGIN
  execute every registered invocation in canonical phase order
  record effect manifests
  build domain/store manifests
  execute transaction-safe validation catalogs
  reconcile expected counts/digests/references
COMMIT
```

Any invocation, cancellation, budget, or validation failure rolls back the logical transaction.

### 4. Checkpoint and close

- run frozen checkpoint rule;
- verify no required frame remains in WAL;
- close all statements/connections;
- apply durability sync sequence;
- ensure no published view can open staging.

### 5. Seal and materialize

- construct generation and artifact manifests noncyclically;
- compute member/object checksums;
- atomically move/materialize to unique final generation location;
- verify final location and bytes;
- mark as sealed inactive in registry/recovery inventory.

### 6. Reopen validation

- acquire temporary validation lease;
- open exact final generation read-only;
- run store open/integrity/read catalog;
- return `OpenValidatedInactiveGeneration`.

Domain coordinator then runs ProjectView/GraphView golden validation.

### 7. Head CAS

Only after coordinator authorization:

```text
compare_and_swap_publication_head(
    head_key,
    expected_head_id,
    new_coherent_head_payload,
    exact sealed generation ID/artifact ID
)
```

CAS success completes publication. CAS conflict leaves an inactive validated generation and unchanged current head.

## Cancellation boundaries

```text
before transaction
    remove/record empty staging; no target

during transaction
    rollback; no target

after commit before seal
    close and classify staging; no headed target

after seal before open validation
    sealed inactive; exact recovery required

after open validation before CAS
    validated inactive; may be adopted only by exact coordinator revalidation

during CAS
    result must be resolved from registry; no ambiguous blind retry

after CAS success
    publication complete; cancellation is too late
```

## Fault injection

Inject process death, IO error, disk full, checksum mismatch, SQLite error, validation failure, object collision, cancellation, and CAS conflict after every durable step. Recovery must classify the state without modifying a valid prior head.

## Idempotence

Replaying an exact request:

- if target is current and all identities match: typed `AlreadyPublished`;
- if sealed inactive and exact adoption gate holds: explicit recovery/adoption path;
- if target exists with different manifest/bytes: corruption/conflict;
- otherwise new staging build.

No request silently appends to or repairs an existing generation.
