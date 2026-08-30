# `wow-store` E1-A decisions

**Status:** normative for the storage foundation and immutable ReferenceStore path.

## STORE-001 — Store depends only on `wow-core`

**Decision:** no domain crate dependency.

**Consequence:** storage lifecycle/identity/integrity remain reusable and acyclic.

## STORE-002 — Store owns mechanics, consumers own semantics

**Decision:** domain crates provide compile-time schema/operation bundles and encode/decode their own records.

**Consequence:** store does not know API/entity/restriction/project/rule meanings.

## STORE-003 — No generic raw SQL public surface

**Decision:** only repository-owned registered schema/migration/prepared-operation catalogs can execute SQL.

**Consequence:** service/CLI/MCP/LSP/source/user input cannot submit arbitrary SQL.

## STORE-004 — Standard metadata schema is domain-neutral

**Decision:** store owns metadata for schema, migration, generation, publication, integrity, objects, references, and retention only.

**Consequence:** domain columns/tables remain in consumer-owned schema namespaces.

## STORE-005 — Schema bundle is exact and digest-bound

**Decision:** namespace/version/parent graph/DDL/migrations/validation/capabilities have stable IDs and canonical digest.

**Consequence:** unknown, modified, or reordered semantic bundles fail rather than drift silently.

## STORE-006 — Migration graph is explicit and acyclic

**Decision:** no automatic version guessing, skipped migrations, or “best effort” upgrade.

**Consequence:** each source-to-target path is known, unique/selected deterministically, and testable.

## STORE-007 — Released ReferenceStores are never migrated in place

**Decision:** a new target schema/profile/reference generation is built in staging and atomically published as a new immutable store.

**Consequence:** rollback and historical references remain intact; readers never see a mid-migration release artifact.

## STORE-008 — E1-A activates immutable ReferenceStore only

**Decision:** ProjectStore WAL/mutable path remains Deferred to E2.

**Consequence:** no premature writer actor/checkpoint/watcher/project schema implementation.

## STORE-009 — One ReferenceStore contains one exact profile/reference generation

**Decision:** no mixed profiles or floating current identity inside a store.

**Consequence:** open/manifest validation can reject cross-profile leakage deterministically.

## STORE-010 — ReferenceStore build occurs in staging

**Decision:** schema/data/object/integrity/manifest work completes outside the published generation.

**Consequence:** active readers never observe partial writes.

## STORE-011 — Seal precedes generation publication

**Decision:** a generation becomes immutable and validation-complete before it is moved into published namespace.

**Consequence:** no mutable published candidate.

## STORE-012 — Generation publication precedes active-pointer publication

**Decision:** the small active pointer is updated only after the generation exists and validates at its final path.

**Consequence:** crash before pointer replacement leaves previous active generation valid.

## STORE-013 — Active pointer is a versioned manifest, not a required symlink

**Decision:** use same-volume atomic file replacement through a platform adapter.

**Consequence:** Windows/non-symlink environments remain first-class.

## STORE-014 — Durability level is explicit and measured

**Decision:** publication records the flush/fsync/rename guarantees actually achieved by the platform/binding adapter.

**Consequence:** no unsupported power-loss/atomicity claim.

## STORE-015 — Published generation identity never changes

**Decision:** sealed files/manifests/objects are immutable.

**Consequence:** cache/source handles/checksums remain reproducible.

## STORE-016 — Runtime ReferenceStore opens least-privilege read-only

**Decision:** use read-only/query-only/immutable/defensive modes where supported and verified.

**Consequence:** accidental writes, journals, schema changes, and extension behavior are blocked.

## STORE-017 — Released ReferenceStore has no WAL/SHM/journal sidecars

**Decision:** staging journals/checkpoints are finalized/removed before sealing.

**Consequence:** release artifact is self-contained and read-only.

## STORE-018 — SQLite configuration is part of store compatibility

**Decision:** behavior-affecting library version/compile options/open flags/PRAGMAs/page/schema settings are recorded and probed.

**Consequence:** consumers do not assume another SQLite environment behaves identically.

## STORE-019 — Foreign keys are enabled and verified

**Decision:** opening/building/migrating validates foreign-key behavior and runs checks where applicable.

**Consequence:** relational integrity is not an optional development setting.

## STORE-020 — Extension loading and arbitrary ATTACH are prohibited

**Decision:** no runtime extension or external DB execution path.

**Consequence:** analyzed/source/user data cannot extend the SQL execution boundary.

## STORE-021 — Untrusted SQLite is imported/rebuilt, not opened writable as owned state

**Decision:** external DB files cannot become Project/ReferenceStore truth by direct mutation.

**Consequence:** schema/integrity/security validation stays controlled.

## STORE-022 — Logical ObjectId hashes canonical uncompressed bytes

**Decision:** storage encoding/compression does not define logical identity.

**Consequence:** recompression can preserve references and deduplication.

## STORE-023 — Encoded payload identity is separate

**Decision:** codec/version/parameters, payload digest, logical/encoded lengths are recorded independently.

**Consequence:** corruption/codec drift is detectable without changing logical ObjectId.

## STORE-024 — Object paths derive only from validated digest

**Decision:** fixed-format digest fanout under a configured root; no source name/path/excerpt.

**Consequence:** path traversal/name collision/privacy leakage are prevented.

## STORE-025 — Object writes are atomic and verify existing content

**Decision:** same-volume temp write, explicit durability step, digest verification, atomic publication; existing valid object deduplicates.

**Consequence:** a same-ID mismatch is corruption and never overwritten.

## STORE-026 — Object references are generation-explicit

**Decision:** each retained store generation records exact ObjectId references.

**Consequence:** garbage collection can be conservative and auditable.

## STORE-027 — GC deletes only proven unreferenced/unleased objects

**Decision:** uncertainty, retained generation, active read lease, or incomplete reference scan blocks deletion.

**Consequence:** no availability/correctness sacrifice for cleanup.

## STORE-028 — Database presence is not evidence completeness

**Decision:** store never infers negative authority from missing rows/tables.

**Consequence:** coverage/provenance remains owned by domain/core records.

## STORE-029 — Integrity failures reject activation

**Decision:** schema, ledger, foreign-key, SQLite integrity, manifest, file/object digest, generation/profile mismatch are fatal for activation.

**Consequence:** no silent repair or partial trust.

## STORE-030 — Corruption recovery is explicit rebuild/rollback

**Decision:** do not auto-edit corrupt state into a new trusted generation.

**Consequence:** provenance and last-known-good remain clear.

## STORE-031 — Canonical logical digest is distinct from raw SQLite bytes

**Decision:** raw file byte reproducibility is claimed only after measured deterministic build/export; logical/store manifest identity is always deterministic.

**Consequence:** page allocation/SQLite metadata cannot create false reproducibility claims.

## STORE-032 — No random/time/temp data in canonical manifests

**Decision:** publication time/host/path may be supplemental noncanonical metadata only.

**Consequence:** equivalent logical build yields equivalent IDs/manifests.

## STORE-033 — One writer for future mutable stores

**Decision:** ProjectStore E2 will serialize writes through one owner/actor and publish generations transactionally.

**Consequence:** no multi-writer race or partial generation visibility.

## STORE-034 — ProjectStore WAL is not ReferenceStore policy

**Decision:** WAL applies only to owned mutable store when activated; sealed ReferenceStore remains sidecar-free.

**Consequence:** journal mode is store-kind-specific.

## STORE-035 — Cancellation never publishes partial generation/object

**Decision:** candidate/temp artifacts remain unreferenced and are cleaned/quarantined safely; active pointer unchanged.

**Consequence:** cancellation is not a commit boundary.

## STORE-036 — Store reports last-known-good; does not relabel it

**Decision:** failed candidate/publication and retained active generation keep original IDs.

**Consequence:** higher layers cannot confuse old state with requested target.

## STORE-037 — Store APIs are bounded and typed

**Decision:** file/page/object/migration/statement/result limits explicit; no unrestricted query/connection exposure.

**Consequence:** denial-of-service and architectural bypass are testable.

## STORE-038 — Static schema/operation catalogs are trusted repository code, not source data

**Decision:** only compiled/versioned catalogs reviewed in this repository are executable.

**Consequence:** source comments/config/manifests cannot inject SQL.

## STORE-039 — No release signing/distribution in store

**Decision:** store emits manifests/checksums/publication records; E7 owns release channel/signing.

**Consequence:** persistence remains transport-neutral.

## STORE-040 — Freeze SQLite/schema/object/publication vectors before code

**Decision:** exact binding/version/capability, schema/migration, generation/manifest/object/publication IDs/digests remain null while documentation-only and become mandatory before first Rust commit.

**Consequence:** implementation cannot select convenient unreviewed storage behavior.
