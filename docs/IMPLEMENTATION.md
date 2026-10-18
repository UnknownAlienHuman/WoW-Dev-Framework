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
[results/interfaces](#results-and-interfaces), [donors](#sources-and-donors).

## Code ownership

Crate paths omit the leading `crates/`; `apps/`, `modules/` and `bridges/` are
workspace-relative.

| Responsibility | Existing owner |
| --- | --- |
| Lua syntax, analysis and incremental sessions | `wow-emmy`; public upstream parser/code-analysis APIs. |
| Blizzard documentation, source facts, normalization and corrections | `wow-reference/src/native*.rs` and their modules. |
| Ketho types and annotation output | `wow-annotations/src/{native,ketho,aliases,literals}.rs`; `wow-ketho-literals` is the native literal algorithm, `wow-render-contract` its shared data types. |
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

**Start #196 in `apps/wow-reference-builder/src/lib.rs`.** Promote the reusable
driver flow from `wow-annotations/examples/support/mod.rs` into the supported
command. The example is development tooling; `apps/wow/src/native.rs` currently
exports prepared bytes and is not a source-to-library generation command.

1. Reuse source-ref/TOC selection and materialization. The driver reads committed
   blobs; its current TOC support is literal `.lua` entries. Keep those limits
   visible until deliberately extended.
2. Use `wow-reference/src/native.rs::ingest_document` and the existing native
   normalization, alias and correction modules. Reuse Emmy's parser; do not
   execute donor Lua or invent another source evaluator.
3. Call `wow-annotations/src/native.rs::project_with_alias_catalogs_and_literal_bridge`
   and the existing emitter. Produce ordinary `.lua` files and a small readable
   source/partial-result report. Share this producer with local native input;
   keep needed source maps as typed data instead of reconstructing the library
   to recover them. Document the consumer's library-directory setup.
4. Remove displaced pack requests/administration in the builder and affected
   `wow-service/src/{reference_pack,reference_pack_admin,reference_admin,annotation_admin}.rs`
   callers. Native generation must not require a pack registry or signed report.

The [Ketho map](KETHO_RUST_PORT.md) owns semantic details and a runnable current
driver command. Preserve aliases, defaults, arrays, receivers, callbacks,
corrections, literals and source maps; reuse its existing consumer probes.
#202 shares a parsed catalog tree between existing validators.
#203 keeps `wow-ketho-literals` callable natively, then removes
`modules/ketho-literals`, `bridges/literal-host`, exclusive options and CI.

## Addon configuration and Emmy analysis

**#200 supplies normal configuration; #193 fixes the analysis/diagnostic path.**
`wow-service/src/local/disk_input.rs::DiskInput.library` already accepts a root
and declared Lua files through Project's `read_lua_inventory`. Build that bounded
inventory from the configured library directory. Remove user-supplied probe,
contract and digest fields; do not require native source to use generated Lua.

`local/native_input.rs::from_native_manifest` generates Lua and passes it to
`local/input.rs::assemble_with_load_plan` as Library files. Ordinary checks reuse
that Library. By contrast, `local/native_library_input.rs::rebuild_for_project`
re-ingests and re-renders retained inputs for `graph/build/native_api.rs::publish`
and `LiveProjectRead::requalify_native_library`; `verify_library_files` builds
another snapshot for identity comparison. #196/#107 reuse the produced Library
and needed callable-origin mappings, then remove that duplicate producer. Do not
make requalification the new check/load path.

`wow-emmy/src/session.rs::AnalyzerSession` owns `syntax` (Main only, diagnostics
enabled) and `semantic` (Main + Library, diagnostics disabled for physical input).
`reports().syntax` supplies Project diagnostics. Moreover,
`syntax/analyzer.rs::accepted_code` passes only syntax/doc-syntax and assignment
type errors; it drops upstream `param-type-mismatch`. #193 must register the
selected inputs once, collect Main diagnostics and semantic facts from the same
`EmmyLuaAnalysis`, and pass the useful upstream diagnostic codes through. Reuse
upstream checking, not a new type checker or diagnostic approval registry.

Trace `wow-project/src/{analyzer,snapshot}.rs`, including its separate TOC/XML
cold branch. Preserve Main/Library roles, virtual spans, cancellation and
add/change/remove behavior. An annotation-defined bad argument must reach the
ordinary addon check at its Main location; a valid call stays clean. Reuse a
small existing fixture shape, not the external-consumer campaign. Queries read
the produced facts or index without creating an analyzer.

Use upstream's public engine APIs through `wow-emmy`. Standard Lua editor
features use its existing `emmylua_ls` server: its `handlers` module is private,
not an SDK to import or copy. [EmmyLua guide](EMMYLUA_UPSTREAM.md) owns this boundary.

## Blizzard UI index, search and context

**#131 replaces the Project read boundary; #169 implements direct SQLite search;
#106 connects the user workflow.** `wow-project/src/replay/publication.rs` reads
through `AcquiredProjectPair::read` → `hydrate_owner`, rebuilding analysis and
recapturing the replay for equality. Even `wow-store/src/project/read.rs` opens
a read by loading every partition and decoding/re-encoding/hashing its JSON;
`record` reads the requested partition again. Bypassing Emmy alone is insufficient.

Start #131 with stored file paths/text and the selected source/flavor: open in a
later process and read a file without source/analyzer callbacks or a whole-index
payload validation pass. Use ordinary SQLite rows/transactions in `wow-store`;
do not serialize another snapshot framework. #169 adds name/span and FTS queries
to this same index. Existing in-memory Project updates can stay while their
readers move; updating source and reading facts have different work to do.

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

## Results and interfaces

#194 starts at `wow-core/src/{envelope,canonical}.rs` and actual Service callers:
plain typed values internally, one serialization at CLI/protocol output, bounds
outside payload identity. Delete self-hashes, size fixed-point loops and durable
read-operation journals with their callers. #195 passes existing recognizer facts
directly; #107/#118 remove graph proof/lineage machinery without rebuilding useful
source relationships.

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
