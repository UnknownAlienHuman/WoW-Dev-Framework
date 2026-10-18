# `wow-store` contribution guidance

Follow repository [`AGENTS.md`](../../AGENTS.md),
[`docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md),
[`docs/REFACTOR_QUEUE.md`](../../docs/REFACTOR_QUEUE.md), and the shared
[`crates/AGENTS.md`](../AGENTS.md).

`wow-store` owns ordinary local SQLite persistence for the project index and
useful search data. It owns schema setup, queries, cache records, and
transactions that prevent partially written derived state. Keep application
rows and migration details behind its API; do not expose raw connections or
arbitrary SQL to callers. Store is not a generic durable-effect, lease,
operation-journal, content-addressed publication, backup, quarantine, or
recovery platform.

Retain only the current store representation. Project databases and generated
indexes are derived from user source/configuration, so it is acceptable to
rebuild them. Remove old schema variants and elaborate retained-store migration
paths when current user inputs are sufficient to reconstruct the index. Never
delete or rewrite addon source or user configuration as part of a rebuild.

Use parameterized queries, transactions for multi-step writes, and SQLite
constraints where they make normal data invariants clear. Bound query results
and indexing work; keep cancellation responsive on large inputs. Treat corrupt
or unsupported derived data as a clear rebuild/error condition rather than
silently returning fabricated records.

Follow pinned SQLite behavior where an implementation choice depends on it.
Use focused tests for transaction rollback, query behavior, or rebuild behavior
when those paths change; broad durability/crash/platform campaigns are not
routine prerequisites.
