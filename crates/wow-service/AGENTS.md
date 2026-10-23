# Use-case composition

Follow root [AGENTS.md](../../AGENTS.md), the full
[manifesto](../../docs/ARCHITECTURE.md) and the relevant
[implementation route](../../docs/IMPLEMENTATION.md).

Compose the configured check/query and return plain typed results. Parsers,
normalization, type analysis and SQL stay with their owners. Select checks before
work and keep findings separate from requested-scope incompleteness.
