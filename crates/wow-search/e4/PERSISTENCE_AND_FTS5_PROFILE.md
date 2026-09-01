# E4-A SearchStore and FTS5 physical-profile contract

**Status:** normative candidate physical design; executable values remain blocked until probed and frozen.

## Selected candidate profile

```text
search-shard-immutable-sqlite-fts5-v1
```

One immutable SQLite artifact represents one exact SearchShard. It is a derived sidecar, not part of owner truth.

## Dependency boundary

`wow-search` defines:

- logical tables/records and invariants;
- document/field/index/query schemas;
- FTS content/column/tokenizer/query/rank semantics;
- prepared logical operations;
- expected manifests and validation/golden queries.

`wow-store` defines:

- SQLite library/binding/VFS/open flags;
- file and transaction lifecycle;
- staging/finalization;
- read-only/query-only reopening;
- content-addressed object handling;
- durability/integrity plumbing;
- retention/recovery/GC.

No raw SQL, connection, table name, PRAGMA, extension, rowid, path, or transaction callback crosses the public seam.

## Required executable probe

Freeze:

```text
SQLite source/library version and digest
Rust binding/version/feature and Cargo.lock digest
compile_options and source ID
FTS5 built-in availability
available built-in tokenizers and exact behavior
unicode/version/diacritic/tokenchars/separators behavior
detail/content/contentless/columnsize/prefix option behavior
bm25/rank/snippet/highlight behavior and numeric representation
integrity-check and secure/open/read-only/query-only behavior
page/journal/temp/cache/limit settings
platform adapter and filesystem semantics
```

No loadable-extension fallback if FTS5 or a tokenizer is missing.

## Tokenizer choice

Default candidate:

```text
built-in unicode61-derived text profile
```

Exact arguments are not accepted until probe/evaluation. Identifier similarity can be implemented in deterministic logical indexes rather than relying on a built-in trigram tokenizer unless the exact probe/profile selects and freezes it.

No source-controlled tokenizer, synonym dictionary, stemmer, auxiliary rank function, or extension.

## Logical record families

```text
search_shard_metadata
search_source_binding
search_profile_manifest
search_partition
search_partition_membership
search_document
search_field_value
search_field_origin
search_exact_key
search_alias_key
search_member_key
search_prefix_key
search_identifier_feature
search_shape_feature
search_fts_content and FTS virtual table mapping
search_validation/golden reports
optional search_result_set object references
```

Physical names may differ; public identity does not.

## Row identity

SQLite rowids/docids are private and deterministically mapped from a build-local ordered document list when FTS requires integer IDs. They are not stable public IDs and never enter result identity or continuation.

A validation table maps private rowid to exact `SearchDocumentId` and verifies one-to-one closure.

## Build mode

```text
create new empty staging database
-> apply exact static schema/profile
-> write complete document partitions and logical indexes
-> write FTS content in canonical document/field order
-> run FTS maintenance required by profile
-> run logical/integrity/golden validation
-> commit/checkpoint/close
-> publish immutable artifact
-> reopen read-only/query-only
-> validate again
```

No in-place update of a published shard.

## FTS rank semantics

- use raw rank only inside the same shard/profile;
- convert to deterministic local ordinal with stable tie keys;
- do not store platform-dependent float as cross-shard canonical evidence;
- freeze column weights and rank expression;
- snippet/highlight output is optional nonauthority presentation data;
- exact identifier/alias indexes do not rely on FTS.

## Physical reproducibility

Classify separately:

```text
LogicalEquivalentPhysicalMayDiffer
ByteIdenticalWithinFrozenProfile
ByteIdenticalRequired
```

E4-A requires logical determinism. Byte identity is claimed only after repeated platform/profile evidence.

Physical digest can identify the artifact object but does not replace logical `SearchShardId`.

## Integrity

Validate at least:

- SQLite quick/integrity checks per profile;
- application/user/schema version;
- migration ledger and compile/runtime profile;
- no extra tables/triggers/extensions;
- document/field/origin counts and digests;
- private rowid-to-document closure;
- FTS content/index integrity and exact field mapping;
- exact/alias/member/prefix/shape index closure;
- no stale/removed partition documents;
- golden lane/ranking/miss results;
- read-only/query-only enforcement.

## Limits

Set explicit SQLite and search limits for:

- database/page/row/value size;
- SQL length/variables;
- expression/compound/column limits;
- FTS term/prefix/phrase complexity;
- result rows;
- memory/temp storage;
- busy/timeout/cancellation;
- snippet bytes.

No unlimited defaults.

## Security

- owned root only;
- no arbitrary database import/adoption;
- no ATTACH/DETACH;
- no extension loading;
- no writable open for query;
- no source/runtime code execution;
- no network/process/editor/client access;
- no SQL from users/source/docs;
- errors/logs omit private paths/source bodies/query secrets.

## Retention and recovery

SearchShard is immutable. Incomplete/corrupt artifacts are quarantined or deleted under store policy. A missing derived shard can be rebuilt from exact retained owners; it cannot be silently substituted with another generation.

Result-set objects and continuations are retained by exact references and removed only after lease/root closure.
