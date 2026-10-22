# WoW Dev Framework — source and architecture audit

**Baseline:** `3c3d5bc526105259ac80379dd475b5565993f5c0` (`main`, 2026-10-21).
**Purpose:** sufficient source-level basis to rewrite the documentation and the
product around the three workflows in [ARCHITECTURE.md](ARCHITECTURE.md).
**Status:** the architecture audit is complete at package, entrypoint and active
workflow level. It is not a release qualification, benchmark or claim that every
individual function has been semantically proved by execution.

This document replaces `AUDIT_2026-10-17.md` and
`DOCUMENTATION_REWRITE_BASIS.md`. It records decisions and evidence for the
rewrite; it is not mandatory context for every small implementation task.

## 1. What was checked

The review covered:

- all 16 workspace packages, their manifests, dependency direction, public
  entrypoints and largest implementation clusters;
- the CLI command surface, builder, stdio LSP/MCP, xtask and CI workflows;
- the complete producer → reader → user-result path for annotation generation,
  addon checking, Project persistence, Search and Context;
- all 847 Rust files by automated inventory/pattern analysis, with direct source
  review of the owners and consumers named below;
- all 329 Markdown files by routing, link, scope-term and reachability analysis,
  with direct review of active root/owner instructions and the old E1–E7 trees;
- the original Gemini audit at `e6961862`, subsequent refactors through #236,
  current Issues and our own regressions;
- pinned Ketho annotation behavior and the public EmmyLua analysis/session APIs.

The reviewed tree contains about **361,000 Rust lines**, including tests,
examples and support code, plus about **49,000 Markdown lines**. `wow-service`
alone contains about 94,500 Rust lines. These mixed counts are scale indicators,
not deletion targets or measured runtime costs.

The structural inventory also found 2,178 Rust structs with fields; a field named
`schema` occurs in 309 of them. Search has four parallel pagination policies and
four matching Service continuation/result wrappers. Context contains hundreds of
`*Wire` references mirroring Project/Graph records. Recognizer adapters repeat
the same proposal/receipt/input shapes across families. These counts include
private and test code, but the duplicated shapes occur in production owners and
support the platform-duplication diagnosis below.

Not run as part of this audit: the full Rust test suite, a real addon workflow,
a complete Blizzard/Ketho corpus, Windows editor sessions or a performance
benchmark. Each is implementation acceptance for the corresponding claim, not a
prerequisite for deciding the replacement architecture.

## 2. Executive judgment

The repository is not one oversized implementation of the desired product. It
is several general platforms layered around fragments of the desired product:

1. a useful EmmyLua analysis owner, plus compatibility/probe and private frontend
   paths that bypass or discard its result;
2. a useful native Ketho projection, plus artifact, certification, identity and
   duplicate producer paths;
3. a Project model that can parse roots/TOC/XML, plus replay-based persistence
   that reconstructs the complete analysis to read saved data;
4. three storage designs, including a SQLite database encoded inside JSON inside
   another SQLite database;
5. typed source facts converted into generic matcher packs, then into a generic
   proof graph, then wrapped again for Service and Context;
6. a large CLI exposing internal profiles, IDs, receipts, snapshots, budgets and
   lifecycle commands instead of the three user workflows;
7. thousands of lines of historical contracts and acceptance choreography that
   continue to compete with the current manifesto.

The central failure is **handoff design**. Work was repeatedly declared complete
when one crate, DTO or generator worked, while the actual producer, actual
reader and user-visible result still disagreed. Refactoring repeated the same
mistake by deleting visible legacy names while preserving the platform beneath
them, and by reporting line reduction or green Clippy as product progress.

The rewrite must be organized around exactly three end-to-end operations:

1. **Generate Library:** Blizzard source/ref/flavor/TOC → ordinary `.lua`
   annotations and source mappings.
2. **Check addon:** root/TOC + Library → real EmmyLua diagnostics at addon
   locations, with useful messages and honest incompleteness.
3. **Index/query source:** selected Blizzard/addon source → persisted text,
   symbols and useful TOC/XML relations → later-process search and readable
   excerpts.

Everything else must either directly support one of these operations or be
removed.

## 3. Five causes that created the current system

### 3.1 Representation completion instead of workflow completion

A schema, pack, graph, report, builder or frontend was accepted independently.
No owner was forced to prove that the next real consumer could use its output.
Examples: builder report `/2` feeding a verifier that only accepts `/1`; tests
using the old example producer; one shared Project analyzer while LSP still uses
isolated one-file analysis.

### 3.2 Meaning lost behind identity and proof metadata

Adapters retained digests, categories, capabilities, receipts and provenance but
removed the information a developer needs. Emmy messages are dropped, severity
is forced to Error and LSP emits a generic sentence. Context escapes source
punctuation into `\uXXXX`. Useful result data must be primary; identity metadata
is kept only where a concrete caller needs it.

### 3.3 Persistence as recomputation and certification

Saved Project data is treated as a recipe to recreate Emmy/Graph owners and
compare a full recapture. Search serializes an inner SQLite image, hex-encodes it
inside JSON and stores it in another SQLite store. A local index should instead
store directly queryable current rows and return them without analysis.

### 3.4 Fixed product behavior routed through generic platforms

Known Rust relationships are converted into generic facts/packs/plans/matches;
source relations become graph proposals, validation reports and partitions;
two hardcoded checks use a general rules engine. This creates more invalid states
and duplicated tests than the product behavior itself.

### 3.5 Competing sources of truth

Architecture, Issues, package specifications, tests and status matrices describe
different revisions. Old comments still direct agents to merged branches and
completed tasks. Mechanical string replacement was mistaken for reconciliation.
The rewrite needs one manifesto, one implementation guide, one queue, one audit
and small owner notes only where code needs them.

## 4. Workflow audit

## 4.1 Generate a native annotation Library

### Current chain

`wow-reference-builder generate` reads Git/TOC input, normalizes through
`wow-reference`, projects through `wow-annotations`, and writes `.lua` files plus
`source-report.json`.

Useful implementation already exists:

- native Blizzard document ingestion and normalization;
- aliases, corrections, selected catalogs and typed literals;
- one statement parse shared by catalog validators;
- one final renderer emission rather than trial renders;
- Ketho semantics for defaults, arrays/stride, receivers, ordered parameters and
  returns, callbacks and source mappings;
- direct native literal generation; the Wasm host/guest route is removed.

### Confirmed defects and duplication

1. **Broken public handoff.** The builder emits
   `wow-native-source-build/2`; `cargo xtask verify-library` accepts `/1` and
   expects the removed embedded `library` object. The current source-bundle
   workflow connects these incompatible shapes.
2. **Tests invoke another producer.** Annotation consumer tests include
   `examples/support/mod.rs`, not the supported builder. Its Git and catalog
   helpers duplicate the builder helpers byte-for-byte.
3. **Producer and reader limits disagree.** Builder output permits 4096 files,
   8 MiB per file and 64 MiB total; ordinary Library reading permits 1024,
   1 MiB and 16 MiB. Both are bounded, but the handoff is not guaranteed.
4. **One Git child per input file.** The builder repeatedly calls
   `git cat-file blob`; a selected tree/batch read is simpler and avoids thousands
   of child processes on a large TOC.
5. **TOC/API mode is not validated.** The alias reader already checks an exact
   regular-file tree entry; the TOC/API helper reads any blob, including a
   symlink entry's target text.
6. **Cancellation cannot interrupt Git I/O.** The helper blocks in
   `read_to_end`/`wait` without the operation stop state.
7. **Report is larger than the real consumer contract.** It publishes extensive
   identity, resolution, correction and sidecar inventories although ordinary
   Library loading consumes files. Keep only selected source/ref/flavor,
   admitted/failed inputs, limitations and mappings actually needed later.
8. **Compatibility/certification remains active.** `wow-annotations` still has a
   roughly thousand-line compatibility/certification platform; Service accepts
   external reports. This is explicitly outside the product.
9. **Reference has parallel products.** Native input/view is useful; generated
   API, topology, persistent reference publication and standalone binaries need
   a real workflow consumer or removal.

### Required replacement

One builder owns source selection and output. It validates exact regular files,
uses bounded batch/tree reads with cooperative child cleanup, calls the existing
native Reference and annotation owners, writes ordinary `.lua` files and a small
report. The same output is loaded directly by the addon checker. Application
checks invoke this producer; emitter/type/source-map checks remain with their
owners. Remove the old producer, stale verifier, certification reports and
unconsumed artifact identities.

## 4.2 Check an addon with EmmyLua

### Current chain

`wow status/check --config ...` reads a materialized Service input containing
Project, Reference, analyzer and native evidence. Project can now register Main
and Library once in `AnalyzerSession`; that shared Emmy analysis supplies
Project diagnostics, member/call facts and flow.

### Confirmed defects and duplication

1. **No normal configuration.** The CLI requires a config file plus internal
   Project/generation/file/rule selectors. Service input carries reference views,
   compatibility reports, pins, contract IDs, digests and receipts instead of
   root, TOC, flavor and Library path.
2. **Standard Lua declarations are never initialized.** The shared engine sets
   Lua 5.1 but does not call upstream `init_std_lib(None)`.
3. **Most Emmy diagnostics are silently discarded.** `accepted_code` admits only
   syntax, doc-syntax, assignment mismatch and parameter mismatch.
4. **Diagnostic meaning is discarded.** The collector does not retain upstream
   `message`, forces normalized severity to Error and LSP substitutes a constant
   generic sentence.
5. **Main Project and frontend paths still differ.** LSP diagnostics call a
   standalone one-file analyzer, not the configured Main+Library owner. Overlay
   and standalone helpers also create separate analysis paths.
6. **Rule selection occurs after execution.** Service runs both current rule
   providers and filters evaluations afterwards, so unrequested work consumes
   budget and can affect completeness.
7. **Status conflates findings and unrelated partial components.** Any partial
   capability can dominate the result even when the requested analysis found a
   concrete problem.
8. **Two checks justify a general rules platform.** `wow-rules` has two product
   checks but carries capability, evidence, budget, digest and multi-pass output
   machinery.
9. **Fixed relationships pass through generic recognizers.** Source adapters
   already discover typed facts, then fabricate packs/plans/matches and convert
   results back into relations. Existing defects (#146, #150, #166) demonstrate
   the extra semantic surface.
10. **Graph use can regenerate the Library.** Native graph publication re-ingests
    Reference and re-renders annotations to compare producer identity rather than
    reusing the loaded Library and origin mappings.
11. **Update coverage is incomplete.** Physical deltas rebuild retained semantic
    indexes, while Service live update rejects TOC/package load plans. A changed
    load selection matters even when Lua text is unchanged.
12. **Compatibility probe remains.** `wow-emmy` still imports/validates a rolling
    compatibility report and ships a probe binary although Cargo already pins the
    dependency used by the product.
13. **Cancellation is lost inside multi-file diagnostics.** The operation stop
    state is not threaded through the complete collector; per-file calls receive
    a fresh token and the session notices cancellation only around the whole
    collection. The narrow repair is tracked in #135. Upstream remains
    synchronous within one file, so between-file cancellation must not be sold as
    preemption.

### Required replacement

Configuration contains addon root, selected/auto-discovered TOC, flavor,
Library directory and optional bounded overrides. One shared Emmy owner loads
Lua 5.1 standard declarations, Library and Main. It emits upstream diagnostic
message/code/severity/range for Main, with only explicit WoW-specific
suppressions. Selected checks execute directly and report findings separately
from completeness. LSP/MCP call this same operation; general Lua language
features remain upstream EmmyLua. Update reloads or rebuilds the same owner when
TOC/XML/load selection changes. Remove compatibility reports, generic packs and
Library requalification as their consumers switch.

## 4.3 Persist, search and read Blizzard/addon source

### Current chain

Project owns roots, files, TOC/XML and analysis. Publication stores a replay and
graph companion. Search creates product-specific shards and Context builds map,
pack and continuation objects. The CLI exposes build/query/retire/GC/federation,
entity, graph and context lifecycle commands.

### Confirmed defects and duplication

1. **Read reconstructs the product.** `ProjectReplay::hydrate_owner` recreates
   workspaces, analysis and publication, then compares a complete recapture.
   Graph companion read performs additional reconstruction/validation.
2. **Three storage designs coexist.** Generic object/catalog Store, Project
   partition Store and Search shard Store have separate lifecycle and schema
   concepts.
3. **SQLite is nested in SQLite.** Search builds an in-memory FTS database,
   serializes its bytes, hex-encodes them into JSON, and stores that object in the
   generic Store. Reopen decodes and deserializes it again.
4. **Current FTS is not body search.** The Project-file shard indexes document ID
   and metadata label/path, while source-name observations are separate. It does
   not satisfy repository text search.
5. **Self-sizing pagination survives.** Four query families run up to eight
   serialize/hash passes because delivered byte count is embedded in the page and
   cursor being measured.
6. **Context is a second platform.** Map, skeleton, external pack, universe,
   continuation, status and wire models exceed what is needed for a located
   source excerpt.
7. **Output is unreadable.** Context rendering escapes ordinary source
   punctuation as Unicode escapes.
8. **Graph duplicates relationships and proof state.** Foundation graph,
   proposals/validation records and final graph coexist; validation often rebuilds
   values and compares `Self::new(...) == self`.
9. **Out-of-scope lifecycle remains active.** Backup, retention, GC, quarantine,
   replacement, recovery, registry and release modules are still exported by
   `wow-store` and exposed by Project/CLI despite current documentation saying
   generic administration was removed.
10. **Search products exceed the user need.** Federation, retained-result
    lifecycle, exact reference products, entity explain and multiple cursor
    families are not required for source text/symbol/TOC/XML search.
11. **No real process-reopen workflow is checked.** CI compiles the code but does
    not prove a body-only term can be found after restart with a readable matching
    excerpt.

### Required replacement

One ordinary SQLite database owns current derived data: selected source/ref/
flavor, file path/text/content version, symbols/spans and only consumed TOC/XML
or source relations. It uses normal tables and FTS5 with documented token/phrase
semantics. Reopen performs SQL reads, not Emmy/Graph reconstruction. Updates
transactionally replace changed/deleted rows. Search returns bounded plain
results; Context reads matching indexed text and renders path, range and fenced
source. Remove replay recapture, nested databases, generic object lifecycle,
self-sizing cursors, federation and proof graph machinery as consumers move.

## 5. Package disposition

The table is a rewrite decision, not a demand to preserve the current crate
count.

| Package | Keep | Rewrite/remove |
| --- | --- | --- |
| `wow-core` | normalized path/range, small IDs/errors, finite bounds, content hash where a caller really needs it | universal canonical JSON, evidence/coverage/generation/profile/finding platform; Ketho lowering; unused integrity helpers |
| `wow-reference` | one normalized Blizzard API/source model, native aliases/corrections and selected flavor facts | standalone generated/topology products, persistent Reference publication and compatibility aliases without a workflow consumer |
| `wow-annotations` | native Ketho projection, literals, source maps, concise partial issues | compatibility certification, artifact/report identity families, duplicated producer support |
| `wow-ketho-literals` | native typed literal algorithm | nothing adjacent; merge into annotations later if a separate crate no longer helps |
| `wow-emmy` | one shared upstream analysis/session, updates and full useful diagnostics | compatibility report/probe, private duplicate analysis routes, diagnostic allowlist/metadata wrappers |
| `wow-project` | roots, TOC/XML/load selection, in-memory current project/update owner | replay reconstruction, graph/capability/fingerprint platform, 45-field platform configuration, evidence-heavy input |
| `wow-store` | one current SQLite schema, transactions and direct queries | generic object/catalog Store, nested DB, epochs/journals/leases and backup/retention/quarantine/recovery/replacement administration |
| `wow-search` | SQL/FTS query projection with explicit semantics and finite plain results | federation, retained-result lifecycle, reference/entity product families, self-sizing cursors |
| `wow-context` | matching path/range/readable excerpt plus small relevant relation context | external packs, map/skeleton/universe/continuation/status/wire platform |
| `wow-recognizers` | direct typed extraction only where Emmy/Project does not already provide it | generic pack/plan/matcher for fixed built-in behavior and its synthetic fixture inventory |
| `wow-rules` | small directly selected checks, or merge them into their owner | generic provider/capability/evidence/budget/report engine for two rules |
| `wow-graph` | at most a small typed relation/query module if SQL rows are insufficient | proposal/proof/partition/registry/axis/explain persistence platform |
| `wow-service` | thin check, index/update, search/context use cases and frontend DTOs | domain algorithms, graph building, duplicated pagination, requalification and lifecycle administration |
| `apps/wow-reference-builder` | supported Library command | duplicate test producer and stale verifier coupling |
| `apps/wow` | simple CLI plus thin LSP/MCP adapters to the same operations | internal graph/search/project lifecycle commands, host/profile matrices and mandatory IDs/digests/budgets |
| `xtask` | repository checks and skill/instruction synchronization | custom source update/materialization/certification/release-like workflows not required by product operation |

## 6. CLI and protocol audit

The current executable advertises internal Project IDs/generations, graph
snapshots/requests, Search stores/receipts/retire/GC/federation, Context maps and
profiles, overlay requests, source census/reference configs and Project
publish/update/read/reconcile/recover/skeleton. Some commands require dozens of
mandatory flags and JSON files. This is an operator console for the old platform,
not a development tool.

The target CLI is small:

```text
wow library build --source <checkout> --ref <ref> --flavor <flavor> [--toc <path>] --out <dir>
wow check [--root <addon>] [--toc <path>] --library <dir> [--format text|json]
wow index --source <checkout> --ref <ref> --flavor <flavor> --db <path>
wow search --db <path> <query>
wow context --db <path> --hit <id>
```

Names may change, but ordinary use must not require internal IDs, digests,
receipts, profiles or budget JSON. Finite safe defaults stay internal; advanced
options exist only for demonstrated user needs.

LSP/MCP are adapters to check/search/context. They must return protocol-standard
empty results, accept bounded extensible metadata/unknown notifications and keep
the session usable. They must not maintain a second semantic model or host
schema per feature combination.

## 7. Tools and CI audit

`xtask` contains repository checks plus custom source updater/materializers,
manifest generation and the obsolete Library verifier. Normal product operation
should accept an explicit checkout/ref and not need a custom update/release
platform. Retain a narrow maintenance helper only where it removes real repeated
work and does not become a product contract.

Current CI proves formatting, workspace compilation and Clippy. The
current-source workflow is not valid end-to-end because it joins builder `/2` to
verifier `/1`. There is no workflow proving the three user operations.

Keep Quick as the shared build gate, but stop treating its source archive,
status comments or line counts as behavior. Add only scoped workflow checks:

- small builder output loaded by the real Library reader;
- valid and invalid Library calls through actual CLI check;
- persisted text/symbol query after process reopen and a readable excerpt;
- one stdio request sequence when LSP/MCP behavior changes.

## 8. Documentation and Issue audit

The baseline had 329 Markdown files; this rewrite leaves 328 after replacing two
intermediate audits with this one. Large superseded trees remain under package
`e1`, `e2`, `e3` and old platform directories; 220 files are not reachable from
the canonical entry documents. Old scope terms still occur thousands of times.
Seven known relative links in old Store lifecycle notes still target removed E2
specifications; #112 must consolidate or delete those notes rather than restore
the deleted contracts. Canonical entry and owner links resolve through the
current implementation routes.

`PROJECT_COMPLETION_MATRIX.md` and parts of `IMPLEMENTATION.md` claimed generic
storage administration, pack/materialization and other platform paths were
removed while active code and CLI remained. The umbrella Issue #191 pointed at a
merged branch and a completed task. These are not harmless archives: agents use
them as instructions.

The canonical documentation set after the rewrite is:

1. root `README.md` — user purpose, actual supported commands and status;
2. root `AGENTS.md` — short working rules;
3. `ARCHITECTURE.md` — the manifesto;
4. `AUDIT.md` — this rewrite basis;
5. `IMPLEMENTATION.md` — target owners and current replacement routes;
6. `REFACTOR_QUEUE.md` — finite ordered work;
7. `PROJECT_COMPLETION_MATRIX.md` — exercised user behavior only;
8. `EXECUTION_MODEL.md` — coordination and proportional checks;
9. concise owner guides for annotations/Ketho, Emmy, source index and frontends;
10. actual CLI help/user guide.

Transfer useful facts once: source/flavor selection, Main/Library roles, exact
locations, ambiguity, TOC/XML/virtual-Lua coordinate behavior and Ketho lowering.
Delete historical gate counts, frozen catalogs, migration promises, proof
choreography, compatibility matrices and status diaries. Git preserves history.

## 9. Findings from the original Gemini audit

### Confirmed and still relevant

- oversized Service/domain leakage;
- replay hydration and nested storage;
- generic matcher/proof graph around fixed behavior;
- self-sizing serialization loops;
- unreadable Context rendering;
- CLI/LSP/MCP usability and protocol problems;
- duplicate/parallel annotation/reference pipelines;
- stale instructions and machine-state ceremony.

### Delivered since the old audit

- one Project Main+Library analysis instead of the original split engines;
- one statement parse and one final annotation emission;
- native literal path, no Wasm bridge;
- deletion of Core E0 envelopes, one-shot registries, lineage/comparison,
  Reference Pack administration and Project catalog/replay tag ladders;
- removal of many historical contracts and plans.

These are component reductions, not proof of the three workflows.

### Corrections and overclaims

- no measured evidence supports “70–80% runtime/code cost” or a 70–80% speedup;
- the cited XML accumulated-vector charge occurs after the item loop and does not
  by itself prove quadratic work;
- `allow(dead_code)` is a warning sign, not proof of no callers;
- `generated_api.rs` has live standalone/test consumers even if those consumers
  may be outside the target product;
- deleting wire types must not let deserialization manufacture validated
  executable owners;
- build/Clippy or source inspection does not establish runtime behavior.

## 10. Errors introduced by our refactoring

The rewrite must explicitly prevent these repetitions:

1. #225 introduced a public builder but left tests and verification on the old
   producer/contract.
2. #232 mechanically replaced schema assertions; one negative test now returns
   through `?` before reaching its intended assertion, and other tests still
   assume semantic admission happens during decode.
3. Documentation edits removed names without tracing whether the runtime platform
   still existed.
4. Completed component work was repeatedly presented as completed workflow work.
5. Temporary workflow scripts and repeated CI retries replaced inspection after
   failed assumptions.
6. Total line deletions combined production, tests and documentation and were
   presented as architectural progress without showing user behavior.
7. A four-code diagnostic filter was documented as a valid product boundary even
   though it discards useful upstream meaning.
8. Active Issues and comments continued to route agents to merged/completed work.

Rule for future changes: trace **user input → actual producer → serialized or
in-memory handoff → actual reader → visible result**. A task closes only when the
claimed user result is exercised. Remove displaced code, options, tests and docs
in the same change.

## 11. Rewrite sequence

### Phase 0 — establish truthful instructions

- make this audit, the manifesto, implementation guide, queue and completion
  matrix the only current root sources;
- reconcile #191/#125 and the active workflow Issues;
- retire or delete stale E1–E7/platform docs and broken links as their useful
  facts move to owner guides;
- do not create another status framework or acceptance ledger.

### Phase 1 — Library to addon diagnostics

- align builder output with the real Library reader; remove old producer,
  verifier and compatibility/certification reports;
- use batch/tree Git reads, regular-file admission and cooperative child cleanup;
- implement normal root/flavor/TOC/Library configuration and upstream Lua 5.1
  standard declarations;
- preserve full useful Emmy message/code/severity/range;
- execute only requested checks and separate findings from completeness;
- prove valid vs invalid typed Library call at the addon line through the CLI.

### Phase 2 — direct persistent source index

- create one current SQLite schema for files/text/symbols/relations;
- write/update rows from Project without replay recapture;
- remove nested SQLite, replay hydration and Store lifecycle platform;
- query body and symbol data after process restart;
- return readable source excerpts and remove Context pack machinery along that
  path;
- prove changed/deleted source leaves no stale hit.

### Phase 3 — direct relationships and thin composition

- replace fixed recognizer packs with direct typed relations;
- reuse loaded Library/source origins instead of requalification;
- remove generic graph proposal/proof/partition persistence where no remaining
  user query requires it;
- shrink Service to the three use cases and plain results;
- remove internal CLI commands and configuration exposed only by deleted paths.

### Phase 4 — stdio frontends and cleanup

- make LSP/MCP call the same configured check/search/context operations;
- preserve standard framing, positions, empty results, metadata and cancellation;
- delete old host/profile, result, fixture, owner-note and Issue families;
- merge or remove crates whose remaining responsibility no longer justifies a
  boundary;
- run one representative real workflow and measure the actual bottleneck once.

## 12. Acceptance for the rewritten product

A source-level rewrite is not complete until these user observations hold:

1. supported builder output loads directly as Library without conversion,
   requalification or the original checkout;
2. a valid annotated call is clean and a wrong argument returns the real upstream
   message, severity and addon location;
3. Ketho defaults including false/zero, arrays, callbacks and an independent
   valid declaration survive representative generation;
4. after restart, a term present only in a source body and a known symbol are
   found from the persisted index;
5. a hit returns a readable matching excerpt with correct path/range and selected
   source/ref/flavor;
6. changing/deleting source removes stale diagnostics and search hits, including
   TOC/XML load-selection changes;
7. missing, ambiguous, unsupported and partial input is reported honestly;
8. LSP/MCP return the same useful results with standard protocol behavior;
9. user source/configuration is never mutated, work/output is bounded and
   cancellation is observed at the granularity the implementation actually
   supports.

No historical schema matrix, signed pack, lineage proof, release executor,
all-editor campaign or arbitrary percentage reduction is an acceptance gate.
