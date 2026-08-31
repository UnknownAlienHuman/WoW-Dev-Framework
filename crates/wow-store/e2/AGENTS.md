# AGENTS.md — `wow-store` E2-D

## Scope

Implement only the ProjectStore physical profile, registered generation transaction, sealed read views, atomic head primitive, recovery, retention, and GC.

Do not implement project, graph, analyzer, recognizer, diagnostic, search, runtime, or application semantics.

## Required reading

1. repository and `crates/` agent rules;
2. [`../README.md`](../README.md) and the root E1-A contracts;
3. all files in this E2-D package;
4. [`../../wow-project/e2/PUBLICATION_BOUNDARY.md`](../../wow-project/e2/PUBLICATION_BOUNDARY.md);
5. [`../../wow-project/e2d/README.md`](../../wow-project/e2d/README.md);
6. [`../../wow-graph/e2/PERSISTENCE_BOUNDARY.md`](../../wow-graph/e2/PERSISTENCE_BOUNDARY.md).

## Before coding

- Freeze the exact SQLite/runtime/library pin and compile options.
- Freeze schema, registered-operation, validation-catalog, physical-profile, and durability IDs.
- Freeze staging, sealing, CAS, lease, recovery, and GC vectors.
- Freeze every fixture/member SHA-256.
- Verify prerequisite `wow-core` implementation and E1-A store foundation.
- Do not replace a null fixture pin with a guessed or floating value.

## Transaction discipline

- One writer owns one staging generation.
- Validate the complete plan before expensive writes.
- Execute only registered operations with typed bounded payloads.
- No caller SQL, callback, closure, table name, PRAGMA, path, or connection.
- Domain validation failures abort the generation.
- Commit, checkpoint, close, and seal before read-only validation.
- Head CAS occurs only after coordinator post-seal validation.
- A CAS conflict never mutates the current head or rebases automatically.

## Immutability discipline

- Published generation files and manifests are immutable.
- No in-place migration, repair, compaction, or metadata rewrite.
- Repair/repack creates a new artifact/generation and preserves provenance.
- Physical row IDs never escape.
- `immutable=1` or equivalent optimization is permitted only after ownership, sealing, checksums, and read-only preconditions are proven.

## Lease and GC discipline

- Resolve the publication head once.
- Acquire a lease for one exact generation before opening it.
- A lease never follows a newer head.
- Current, last-known-good, pinned, leased, recovery, quarantine, and evidence roots are retained.
- GC is mark-and-sweep from validated roots and manifests.
- Age may rank eligible garbage but never proves eligibility.

## Security discipline

- No arbitrary database/file opening.
- No symlink/reparse traversal.
- No source execution, network, process, editor, client, extension, or plugin behavior.
- No dynamic SQL identifiers or user migration scripts.
- Bound rows, payloads, objects, statements, pages, WAL, validation output, leases, and recovery inventory.
- Redact private paths/tokens from public records.

## Completion report

```text
store profile and exact prerequisite pins
base/head and target generation IDs
registered bundles and operation counts
logical/physical/object manifests
transaction/seal/open/CAS result
fault/cancel phases tested
lease/read-consistency result
recovery/retention/GC result
determinism result
deferred capabilities and unperformed runtime checks
```
