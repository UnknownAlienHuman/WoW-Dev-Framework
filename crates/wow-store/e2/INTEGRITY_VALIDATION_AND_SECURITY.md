# ProjectStore integrity, validation, and security

**Status:** normative.

## Validation layers

### Request and bundle

- exact IDs/versions/compatibility;
- canonical plan and acyclic prerequisites;
- payload schemas and budgets;
- expected base/head;
- no unregistered operation/read/validation IDs.

### Transaction

- statement/effect success;
- schema/foreign/reference closure;
- expected logical partition replacement;
- row/count/digest manifests;
- object-reference closure;
- domain validation reports.

### Seal

- checkpoint/close complete;
- mandatory members present;
- no mutable staging/WAL dependency;
- safe owned final location;
- exact bytes/checksums;
- noncyclic manifests.

### Open

- read-only exact artifact;
- SQLite/profile/application/schema versions;
- integrity checks;
- registered golden reads;
- generation/manifests match request.

### Registry/head

- valid head schema/payload digest;
- referenced generation sealed/open-validated;
- expected prior head;
- one atomic record;
- no mixed or dangling generation IDs.

## Threat model

Reject or contain:

- malicious/corrupt database and manifest;
- path traversal, symlink, junction, device, URI, case collision;
- dynamic SQL/table/PRAGMA/extension/UDF injection;
- oversized payload/row/object/WAL/database/validation report;
- content-address collision or object substitution;
- stale-base and concurrent-writer races;
- crafted registry/head/lease records;
- crash between every durable operation;
- private path/token/source leakage;
- source/repository instructions attempting to change store behavior.

## Input limits

Bound:

```text
schemas/catalogs/operations/invocations
payload and record sizes
rows/partitions/index entries
database/WAL/object/member bytes
statements and transaction duration
validation checks/output
open handles/readers/leases
recovery inventory
retention roots/GC candidates
```

Limit failure is typed. No truncated generation can publish as complete.

## SQLite restrictions

- no runtime extension loading;
- no caller attach path;
- no caller DDL/DML/PRAGMA text;
- no untrusted collations/functions;
- no writable open of sealed generation;
- exact compile/runtime option audit;
- prepared registered statements only;
- defensive/trusted-schema/profile options frozen where supported;
- integrity checks do not replace domain validation.

## Privacy

Canonical/public records exclude:

```text
absolute root and temp path
username/home/drive/UNC details
credentials/tokens/private URLs
raw source content
arbitrary SQL or database pages
process/thread IDs
runtime Secret-capable values
```

Use logical IDs, relative owned member IDs, digests, bounded structured errors, and protected debug handles.

## Independent validation

Validation must be able to fail independently of the writer. Golden read results and manifest reconciliation are recomputed from the sealed artifact. A writer-produced “success” flag is insufficient.

## Security mutation tests

- operation payload tries SQL/DDL/PRAGMA/table injection;
- artifact is symlink/reparse/device/foreign DB;
- checksum matches manifest but logical rows differ;
- object digest path contains wrong bytes;
- head references unsealed or another profile generation;
- lease registry is missing/ambiguous;
- GC plan races with a new lease/head;
- source comments contain agent/tool commands;
- oversized/cyclic manifests and reference bombs;
- cancellation/fault at every phase.
