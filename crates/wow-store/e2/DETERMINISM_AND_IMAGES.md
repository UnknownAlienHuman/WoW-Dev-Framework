# E2-D logical determinism and physical image comparison

**Status:** normative reproducibility contract.

## Distinct determinism questions

```text
Logical determinism
    Do equivalent exact inputs produce the same IDs, records, manifests, query results, and publication set?

Physical image determinism
    Do they produce byte-identical `project.sqlite` and member files under one frozen runtime/profile?

Operational determinism
    Do retries/crashes/concurrency classify to the same semantic state without depending on timing/order?
```

These are reported separately.

## Logical determinism — mandatory

Equivalent exact request/candidate/bundle/object inputs must produce identical:

```text
canonical generation plan
StoreGenerationId
logical project/graph records and manifests
object reference-set manifest
registered operation result counts/digests
project and graph golden query results
project/graph attestation semantic payloads
PublicationSetId/Manifest
canonical publication receipt fields
retention root/closure and GC dry-run plan for the same registry state
```

Independent of:

```text
worker count
input map/list arrival order where semantics are unordered
thread/task completion
SQLite row IDs/page placement
staging root/attempt ID
wall clock/host/process
filesystem enumeration
registry sequence/epoch values excluded from identity
```

## Canonical execution plan

Operation invocations are ordered by explicit phase, semantic ordinal, dependency closure, owner/operation ID, and invocation ID. If two operations are semantically order-dependent but the contract does not declare order, reject the plan rather than choose source/arrival order.

All writes use deterministic record ordering where domain bundles require it, while constraints/manifests—not insertion order—define truth.

## StoreGeneration versus StoreImage

```text
StoreGenerationId
    canonical logical state

StoreImageId
    SHA-256 exact SQLite bytes
```

One logical generation can have:

- exactly one accepted image under a `ByteIdenticalRequired` profile; or
- multiple independently built but logically equivalent image IDs under an explicitly permitted logical-equivalence profile.

A publication set always names the exact accepted image it uses.

## Physical reproducibility classes

### `ByteIdenticalRequired`

All exact inputs and the frozen runtime/VFS/platform class must produce identical database/member bytes. Any difference blocks profile acceptance/publication.

Use only after proving:

- exact SQLite version/compile options/VFS;
- page size/encoding/journal/checkpoint/vacuum profile;
- schema and insertion/index creation order;
- no volatile metadata/pages;
- clean staging history;
- platform/filesystem compatibility.

### `ByteIdenticalWithinProfileReported`

Byte equality is expected/tested within the exact profile but a mismatch can be classified and investigated; publication policy decides whether it blocks.

### `LogicalEquivalentPhysicalMayDiffer`

Logical manifests, query results, constraints, and store validation must match; exact image digest may differ. Physical differences are always recorded and never described as byte reproducible.

E2-D defaults to the most conservative proven class selected by benchmark/probe, not aspirational byte equality.

## Image comparison

```text
ProjectStoreImageComparison
    exact logical StoreGenerationId
    left/right StoreImageId and runtime profiles
    member/path/length/digest comparison
    SQLite schema/catalog/application/user/page metadata comparison
    logical table/index/constraint/count/digest comparison through registered validation
    golden project/graph query comparison
    object reference-set comparison
    difference records and classification
    reproducibility decision
```

Do not compare raw database pages only and call logical state different without semantic checks.

## Difference classes

```text
EqualBytes
EqualLogicalAndPhysicalMetadataDifferentBytes
EqualLogicalRuntimeProfileDifferent
LogicalRecordDifference
SchemaOrIndexDifference
ConstraintOrValidationDifference
GoldenQueryDifference
ObjectReferenceDifference
MemberOrChecksumDifference
CorruptOrUnreadable
NotComparable
```

Every difference links exact profiles and checks.

## SQLite physical sources of variation

The profile/tests explicitly account for:

```text
SQLite version/compile/VFS
page size/freelist/page allocation
journal/WAL/checkpoint history
index creation/insertion order
VACUUM/ANALYZE/statistics
application/user/schema versions
file timestamps outside DB
filesystem sparse/allocation metadata (noncanonical)
crash/recovery path
```

No unknown physical variation is hidden.

## Registry and receipt determinism

Registry epochs, activation sequence, lock owner, timestamps, lease heartbeat, and attempt counters are operational and excluded from canonical publication-set identity.

Canonical receipt contains exact semantic predecessor/target/outcome IDs. Operational receipt supplements may record safe timestamps and attempt details separately.

## Object determinism

Object identity is exact uncompressed/declared-byte SHA-256 according to the object profile. If compression is stored:

- define whether digest covers source or encoded bytes;
- pin compression implementation/options for encoded-byte reproducibility;
- separate logical object ID from encoded image ID if needed;
- exclude volatile archive headers/timestamps.

## Rebuild matrix

Repeat complete build/seal/read validation with:

```text
1, 2, and N workers
randomized input record order
randomized independent invocation arrival order
new staging roots
fresh registry with same logical predecessor
same host profile repeated
supported platform/VFS classes
crash/recovery path versus clean path when declared equivalent
```

Require exact logical equality and the declared physical class.

## No-change

If target logical inputs equal current publication set under exact bundle/profile identity:

```text
outcome = NoChange
no generation database rebuild by default
no new StoreGenerationId/PublicationSetId
no pointer update
exact current receipt/view returned with explicit no-change result
```

A diagnostic physical rebuild can be requested only through a separate comparison operation and cannot silently replace current.

## Mutation tests

- omit one logical record/manifest field from StoreGeneration hash;
- include staging path/time/row ID in canonical ID;
- shuffle invocation/record order;
- alter one project or graph record while preserving count;
- alter index/constraint but not table rows;
- alter object reference with same count;
- change runtime profile without changing image manifest;
- treat different physical bytes as equal without logical comparison;
- claim physical equality from logical equality only;
- no-change creates a new arbitrary generation.

All must fail/classify exactly.

## Hard stops

- no SQLite bytes as domain generation identity;
- no physical byte-equality claim without frozen profile and test;
- no logical equality decided from row counts alone;
- no operational metadata in canonical IDs;
- no nondeterministic iteration/output ordering;
- no fixture auto-rewrite by tests;
- no relaxed validation to make two images appear equivalent.
