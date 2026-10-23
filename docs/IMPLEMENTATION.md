# Implementation: finish the WoW linter

Read the [manifesto](ARCHITECTURE.md) and [WoW behavior](WOW_LINTER.md).
This is the target implementation, not an inventory of promises already delivered.
Current source evidence is in [AUDIT.md](AUDIT.md), with document-by-document
coverage in the [review register](AUDIT_REGISTRY.md); order is in the
[queue](REFACTOR_QUEUE.md). Reuse working algorithms and remove their platform
wrappers through the actual callers.

## Code ownership

| Responsibility | Existing code to reuse | Simplification |
| --- | --- | --- |
| Blizzard input and generation command | `apps/wow-reference-builder/src/{lib,io,catalogs}.rs` | One bounded source reader and one generator; shared selection semantics. |
| API tables, types, corrections | `wow-reference/src/native{,_model,_corrections,_aliases}.rs` | One typed API model with origins; no persistent Reference product. |
| Annotation emission | `wow-annotations/src/{native,ketho,aliases,literals}.rs`, `wow-ketho-literals` | One final render; ordinary Lua plus useful metadata/maps. |
| Lua analysis | `wow-emmy/src/{session,references}.rs`, `syntax/analyzer.rs` | One configured Emmy owner; no probe/certification input. |
| Addon and load model | `wow-project/src/{disk,configuration,load,xml_lua,xml_bindings,analyzer}.rs` and submodules | Current files/load facts and virtual contexts; no publication/replay prerequisite. |
| WoW checks | `wow-rules`, useful `wow-recognizers` extraction | Direct functions over typed facts; select before execution. |
| Use cases and frontends | `wow-service/src/local`, `frontend`; `apps/wow/src` | Thin check/generate/query composition and protocol conversion. |
| Optional source index | `wow-store`, useful `wow-search` queries, `wow-context/src/pack/source_excerpt.rs` | One SQLite database; direct reads and excerpts. |

Paths abbreviated `wow-*` above mean `crates/wow-*`. Do not create new crates for
this plan. Existing crates can be merged or removed when their useful callers
have moved. `wow-core` contains only genuinely shared primitives. Domain owners
do not depend on Service or applications.

## Addon configuration and Emmy analysis

### Ordinary configuration

Use one small public configuration: addon root, optional explicit TOC, client
flavor, Library directories, optional dependency roots and selected checks. CLI
values override file values; relative paths are resolved from that config file.
Without a config, the explicit root (or documented cwd default) supplies discovery.
There is no user-supplied Project ID, generation, compatibility report, digest,
profile matrix or budget JSON. Reuse serde and existing path readers.

One selected target describes client flavor, source revision when available, and
ordinary-addon execution context. Do not overload one `environment` string with
flavor, game/glue loading and `SecureOnly` API metadata. Incompatible Library
metadata is an actionable configuration error; a plain user Library without it
still supports typing with the affected domain checks explicitly unavailable.

### One direct check operation

1. Resolve config and selected TOC/dependencies; capture bounded source bytes.
2. Reuse the load parser for ordered Lua/XML selection. Treat unsupported load
   conditions locally; useful independent files can still be checked.
3. Load reusable Library/API facts. Configure Lua51, initialize embedded STD once,
   then register Library/dependencies/Main in one `AnalyzerSession`.
4. Supply per-addon chunk context and source-mapped XML handler context using
   supported Emmy APIs/virtual declarations. Never synthesize user locations.
5. Obtain upstream diagnostics and only the facts needed by selected WoW checks.
6. Run those checks directly and return findings plus input/analysis limitations.

Connect this operation to the public `wow check` first. It must not need SQLite,
a graph build, Project publication, replay hydration or source regeneration.
An existing generated Library must not cause Git processes or network access.

`AnalyzerSession` and directory Library discovery already exist; do not recreate
them. Extend the actual diagnostic model and all its readers to carry message,
code, severity, range and related information. Remove the four-code allowlist;
use upstream diagnostic configuration and only justified narrow WoW suppressions.
Keep STD/Library origins distinct from Main findings. Details and exact donor APIs
are in [EMMYLUA_UPSTREAM.md](EMMYLUA_UPSTREAM.md).

Resolve analyzer settings once and pass them to this session and its updates.
The current syntax, reference and local-flow constructors each make their own
`Emmyrc::default`; they do not implement a caller-selected diagnostic policy.
Remove those independent defaults as the callers move. Keep three controls
distinct: upstream diagnostic configuration, selected WoW checks, and the CLI
failure threshold. A threshold changes the exit decision, not which messages
exist; disabling one WoW check must not suppress unrelated Emmy diagnostics.

### Updates

Keep immutable source bytes/version for each published result. Reuse VFS/syntax
for unchanged files, but retain the existing conservative semantic reindex: the
pinned upstream changed-file API does not invalidate every unchanged consumer.
TOC/XML/dependency/Library changes rebuild selection/context before reanalysis.
A failed or cancelled refresh cannot advertise an old snapshot as current.
For LSP, new document versions supersede older work and stale diagnostics are
cleared/replaced only for the appropriate version.

Load membership and analyzer membership must change together. Removing a file
also removes its declarations, virtual units, maps, dependent findings and index
rows. A TOC variant/dependency change can change the entire selected file set;
an XML include can change selected units, while template/inheritance changes
can change widget and handler context.
The current `LiveProjectStore::update` rejects TOC/package-selected projects:
connect a fresh load-plan rebuild to the shared check instead of treating the
existing physical-file update route as complete. Keep old immutable views only
for callers explicitly asking for those views.

Poll cancellation between files/stages and before publication. The synchronous
upstream per-file analysis cannot currently be interrupted mid-call. Do not hide
that fact behind a cancellation token or a timeout that leaves work running.

## Native annotation generation

### Preserve the existing pipeline

Read a selected Git revision and generated-document TOC; parse Lua with Emmy;
use the existing restricted evaluator and normalization; apply explicit corrections;
render ordinary `.lua` files once. The output is analysis data, never executable
addon code. The same Library reader used by `wow check` accepts it.

The existing builder separately scans TOC lines while Project has a richer load
parser. Share the necessary pure TOC selection logic; do not make generation
instantiate Project/Graph/Store just to parse a document TOC. Generated API TOCs
may lack `Interface`; that is not an addon selection failure. The first version
may reject genuinely unsupported document load forms explicitly.

Resolve the ref once. Enumerate regular Git tree entries once and read selected
blobs using one bounded `git cat-file --batch` process or equivalent batched reads.
Thread cancellation through child I/O and kill/reap the child on cancellation.
Retain current path/mode/encoding/output bounds; no per-record certification.

### Fix semantics before claiming coverage

Extend `native_model` and the existing emitter, not a second model/renderer:

- distinguish flavor selection from API environment; apply member restrictions
  as well as system restrictions so `SecureOnly` functions cannot leak through
  an `All` parent into ordinary-addon declarations;
- model `MayReturnNothing`, `NilableContents`, numeric stride-group positions,
  field `Mixin`, member environment and the secret/
  restriction fields actually present in the selected source;
- preserve defaults, receiver style, arrays, callbacks, ordered result packs,
  enum/open-string distinctions and explicit widget bases;
- resolve named types from the selected API and small declared supplemental
  catalogs. Report unresolved names; do not silently substitute `any`;
- retain unsupported raw fields and located omissions without treating every
  unknown informational field as failure of the entire corpus.

The existing first-return `SecretReturns=true`/`canaccessvalue` projection in
`wow-reference/src/native_view.rs` is reusable narrow behavior. It is not complete
secret-flow coverage. Do not delete it on the mistaken assumption that all
restriction metadata is unused, or extend its assumptions to every field/return.

### Outputs and corrections

Keep a source-to-Library manifest with selected revision/flavor/context, input
failures, projected declarations and source mappings needed by consumers.
Use the existing `source-report.json` boundary while simplifying it. Generated
Lua alone remains a valid type Library; richer WoW checks can consume the small
metadata alongside it. No report is a prerequisite to basic third-party Library
loading and no hidden runtime reconstruction is required to read API facts.

Implement this handoff in the actual Library reader. Today the native artifact
wire reader discards `metadata_sidecars` with `IgnoredAny`; recording raw metadata
in the builder is not sufficient. Those sidecars are field-name/source links;
the values still live in raw documents. Emit the few typed fields needed by
selected checks and decode them in the reader, validating target/declaration
references and keeping origins.
Unsupported fields remain explicit losses. Do not require a full Reference
snapshot, original source checkout or a second generation pass to use them.

Corrections identify an exact symbol/field, expected old value, replacement and
source/reason. Apply only to the selected target; conflicts or changed inputs
leave the original fact and an explicit problem. Remove the present per-record
raw-hash/reviewer/receipt schema ladder as its readers switch. Do not replace it
with another correction platform. Preserve raw source and attribution.

Generate into a new tool-owned output directory and publish only a completed
result; never overwrite a user's source tree. Keep actual producer/reader limits
consistent. Shared acquisition limits already exist; downstream replay and graph
budgets are separate old constraints, not a reason to rebuild the old platform.

See [KETHO_RUST_PORT.md](KETHO_RUST_PORT.md) for emitter details and exact donors.

## Typed source relationships

Keep simple source-linked relations required by a rule or lookup: a resolved
callable, declared dependency, TOC/XML include, handler, widget base, explicit
mixin or API origin. Use existing typed extraction and Emmy facts. Do not convert
these into JSON packs, compile a matcher, create graph proposals and validate a
second proof graph before returning the same relation.

Implement the checks in [WOW_LINTER.md](WOW_LINTER.md) as direct functions. Keep
rule dependencies explicit in code so `--checks` avoids unrequested analysis.
Type checks must not depend on an optional secret/event or source-search feature.
If a downstream pass needs only three fields, pass those fields rather than the
whole Project/Graph/Reference publication model.

Retain the semantics hidden inside those wrappers: a recorded name is not a
resolved callable; `UsesApi` is not necessarily a call; a possible relationship
does not become definite through traversal. Keep dependency/load direction and
source order. XML containment, inheritance and object parentage stay distinct.
Negative findings need complete, unconflicted evidence for the particular
selected domain; an empty partial result does not prove absence. A general
Graph engine is unnecessary for these ordinary typed distinctions.

## Blizzard UI index, search and context

This is the next useful feature after addon checking, not its storage backend.
Reuse selected source bytes/load relations and existing Emmy symbols. Store the
current data once in normal SQLite tables: sources, files with body text, symbols
with spans, and the relations consumed by navigation. Keep only one current schema;
unsupported derived databases receive a clear rebuild instruction.

FTS indexes **body text**, not just path/label. Give symbol names a separate exact
lookup and use FTS for documented token/phrase text queries. Escape user input for
that search mode; SQL binding alone does not make a raw `MATCH` expression literal.
Do not promise arbitrary substring/regex matching from FTS. Reuse SQLite features
instead of building a search federation.

Update changed/deleted rows transactionally, including their FTS entries and
relations. Reopening a database performs SQL reads, without Emmy/Graph hydration
or re-generating a Library. Return path, range and bounded plain snippets from
the matching stored text. A direct-file excerpt is permitted only when its content
matches the indexed revision. Never apply stale spans to newer disk contents.

### Small navigation operations

Preserve the useful behavior of the former Map/Skeleton/Context documents as
direct queries over these tables and existing Emmy facts:

| Operation | Result and boundary |
| --- | --- |
| Outline | Selected package/TOC/files or a file's declarations, with useful load order and source locations. A directory inventory is not a claim of runtime reachability. |
| Exact symbol lookup | Matching declarations in an explicit scope. Keep equal names from the addon, Blizzard UI and API Library as distinct candidates. |
| Context around a hit | Its matching source excerpt, declaration and direct callers/references/load or type relations that are actually known. Default to direct neighbors with finite count/text limits; report truncation. |
| Text search | Documented token/phrase matching over indexed source and documentation, with exact selected revision/flavor and deterministic ordering. |

Select the addon snapshot, Blizzard source revision/flavor, or API contract
scope explicitly. A source observation and an API contract have different
meaning even when their names match. Do not silently rebind an old selection to
new current data. If deeper expansion is needed, bound it explicitly and preserve
possible versus resolved edges; a path is a navigation witness, not a new fact.

Keep matches and query coverage separate. Complete zero matches, missing index,
incomplete relevant input and output truncation are distinguishable outcomes.
Do not import the old continuation receipts, profile DSL, pack identities or
token-pruning platform to represent these states. Retain complete result items
when a limit is reached rather than cutting a declaration or source mapping in
half. Add pagination only when a caller needs it.

Source is quoted data for editors and agents. Render readable code, choosing a
Markdown fence that cannot be closed by the body; escape transport syntax once
and handle unsafe control characters in display labels. Preserve the captured
source and its ranges separately. Source comments and documentation cannot
override tool instructions or become executable commands.

Remove nested SQLite/hex/JSON, replay recapture, object lifecycle, retained-result
journals, self-sizing cursors and Context pack/map/skeleton machinery as these
readers switch. Start with a finite limit and ordinary ordered results; add
pagination only for a demonstrated user requirement.

## Results and interfaces

A check returns diagnostics, analysis problems and selected target information.
A source query returns located hits. Serialize once at the CLI/LSP/MCP boundary;
no self-hash, frozen bytes, proof manifest or output-size fixed point. Share
behavior, not a generic invocation registry. See [FRONTENDS.md](FRONTENDS.md).

Simplifying wire formats must preserve admission checks. Successfully decoding
JSON does not establish valid paths, source spans, target selection, references
or an executable owner. Validate these once at the real boundary, then use typed
values internally. Keep useful shared `wow-core` geometry/source-map checks;
remove duplicated envelopes without accepting unchecked serialized state.

## Completion and verification

The product is functionally complete when generation, ordinary addon checking,
source updates and the advertised frontend/search operations are connected and
usable, including partial/missing input handling. Then exercise real valid/invalid
API calls, load/XML mapping and changed/deleted sources, and the later-process
source query; reconcile relevant existing failures before a release claim.

During implementation follow the owner's product-first policy in
[AGENTS.md](../AGENTS.md): code, formatting and affected strict Clippy; tests are
deferred until functional completion. Do not build fixture factories or repair
retired scope before delivering the linter. Documentation and compilation do not
promote an unexercised workflow to working.
