# `wow-project`

`wow-project` is the source/project owner for configured addon and Blizzard UI
roots. Its current implementation includes source inventories, project
snapshots and updates, disk acquisition, TOC/XML replay, graph proposals,
overlays, and a large generation/publication model. The target is a practical
root/flavor/TOC configuration, bounded non-executing source loading, accurate
Lua/XML locations, and reader-ready facts for Emmy analysis and useful search.

## Where to look

- Public types and modules: [`src/lib.rs`](src/lib.rs)
- Config and generation inputs: `src/configuration.rs`, `src/generation.rs`
- Disk and local source acquisition: [`src/disk/`](src/disk/)
- Source inventory and project view: `src/inventory.rs`, `src/snapshot.rs`,
  `src/registry.rs`
- TOC/XML and source graph behavior: `src/load/`, `src/xml_bindings.rs`,
  `src/xml_lua.rs`, `src/graph.rs`
- Existing executable behavior: `src/update.rs`, `src/publication.rs`,
  `src/platform_source/`, and tests under [`tests/`](tests/)

Current code still has more publication, update, overlay, and exact-generation
machinery than the target requires. The intended setup/index workflow is tracked
by [#200](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/200),
useful TOC/XML relationships by [#117](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/117),
and reader-ready current index by
[#131](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/131).
These tasks describe pending replacement work; existing source files are not
proof that the simplified workflow is complete.

Use upstream EmmyLua through `wow-emmy`; do not add another Lua parser. Never
execute addon or repository code. Keep file access within configured roots and
bound file counts, bytes, parse work, and output. Preserve exact source byte
spans for diagnostics and navigation. User source and config stay untouched;
project snapshots and indexes can be regenerated. Report ambiguity and
unsupported/failed parses honestly.

Read root [`AGENTS.md`](../../AGENTS.md), [architecture](../../docs/ARCHITECTURE.md),
and [reduction queue](../../docs/REFACTOR_QUEUE.md), then inspect the concrete
owner and consumer path being changed.
