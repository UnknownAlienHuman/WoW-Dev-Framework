# `wow-graph`

`wow-graph` stores and queries useful source relationships for addon and Blizzard
UI development. Its current core is generation-local: typed entities and relations,
validated proposals, producer partitions, immutable snapshots, and bounded direct
queries. It does not parse source or discover current projects; Project, Reference,
Search, and source producers supply facts through their existing owners.

## Read and change

Start with the selected task, then inspect the relevant implementation under
[`src/`](src/lib.rs) and its direct caller. Public query entry points are backed by
`direct`, `axes`, `paths`, `subgraph`, and `explain`; proposal and partition
handling lives in `proposal`, `assertion_validation`, and `partition_session`.
Persistent graph storage is an explicit `wow-store` boundary. Keep graph logic out
of the CLI and service layers.

For source-to-graph behavior, the executable path is [`wow graph build`](../../apps/wow/GRAPH_BUILD.md).
It builds bounded graph data through the same Project materialization route used
by `wow check`. Query a saved snapshot with `wow graph entity`, `neighbors`,
`subgraph`, `axis`, `explain`, or `path`; the current command forms are listed by
`wow --help` and `wow graph --help`.

## Boundaries

Keep only relationships consumed by search, navigation, or actionable diagnostics.
Exact graph identity and source locations come from their owners. A static edge,
match, or path does not prove runtime execution, safety, or API authority. Query
and traversal work must remain bounded and must preserve incomplete or ambiguous
input as such.

Cross-generation lineage, migration, and impact code remains excess pending its
reduction in the refactoring queue; it is not a required framework or acceptance
system. Do not expand it to satisfy old contracts. The graph crate is active and
useful today, but its remaining layers are not thereby declared complete or
already removed.
