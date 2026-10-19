# Current Project publication

Project publication persists a rebuildable local Project/graph snapshot for the
current tool. It is not a compatibility archive or a database migration product.

## Owners and flow

1. `ProjectPublisher` owns validated Main, Library, TOC/XML and selected Blizzard
   source inputs.
2. `ProjectReplay::capture` retains only the inputs needed by the current native
   reconstruction path.
3. `ProjectPublicationBundle` combines the replay, graph records and bindings.
4. `wow-store::project::ProjectStore` commits the generated records in one SQLite
   publication transaction.
5. `LiveProjectStore` selects the current catalog for the requested ordinary or
   named component route and exposes Current/Exact reads.

`ReadSnapshot` validates the selected manifest and membership when it opens, then
validates each requested record on read. `AcquiredProjectPair::read` still rebuilds
Project/Emmy state from the retained replay before returning a `ProjectView`; #131
replaces that reconstruction with reader-ready indexed facts.

## Required behavior

- Main and Library remain separate inputs to one Emmy analysis.
- TOC/XML selection, source flavor and source locations survive publication.
- Project and graph records refer to one selected generation.
- Writes are transactional; a failed publication does not replace Current.
- Reads remain bounded and cancellable.
- Source and user configuration are never deleted during cache rebuild.

## Format policy

Historical exact V1–V16 store catalogs are not registered or reopened. An old or
incompatible derived database must report a rebuild requirement and be recreated
from current source/configuration.

Current named component catalogs and the ordinary current catalog remain while
their real callers exist. The replay record still contains several current
recipe shapes selected by configuration; #112 removes those variants as their
callers are consolidated. Do not introduce `v17`, a converter, an archive reader,
a disabled compatibility mode or another catalog registry.

## Where to change code

- Capture/hydration: `src/replay/mod.rs` and its small owner modules.
- Store records/catalogs: `src/replay/publication.rs`.
- Service publication/read: `wow-service/src/live_project.rs` and operations.
- Direct reader replacement: #131.
- Legacy format deletion: #112.

The architecture and implementation documents define product scope. Git history
is the record of removed formats; runtime code is not.
