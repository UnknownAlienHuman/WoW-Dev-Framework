# `wow-project` contribution guidance

Follow repository [`AGENTS.md`](../../AGENTS.md),
[`docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md),
[`docs/REFACTOR_QUEUE.md`](../../docs/REFACTOR_QUEUE.md), and the shared
[`crates/AGENTS.md`](../AGENTS.md).

`wow-project` turns an explicit configured addon root into an index useful for
Blizzard UI development. It owns source inventory, flavor/config selection,
TOC discovery and load order, XML/Lua project facts, and project-level query
projections. Use existing EmmyLua and annotation owners for parsing and
normalization rather than creating parallel parsers or second semantic paths.

Configuration is the user's source of truth. Keep it readable and stable;
resolve flavor, TOC, and root ambiguity explicitly instead of guessing from
machine state, Git metadata, or addon names. A simple config may identify root,
flavor, and TOC; discovery may offer bounded candidates for the user to choose.

Never execute Lua, addon scripts, repository hooks, package managers, or
arbitrary generators. Keep file access rooted, reject traversal, and bound file
count, bytes, nesting, facts, and output. Preserve exact byte offsets/spans so
hover, definition, diagnostics, and references point to the source users see.

Project index data and annotations are derived. Rebuild using one current format
from retained source/configuration rather than carrying multiple old snapshot
formats or complex migration machinery. Do not modify addon files or config
while indexing. Report parse/read failures accurately and avoid claiming facts
that were not analyzed.

Inspect the direct app/service callers before changing a projection. A
single-owner project operation may be called directly by an application when
that yields the simplest usable path; cross-owner workflows can use Service.
Use focused existing fixtures for relevant TOC/XML/Lua and location behavior.
