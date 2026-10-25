# Implementation: finish the WoW linter

Read [current status](CURRENT_STATUS.md), [architecture](ARCHITECTURE.md),
[WoW behavior](WOW_LINTER.md) and the ordered
[queue](REFACTOR_QUEUE.md). Source decisions and observation status live in
[IMPLEMENTATION_TRACE_REGISTER.md](IMPLEMENTATION_TRACE_REGISTER.md).

## Current checkpoint

PRs #246–#250 already delivered:

- complete configured Emmy diagnostic payloads;
- `ConfiguredAnalyzerSession` over exact Main/Library/optional virtual snapshots,
  Lua 5.1, embedded STD and caller diagnostic settings;
- diagnostics-only versus explicit semantic collection;
- direct `wow-service::addon_check::check_addon` and its diagnostic/limitation
  result without the legacy generic platform;
- compiled Emmy identity without fake compatibility-report hashes;
- exact Main snapshot admission for configured-owner reuse.

Do not recreate those owners. The missing public path is ordinary
configuration/acquisition/context -> exact snapshots -> existing `check_addon` ->
frontend rendering and status. No ordinary addon command has been observed.

## Code ownership

| Responsibility | Owner to reuse | Remaining change |
| --- | --- | --- |
| Blizzard source/generation | `apps/wow-reference-builder/src/{lib,io,catalogs}.rs` | one selected bounded Git read pipeline and one generator |
| API facts/corrections | `wow-reference/src/native{,_model,_corrections,_aliases}.rs` | preserve reviewed facts/origins without making Reference mandatory for type checking |
| Annotation output | `wow-annotations`, `wow-ketho-literals` | one Lua render plus optional shared manifest/maps |
| Lua analysis | `wow-emmy::ConfiguredAnalyzerSession` and existing semantic owners | reuse delivered owner; add only demonstrated mapped context/update seams |
| Addon/load model | `wow-project/src/{disk,load,xml_lua,xml_bindings,analyzer}.rs` | ordinary target, selected acquisition, typed order/context/triggers/bindings and pure virtual-unit preparation |
| Direct check | `wow-service/src/addon_check.rs` | ordinary config orchestration and selected WoW checks |
| CLI/LSP/MCP | `apps/wow/src`, `wow-service/src/frontend` | thin adapters to the same Service owners |
| Current source index | `wow-store` plus useful direct query/excerpt code | one direct SQLite database over selected current data |

Do not add crates. Remove or merge old crates only after real callers move.
Domain owners do not depend on Service or applications.

## Addon configuration and Emmy analysis

### Ordinary configuration

The public configuration contains:

- addon `root`;
- optional explicit `toc`;
- supported `flavor`;
- explicit `channel` when product gating matters: `stable`, `ptr` or `beta`;
- optional positive **target** `target_interface`;
- one or more local `library` directories;
- optional `dependency_roots`;
- optional selected WoW `checks`;
- optional pinned upstream `emmy_diagnostics`;
- `fail_on`, default `error`.

The positive target Interface is not the same as the source TOC value: selected
Blizzard source can contain `Interface: 0`, which is retained as unknown/sentinel
compatibility rather than malformed text.

Resolution rules:

1. CLI overrides config. Relative CLI directories use cwd; relative config
   directories use the config file directory. Relative TOC uses resolved root.
2. Require explicit flavor and at least one explicit Library directory. Do not
   infer flavor/channel from folder, ref, Interface or addon title, discover a
   different Library, download data or regenerate declarations.
3. `library` is a nonempty path array. Repeated CLI `--library` replaces config.
   Library files are declarations, never executable addon units.
4. Preserve explicit TOC through mismatch or unknown compatibility and report it.
   Without one, choose only an unambiguous supported candidate. First support
   retail/Mainline, exact `<addon>.toc` and `<addon>_Mainline.toc`; case-fold
   collisions/equal candidates require choice. A zero-Interface candidate does
   not outrank a positive compatible candidate without a documented product rule.
5. Ordinary installed-client default may be `stable` only when that target is
   explicit. Use the reviewed donor model for `OnlyBetaAndPTR: 1`: PTR/Beta
   included, Stable excluded and unknown channel unresolved. Preserve the donor
   evidence class. Channel is separate from flavor, game/glue and global/secure
   context.
6. Use explicit target Interface, otherwise compatible selected-Library metadata.
   Conflicting targets are configuration errors. Without either, only dependent
   checks are limited. Never infer target Interface from the addon TOC or use
   source sentinel zero as the target.
7. Dependency roots are AddOns-like parents. Select only named required and
   available optional transitive dependencies using the finite aliases:
   - required: `Dep`, `Dependencies`, `DependsOn`, `RequiredDep`, `RequiredDeps`;
   - optional: `OptionalDep`, `OptionalDeps`.
   Reject ambiguity. Missing required dependency limits dependent work; absent
   optional dependency is normal.
8. Model `LoadWith` as the reviewed donor-backed reverse trigger, not a
   dependency. Preserve source package/target/resolution/order and evidence class
   without granting prior availability or claiming complete client timing.
9. Decode `emmy_diagnostics` as pinned `EmmyrcDiagnostic`; invalid fields are
   errors. No `.emmyrc` discovery. `diagnosticInterval` is scheduling only.
10. `fail_on` changes exit status only. Empty/omitted `checks` runs no WoW rule
    while Emmy diagnostics remain enabled. Unknown check names are errors;
    recognized unavailable checks are scoped limitations.

No ordinary user supplies Project IDs, generations, profiles, reports, digests,
receipts or graph budgets.

### First implementation slice

The first public result is an offline Lua/type check with an existing Library:

1. resolve ordinary target and selected TOC/dependencies/triggers;
2. capture exact bounded physical, UI XML, implicit bindings and Library input;
3. prepare mapped virtual Lua units and a validated exact Project context;
4. build path-sorted Main/Library/virtual snapshots without discarding selected
   package/TOC/XML/callback order and phase;
5. call existing `wow_service::addon_check::check_addon`;
6. render diagnostics plus limitations and apply documented exit status.

No WoW rule is enabled by default. Missing addon chunk, UI handler, binding or
unresolved TOC security context stays explicit `Partial`; never drop the unit and
report clean. This path requires no SQLite, graph, publication, replay, Reference
or source regeneration.

<a id="one-direct-check-operation"></a>

### One direct check operation

1. resolve public config and supported target;
2. select TOC, dependencies, triggers and product-channel gates;
3. parse/retain compatibility and security metadata, including source sentinel
   zero and unsupported `Secure` values;
4. capture exact bounded physical/XML/bindings/Library source;
5. build typed package/unit execution context and exact selected order;
6. prepare mapped UI-handler/free-inline/binding virtual units;
7. open or reuse one compatible `ConfiguredAnalyzerSession`;
8. supply only supported source-mapped addon chunk/callback context;
9. collect language diagnostics and facts requested by selected checks;
10. run those direct checks;
11. return target summary, diagnostics and scoped limitations.

Reference/Rules exist only when a selected implemented check needs them. Optional
component health never becomes a global clean gate.

### Load units and addon context

#### Selected acquisition

Reuse the existing TOC/XML parser, exact source records and maps. Do not replace
selection with recursive collection of every Lua/XML file or another parser.
Acquire only explicit/unambiguous TOCs and relevant dependency closure.

Use one finite metadata/directive table shared by TOC and package projection.
Preserve source span, predicates, originating record and evidence class for all
facts. Recognize:

- dependency aliases above;
- source-located `UseSecureEnvironment`, `LoadIntoEnvironment` and
  `SuppressLocalTableRef` plus their reviewed donor-backed adapter semantics;
- `LoadOnDemand`, bootstrap and supported `LoadFirst`/phase, with source versus
  donor evidence retained;
- `OnlyBetaAndPTR`, explicit product channel and the reviewed donor gating model;
- source-located `LoadWith` plus the reviewed donor reverse-trigger model;
- bounded `DefaultState` lifecycle metadata;
- `SavedVariables`, `SavedVariablesPerCharacter` and
  `SavedVariablesMachine` declarations;
- `Interface: 0` as unknown/sentinel compatibility;
- `Secure` as source-backed **unsupported semantic state** until reviewed source
  establishes its meaning.

Do not alias `Secure` by name to `UseSecureEnvironment`, per-unit environment,
signing, taint or private-table policy. Independent language diagnostics survive;
checks needing that context stay limited.

`DefaultState` does not suppress analysis of an explicitly selected root.
SavedVariables are declarations only; never open user data, infer schema or
initialize values as non-nil.

Unknown cosmetic/custom metadata may be retained without limiting language work.
Unknown selection/environment/namespace/order/callback directives remain located
limitations.

Reduce predicates by operator:

- allow-list: any known true includes; all known false excludes; otherwise
  unresolved;
- exclude-list: any known true excludes; all known false includes; otherwise
  unresolved;
- conjunctive gates: any known exclusion excludes; unresolved remains only when
  no gate already excludes.

Retain unknown predicates as source evidence even when another gate determines
exclusion. Reachable unknown scope never reports clean.

See [TOC_LOAD_CONTEXT_AUDIT.md](TOC_LOAD_CONTEXT_AUDIT.md) and
[TOC_TRIGGER_BINDINGS_AUDIT.md](TOC_TRIGGER_BINDINGS_AUDIT.md).

#### Exact load/context handoff

`LuaWorkspaceSnapshot` is path-sorted/content-addressed, not runtime order.
Project passes a separate immutable validated value containing:

- exact dependency/package/TOC/XML unit order;
- snapshot role, canonical path and content digest;
- originating load record or virtual-unit identity;
- addon/package identity;
- effective environment/private-table policy;
- supported load/callback phase;
- product channel and trigger evidence;
- source Interface value and compatibility state;
- raw/typed supported or unsupported security metadata.

Reject missing, foreign, duplicate or digest-mismatched entries. Never claim
Emmy registration order emulates the client. Changes to these facts change the
context/rebuild identity.

#### Physical and virtual Main

Main may be physical Lua, mapped XML Lua, both, or a valid empty language scope
with load facts. Remove physical-file prerequisites. XML-only addons need no
dummy file.

Keep exact entity/newline/discontinuous maps. A virtual diagnostic is visible
only after exact projection. One malformed included XML unit produces a located
limitation for dependent scope and preserves independent units; unsafe paths and
invalid selected TOC remain hard refusal.

#### Implicit binding documents

A selected addon may contain sibling `Bindings<suffix>.xml` absent from its TOC.
Project must derive a finite candidate list from reviewed flavor/product donor
rules, select at most one under the held root, treat absence as normal, reject
ambiguity/case collision, capture exact bounded bytes, and parse `<Binding>`
bodies through existing quick-xml/source maps. Preserve the evidence class; the
reviewed donor does not establish exact client discovery precedence or timing.

Mapped binding callback units receive the reviewed source-and-donor-backed
`keystate` adapter, addon identity, effective supported environment, exact
document identity and callback phase. They are not top-level TOC chunks and do not receive addon
ellipsis/private-table arguments. Malformed binding scope is a located limitation and preserves
independent diagnostics. Synthetic wrappers never become user locations/facts.

#### Addon chunk context

For supported external Lua chunks, provide one stable private table per addon and
separate lexical chunks. Dependency addons retain distinct tables. Transform
only chunk-owned top-level ellipsis expressions in memory using the mapped helper
seam in [EMMYLUA_UPSTREAM.md](EMMYLUA_UPSTREAM.md#addon-chunk-context).

Keep external chunks, free inline scripts, reviewed UI handler adapters,
reviewed binding callbacks, suppressed private tables, reviewed secure overrides,
unresolved `Secure` metadata and bootstrap/load-first contexts distinct. Only
evidence-backed handler/callback adapters receive implicit parameters, and their
client-authoritative or donor-derived evidence class remains explicit.

### Updates

Reuse an owner incrementally only while exact membership, product channel,
triggers, Interface/security metadata, ordered context, Library identity and
analyzer config are unchanged. TOC/XML/bindings/dependency/trigger/Library/
channel/context/config change rebuilds selection and owner cold.

Delete/rename removes declarations, virtual units, maps, diagnostics and index
rows together. Failed/cancelled/stale candidates never become current. LSP
overlays participate in the same identity. Keep conservative semantic rebuild
until narrower invalidation is observed correct. Cancellation around synchronous
Emmy calls is cooperative.

<a id="outputs-and-corrections"></a>

## Native annotation generation

<a id="preserve-the-existing-pipeline"></a>

### Preserve the existing pipeline

Resolve one Git revision and generated-document TOC; parse Lua as data; use the
restricted evaluator/normalization; apply guarded corrections; render ordinary
Lua once. Share pure TOC selection where useful, but do not instantiate
Project/Graph/Store for generated API documents.

Enumerate regular Git entries once and batch selected blob reads through bounded,
cancellable owned child I/O. Kill/reap on cancellation. Keep path/mode/encoding/
output limits.

Project missing semantics in existing `native_model`/emitter:

- flavor versus API member environment;
- `MayReturnNothing`, `NilableContents`, ordered stride positions;
- mixin and selected restriction/origin facts;
- defaults including false/zero/empty string;
- receiver style, arrays, callbacks, ordered result packs, enums/open strings and
  explicit widget bases;
- unresolved/unrepresentable named types as located limitations, never silent
  `any`.

The narrow first-return Secret/predicate projection remains useful but incomplete.

Keep one optional `source-report.json` DTO owned with annotation output. It binds
selected revision/flavor/context, generated file identity, projection
limitations, exact mappings and only typed facts required by implemented checks.
Project returns exact bounded bytes; Service validates/consumes them. Plain Lua
Libraries remain valid for typing. No converter, original checkout or old native
artifact envelope.

Corrections bind exact target/old value/replacement/source/reason and selected
target. Conflicts preserve original fact plus explicit problem. Publish only a
completed tool-owned output directory; never overwrite user source.

## Typed source relationships

Keep only relations required by real diagnostics/navigation: resolved callable,
selected dependency/trigger/include, handler/binding callback, widget base,
explicit mixin and API origin. Use Project/Emmy facts directly, not generic
pack/plan/matcher/proof graphs.

Preserve distinctions: spelling is not resolution; possible is not resolved; API
use is not necessarily a call; dependency, trigger, load order, callback
availability, compatibility, execution security, XML containment, inheritance and
runtime parentage are different; negative findings need complete unconflicted
selected-domain evidence.

API checks require original receiver binding and API origin; a local named `C_*`
is not a Blizzard namespace. Secret checks treat malformed/stale identity as
input error and coherent unresolved producer as scoped uncertainty. TOC `Secure`
uncertainty is separate from API `SecureOnly` and Secret-value flow.

## Blizzard UI index, search and context

The index follows useful addon checking; it is never the check backend. Reuse the
same selected exact bytes, context, XML/binding maps and Emmy symbols. Store one
rebuildable SQLite schema: exact source/revision/flavor/channel, files with body,
symbol spans, only consumed relations, and FTS over source/documentation bodies.

Use separate exact symbol lookup and documented token/phrase FTS. Bind SQL
parameters and escape FTS syntax. Update files/symbols/relations/FTS
transactionally. Reopen/query without Project/Emmy/Graph reconstruction. Disk
excerpt requires matching content identity. Add real SQLite interruption and
bounded lock wait.

Reuse useful direct retained-shard SQL, opt-in Reference documentation search,
one-scan neighbor batching and exact excerpt validation. Replace nested
SQLite/hex/JSON, replay hydration, journals and fixed-point pagination only as
readers move.

<a id="small-navigation-operations"></a>

### Small navigation operations

| Operation | Result |
| --- | --- |
| Outline | selected packages/TOCs/files/callback units with exact order/context/source |
| Exact symbol lookup | distinct declarations in explicit addon/UI/API scope |
| Context around a hit | exact excerpt/declaration plus direct known load/trigger/callback/reference/type relations |
| Text search | deterministic token/phrase matches for exact selected revision/flavor/channel |

Complete zero matches, missing index, incomplete input and truncation are distinct.
Source is quoted data with readable safe transport escaping; captured bytes/ranges
remain unchanged.

## Results and interfaces

A check returns target summary, diagnostics and limitations. A source query
returns located hits/context. Serialize once at CLI/LSP/MCP; no self-hash, proof
manifest or response journal.

CLI preserves message/code/severity/location and limitations. Failure threshold
affects exit only. LSP binds exact version/encoding, accepts increasing version
gaps and standard empty/null results. MCP accepts bounded extension metadata,
ignores valid unknown notifications and advertises only implemented tools.

Validate paths, spans, target, context entries and owner identity once at the
real boundary. Preserve primary error/location alongside cleanup errors.

## Completion and verification

Functional completion requires connected generation, ordinary checking, source
updates, advertised frontends and direct search/context, including ambiguous,
missing, partial and truncated input. Record exact commands/protocol sequences and
results in the trace register.

Follow [AGENTS.md](../AGENTS.md): code first, formatting and affected strict
Clippy; tests remain deferred until functional completion by owner direction.
Documentation, compilation and Clippy never promote an unexecuted workflow to
working.
