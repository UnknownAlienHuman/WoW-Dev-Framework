# E2-D security and resource budgets

**Status:** normative.

## Trust boundary

Owned schema/catalog/profile files are reviewed controls. Project/graph parameters, source-derived strings, manifests, paths, objects, and database contents are still validated as untrusted bounded data.

## Prohibited

- source/addon/Lua/XML/TOC/generated-code execution;
- hooks, build/test/release scripts, package managers, shell, process, network, editor, or WoW-client access;
- SQL/DDL/PRAGMA/table/index/column names supplied by caller/source;
- SQLite extension loading or arbitrary ATTACH;
- opening an untrusted external DB writable as owned state;
- private paths/tokens/source bodies/secret-capable runtime values in public manifests or errors;
- multi-writer/external-reader support without a new contract;
- unbounded generations, partitions, rows, joins, result bytes, WAL, backup, or GC;
- current/last-known-good substitution;
- row absence as domain negative authority.

## Root and path policy

Use one private configured root and validated root-relative epoch/database/object/backup/quarantine paths. Reject traversal, absolute, UNC, device, URI, NUL, case-collision, symlink/reparse escape. Names derive from fixed-format IDs, not source filenames.

## SQLite hardening

Pin/probe extension loading disabled, trusted-schema/defensive mode, foreign keys, attach restrictions, explicit limits, prepared statements, schema digest comparison, bounded integrity checks, and finite busy behavior.

## Input budgets

Bound schema bundles/objects/migrations/catalogs, partition versions and rows/bytes, membership rows, objects/references, validation output, generation count, inactive targets, transaction work, and report bytes. Over-budget target cannot activate.

## WAL/read budgets

Bound readers, reader lifetime classes, WAL frames/bytes, checkpoint work, writer lock wait, read rows/result bytes, and graph neighborhood/path work. No unlimited values.

## Backup/recovery/GC budgets

Bound backup size/work, recovery scan counts, GC candidates/deletes/bytes, quarantine, and epoch count. Cancellation leaves current unchanged and no background continuation.

## Attack tests

Malformed schema, oversized values, FK/reference bombs, high-fanout graph, huge membership, cardinality attacks, WAL growth under readers, corrupt WAL/SHM/main DB combinations, same-ID/different-content, source/prompt injection in parameters/errors, private leakage, and path escape.

## Secret/runtime boundary

Persist static source/tool-derived facts only. SavedVariables contents, event payloads, combat state, runtime spell secrecy, and secret-capable values are not E2-D inputs.

## Cancellation

Check before/after bounded batches, transactions, validation catalogs, checkpoints, backup chunks, and GC batches. For noninterruptible commits, observe and classify the exact outcome.
