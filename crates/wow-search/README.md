# `wow-search`

`wow-search` builds bounded local search over exact Project, Blizzard UI, and
Reference data. It retrieves source files and symbols for inspection; it does not
own their semantic truth, infer a replacement, or establish lineage or runtime
impact. Search and ranking code stays in this crate, persistence stays in Store,
and Service/app provide composition and commands.

## Current route

The implemented native path builds and queries Project file metadata and selected
Reference/source observations. Public APIs are in `src/lib.rs` and the focused
`entities`, `reference`, `source_observations`, and `federation` modules. For the
existing CLI route and input shape, follow
[`SEARCH_BUILD_QUERY.md`](../../apps/wow/SEARCH_BUILD_QUERY.md) and the command
list from `wow --help` / `wow search --help`.

Reuse existing SQLite/FTS5 and Store paths. Queries need finite bounds and should
return located candidates with their basis; a text, fuzzy, or similar-name hit is
a candidate only. A miss over incomplete input is not proof of absence. Do not
reparse source in Search or silently build missing indexes during a query.

More search lanes, durable combinations, and legacy lineage-oriented behavior
remain excess pending queue reduction. They are not a required compatibility,
proof, benchmark, or corpus framework, and should not be expanded solely to
satisfy older plans.
