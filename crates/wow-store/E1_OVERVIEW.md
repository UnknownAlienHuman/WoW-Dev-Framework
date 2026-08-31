# `wow-store` E1-A contract route

**Status:** E1-A implementation-ready documentation; Rust implementation has not started.

The original crate-level E1-A overview remains [`README.md`](README.md). E1-A defines the reusable SQLite/runtime, schema/migration, immutable ReferenceStore, object-store, integrity, transaction, read-snapshot, recovery, and security primitives on which E2-D depends.

Read the E1-A package in this order:

1. [`README.md`](README.md)
2. [`AGENTS.md`](AGENTS.md)
3. [`DECISIONS.md`](DECISIONS.md)
4. [`DATA_MODEL.md`](DATA_MODEL.md)
5. [`SQLITE_PROFILE.md`](SQLITE_PROFILE.md)
6. [`SCHEMA_AND_MIGRATIONS.md`](SCHEMA_AND_MIGRATIONS.md)
7. [`TRANSACTIONS_AND_PUBLICATION.md`](TRANSACTIONS_AND_PUBLICATION.md)
8. [`REFERENCE_STORE.md`](REFERENCE_STORE.md)
9. [`PROJECT_STORE.md`](PROJECT_STORE.md)
10. [`OBJECT_STORE.md`](OBJECT_STORE.md)
11. [`INTEGRITY_AND_SECURITY.md`](INTEGRITY_AND_SECURITY.md)
12. [`ERROR_MODEL.md`](ERROR_MODEL.md)
13. [`TEST_MATRIX.md`](TEST_MATRIX.md)
14. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
15. [`CONTRACT.json`](CONTRACT.json)
16. [`examples/`](examples/README.md)

E2-D adds the ProjectStore-specific physical profile, coherent project+graph generation image, activation CAS, exact reader leases, crash recovery, and retention/GC contract under [`e2/`](e2/README.md). It does not replace or weaken E1-A.
