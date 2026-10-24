# Implementation: finish the WoW linter

Read [current status](CURRENT_STATUS.md), the
[manifesto](ARCHITECTURE.md) and [WoW behavior](WOW_LINTER.md). This document is
the implementation design and owner map. [REFACTOR_QUEUE.md](REFACTOR_QUEUE.md)
orders work; [IMPLEMENTATION_TRACE_REGISTER.md](IMPLEMENTATION_TRACE_REGISTER.md)
records source evidence, delivered slices and exact observations.

## Current checkpoint

PRs #246–#250 already delivered:

- complete configured Emmy diagnostic payloads;
- `ConfiguredAnalyzerSession` over exact Main/Library/optional virtual snapshots,
  Lua 5.1, embedded STD and caller diagnostic settings;
- diagnostics-only versus explicit semantic collection;
- direct `wow-service::addon_check::check_addon` and its diagnostic/limitation
  result without the legacy generic platform;
- compiled Emmy identity without fabricated compatibility-report hashes;
- exact Main snapshot admission for configured-owner reuse.

Do not reimplement those owners. The missing first public path is ordinary
configuration/acquisition -> exact snapshots plus validated load/context data ->
existing `check_addon` -> CLI rendering/exit status. None of the lower slices has
been observed through an ordinary addon command.

## Code ownership

| Responsibility | Existing owner to reuse | Required simplification/addition |
| --- | --- | --- |
| Blizzard input and generation | `apps/wow-reference-builder/src/{lib,io,catalogs}.rs` | one bounded selected Git read pipeline and one generator |
| API facts and corrections | `wow-reference/src/native{,_model,_corrections,_aliases}.rs` | preserve reviewed fields/origins; no persistent Reference requirement for type-only checking |
| Annotation output | `wow-annotations/src/{native,ketho,aliases,literals}.rs`, `wow-ketho-literals` | one Lua render plus optional shared manifest/maps |
| Lua analysis | `wow-emmy::ConfiguredAnalyzerSession`, existing session/reference/flow owners | reuse delivered owner; add only demonstrated context/update seams |
| Addon/load model | `wow-project/src/{disk,load,xml_lua,xml_bindings,analyzer}.rs` | ordinary load target, selected acquisition, typed unit context/order and pure XML preparation without publication receipts |
| Direct check use case | `wow-service/src/addon_check.rs` | ordinary config/input orchestration and selected WoW checks |
| CLI/LSP/MCP | `apps/wow/src`, `wow-service/src/frontend` | thin adapters to the same Service owners |
| Optional current source index | `wow-store`, useful `wow-search` queries, exact excerpt helpers | one direct SQLite database over current selected data |

Do not create new crates for this plan. Existing crates can be merged or removed
only after their real callers move. Domain owners do not depend on Service or
applications.

## Addon configuration and Emmy analysis

### Ordinary configuration

The public configuration contains:

- addon `root`;
- optional explicit `toc`;
- supported `flavor`;
- optional positive `target_interface`;
- one or more local `library` directories;
- optional `dependency_roots`;
- optional selected WoW `checks`;
- optional pinned upstream `emmy_diagnostics`;
- `fail_on`, default `error`.

Resolution rules:

1. CLI values override config-file values.
2. Relative CLI directory paths use cwd; relative config directory paths use the
   config file directory. Root defaults to the config directory, or cwd without
   a config. Relative TOC paths use the resolved addon root.
3. Require explicit flavor and at least one explicit Library directory. Never
   infer a flavor from folder spelling, discover another Library silently,
   download data or regenerate declarations.
4. `library` is a nonempty JSON path array. Repeated CLI `--library` replaces the
   configured array. Library directories supply declarations, never executable
   files.
5. Keep an explicit TOC even when Interface metadata differs; report
   compatibility without switching target. Without explicit TOC, select only an
   unambiguous supported candidate inside the addon root.
6. First support `retail`, with `mainline` normalized as its alias and source
   family `Mainline`. Automatic discovery considers exact `<addon>.toc` and
   `<addon>_Mainline.toc`; if both exist, require selection. Reject case-folded
   duplicates. Do not choose by highest Interface.
7. The ordinary in-game global target supplies Mainline game type/family and Game
   location for selection. It does **not** force every selected unit to execute
   globally: package `UseSecureEnvironment` and per-unit `LoadIntoEnvironment`
   determine effective execution context where supported. Glue, bootstrap,
   load-first, suppressed-private-table and other special contexts remain typed
   facts or explicit limitations; never inherit ordinary context silently.
8. Use explicit `target_interface` when supplied. Otherwise use compatible
   selected-Library metadata if present. Conflicting Library targets or explicit
   incompatibility are configuration errors. Without either, compatibility is
   unknown and only dependent checks are limited. Never infer the target from
   the addon TOC or fabricate zero.
9. Dependency roots are AddOns-like parents. Acquire only named required and
   available optional transitive dependencies. Use one finite reviewed alias
   table: required `Dep`, `Dependencies`, `DependsOn`, `RequiredDep`,
   `RequiredDeps`; optional `OptionalDep`, `OptionalDeps`. Reject ambiguous names.
   Missing required dependencies limit affected analysis; absent optional
   dependencies do not block it. Never add every neighboring addon, accept
   arbitrary metadata prefixes as dependencies or treat dependency roots as
   annotation Libraries.
10. Decode `emmy_diagnostics` directly as the pinned `EmmyrcDiagnostic` using its
    camelCase fields/defaults. Invalid values are configuration errors. Do not use
    lenient `.emmyrc` discovery. `diagnosticInterval` is editor scheduling only.
11. `fail_on` accepts `error`, `warning`, `information` or `hint`; it changes exit
    status, not collection or severity.
12. Empty/omitted `checks` means no WoW rule. Emmy language diagnostics remain
    enabled. Unknown check names are configuration errors; recognized unavailable
    checks become scoped limitations while independent findings survive.

No ordinary user supplies Project IDs, generations, profiles, compatibility
reports, digests, receipts or budget JSON.

### First implementation slice

The first public slice is an offline Lua/type check with an existing Library:

1. resolve ordinary config and selected TOC/dependencies;
2. capture exact bounded physical files, supported mapped XML units and a typed
   exact selected load/context receipt;
3. build the exact path-sorted Main/Library/optional virtual snapshots accepted
   by the delivered Service request without discarding the separate selected
   package/TOC/XML order;
4. call `wow_service::addon_check::check_addon` with snapshots and the validated
   context available for limitations/later selected checks;
5. render its diagnostics and limitations;
6. apply the documented exit policy.

No WoW rule is enabled by default. Missing addon chunk or XML handler context
remains explicit `Partial`; do not drop those units and advertise complete addon
semantics. This slice needs no SQLite, graph, Project publication, replay
hydration, Reference or source regeneration.

<a id="one-direct-check-operation"></a>

### One complete check operation

The final check, including later context and selected WoW rules, is:

1. resolve ordinary configuration and supported target;
2. select TOC, dependencies and exact package/TOC/XML unit order;
3. capture exact bounded physical/XML/Library input and typed package/per-unit
   execution context;
4. prepare virtual units and maps;
5. open or safely reuse one `ConfiguredAnalyzerSession` over exact snapshots;
6. validate the ordered context against snapshot role/path/content identity and
   supply supported addon chunk and XML handler context;
7. collect language diagnostics and only semantic facts needed by selected checks;
8. run those checks directly;
9. return exact target information, diagnostics and scoped limitations.

Reference/Rules are constructed only when a selected implemented check needs
them. Optional component health never becomes a global clean gate.

### Load units and addon context

Reuse the existing TOC/XML parser, virtual-unit extraction and maps. Do not
replace selected loading with recursive collection of every Lua file or a second
inline-Lua parser.

#### Selected acquisition

The old package universe can read unselected variants and unrelated packages
before reachability. The ordinary adapter supplies only the explicit or
unambiguous TOC and relevant dependency closure.

Share one finite reviewed metadata/directive table between TOC parsing and
package projection. It must cover the exact dependency aliases above plus typed
semantics actually used by selected source:

- package default execution environment from `UseSecureEnvironment`;
- per-unit environment override from `LoadIntoEnvironment`;
- private-table policy from `SuppressLocalTableRef`;
- `LoadOnDemand`, bootstrap and supported load-first/load-phase facts;
- source span, selection predicates and originating load record for each fact.

Unknown cosmetic/custom metadata remains retained evidence without making
unrelated language checking partial. Unknown metadata or clauses that can change
selection, environment, namespace or order remain located limitations. Do not
collapse all unknown directives into one global closure failure.

Reduce selection predicates according to their operators:

- an allow-list is included after any known match, excluded only when every value
  is known false, otherwise unresolved;
- an exclude-list is excluded after any known match, included only when every
  value is known false, otherwise unresolved;
- across conjunctive gates, known exclusion dominates unknown; unresolved remains
  only when no gate already excludes the unit.

Retain unknown predicates on their source records even when a different gate
makes the unit definitely excluded. They must not poison the selected program,
and a reachable unresolved unit must never be reported clean.

A malformed included XML unit carries path and best available parser range,
marks only its dependent scope incomplete, and preserves already captured
independent units. Unsafe roots/paths and invalid target selection remain hard
refusals. Do not recover by reading outside configured roots.

The exact source trace and concrete Blizzard examples are in
[TOC_LOAD_CONTEXT_AUDIT.md](TOC_LOAD_CONTEXT_AUDIT.md).

#### Physical and virtual Main

Main may contain physical Lua, mapped XML-derived Lua, both, or a valid empty
language scope with load facts. Remove physical-file prerequisites from the
direct route. An XML-only addon must not require a dummy `.lua` file.

Keep entity/newline mappings, XML source pieces and gaps around comments/CDATA.
A virtual diagnostic is user-visible only after exact projection through those
maps. Successful syntax analysis does not establish handler receiver/parameters
or runtime execution.

`LuaWorkspaceSnapshot` remains a canonical path-sorted source set. It is not the
load-order owner. Project must pass a separate immutable validated context value
that lists selected physical and virtual units in exact package/TOC/XML order and
binds each entry to snapshot role, canonical path, content digest and source
load-record/unit identity. Reject missing, foreign, duplicate or digest-mismatched
entries. Do not claim that Emmy registration order emulates the WoW client.

#### Addon chunk context

Path prefixes isolate file identity but do not model WoW chunk arguments. For
supported ordinary external Lua chunks, provide one stable private table per
addon and separate lexical chunks per file. Dependencies get distinct private
tables. Variable spelling is irrelevant.

Use the source-mapped helper design in
[EMMYLUA_UPSTREAM.md](EMMYLUA_UPSTREAM.md#addon-chunk-context): replace only
chunk-owned top-level ellipsis expressions in memory; preserve/compose maps; and
mark helpers/calls context-only so they never become user API/flow facts.

Derive that transformation only from the validated per-unit context. Keep
external chunks, free inline Script bodies, verified handler signatures,
`SuppressLocalTableRef`, package secure defaults, per-unit environment overrides
and bootstrap/load-first contexts separate. Unsupported contexts remain
limitations rather than receiving fabricated values.

### Updates

Keep immutable exact input identity for every result. Reuse a configured owner
only when its exact compatibility contract holds.

- A physical source delta may use incremental machinery only when selected
  membership, ordered load context, Library identity and analyzer configuration
  are unchanged.
- TOC/XML/dependency/Library/target/context/config changes rebuild
  selection/context and owner cold.
- Removing/renaming a file removes declarations, virtual units, maps,
  diagnostics and index rows together.
- A failed, cancelled or stale candidate never advertises an old result as
  current; the previous result remains explicitly tied to its old snapshot.
- LSP overrides participate in the same result identity and cannot publish over a
  newer document version.

Pinned Emmy touched-file updates do not establish complete cross-file semantic
invalidation. Retain conservative reindexing until narrower parity is observed.
Cancellation remains cooperative around synchronous native calls.

<a id="outputs-and-corrections"></a>

## Native annotation generation

<a id="preserve-the-existing-pipeline"></a>

### Preserve one pipeline

Read a selected Git revision and generated-document TOC, parse Lua as data, use
the restricted evaluator/normalization, apply guarded corrections and render
ordinary `.lua` once. The same ordinary Library reader consumes that output.

Share pure TOC selection where useful, but do not instantiate Project/Graph/Store
merely to parse a generated API TOC. Resolve the ref once; enumerate regular tree
entries once; batch selected blob reads through bounded cancellable owned child
I/O. Kill/reap on cancellation. Keep path/mode/encoding/output bounds.

### Project missing semantics explicitly

Extend the existing `native_model`/emitter, not a second model:

- separate client flavor from API environment;
- apply member environment as well as parent system environment;
- model `MayReturnNothing`, `NilableContents`, numeric stride-group positions,
  field mixins and selected restriction/origin facts;
- preserve defaults including false/zero, receiver style, arrays, callbacks,
  ordered result packs, enum/open-string distinction and explicit widget bases;
- resolve named types from selected API/small declared supplemental catalogs;
- retain unresolved/unrepresentable forms as located limitations, never silent
  `any` widening.

The existing narrow first-return Secret/predicate projection remains useful but
is not complete Secret-flow coverage.

### One optional generated-Library manifest

Keep `source-report.json` as the single optional producer/consumer boundary.
`wow-annotations` owns shared serializable manifest/mapping DTOs. The builder
records selected revision/flavor/context, generated file path/digest/length,
projection limitations, exact mappings and only typed facts required by
implemented checks.

Project returns exact optional bounded manifest bytes; Service validates and
consumes the same DTO. Plain `.lua` Libraries remain valid for typing and limit
only dependent WoW checks. Do not require the original checkout, embed the whole
source corpus, add a converter or retain the unrelated old native-artifact
envelope.

Corrections identify exact target field, expected old value, replacement,
source/reason and selected target. Conflicts or changed inputs retain the original
fact plus explicit problem. Generate into a new tool-owned output directory and
publish only a complete result; never overwrite user source.

## Typed source relationships

Keep only relations required by an actual diagnostic or navigation consumer:
resolved callable, selected dependency/include, handler, widget base, explicit
mixin and API origin. Use existing typed Project/Emmy facts directly. Do not
serialize them through generic pack/plan/matcher/proof graphs.

An API rule needs the original receiver binding and selected API origin, not only
receiver spelling. Carry/query Emmy lexical identity so a local named `C_*`
cannot become a Blizzard namespace. Unknown alias/dynamic provenance remains
unknown; do not create another resolver.

Preserve distinctions hidden by old wrappers:

- recorded spelling is not a resolved callable;
- possible is not resolved;
- `UsesApi` is not necessarily a call;
- load direction, source order, XML containment, inheritance and runtime parentage
  are different relations;
- negative findings require complete unconflicted evidence for the selected
  domain;
- selected APIDocumentation absence is not proof of all runtime/global absence.

For Secret checks, malformed/stale fact identity remains an input error, while an
unresolved coherent producer is semantic uncertainty and limits only that check.

## Blizzard UI index, search and context

The index is the next feature after addon checking, never its backend. Reuse the
same selected exact bytes, load/XML relations and existing Emmy symbols. Store
one current rebuildable SQLite schema:

- selected sources/revisions/flavors;
- files with exact body text;
- symbols with spans;
- only relations consumed by navigation;
- FTS rows over source/documentation body text.

Use exact symbol lookup separately from documented token/phrase FTS. Bind SQL
parameters and escape FTS query syntax; binding alone does not make raw `MATCH`
text literal. Do not promise arbitrary substring/regex search.

Reuse useful existing direct retained-shard SQL, opt-in Reference documentation
search, one-scan neighbor batching and exact source-excerpt validation. Replace
nested SQLite/hex/JSON, replay hydration, response journals and fixed-point page
sizing only as their readers move.

Update file/body/symbol/relation/FTS rows transactionally. Reopen/query without
Project/Emmy/Graph reconstruction. A direct disk excerpt is allowed only when
content identity matches the indexed revision. Add real SQLite interruption or
progress handling and bounded lock wait; row limits do not preempt a running
statement.

### Small navigation operations

| Operation | Required result |
| --- | --- |
| Outline | selected package/TOC/files or file declarations with exact load order and source locations |
| Exact symbol lookup | distinct declarations in explicit addon/UI/API scope |
| Context around a hit | matching exact excerpt, declaration and direct known load/reference/type relations with finite limits |
| Text search | deterministic token/phrase matches over indexed bodies for exact selected revision/flavor |

Complete zero matches, valid empty index, missing index, incomplete input and
truncated output are distinct. Keep complete result items at limits. Source text
is quoted data; choose safe readable fences/transport escaping without modifying
captured bytes/ranges.

## Results and interfaces

A direct check returns selected target summary, diagnostics and limitations. A
source query returns located hits/context. Serialize each typed result once at
the CLI/LSP/MCP boundary; no self-hash, frozen bytes, proof manifest or response
journal.

CLI text displays actual message/code/severity/location and a concise limitations
section. JSON serializes the same result. Failure threshold changes exit status,
not diagnostics.

LSP/MCP call the same Service owners. LSP binds diagnostics to exact document
version and negotiated encoding, accepts increasing version gaps and returns
standard empty/null results. MCP accepts bounded extension metadata, ignores
valid unknown notifications without response and advertises only implemented
tools. Protocol request IDs remain transport correlation, not durable content
identity.

Preserve admission checks after decoding: paths, source spans, target selection,
references and owner identity are validated once at the real boundary. Preserve
primary errors and locations alongside secondary cleanup failures.

## Completion and verification

Functional completion requires connected ordinary generation, addon checking,
source updates, advertised frontend behavior and direct search/context, including
missing/ambiguous/partial/truncated input. Then record proportional exact
commands/protocol sequences and results in the trace register.

During implementation follow [AGENTS.md](../AGENTS.md): code first, formatting and
affected strict Clippy; tests remain deferred until functional completion by
owner direction. Documentation, compilation and Clippy never promote an
unexecuted workflow to working.