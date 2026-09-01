# E4-A index build, validation, and publication

**Status:** normative.

## Ownership boundary

`wow-search` owns:

- owner-record projection to SearchDocuments;
- document/field/partition schemas;
- exact/alias/member/prefix/similarity/shape logical indexes;
- FTS logical content and query profile;
- shard IDs/manifests and validation/golden catalogs;
- search-specific coverage/conflict/omission reporting.

`wow-store` owns:

- SQLite/runtime/VFS/transaction lifecycle;
- physical files and content-addressed objects;
- staging/finalization;
- read-only reopening;
- retention, leases, recovery, and GC.

Owner crates own project/reference/graph facts and negative authority.

## Build request

```text
SearchShardBuildRequest
    exact SearchShardSourceBinding
    SearchProfileSet
    owner read-catalog IDs
    optional exact base SearchShard
    target partition plan
    expected logical counts/digests
    privacy/license policy
    budgets/cancellation
```

No symbolic current, source path, raw SQL, or arbitrary callback.

## Projection pipeline

```text
validate exact owner view and profiles
-> enumerate declared owner partitions under complete bounded catalogs
-> project allowed owner records to typed SearchDocuments
-> validate field origins, privacy, coverage, and conflicts
-> canonicalize and content-address document partitions
-> compare optional exact base partition manifests
-> reuse byte/logically identical immutable partitions
-> identify new/replaced/removed partitions
-> build complete target shard membership
-> materialize logical exact/alias/member/prefix/shape indexes
-> materialize bounded FTS content/index
-> commit PublishedInactive shard artifact
-> reopen through a fresh read-only view
-> run integrity, logical, coverage, privacy, and golden query validation
-> seal as Validated/SealedReadOnly
```

## Partition keys

Partitioning follows owner/public semantics, not SQLite page layout. Initial keys may use:

```text
universe + owner generation + projection producer/version
+ owner partition kind/ID
+ document schema/profile
```

Examples include one project source partition, one graph producer partition, or one reference system partition.

## Complete target membership

Every SearchShard manifest contains a complete ordered target partition map. It is not a recursive delta chain. Removed partitions are absent from target membership and stale documents cannot remain queryable.

## Incremental reuse

Reuse requires exact equality of:

- owner generation-compatible partition identity;
- owner record/document input manifest;
- projection/profile/schema IDs;
- privacy/license policy;
- field/origin/coverage/conflict state;
- canonical SearchDocument partition digest;
- exact/alias/prefix/shape/FTS profile compatibility.

Same path, name, mtime, row count, or cache entry is insufficient.

## No-change

If the exact target membership, profile, source binding, documents, indexes, and validation outputs are unchanged:

```text
NoChange
```

No new semantic shard ID or duplicate physical publication is created.

## Shard identity

`SearchShardId` derives from:

- exact source binding;
- SearchProfileSet;
- complete ordered partition membership/digests;
- logical exact/alias/prefix/shape/FTS manifests;
- logical validation and capability profile versions.

It excludes:

- SQLite row/page IDs;
- physical file path;
- WAL/checkpoint state;
- clock/process/thread/host;
- insertion or worker order;
- physical byte digest unless the accepted profile explicitly includes a byte-reproducibility class.

## Validation

Required validation classes:

```text
owner binding and generation closure
document/field/origin/evidence closure
partition membership and stale-removal closure
exact/alias/member/prefix index correctness
FTS content-to-document mapping and integrity
shape and similarity index correctness
privacy/license/source-field enforcement
coverage/conflict/omission reconciliation
golden query and ranking vectors
1/2/N worker and shuffled-order determinism
read-only reopening and physical-integrity checks
```

Validation is read-only and nonrepairing.

## Failure and cancellation

- no current/search catalog advancement before validation;
- build failure/cancellation leaves prior shards untouched;
- invalid inactive artifacts are failed/quarantined under original identity;
- no automatic fallback to previous shard while reporting target success;
- no background build after cancellation;
- response loss may be recovered through exact idempotency records owned by the store profile, not by rebuilding blindly.

## Registration/catalog

Search core has no mutable current pointer. A higher owner/service catalog may register:

```text
exact owner generation -> exact validated SearchShardId
```

Registration cannot alter shard identity. Search requests still bind an exact shard set.

## Removal closure

Deleting or changing an owner partition must remove every stale target:

```text
SearchDocument
field value/origin
exact/alias/member/prefix/similarity/shape index entry
FTS content/index entry
candidate golden result reference
detail/source handle mapping
coverage/conflict/omission summary
result-set continuation root when no longer retained
```

## Rebuild/discard

SearchShard is a derived sidecar. It can be discarded and rebuilt from exact retained owner generations. Rebuild must reproduce logical identity and query results; failure to reproduce is an integrity problem, not permission to silently choose a new profile.
