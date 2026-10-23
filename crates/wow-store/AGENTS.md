# Current source index

Follow root [AGENTS.md](../../AGENTS.md), the full
[manifesto](../../docs/ARCHITECTURE.md) and the relevant
[implementation route](../../docs/IMPLEMENTATION.md).

Keep direct SQLite tables/queries and transactional updates. The index is
derived data; never use cached paths to delete user source. Remove nested
databases and generic backup/retention/recovery services through their callers.
