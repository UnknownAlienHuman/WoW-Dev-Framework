# Native exact-generation search

## Durable selected shard

`index build` and `index retire` also accept an explicit `--file-ids` JSON array
instead of `--file-id`. The two options are mutually exclusive. The set is finite,
nonempty, duplicate-free and canonically ordered; index build needs no query or
representative document. Its response retains all selected exact IDs/relative
labels, the `project_file_set_v1` component and the original source-hold receipt.
`query --file-set` explicitly selects that component and keeps `--file-id` as an
actual query member. An omitted/wrong component refuses rather than trying hashes.
Retirement requires the complete original set and exact hold receipt; another
valid selection/hold cannot retire that stored shard. The same native Store v2,
operation replay, source-generation holds, catalog/lease closure and GC apply.

`wow search index build` prepares one exact ProjectFile or explicit finite-set metadata shard from an
explicit retained ProjectStore generation and publishes it into a separate
explicit SearchStore file. Publication atomically commits the immutable shard,
its direct catalog root, response object and completed operation journal entry.

The selected `wow-search.durable-shard-store.v2` profile uses a Store-owned
physical instance identity. Each retained shard also holds its exact source
generation in ProjectStore; two SearchStore instances keep independent holds.
Copied SearchStore files and older identity-free profiles are refused. Retirement
validates the original source epoch and hold before releasing it.

`wow search query` opens the exact returned shard/object/profile/generation/file
handle in a later process. It validates the retained owner manifest and physical
document before exact ID/label and literal FTS queries; it does not rebuild the
shard or resolve Current. `candidate_only` remains Candidate evidence and uses
CLI exit 2. The metadata permission and every finite bound remain explicit.

`wow search index retire` validates the same handle and atomically removes its
catalog root with the retirement response/journal completion. `wow search index gc`
runs bounded explicit Store GC and reports whether the selected target object was
deleted. The native deletion and GC receipt commit together; an interrupted
outer operation adopts that receipt on retry without collecting again.
A released query lease must not keep that retired object alive until its
expiry. Build, retirement and GC completion use exit 0; owner refusals retain
their error mapping. Store initialization may create the explicitly named new
SearchStore before its operation journal exists; no shard is published by that
initialization.

Each operation binds an exact ID/request digest. Completed retries return stored
response bytes; a conflicting request fails, and an interrupted Prepared outcome
requires reconciliation. These are immutable metadata observations, not current
source availability or a promise that a retired handle can still be queried.
Physical process reopen is separate from power-loss/platform acceptance.

Use `wow search --help` for the closed build/query/retire/GC grammar. Source bodies,
snippets, inferred aliases, complete ranking/miss/continuation, lineage and full
E4-A/E4-C acceptance remain outside this selected route in
[#169](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169).

## One-shot build/query

The optional `--file-ids '["project-file:a.lua","project-file:b.lua"]'` selects
one explicit finite file set for `search build-query`. `--file-id` remains the
exact query root and must belong to that set. The service rejects empty,
duplicate, over-capacity and absent IDs; selection never discovers files.
`--max-documents` caps the raw selection before ordered-set work. All documents
share one exact Project/Graph acquisition and one continuing manifest-byte bound.

This selection uses the named file-set profile/manifest/reader and
`wow-service/search-file-set-result-envelope/1`. Its ordered `documents` retain
only exact IDs/relative labels; literal FTS can return candidates from any member.
No FTS result or miss gains exact or negative authority. Omitting `--file-ids`
preserves the original single-file request/result identities. This operation
remains one-shot; use the separate durable commands for retained file-set handles.
Full E4/fault/platform qualification remains open in #169.

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
one LF. Candidate-only output uses exit 2, cancellation 130, request refusal 64 and output failure
4; other owner failures retain their mapped error code.

This first route uses an in-memory generic search Store and a process-local
operation registry. It returns no cursor and does not establish cross-process
durability, complete corpus/field coverage, ranking, authoritative misses, lineage
or full E4-A/E4-C acceptance. Those obligations remain in
[#169](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169).
