# `wow-store` contract router

**Status:** planned storage documentation is complete through the E7-B generic release and installation seam; no Rust code exists.

`wow-store` owns generic physical persistence only. It depends directly on `wow-core` and never imports Reference, project, graph, recognizer, rule, search, context, provider, session, editor, release, installation, or application semantics. Domain owners and service supply registered schemas, canonical logical bytes, prepared operations, validation contracts, and retention edges.

## Contract routes

### E1-A — storage foundation and immutable ReferenceStore

The complete pre-E2 overview is preserved as [`E1_A_OVERVIEW.md`](E1_A_OVERVIEW.md). Its earlier ProjectStore boundary is preserved as [`PROJECT_STORE_PRE_E2_BOUNDARY.md`](PROJECT_STORE_PRE_E2_BOUNDARY.md).

Read the root E1-A package for registered schemas and migrations, immutable objects, publication and read-back, durable effects, leases, retention and GC, backup and restore, corruption handling, and generic store lifecycle.

### E2-D — ProjectStore and coherent project and graph publication

Read [`e2/README.md`](e2/README.md). The selected physical profile is:

```text
project-store-wal-manifested-partitions-v1
```

It uses one owned SQLite database per ProjectStore epoch, WAL with one writer, immutable content-addressed partition versions, complete generation membership maps, published-inactive construction, fresh read-back validation, separate current CAS, snapshot-bound readers, and explicit retention and GC.

### E6-B — external Candidate generic persistence

Read [`E6_B_EXTERNAL_CANDIDATE_STORAGE.md`](E6_B_EXTERNAL_CANDIDATE_STORAGE.md).

The store may persist registered immutable objects and append-only records for provider session references, query and dispatch receipts, bounded raw response objects, E6-A result sets and artifacts, mapping, selection, and context manifests, reconciliation, retention, and audit.

It does not call providers, own credentials or provider sessions, parse results, validate Candidate authority, map locators, choose candidates, build context, or interpret privacy and license semantics. There is no current or default external result pointer.

### E7-A — session and response-journal persistence

Read [`E7_A_SESSION_AND_RESPONSE_JOURNAL.md`](E7_A_SESSION_AND_RESPONSE_JOURNAL.md).

Store provides generic registered object, append-only state, CAS, lease, retention, and audit mechanisms for:

```text
FrontendOperationRegistry and compatibility manifests
frontend and service session generations
explicit workspace, project, and profile binding receipts
project-owned overlay references and bounded content objects
operation tickets, request and response journals, replay, and reconciliation
progress and artifact stream state when the profile retains it
close, expiry, quarantine, and recovery records
```

Store does not parse local-daemon, LSP, or MCP frames; patch documents; convert coordinates; authenticate clients; decide session semantics; or turn delivery state into semantic authority.

### E7-B — release, channel, installation, update, and support persistence

Read [`E7_B_RELEASE_STORAGE.md`](E7_B_RELEASE_STORAGE.md).

Store provides generic immutable object, append-only state, expected-current CAS, durable effect, lease, retention, backup and restore, and GC mechanisms for:

```text
release source, materialization, plan, and build records
unsigned and signed artifacts and independent-build evidence
SBOM, provenance, license, notices, checksums, and test reports
ReleaseBundle, ReleaseSupportMatrix, and ReleaseCandidate
provider publication, channel, and signed update manifests
installation, staging, backup, current, and LastKnownRunnable records
registered store and configuration migration receipts
update, rollback, revocation, retirement, incident, authorization,
and reconciliation records
```

Store does not build, compare, scan, sign, publish, install, migrate, update, roll back, revoke, retire, or decide support and release eligibility.

## Direct dependency

```text
wow-store -> wow-core
```

Domain crates and service depend on `wow-store` through narrow contracts. The reverse dependency is forbidden.

## Generic operations

Depending on the active implementation package, the store may expose narrow typed primitives for:

```text
registered schema and migration validation
content-addressed immutable object publication and read
append-only catalog, state, effect, and audit records
snapshot-bound deterministic lists
expected-current compare-and-swap
durable operation reconciliation
leases and retention edges
backup, restore, and corruption validation
reachability-based garbage collection
```

Physical database, transaction, WAL, table, row, page, object key, and filesystem-layout details remain internal.

## Hard boundaries

- no domain, provider, project, session, release, installation, or support interpretation;
- no raw SQL, connection, transaction callback, table, row ID, object key, or arbitrary filesystem root exposed to service, applications, or tools;
- no arbitrary serializer, source parser, process executor, provider client, signer, publisher, installer, or credential access;
- no source execution or editor document patching;
- no implicit current, latest, default, LKG, or LKR selection;
- no physical commit treated as semantic validation;
- no GC or cleanup from age, name, path, or previous-position inference;
- no deletion of current, active, supported, rollback, unresolved-effect, incident, or evidence state;
- no detached background cleanup outside an exact owned operation.

## Publication and read-back

Domain and service owners prepare exact logical objects and transitions. Store validates generic schema, expected-state, digest, and transaction invariants and commits them. Service or the domain owner closes the writer, reacquires a fresh exact read, and validates semantic meaning. A physical commit alone is never complete success.

## Migrations

Migrations are registered, versioned, deterministic owner and store operations with exact source and target schemas, transaction and crash behavior, backup and restore, read-back, and rollback compatibility. No raw SQL or arbitrary script crosses a public seam.

## Response loss

Every write, CAS, backup, restore, publication, or GC effect reconciles by exact `OperationId + CanonicalRequestDigest` and target identity. Same operation ID with a different digest fails. A possible commit with a lost response remains unknown; no duplicate effect or newest-state selection is permitted.

## Current implementation state

```text
planned documentation frontier: E7-B generic storage seams
implementation frontier: not-started
first implementation package: I1-A after wow-core
repository next package: I0-A / wow-core E0-A
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```

Directory and seam presence does not bypass prerequisite implementation, fixtures, checksums, crash tests, benchmarks, adapters, clients, platforms, or release evidence.
