# Implementation guide

Read the whole [manifesto](ARCHITECTURE.md), then the relevant route below and
the selected Issue. This guide maps the existing code to the intended simpler
implementation; it does not claim that the replacements already work.
[Queue](REFACTOR_QUEUE.md): order and dependencies.
[Completion](PROJECT_COMPLETION_MATRIX.md): delivered behavior and remaining gaps.
[Execution](EXECUTION_MODEL.md): coordination and proportionate checks.

Routes: [annotations](#native-annotation-generation),
[addon analysis](#addon-configuration-and-emmy-analysis),
[Blizzard search/context](#blizzard-ui-index-search-and-context),
[source relationships](#typed-source-relationships),
[results/interfaces](#results-and-interfaces), [donors](#sources-and-donors).

## Code ownership

Crate paths omit the leading `crates/`; `apps/`, `modules/` and `bridges/` are
workspace-relative.

| Responsibility | Existing owner |
| --- | --- |
| Lua syntax, analysis and incremental sessions | `wow-emmy`; public upstream parser/code-analysis APIs. |
| Blizzard documentation, source facts, normalization and corrections | `wow-reference/src/native*.rs` and their modules. |
| Ketho types and annotation output | `wow-annotations/src/{native,ketho,aliases,literals}.rs`; `wow-ketho-literals` owns the typed literal data and native algorithm. |
| Roots, TOC, XML, loaded source and project facts | `wow-project`. |
| Useful source relationships and WoW diagnostics | `wow-graph`, `wow-recognizers`, `wow-rules`; keep facts with actual search/navigation/diagnostic consumers. |
| SQLite, transactions and queries | `wow-store`; `wow-search` supplies useful query/projection logic. |
| Located excerpts and readable context | `wow-context`. |
| Multi-owner composition and user interfaces | `wow-service`; `apps/wow` for CLI/stdio, `apps/wow-reference-builder` for the current builder. |
| Shared values | `wow-core`; only values genuinely shared by these callers. |

This is a map of existing code, not a required crate count. Cargo manifests own
actual dependencies. Keep algorithms with their owner; a single-owner command
may call it directly. Merge forwarding/copying layers rather than adding Service
wrappers. Do not promote a local helper or DTO into a framework.

## Native annotation generation

**`apps/wow-reference-builder` now owns the supported `generate` command.** It
reads an exact local Git revision/TOC and writes ordinary annotation files plus a
source report. `apps/wow/src/native.rs` remains the prepared-input path used by
local analysis, not a second generator.

1. Reuse source-ref/TOC selection and materialization. The driver reads committed
   blobs; its current TOC support is literal `.lua` entries. Keep those limits
   visible until deliberately extended.
2. Use `wow-reference/src/native.rs::ingest_document` and the existing native
   normalization, alias and correction modules. Reuse Emmy's parser; do not
   execute donor Lua or invent another source evaluator.
3. Call `wow-annotations/src/native.rs::project_with_alias_catalogs`
   and the existing emitter. Produce ordinary `.lua` files and a small readable
   source/partial-result report. Share this producer with local native input;
   keep needed source maps as typed data instead of reconstructing the library
   to recover them. Document the consumer's library-directory setup.
4. Builder and Service pack/materialization paths are removed. Generated Lua
   enters the analyzer once as ordinary Library files; source maps and callable
   origins remain typed sidecars. Native generation requires no pack registry,
   distribution manifest gate, signed report, second byte copy or compatibility
   archive.

The [Ketho map](KETHO_RUST_PORT.md) owns semantic details and a runnable current
driver command. Preserve aliases, defaults, arrays, receivers, callbacks,
corrections, literals and source maps; reuse its existing consumer probes.
Functions, structures/callback tables, enums, constants and events now use
renderer-owned declaration validation; only the final mapped aggregate render
emits bytes. Statement catalogs likewise parse once and share one Emmy
`LuaChunk` across the existing validators. Keep these owner boundaries when
extending Ketho behavior; do not reintroduce trial artifacts or a second type or
validation model.
Literal generation calls `wow-ketho-literals` directly; no guest, host, wire or runtime selection path exists.

## Addon configuration and Emmy analysis

**#200 supplies normal configuration; one shared Emmy analysis now owns diagnostics and facts.**
`wow-service/src/local/disk_input.rs::DiskInput.library` already accepts a root
and declared Lua files through Project's `read_lua_inventory`. Build that bounded
inventory from the configured library directory. Remove user-supplied probe,
contract and digest fields; existing Lua files are enough to implement #200
before #196. The generation-to-check integration belongs to #201.

`local/native_input.rs::from_native_manifest` generates Lua and passes it to
`local/input.rs::assemble_with_load_plan` as Library files. Ordinary checks reuse
that Library. By contrast, `local/native_library_input.rs::rebuild_for_project`
re-ingests and re-renders retained inputs for `graph/build/native_api.rs::publish`
and `LiveProjectRead::requalify_native_library`; `verify_library_files` builds
another snapshot for identity comparison. #196/#107 reuse the produced Library
and needed callable-origin mappings, then remove that duplicate producer. Do not
make requalification the new check/load path.

`wow-emmy/src/session.rs::AnalyzerSession` owns one
`RegisteredMemberAnalysis`: Main and Library are registered once, then the same
`EmmyLuaAnalysis` supplies Main diagnostics, member/call facts and local flow.
The TOC/XML cold branch uses the same composition without constructing separate
syntax, member and flow engines. `syntax/analyzer.rs::accepted_code` keeps the
useful current slice, including upstream `param-type-mismatch`; diagnostics are
reported only for Main files while Library declarations participate in checking.
Reuse upstream checking, not a new type checker or diagnostic approval registry.

Preserve Main/Library roles, virtual spans, cancellation and add/change/remove
behavior. Follow the [engine setup](EMMYLUA_UPSTREAM.md#engine-setup) for Lua 5.1
and standard declarations. Normal Project checks now receive annotation-defined
parameter errors at the Main location. Queries consume produced facts or the
index without creating another analyzer.

`session.rs::apply_delta` keeps unchanged syntax trees in the shared VFS, then
clears and rebuilds retained semantic indexes. `reused_files` counts unchanged
Main inputs, not saved semantic work. #109 preserves final results through the
[current update route](../crates/wow-project/UPDATE_MODEL.md). Do not replace this
rebuild with changed-only upstream calls on assumption: the pinned bulk updater
does not schedule unchanged global/type consumers.

Use upstream's public engine APIs through `wow-emmy`. Standard Lua editor
features use its existing `emmylua_ls` server: its `handlers` module is private,
not an SDK to import or copy. [EmmyLua guide](EMMYLUA_UPSTREAM.md) owns this boundary.

## Blizzard UI index, search and context

**#131 replaces the Project read boundary; #169 implements direct SQLite search;
#106 connects the user workflow.** `wow-project/src/replay/publication.rs` still
reads through `AcquiredProjectPair::read` → `hydrate_owner`, rebuilding analysis
and recapturing replay for equality. `ReadSnapshot::acquire` now validates the
selected manifest/membership without loading every partition; `record` validates
only the requested payload. Removing Project/Emmy reconstruction remains the
larger boundary.

Continue #131 with stored file paths/text and selected source/flavor: open in a
later process and read a file without source/analyzer callbacks. Use ordinary
SQLite rows/transactions in `wow-store`; do not serialize another snapshot
framework. #169 adds name/span and FTS queries to this same index. Existing
in-memory Project updates can stay while readers move; updating source and
reading facts have different work to do.

Current CLI search is `apps/wow/src/search.rs` → `wow-service/src/search.rs` →
`wow-search` / `wow-store/src/search/mod.rs`. The ProjectFile shard stores `document_id` and
`owner_metadata_label`; FTS searches these file labels/paths, **not source bodies**.
It serializes an inner SQLite image as hex/JSON into Store. Moving only those
metadata rows does not implement source-text search.

`wow-service/src/search/source_observations.rs::execute` already queries stored
native names without Project hydration. Its separate `acquire_bound` federation
path and the shard builder do hydrate Project. Reuse the facts and name lookup
in `wow-search/src/source_observations.rs`; remove unneeded shard/proof wrapping
as they move into the current index. Do not claim every existing query analyzes.

- Store current file paths/text and available symbol names/spans in the ordinary
  local index; reuse the selected Project/Emmy facts. Keep only TOC/XML links
  consumed by the requested navigation. Use existing rusqlite/FTS5 and normal
  transactions; remove the image/hex/JSON path and its exclusive manifests/holds.
- Query the tables/FTS directly with finite defaults and parameters. #106 must
  find a term occurring in a source body but absent from its filename, plus a
  real symbol. #109 covers changes/removals; #113 measures this actual workflow.
- Connect the hit to the indexed text or matching selected source and emit its
  path, range and excerpt. Never apply old offsets to changed text. The existing
  bounded excerpt reader is `wow-context/src/pack/source_excerpt.rs`, consuming
  `wow_project::graph::ProjectSourceReadReport`; reuse the reading logic.
  Current `wow search context build` explicitly excludes source excerpts: #122
  connects this missing handoff and removes semantic-pack wrapping.

Use one current database layout. Rebuild incompatible derived data from source
and configuration; switch only after success and preserve user files. If inputs
are absent, report the rebuild requirement. #111 removes historical epochs,
backup/quarantine/migration/retention administration once normal reads and
rebuilds replace it. No compatibility reader or new lifecycle subsystem.

## Typed source relationships

**Continue #195 in `wow-recognizers/src/source_state_core/{adapt,project}.rs`.**
Service's second pack compilation and graph reprojection are removed: the
recognizer-owned batch is published once and the Service suite checks that final
path. The recognizer still converts admitted typed accesses into generic facts,
fixed packs and matcher output before converting them back into graph proposals.

Map the admitted typed facts directly to the consumed relationships. Keep root
versus literal-path targets, account/character scope, source locations,
ambiguity, incomplete-input results and the existing `execute_graph_build`
check. Remove `evaluations`, pack/plan digests and generic matcher DTOs as their
last consumers disappear; they are not a required user result. Do not replace
them with another adapter or proof layer.

Trace the other live `src/source_*` adapters through their actual callers and
remove the same redundant conversions where present. #103 removes unconsumed
capabilities; a lint annotation alone
does not prove one unused. #107/#118 remove graph proof/lineage machinery.
No replacement rule compiler, plugin registry or proof-receipt framework.

## Results and interfaces

Local status/check values in `wow-service/src/service.rs` are plain typed results:
no generic envelope, request/result self-digest, schema echo or service-version
echo. The application serializes once with an external byte ceiling. #194 now
continues only through separate Context/Search/graph result families; simplify
each from its actual caller rather than introducing another universal envelope.

CLI, LSP and MCP consume the same operation result. Keep framing and position
conversion in `apps/wow`, not copies of analysis/search algorithms. #197 supplies
normal defaults; #198/#199 fix existing standard protocol behavior. Use the
current recipes in [FRONTENDS.md](FRONTENDS.md); add only useful supported stdio
operations, not a host profile or registry per feature.

## Sources and donors

| Source | Take / read |
| --- | --- |
| [EmmyLua](https://github.com/EmmyLuaLs/emmylua-analyzer-rust) | The parser, `crates/emmylua_code_analysis/src/lib.rs` session/update/library APIs and standard server. Use the compiled Cargo pin; change it only for a concrete need. |
| [Ketho](https://github.com/Ketho/vscode-wow-api) | `luasrc/annotate/init.lua` type/field/callback emission; loader/corrections and annotation catalogs through the [existing Rust map](KETHO_RUST_PORT.md). Port useful behavior and notices, not extension activation or build/runtime dependencies. |
| [Blizzard UI source](https://github.com/Gethe/wow-ui-source) | Actual API documentation, Lua, XML and TOCs at the requested revision/flavor. Reuse [source materialization](SOURCE_MATERIALIZATION.md) and [updates](SOURCE_CHECKOUT_UPDATES.md); preserve dirty/diverged checkouts. |
| [SQLite FTS5](https://www.sqlite.org/fts5.html) | Existing virtual tables, ordinary queries and updates; use the repository's rusqlite dependency, not another search engine. |

Resolve source revisions when changing source-sensitive behavior. Generic
plumbing needs no fresh WoW corpus. Optional private context stays advisory and
out of public logs, fixtures and Issues; it is not an implementation prerequisite.

## Finish the selected replacement

Trace its producer and real consumer, implement the useful path, then remove the
superseded code/options/tests/docs in the same change. Use the smallest existing
check that exercises the changed behavior; add a regression test only for an
uncovered concrete risk. Do not rebuild every old fixture matrix. Follow
[EXECUTION_MODEL.md](EXECUTION_MODEL.md) for commands and publication, and update
the affected route/Issue when the implementation changes.
