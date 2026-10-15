# Native exact-generation search

`wow search build-query` builds and queries one immutable ProjectFile metadata
shard through the Search, Store and Service owners. Use `wow search --help` for
the closed grammar and required finite bounds. The command requires an explicit
absolute store location, exact retained generation, ProjectFile ID, literal query
and `--allow-local-metadata-only`; it accepts no `Current` selector.

The selected document contains the exact owner file ID and its relative-path
label. Exact ID and label lookups preserve that limited authority. Literal FTS
results are explicitly Candidate evidence, and zero text hits prove no absence.
Source bodies, comments, snippets, inferred aliases and license conclusions are
outside this projection.

The Service validates the physical SQLite/FTS5 shard and reopens it read-only.
It releases the shard lease and checks the retained read/store closure before
returning canonical `wow-service/search-result-envelope/1` bytes. The CLI adds
one LF. Success is exit 0, cancellation 130, request refusal 64 and output failure
4; other owner failures retain their mapped error code.

This first route uses an in-memory generic search Store and a process-local
operation registry. It returns no cursor and does not establish cross-process
durability, complete corpus/field coverage, ranking, authoritative misses, lineage
or full E4-A/E4-C acceptance. Those obligations remain in
[#169](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169).
