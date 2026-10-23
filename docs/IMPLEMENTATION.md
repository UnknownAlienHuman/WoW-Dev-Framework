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
flavor, Library directories, optional dependency roots and selected checks.
Optional `target_interface`, `emmy_diagnostics` and `fail_on` supply compatibility,
upstream diagnostic settings and the exit threshold; they are not separate profiles.
Resolve it once before acquiring source:

- CLI values override file values. Relative CLI directory paths use the process
  working directory; relative file-config directory paths use the config's
  directory. The root defaults to the config's directory, or cwd without a config.
  A relative TOC path is always relative to the resolved addon root.
- Require an explicit flavor and at least one explicitly selected local Library
  directory, supplied by CLI or config. Do not infer flavor from a folder name,
  find a different Library silently, download data or regenerate declarations.
  `library` is a nonempty array of paths in JSON; repeated `--library` options
  replace that array. These directories supply declarations, not files to execute.
- An explicit TOC remains selected even when its Interface metadata differs.
  Report that compatibility problem without switching TOCs. Without an explicit
  TOC, inspect the addon root and select only an unambiguous candidate under the
  supported flavor rules; otherwise list candidates and require selection.
  Current Project loading takes explicit TOCs, so this small discovery adapter is
  work to implement, not an existing loader capability.
- The first supported public flavor is `retail`, with `mainline` accepted as an
  alias and normalized once. Its source family is `Mainline`. Automatic discovery
  considers the addon root's exact `<addon>.toc` and `<addon>_Mainline.toc` names;
  if both exist, require an explicit TOC instead of guessing client precedence.
  Reject case-folded duplicates. Other flavor/condition mappings need their own
  source-backed entries before being advertised; an explicit path does not make
  an unsupported flavor supported. Classic remains planned work, not an inferred
  synonym for every non-retail client. Never choose the highest Interface value.
  The ordinary in-game global addon context supplies
  `game_types["mainline"] = true`, `family = "Mainline"`, `location = Game` and
  `environment = Global` to the existing `TocLoadContext`. Unlisted game aliases,
  `game` and `text_locale`
  remain unknown unless separately selected and supported. Do not infer them
  from the host locale, Interface number or folder spelling. Secure/glue/special
  loads do not inherit this ordinary context silently.
- `target_interface` is an optional positive integer. Use it when explicitly
  supplied, otherwise use a compatible selected Library's recorded target if
  available. Conflicting Library targets or an explicit incompatible target are
  configuration errors. Without either, compatibility is unknown: basic typing
  still works, but do not infer the target from the TOC being checked or use zero.
  Represent this absence in the ordinary load input instead of forcing the old
  positive-only `ProfileIdentity` to contain a fabricated target. Missing target
  information limits only checks that actually need it.
- Dependency roots are explicitly supplied AddOns-like parent directories.
  Acquire only named TOC dependencies and their transitive dependencies from
  those directories, then pass the selected packages to the existing load owner.
  Reject ambiguous name matches. Absent optional dependencies are nonblocking;
  missing required dependencies limit the affected analysis. Do not add every
  nearby addon, and do not treat a dependency directory as an annotation Library.
  Pass only selected TOCs and the relevant dependency closure to acquisition.
  The old package-universe reader also reads unselected variants and unrelated
  supplied packages; their existence, encoding or size must not become a
  prerequisite for checking the chosen flavor.
- `emmy_diagnostics` is the pinned upstream `EmmyrcDiagnostic` object, using its
  camelCase field names and defaults. Deserialize it directly into that type;
  malformed codes, severities or values are configuration errors. Set it on the
  one framework-owned `Emmyrc` with runtime Lua51. Do not discover another config
  or use the lenient upstream file loader. An omitted object enables default
  upstream diagnostics. `diagnosticInterval` affects editor scheduling only.
- `fail_on` defaults to `error` and accepts `error`, `warning`, `information` or
  `hint`. It changes the exit threshold, not diagnostic collection or severity.
- Omitted or empty `checks` selects no WoW rule; Lua diagnostics still follow
  upstream diagnostic settings. Unknown check names are configuration errors.
  A recognized check whose implementation or required input is unavailable is
  reported unavailable, while independent Lua findings remain usable.

There is no user-supplied Project ID, generation, compatibility report, digest,
profile matrix or budget JSON. Reuse serde and existing path readers.

One selected target describes client flavor, source revision when available, and
ordinary-addon execution context. Do not overload one `environment` string with
flavor, game/glue loading and `SecureOnly` API metadata. Incompatible Library
metadata is an actionable configuration error; a plain user Library without it
still supports typing with the affected domain checks explicitly unavailable.

### First implementation slice

Queue item 1 delivers offline Lua/type diagnostics with an existing Library,
one configured Main/Library/STD session and the ordinary CLI result. Reuse the
current ordered TOC/package selection and already supported mapped XML units;
do not replace them with a recursive collection of every Lua file. No WoW rule
is enabled by default in this first slice. API/Secret checks become selectable
as functioning checks after their binding and uncertainty handling is repaired
in queue item 4; merely executing the old providers does not satisfy that work.

Full addon chunk context, implicit XML handler context, XML-only admission and
selected-load updates are completed in queue item 3. Until then, unsupported
input or missing context must be an explicit limitation of the affected scope;
do not drop those units and report a complete addon check. Preserve useful
independent diagnostics. Item 1 can finish a useful language-checking path, but
it does not complete the full WoW addon workflow or retire these requirements.

### One direct check operation

The complete operation below includes the later context and WoW-check work;
the first slice above defines the initial implementation boundary.

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

Derive completeness from the requested checks and their actual input dependencies.
The current `local/projection.rs::components` has no selected-check input and
marks native annotation input degraded unconditionally; `service.rs::derive_status`
then treats any partial component capability as a partial check. Fix this along
with early check selection. Keep real limitations with the affected check/file;
an unused secret-analysis capability or an auxiliary navigation limitation must
not prevent a completed type check from returning its normal outcome.

### Load units and addon context

Reuse physical Lua files and the existing XML virtual-unit path in one analysis.
`xml_lua::prepare` already handles both a single TOC and reachable package plans;
`analyzer.rs` adds those units to the shared Main/Library session. Preserve its
entity/newline maps and separate pieces across XML comments/CDATA delimiters.
Adding callback context must not replace this with a second inline-Lua parser.

Keep acquisition failures local when selection is still meaningful. A malformed
included XML unit must carry its path and best available parser range, leave its
dependent closure incomplete, and preserve already captured independent Lua/XML
units. The current `Loader::expand` propagates that parse error for the whole
input, while `local/disk_input.rs::acquisition_error` loses its source fields.
Fix both boundaries. Invalid request/root selection and unsafe paths still need
explicit refusal; do not recover by reading outside configured roots.

Remove the assumption that Main needs a physical `.lua` file. The current
single-TOC loader, package Main conversion and `ProjectInputInventory::build`
reject an empty physical inventory before XML units can supply the program.
Select analysis units after load parsing: XML-only addons with inline Lua still
need analysis, and a selected load-only/template package still has load facts.
Represent an empty language scope explicitly; never require a dummy source file
or confuse it with failed acquisition.

Path prefixes such as `packages/<addon>/...` prevent file collisions; they do not
model WoW chunk arguments. For supported ordinary external Lua chunks, add one
shared private-table context per selected addon and a separate lexical chunk
scope per file. Special load kinds follow their explicit context or remain
limited; see [the load boundaries](WOW_LINTER.md#4-toc-and-xml-form-the-load-model).
The addon-name/namespace values must reach top-level `...` through a supported
Emmy integration, preserving
declared load order and source maps. Dependency addons keep their own private
tables; their selected globals can still provide context. Explicit local names
for these values are arbitrary. Do not recognize them by the spelling `ns`,
`addonName` or a filename, and do not rewrite unrelated nested varargs.

The pinned Emmy treats an unbound chunk `...` as `any...`; adding a workspace or
prepending an unrelated declaration does not type its two positions. The chosen
adapter is a source-mapped virtual expression replacement, described in
[Emmy chunk context](EMMYLUA_UPSTREAM.md#addon-chunk-context): one stable private
table and generated two-result helper per addon, with only chunk-owned ellipsis
expressions replaced in memory. This is a source-supported implementation design,
not delivered context support. All diagnostics, navigation and WoW-fact readers
must use the same transformed view and original-source maps.

Keep TOC execution order as separate load data. Emmy's workspace indexing order
does not simulate chunk execution, prove initialization before use, or make a
conditionally loaded global available at every runtime point.

Share a finite reviewed dependency-key table between TOC parsing and package
projection. The selected Blizzard `Blizzard_FrameXMLBase.toc` uses `## Dep:`,
which both current key lists miss. Preserve that dependency and its condition;
do not classify every unknown metadata key as another dependency alias.

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
Include selected XML/context adapter versions in configuration/cache identity for
both standalone and package loads. Current package configuration omits the XML
constants, although its final analyzer/report identities and replay recapture
already bind them; this is not evidence of a stale final result.

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

An API rule needs the original receiver binding, not just its spelling and a
member-resolution flag. Extend/query the existing Emmy fact boundary so a typed
local named `C_*` cannot be mistaken for the selected API namespace. Reuse exact
Reference lookup and its conflict/partial/absence distinctions. Alias or dynamic
provenance that Emmy cannot establish stays unknown; do not add string-based
alias chasing or a second resolver.

Retain the semantics hidden inside those wrappers: a recorded name is not a
resolved callable; `UsesApi` is not necessarily a call; a possible relationship
does not become definite through traversal. Keep dependency/load direction and
source order. XML containment, inheritance and object parentage stay distinct.
Negative findings need complete, unconflicted evidence for the particular
selected domain; an empty partial result does not prove absence. A general
Graph engine is unnecessary for these ordinary typed distinctions.

The current native completeness check covers the selected APIDocumentation
manifest. It does not prove that this manifest lists all Blizzard globals,
undocumented APIs or Lua UI exports. State that a name is absent from the selected
contract or unresolved in the selected environment; stronger runtime-absence
wording needs stronger evidence. Keep this scope in the finding's data and text.

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

Reuse the current useful query paths. `Service::durable_query_shard` already
opens a retained search shard and queries it without Project/Emmy hydration.
The opt-in Reference documentation inventory also indexes normalized Blizzard
documentation text in its label column. The missing body index concerns actual
Lua/source text; do not claim all existing search is name-only or rebuild these
working SQL readers. The current graph `neighbors_batch` likewise shares one
physical edge scan, despite its conservative per-query logical budget.

Update changed/deleted rows transactionally, including their FTS entries and
relations. Reopening a database performs SQL reads, without Emmy/Graph hydration
or re-generating a Library. Return path, range and bounded plain snippets from
the matching stored text. A direct-file excerpt is permitted only when its content
matches the indexed revision. Never apply stale spans to newer disk contents.

Make query cancellation real at the database boundary: row/output limits do not
bound the time spent inside a synchronous SQLite statement or lock wait. The
current `SearchStoreReader` has no stop/deadline or SQLite interruption hook.
Use the connection's interruption/progress mechanism and bounded lock waiting,
with cancellation checks before and after reads. Keep an interrupted query
distinct from complete zero matches; do not publish partially built success.

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
An acquired empty source set is a valid empty index, not missing/corrupt input;
the old shard builder's nonempty-document guard must not become this contract.
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

Preserve the useful old diagnostic-presentation rule: two messages are related
only when their source, semantic operation and cause establish the relation.
Keep distinct findings and their origins; presentation grouping must not change
severity, completeness or the exit decision. The current local check supplies
no causal relations to `presentation::build`, so the existence of that graph
does not establish working root-cause folding. A plain related-diagnostic field
is sufficient when a real producer provides the evidence.

Simplifying wire formats must preserve admission checks. Successfully decoding
JSON does not establish valid paths, source spans, target selection, references
or an executable owner. Validate these once at the real boundary, then use typed
values internally. Keep useful shared `wow-core` geometry/source-map checks;
remove duplicated envelopes without accepting unchecked serialized state.

Preserve failure cause and source location through the same boundary. Current
session adapters collapse most syntax/local-flow failures into `NativeFailure`,
and Context cleanup can replace an earlier semantic error with a generic close
failure. Keep the primary failure with a secondary cleanup problem when both
occur. A failed close cannot publish success or cause a mutation to be retried.
This requires ordinary typed errors, not the former lease/error-code platform.

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
