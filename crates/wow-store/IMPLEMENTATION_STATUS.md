# `wow-store` implementation status

## Implemented E1-A boundary

- SQLite-backed synchronous durable store with private physical schema and no raw connection or SQL in the public API.
- Canonical JSON immutable objects addressed by domain-separated SHA-256 identity.
- Exact object kind, schema version, content digest and byte verification on every read and integrity scan.
- Atomic multi-object writes and catalog compare-and-swap mutations in one immediate transaction.
- Exact operation journal keyed by `OperationId + RequestDigest`, including prepared, completed, no-effect, failed and outcome-unknown terminal states.
- Idempotent replay for the same request and explicit conflict for operation-ID reuse with different request bytes.
- Logical caller-supplied lease epochs; no wall clock, UUID, randomness or hidden background expiry.
- Catalog, completed-operation and active-lease retention roots.
- Deterministic bounded garbage collection with explicit continuation state.
- SQLite integrity and foreign-key validation plus a deterministic logical manifest independent of physical database bytes and insertion order.
- Durable configuration/application/schema identity validation on reopen.

## Authority and security boundary

The store proves persistence, exact identity, atomicity and reference integrity only. It does not interpret stored JSON, infer semantic authority, select a current generation, execute source, expose raw SQLite handles, run migrations supplied by callers, or claim that physical SQLite files are reproducible artifacts.

All filesystem paths remain construction inputs to the owner and are absent from logical records, errors, receipts and manifest identity. Logical lease epochs are explicit caller data rather than ambient time.

## Current nonclaims

- No ReferenceStore or ProjectStore domain schema yet.
- No schema-migration registry or crash-injection harness yet.
- No snapshot/reader lease shared across processes.
- No service operation, CLI storage command, daemon, LSP or MCP integration.
- No encryption, signing, backup, restore, installation or release lifecycle.
- No Windows packaging claim until the repository CI and later E7 gates pass.

## Next package

E1-B may use this substrate to persist exact Reference generations and publication catalogs through typed owner records. Domain owners must not bypass this API with raw SQL or depend on SQLite row identity.
