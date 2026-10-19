# `wow-store`

`wow-store` is the local persistence owner. It is an active crate over SQLite;
`src/store.rs` implements a generic catalog/object/effect store, while
`src/project/` contains a much larger manifested ProjectStore with WAL,
retention, GC, backup, quarantine, replacement, and migration machinery. The
target is a straightforward local SQLite index for reader-ready project/search
facts, with parameterized queries and transactions that prevent partial writes.
It is not a general durable-effect or recovery platform.

## Where to look

- Public exports and the project module: [`src/lib.rs`](src/lib.rs)
- Generic database operations: `src/store.rs`
- Project database owner: `src/project/`
- Search persistence/query code: [`src/search/`](src/search/)
- Existing focused coverage: [`tests/`](tests/) and module tests

Retain ordinary SQLite schema setup, query bounds, and transactional updates.
Store plain records and FTS rows directly; keep SQL and connections private.
Project indexes are derived from source and configuration, so one current format
can be rebuilt. Snapshot open validates manifest membership; requested records
validate their own payload on read instead of scanning every partition. On
unsupported/corrupt derived state, give a clear rebuild/error path; never delete
or rewrite user source/configuration.

The physical epoch-migration/selection platform has been removed. Older
backup, quarantine, recovery and generic retention machinery remains for the
bounded reductions in [#111](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/111)
after the current index path in [#131](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/131).
Search's direct SQLite/FTS integration is tracked by
[#169](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169).
These are queued reductions, not completed runtime behavior.

Use the root [`AGENTS.md`](../../AGENTS.md), [architecture](../../docs/ARCHITECTURE.md),
and [reduction queue](../../docs/REFACTOR_QUEUE.md) with the actual consumer and
schema touched by a change. Follow pinned SQLite behavior when it affects a
concrete implementation choice; broad crash/platform acceptance campaigns are
not routine requirements.
