# What the WoW linter must understand

This is intended behavior, not a claim that the current code implements every
row. The [audit](AUDIT.md) identifies existing gaps and the
[review register](AUDIT_REGISTRY.md) records what was actually checked; the
[implementation guide](IMPLEMENTATION.md) maps them to code. Sources and reusable
algorithms are linked in [DONORS.md](DONORS.md).

## 1. Select the right program and environment

Keep these selections distinct:

| Selection | Meaning |
| --- | --- |
| Blizzard source revision | Exact content used to generate API facts; a moving branch is resolved once. |
| Client flavor/game type | Which client variant and TOC/load conditions are selected. Never union Mainline and Classic declarations to make errors disappear. |
| Game/Glue load context | Which UI code can load according to the selected TOC. A system named `Glue` does not determine this. |
| API execution context | Ordinary addon code by default. `SecureOnly` metadata is distinct from client flavor and must be considered at system **and member** scope. |
| Main, dependencies and Library | Main is the addon under diagnosis; declared dependencies and libraries provide context. They are not arbitrary files found nearby. |

The distinction is observable in the reviewed Blizzard source: a
[Mainline TOC](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_UIPanels_Game/Blizzard_UIPanels_Game_Mainline.toc)
contains separate game/game-type directives, while
[FrameScript](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_APIDocumentationGenerated/FrameScriptDocumentation.lua)
contains a `SecureOnly` function inside an `All` system. `All` is not evidence
that an API exists in every WoW flavor. A library generated for one selection
must not silently satisfy another.

Existing annotation directories remain usable without a generator report.
Their types can be analyzed; absent source/context metadata makes the affected
WoW-specific checks unavailable, not the entire Lua analysis unusable.

## 2. Reuse Lua analysis; model the WoW environment

Use EmmyLua's explicit Lua 5.1 setting. Do not enable LuaJIT or arbitrary newer
syntax to make a file parse. A genuine Blizzard syntax extension requires a
source example and a narrow upstream/adaptor solution, not a second parser.

Load upstream standard declarations for Lua typing, then the selected WoW
Library and addon. Stock Lua declarations do **not** prove that a host API exists
in WoW. Keep runtime availability separate and keyed to the resolved symbol;
a local variable shadowing a standard/global name is not a WoW API call. Reuse
Blizzard/Ketho declarations for WoW aliases and extensions. Do not globally
suppress undefined names, accept every `C_*` name, or type unknown values as `any`
to hide missing environment support.

WoW loads addon chunks through TOC/XML, not by inventing a `require()` graph.
Model addon-name/private-table chunk arguments and their shared namespace within
one addon using upstream-supported context/virtual declarations. Preserve each
file's lexical scope. Generated context must map back to user source and never
be written into addon files. The [Wowless loader reference](DONORS.md#wowless)
is a useful comparison for this integration, not runtime proof.

Keep the private table distinct for each addon, including selected dependencies;
sharing within one addon must not merge their lexical locals or all addon
namespaces. The current package path prefix provides file identity only. The
traced Main/Library inputs do not yet bind chunk arguments to a per-addon table.

That default context applies only where the selected load unit justifies it.
The reviewed Blizzard
[RecruitAFriend TOC](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_RecruitAFriend/Blizzard_RecruitAFriend.toc)
contains `SuppressLocalTableRef: 1`; other selected source uses secure load
environments. These are not cosmetic metadata. The comparative Wowless loader
also treats suppressed tables and bootstrap contexts separately. Preserve the
directives and leave unsupported context unknown instead of injecting the same
non-nil private table into every Blizzard-owned chunk.

SavedVariables names come from the selected TOC. Declaration does not make their
contents non-nil before initialization, provide a complete schema, or justify
reading the user's saved-variable files. Bundled Ace/LibStub libraries are normal
selected dependencies; dynamic embed lookup stays unresolved unless a small
explicit adapter can establish it.

## 3. Preserve the type contract

| Blizzard fact | Required treatment |
| --- | --- |
| Ordered `Arguments` / `Returns` | Preserve order, Lua multiple-result behavior and receiver style. Let EmmyLua check types and argument counts. |
| `Type`, `InnerType`, `Nilable`, `NilableContents` | Preserve aliases, arrays/unions, container nilability and element/pack-slot nilability separately. Missing metadata and explicit `false` are different. |
| `Default` | Preserve `false`, zero and empty string as present values. Do not use truthiness to decide optionality. |
| `MayReturnNothing` | Keep the callable's zero-result alternative, even when individual return fields say `Nilable = false`. At assignment, absent results become nil. |
| `StrideIndex` / varargs | Preserve the fixed prefix, ordered repeated group and each position within it. A positive index is not merely a boolean variadic flag. Unsupported packs are visible projection gaps. |
| `CallbackType`, event payload | Preserve ordered parameters and returns. A callback name alone is not a complete signature. |
| Structure fields and `Mixin` | Retain the data type and explicit mixin relation; do not flatten guessed inheritance or duplicate every method. |
| `ScriptObject`, widget bases | Use explicit reviewed class/receiver/base relationships. Folder or type-name similarity does not prove inheritance. |
| Enumerations / literals | Keep closed enums closed and extensible string domains open. Membership claims require a sufficiently complete selected domain. |
| Member `Environment`, secret/restriction flags | Carry them beside type facts; ordinary annotation syntax cannot express every game constraint. |

The reviewed Ketho emitter is a useful donor, but it does not lower all of this
metadata. In particular, `MayReturnNothing`, member `Environment`, `Mixin` and
several restriction fields require work in this project, as do multi-position
stride packs and `NilableContents`. See
[Ketho/native projection](KETHO_RUST_PORT.md).

For example, the selected Blizzard source gives `GetMetaPowerInfo` three return
fields with `StrideIndex` 1, 2 and 3; the current emitter rejects that shape.
`C_EquipmentSet.GetItemIDs` has both `MayReturnNothing` and a non-nil table whose
contents may be nil. These are three separate questions: whether results exist,
whether the returned table is nil, and whether an indexed element is nil. The
[source traces](AUDIT.md#stride-packs-and-nilable-contents) identify the input and
the current losses. Do not flatten them into one optional marker.

Use EmmyLua's supported types/overloads. If the engine cannot express correlated
optional returns, retain that fact and use a documented conservative projection
(e.g. nullable result slots); do not claim independent nullable slots exactly
represent the original tuple. Unsupported expressions or unresolved named types
remain visible. Independent valid declarations should survive one failed
neighbor; the resulting library is explicitly partial.

## 4. TOC and XML form the load model

Use one explicitly or unambiguously selected TOC and assess compatibility
separately. Otherwise list candidates. Keep declared file order and nested XML
include/script order. Sorting files for deterministic storage must not change
load semantics. Normalize separators for lookup while retaining original spans;
report missing paths, case collisions, cycles and invalid references.

Required and optional dependencies are different. Resolve only configured addon
roots. An optional dependency being absent is not an error; an unknown required
dependency prevents dependent availability claims. LoadOnDemand and conditional
loading remain conditional. A TOC Interface mismatch is a compatibility diagnostic,
not permission to abandon all useful analysis or guess another client.

Cosmetic/custom TOC metadata must not mark all type checking incomplete. An
unknown directive that can change loading affects the load-dependent checks.
Keep selected, excluded and unresolved files distinct. Report a Lua file omitted
from the load plan only when that warning is useful and its exclusion is proven.

Track file discovery, XML parsing, inline Lua analysis and handler-context
support separately. An extracted handler body must not keep a blanket
`InlineLuaNotAnalyzed` blocker on file closure after its syntax has been
analyzed. Unsupported handler context can still limit the relevant semantic
check. Inventory membership also differs from load reachability: finding a file
under a configured root does not prove that the selected addon loads it.

An addon may contain XML with inline Lua and no physical `.lua` file. Analyze
those virtual units and retain load/template diagnostics even when there is no
language body. The current Main admission rejects an empty physical inventory;
it must be corrected before the existing inline-analysis path can handle this
case. A successfully selected empty language scope and failed source acquisition
must remain different outcomes.

Use the existing quick-xml parser and source maps. Resolve `Include`/external
`Script`, template inheritance, widget types, script handlers and explicit mixins
only from selected data. XML inheritance, lexical containment and runtime object
parentage are different relationships. Unknown templates or dynamically selected
mixins do not establish a missing method.

Inline Lua needs the context justified by its exact XML script kind. Supply a
handler receiver and callback parameters only when that element's signature is
source-verified; otherwise leave that context unknown. A free inline `Script`
body and an external Lua chunk are different units. Do not replace a
handler's `...` with addon arguments or assume every inline body receives the
external chunk's private table. The current unwrapped inline analysis does not
provide these contexts. Synthetic wrappers must have
source maps; wrapper-only errors must not be reported against invented user
locations. XML entities/newline transformations can map a virtual range to
multiple original spans. Use the actual mapping; never add one constant offset.

## 5. Checks to deliver

| Check | Reliable input and result | Boundary |
| --- | --- | --- |
| Lua syntax/types | EmmyLua diagnostics with the selected environment and Library | Preserve upstream text/code/severity; a warning is not forced to Error. |
| TOC/XML integrity | Selected paths, references and known load conditions | Static load facts do not prove that callbacks execute. |
| API context/availability | Resolved symbol plus selected source/context metadata | A missing generated declaration is not proof that Blizzard removed an API. Say unresolved in this environment unless stronger evidence exists. |
| Widgets/templates/mixins | Known object type, explicit inheritance and script signatures | Dynamic object creation/dispatch stays unknown. |
| Events/callbacks | Known registration API, literal event and documented payload/handler | Frame events, EventRegistry signals, Ace callbacks and CVars are separate systems. An event-like string alone does not connect them. |
| Deprecation/replacement | Explicit selected-source metadata or reviewed correction | Do not invent a replacement from a similar name or an old flavor. |
| Secret/restricted use | Exact API/value metadata and a supported local operation | Advisory where execution state is unknown; no whole-program taint/combat proof. |

Start with types, load integrity and API context. Add each remaining check as a
small direct function over existing typed facts. Select checks **before** their
work runs. Share relevant facts, not a JSON recognizer-pack compiler or a plugin
registry. An unrequested rule cannot make a completed type check partial.

Apply that boundary to result status as well as execution. Report which selected
check/file lacks necessary evidence. General component health, unsupported
optional features and missing metadata for a disabled check are not by themselves
limitations of the requested analysis. Keep actual input losses and enabled-check
limitations visible; do not achieve a clean status by discarding them.

At the reviewed baseline, the active rule providers are only API existence and
a narrow local secret-value operation check. Event payload/handler validation,
broader callback summaries and widget checks in this table are work to deliver;
generating a callback annotation does not connect a registration site to a handler.

Before attributing a use to a WoW API, establish its receiver binding and selected
API origin through Emmy. A local with the same `C_*` spelling can have an unrelated
concrete type. The current spelling-based consumer can misclassify its missing
member; [the audit](AUDIT.md#a-typed-local-receiver-can-enter-wow-api-absence-checking)
traces this conditional path. Exact catalog absence alone cannot fix a wrong
binding. Keep unknown provenance unknown and preserve ordinary Lua diagnostics.
The existing API rule covers direct static member calls, with a receiver-name
filter; it does not establish global-function or namespace-alias coverage.

## 6. Secret values and protected behavior

Preserve `SecretReturns`, conditional secret flags, field `SecretValue`,
`SecretArguments`, `HasRestrictions` and `SecureHooksAllowed` with their original
values and source. A generic restriction flag alone does not identify the reason
or prove that the current call is forbidden. Source facts are valuable even when
a rule cannot yet interpret them.

Reuse the existing narrow local-flow work only where it resolves the exact
producer, value and operation. A guard must apply to that value and dominate the
use; a similarly named variable, a guard after use, copying, `pcall`, conversion
or formatting does not itself prove permission to inspect it. Conditional
secrecy, unknown aliasing/calls and unsupported control flow remain unknown.
Do not turn absence of a supported finding into a runtime-safety certificate.

Distinguish unresolved semantics from invalid facts. A correctly linked local
initializer may still have an unresolved member call. The current Secret
consumer can treat this as a fact-graph error when matching restriction metadata
exists; [the source trace](AUDIT.md#an-unresolved-secret-producer-is-treated-as-a-broken-fact-graph)
describes the condition. Report the affected check/use as not evaluated while
retaining real rejection of stale or inconsistent identities. Missing restriction
rows in a partial catalog do not establish unrestricted behavior.

Keep this metadata separate from the ordinary number/string/boolean type model;
secret values are not assumed to be wrapper objects. Do not build a parallel
whole-program dataflow engine for the first usable linter. Show a useful
source-linked advisory where more precise analysis is unavailable. Do not propose
automatic guard insertion that changes addon control flow.

## 7. Useful output and updates

A diagnostic contains a stable code, severity, explanatory message, file/range
and optional related declaration/location. Keep findings separate from analysis
problems such as an incomplete library, unknown load condition or cancellation.
Known findings survive partial input; an unexamined scope never reports clean.
Deduplicate only the same code/cause at the same source location; do not hide
independent symptoms behind a root-cause platform.

Internally use UTF-8 half-open byte ranges tied to the exact text. Convert to
editor positions once with the negotiated encoding and document version. Library
or STD declarations are related locations, not fabricated Main-file spans.

A changed/deleted file, TOC, XML include, dependency or Library must invalidate the
facts that depend on it. Initially retain the correct conservative semantic
rebuild. Publish a coherent new result after success; never serve an old result
as current after a failed update. Source search uses matching indexed text, not
old offsets applied to modified disk contents.

Deletion must remove obsolete declarations, load membership, virtual Lua units,
source maps, check results and search rows together. If the effect of a change
is unknown, rebuild the smallest containing scope known to be safe. An older
immutable result may remain available as explicitly old data; it cannot satisfy
a request for the new text. Client LSP versions increase but need not be
consecutive; internal revisions must not impose that extra protocol constraint.
