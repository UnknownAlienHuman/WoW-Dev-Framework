# Content-addressed object store

**Status:** normative E1-A logical-object, encoded-payload, reference, retention, and garbage-collection contract.

The object store holds large/raw/skeleton/compressed artifacts without embedding source paths or making storage encoding part of logical identity.

## 1. Logical object identity

```text
ObjectId = sha256(canonical uncompressed logical bytes)
```

Canonical bytes are supplied by the owning domain adapter under an explicit media/type/version contract. Store validates bytes/digest but does not interpret WoW semantics.

Rules:

- SHA-256 lowercase fixed-length hex canonical form;
- digest algorithm/version explicit in manifest;
- logical byte length recorded;
- no filename/path/source symbol/name in ObjectId;
- no compression timestamp/container metadata in ObjectId;
- no random/time/temp data;
- an object is immutable after publication.

## 2. Logical object media/type

```text
ObjectLogicalType
    type namespace/name/version
    canonicalization contract ID
    content encoding before storage: raw logical bytes
```

Examples may later include raw APIDoc, source skeleton bundle, source snippet, or runtime probe payload. `wow-store` treats type as validated opaque tag and enforces registered size/codec policy.

## 3. Encoded payload

Storage may encode/compress logical bytes.

```text
EncodedPayloadRecord
    payload ID
    ObjectId
    codec namespace/version
    deterministic parameters/profile
    encoded payload sha256
    logical length
    encoded length
    relative path
```

Rules:

- payload digest verified independently;
- decode result must hash to ObjectId and match logical length;
- codec/parameters registered and versioned;
- nondeterministic codec metadata excluded/normalized or encoded payload reproducibility not claimed;
- multiple encodings may exist for one ObjectId if policy permits; deterministic preferred encoding selected explicitly;
- unknown codec/payload mismatch rejects read/activation.

## 4. Path layout

Conceptual root-confined layout:

```text
objects/<algorithm>/<first-2-hex>/<next-2-hex>/<full-object-id>/<payload-id>
```

Exact fanout freezes in implementation.

Path rules:

- derive only from validated canonical IDs;
- no user/source name/path segments;
- no absolute/traversal/device/reserved component;
- normalize and verify final path remains under configured ObjectStore root;
- reject symlink/reparse escape at every component according to platform adapter;
- fixed maximum path/component length;
- private temp/quarantine subtrees not reader-addressable.

## 5. Write request

```text
ObjectWriteRequest
    logical type/canonicalization contract
    canonical logical byte source/length
    expected ObjectId: optional
    selected codec/profile
    expected payload ID/digest: optional
    candidate build/generation ID
    budgets/cancellation
```

Byte source can be bounded stream supplied by owning process, not arbitrary path opened from source input.

## 6. Write sequence

```text
validate request/type/codec/budgets
-> stream logical bytes through SHA-256 and encoder
-> write encoded payload to same-volume private temp file
-> verify logical length/ObjectId
-> finalize encoder
-> verify encoded length/payload digest
-> flush temp payload according to durability policy
-> derive final root-confined path from validated IDs
-> if final payload absent: atomically publish no-replace
-> if present: validate existing manifest/payload/logical decode contract
-> write/finalize ObjectManifest atomically if needed
-> verify final read/path/digests
-> return ObjectWriteResult
```

No final path derived before IDs validate.

## 7. Existing object/deduplication

When an ObjectId/payload target already exists:

### Valid equivalent

- manifest/type/logical length/codec/payload digest match;
- optional decode verification under policy;
- treat as idempotent dedup success;
- candidate temp removed;
- no overwrite.

### Mismatch/corruption/collision

- same logical ObjectId but manifest/payload/logical bytes mismatch;
- quarantine/report both safely;
- do not overwrite existing or publish candidate under same ID;
- store/publication activation requiring object fails;
- no assumption that cryptographic collision is impossible enough to ignore file corruption/path race.

## 8. Read sequence

```text
validate ObjectId/type/encoding selection
-> resolve root-confined manifest/path
-> validate manifest ID/digest/type/length
-> open payload read-only/no-follow according to adapter
-> stream payload digest check and decode
-> stream logical ObjectId/length check
-> enforce read/output budgets/cancellation
-> return bytes/stream plus verified ObjectReadReport
```

Depending on trusted sealed manifest/profile, a fast path may defer full decode/hash after previous verification, but correctness-sensitive consumers/publication validation require the exact policy and retained integrity evidence. Corruption must be detected before data is trusted.

## 9. Object manifest publication

ObjectManifest is immutable and atomically published. It includes all accepted encoded payload records. Adding another encoding either:

- creates a new immutable manifest version/ID selected explicitly; or
- follows an append-only atomically replaced manifest contract with stable logical ObjectId and explicit manifest generation.

E1-A should choose the simpler single preferred payload per object unless measurement requires multiple encodings.

## 10. Generation references

A StoreGeneration records exact ObjectIds through `ObjectReferenceRecord`.

Reference-set finalization:

- candidate accumulates references;
- every object present/verified before generation seal;
- canonical reference set sorted/digested;
- generation manifest records reference count/digest;
- publication creates retained generation-reference ownership;
- cancellation/failure does not create retained reference ownership;
- shared preexisting object remains safe even if candidate fails.

Store does not interpret opaque owner record key.

## 11. Reader leases

When a reader opens a generation/object stream, retention/GC must know it may still be in use.

E1 implementation may use process-local generation leases plus retained-generation policy. Cross-process lease design requires explicit support and testing; do not pretend process-local state protects another process.

Uncertain lease state blocks deletion.

## 12. Garbage collection

Inputs:

```text
retained generation manifest set
active pointer(s)
last-known-good policy
published-but-not-active recovery/retention set
object reference records
reader/object leases
quarantine/corruption records
budget/cancellation
```

Decision:

```text
eligible=yes
    no retained/active/published-protected generation reference
    no lease
    complete authoritative reference scan
    not quarantined/under investigation
    final path/manifest validates

eligible=no
    referenced/protected/leased

eligible=unknown
    incomplete scan/lease/reference/integrity state
```

Only `yes` deletes.

Deletion:

- remove payload/manifest through root-confined no-follow operations;
- verify target IDs/path before removal;
- record deletion/skips;
- tolerate already-missing only under explicit idempotent policy and report integrity gap;
- cancellation stops future deletions; completed deletions recorded;
- no broad recursive delete from untrusted path.

## 13. Orphan recovery

Crash may leave:

```text
temp payload
published valid object with no retained generation reference
published generation not active but referencing object
quarantine record
```

Recovery scan classifies by validated manifest/IDs and publication/retention records. It never guesses based only on file age/name.

Age may be a nonauthoritative cleanup guard after reference completeness, not proof of unreferenced status.

## 14. Compression/determinism

- choose/pin codec/version/parameters;
- set/normalize timestamps/dictionaries/metadata affecting payload bytes;
- record whether payload bytes reproducible;
- logical ObjectId deterministic regardless;
- payload digest/manifest detects encoded drift;
- codec update can produce a new payload/manifest without changing logical ObjectId if decoded canonical bytes equal.

No compression implementation selected until benchmark/security/license/compatibility review.

## 15. Size and resource controls

Bound:

```text
logical object bytes
encoded object bytes
compression ratio/decompression expansion
codec memory/window/dictionary
object count per generation
path depth/count
temp/quarantine bytes
read/output bytes
GC scan/delete work units
```

Abort safely on violation; no final publication/reference.

## 16. Security

- never execute/object-interpret source;
- codecs chosen/owned by framework, no source-supplied plugin;
- reject malformed encoding before allocation beyond bounds;
- no symlink/reparse/hardlink confusion under root according to adapter;
- no source/private path in manifest/path/error;
- no world-writable/shared untrusted root without explicit threat model;
- existing-file validation prevents preplacement attack from being treated as dedup;
- quarantine has bounded/retention policy and cannot be reader-addressed.

## 17. Canonical reports

```text
ObjectWriteResult
ObjectReadReport
ObjectReferenceSetReport
ObjectGcDecision
GarbageCollectionReport
```

Canonical semantics include IDs/digests/counts/status/reasons; exclude temp path/time/order/system error prose.

## 18. Required operations

```text
validate_object_id
build_object_manifest
validate_object_manifest
select_object_encoding
write_object_atomic
validate_existing_object_for_dedup
read_object_verified
build_object_reference_set
validate_object_reference_set
record_generation_object_references
acquire_release_object_lease
classify_object_gc_eligibility
garbage_collect_objects
scan_recover_orphan_objects
quarantine_corrupt_object
```

## 19. Required tests

- logical ObjectId known vectors;
- same bytes/different chunking/order -> same ID;
- different compression -> same ObjectId/separate payload ID;
- write/read/verify;
- exact dedup no overwrite;
- existing mismatched payload/manifest rejected/quarantined;
- temp/write/flush/rename crash points;
- path traversal/malformed digest/symlink-preplacement rejected;
- decompression bomb/size/ratio/memory limits;
- missing/unknown codec rejected;
- reference set deterministic;
- candidate failure creates no retained refs;
- active/last-known-good/published-protected/leased object not GC'd;
- incomplete scan -> unknown/no delete;
- unreferenced eligible object deleted and reported;
- cancellation stops future deletes, no broad recursive damage;
- temp root/time/source filename does not change logical identity;
- raw/private path/value not leaked.

## 20. Hard stops

- no identity from compressed bytes alone;
- no final path from unvalidated input;
- no overwrite on same-ID mismatch;
- no age-only GC;
- no delete under incomplete/unknown reference/lease state;
- no source path/name in filenames;
- no unbounded decode/compression;
- no plugin/source-supplied codec;
- no object reference before verified publication;
- no referenced object deletion.
