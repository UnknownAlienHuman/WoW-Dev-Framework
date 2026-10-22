# Implementation guide

Read the complete [manifesto](ARCHITECTURE.md), then the selected route and
Issue. [AUDIT.md](AUDIT.md) explains why the replacement is needed;
[REFACTOR_QUEUE.md](REFACTOR_QUEUE.md) orders it. This document states the target
owners and current handoffs. It does not claim that a route already works.

<a id="code-ownership"></a>

## Target ownership

| Responsibility | Target owner |
| --- | --- |
| Blizzard source/ref/flavor/TOC acquisition | `apps/wow-reference-builder` with small source helpers |
| Normalized Blizzard API model, aliases and corrections | `wow-reference` |
| Ketho-style annotation files and source mappings | `wow-annotations` + `wow-ketho-literals` |
| Lua 5.1 Main+Library analysis, diagnostics and updates | `wow-emmy` using upstream public APIs |
| Addon root, TOC/XML/load selection and current in-memory project | `wow-project` |
| Current persisted files/text/symbols/relations and SQL queries | `wow-store` |
| Text/symbol query projection | a reduced `wow-search`, or `wow-store` if no separate algorithm remains |
| Readable located source excerpt | a reduced `wow-context`, or a local query helper |
| Useful WoW relationships/checks | direct typed owners; no generic pack/proof platform |
| User operations and protocol DTOs | thin `wow-service`; CLI/LSP/MCP in `apps/wow` |
| Shared primitives | `wow-core` only when multiple retained owners genuinely share them |

The current crate count is not a requirement. Remove or merge boundaries after
callers move; do not create replacement frameworks.

<a id="native-annotation-generation"></a>

## Route 1 — generate a Library

### Inputs

- explicit Blizzard checkout and commit/ref;
- selected flavor;
- selected or unambiguous TOC;
- output directory, created new/no-clobber;
- finite internal limits and cancellation.

### Existing substance to reuse

- `apps/wow-reference-builder/src/{lib,io,catalogs}.rs`;
- `wow-reference/src/native*.rs`, aliases and corrections;
- `wow-annotations/src/{native,ketho,aliases,literals}.rs` and source maps;
- `wow-ketho-literals` native algorithm;
- Ketho emitter behavior described in [KETHO_RUST_PORT.md](KETHO_RUST_PORT.md).

### Replacement

Read exact regular-file tree entries using a bounded batch/tree Git operation.
Thread stop state through child I/O and reap the child. Parse/normalize/project
once and write ordinary `.lua` files. The report contains selected source/ref/
flavor, admitted/failed inputs, limitations and mappings consumed by later
navigation—nothing else.

The real Library reader must accept every successful output within the same
published limits. Application-level checks invoke this producer. Delete the old
example producer, duplicated Git/catalog helpers, stale `/1` verifier,
compatibility/certification reports and unconsumed artifact identities.

### Observable check

A small committed fixture runs through the public builder, loads through the real
Library reader and preserves representative Ketho output/maps. Then one selected
real source/flavor establishes the claimed coverage only.

<a id="addon-configuration-and-emmy-analysis"></a>

## Route 2 — check an addon

### Inputs

- addon root;
- selected or auto-discovered TOC, with ambiguity reported;
- flavor/source selection when WoW-specific relationships require it;
- generated or existing Library directory;
- output format and optional selected checks.

No user-supplied Project IDs, generations, probe reports, contract IDs, digests,
proof manifests or budget JSON are required for ordinary use.

### Existing substance to reuse

- `wow-project` root/TOC/XML/load readers;
- `wow-emmy/src/session.rs::AnalyzerSession` and registered Main+Library analysis;
- upstream Lua 5.1 configuration and `init_std_lib(None)`;
- current Project update facts and exact coordinates;
- small useful WoW checks after they are selected before execution.

### Replacement

Create one configured analysis owner. Load standard declarations once, then
Library and Main. Preserve upstream diagnostic message, code, severity and range
for Main. Apply only explicit documented WoW suppressions. Findings and
completeness are separate fields; an unrelated optional component cannot hide a
finding or make the requested analysis partial.

Execute only requested checks. Convert typed facts directly to consumed
relationships; remove generic recognizer packs and graph requalification as their
callers move. TOC/XML/load changes reload or rebuild the same owner. LSP/MCP call
this operation rather than standalone one-file analysis.

### Observable check

The actual CLI loads a typed Library function: a valid call is clean; an invalid
argument reports the upstream message/severity at the addon line. Also exercise
one missing/ambiguous input and one changed/deleted or TOC-selection update.

## Route 3 — index, search and read source

### Inputs

- selected Blizzard/addon source root, ref and flavor;
- current database path;
- text/symbol query and finite result limit.

### Existing substance to reuse

- `wow-project` source/TOC/XML/load selection and available Emmy facts;
- rusqlite/FTS5;
- useful native-name/source observations;
- the bounded source-excerpt reading logic in
  `wow-context/src/pack/source_excerpt.rs`.

### Replacement

Use one current SQLite database. Store source identity, file path/text/content
version, symbol name/kind/span and only useful TOC/XML/source relations in normal
tables. FTS indexes source bodies with documented token/phrase semantics. Query
after reopening without Project/Emmy/Graph reconstruction. Updates replace
changed rows and delete removed rows transactionally.

Return bounded plain hits. Read the matching indexed text or exactly matching
selected source and render path, range and fenced source. Never apply old offsets
to changed text.

Remove Project replay recapture, nested SQLite/hex/JSON, generic Store lifecycle,
retained-result/federation/self-sizing cursors and Context pack/map/skeleton
machinery as their consumers switch.

### Observable check

After process restart, find a term absent from filenames but present in a file
body, a known symbol and one useful TOC/XML relation. Return readable matching
source. Change/delete a file and verify stale hits disappear.

<a id="typed-source-relationships"></a>

## Direct relationships and checks

Keep only relations consumed by diagnostics, search or navigation: source
call/target, TOC/XML load/template/script, callable origin and selected state or
library relationships. Preserve location, ambiguity and incomplete coverage.

Do not convert fixed Rust facts into JSON packs, compile them, execute a generic
matcher and map them back. Do not retain graph proposals, validation proofs,
partitions or registries merely because old tests assert their IDs. A small typed
relation table/module is sufficient unless a real user query demonstrates more.

## Results and frontends

Each use case returns a plain typed result with data the caller presents:
findings, completeness, source identity, paths/ranges, hits/excerpts and typed
errors. Serialize once at the frontend with an external output ceiling. Remove
self-hashes, byte fixed points, request journals and replay registries from
ordinary reads.

CLI is the complete local path with useful defaults. LSP/MCP are thin adapters to
the same check/search/context operations. General Lua editor features remain the
upstream EmmyLua server. Follow [FRONTENDS.md](FRONTENDS.md) for protocol details.

## Code that must disappear with its replacement

- annotation and Emmy compatibility/certification reports and probe binaries;
- duplicate annotation producer/verifier and producer-only fixtures;
- evidence-heavy local input variants and user-facing IDs/digests/contracts;
- Project replay hydration/recapture and companion proof reconstruction;
- generic Store object/catalog, nested Search database and backup/retention/
  quarantine/recovery/replacement administration;
- generic recognizer pack/plan/matcher and proof graph paths with no remaining
  consumer;
- Context external pack/map/skeleton/universe/continuation platform;
- Search federation, retained-result lifecycle and self-sizing pagination not
  required by direct source queries;
- internal CLI commands, profiles, schemas, tests and documents owned only by
  those paths;
- old E1–E7/platform specifications after useful facts move to retained owners.

## Donors

| Donor | Reuse |
| --- | --- |
| [EmmyLua](https://github.com/EmmyLuaLs/emmylua-analyzer-rust) | public analysis/session/update/library APIs and standard server/declarations |
| [Ketho](https://github.com/Ketho/vscode-wow-api) | annotation type/default/array/callback/receiver behavior, corrections and notices—not VS Code activation/runtime |
| [Blizzard UI source](https://github.com/Gethe/wow-ui-source) | selected Lua/XML/TOC/API documentation at an explicit revision/flavor |
| [SQLite FTS5](https://www.sqlite.org/fts5.html) | ordinary current tables, transactions and documented token/phrase queries |

Read the exact donor module when its semantics change. Do not infer unsupported
WoW facts or promise parity not exercised by the selected workflow.

## Working rule

For every change trace:

```text
user input -> actual producer -> handoff -> actual reader -> visible result
```

Finish that path and remove its displaced code/options/tests/docs in the same
change. Use the smallest existing consumer check after the path works. Build and
Clippy are required code gates, not behavioral acceptance.
