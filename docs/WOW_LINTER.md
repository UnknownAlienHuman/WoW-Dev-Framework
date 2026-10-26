# What the WoW linter must understand

This is required behavior, not a claim that current code implements every row.
Use [CURRENT_STATUS.md](CURRENT_STATUS.md) for delivery state,
[IMPLEMENTATION.md](IMPLEMENTATION.md) for owners and
[IMPLEMENTATION_TRACE_REGISTER.md](IMPLEMENTATION_TRACE_REGISTER.md) for evidence.

## 1. Select the exact program and environment

Keep these choices distinct:

| Selection | Meaning |
| --- | --- |
| Blizzard source revision | Exact source used to generate API/UI facts; resolve a moving ref once. |
| Client flavor/game type | Retail/Mainline versus other families and TOC game-type conditions. |
| Product channel | Stable, PTR or Beta. `OnlyBetaAndPTR` is not flavor or Interface. |
| Target Interface | Positive compatibility target supplied explicitly or by compatible Library metadata. |
| Source Interface metadata | A selected TOC can carry positive versions or official sentinel `0`; it is evidence, not automatically the target. |
| Game/Glue location | Package/unit selection context. A system named `Glue` does not determine it. |
| Package/file execution context | Package default, per-unit override, private-table policy and load phase. Selection is not execution. |
| TOC security metadata | Source occurrences plus reviewed donor-backed adapters for known directives; unresolved `Secure` remains separate and unsupported. |
| API execution context | Ordinary addon by default. Member `SecureOnly` is distinct from package security and Secret flow. |
| Main/dependencies/Library | Main is diagnosed; selected dependencies provide program context; Library is declarations, not executable source. |

Never union flavors/channels to hide diagnostics. Existing plain Lua annotation
directories remain usable for types; absent metadata limits only checks needing it.

<a id="2-reuse-lua-analysis-model-the-wow-environment"></a>

## 2. Reuse EmmyLua and model WoW around it

Use explicit Lua 5.1 and embedded standard declarations. Stock Lua declarations
do not prove host availability. A real Blizzard syntax extension requires a
source example and narrow upstream/adapter support, not another parser.

Resolve symbols semantically. A local named like a standard global or `C_*`
namespace is not a WoW API by spelling. Unknown aliases/dynamic dispatch remain
unknown; do not suppress undefined names globally or widen missing types to `any`.

External addon chunks are selected by TOC/XML rather than an invented `require`
graph. Pinned Blizzard API documentation defines the ordinary chunk results as
`(addonName, addonTable)`. For supported ordinary chunks, provide those
source-mapped results with one stable private table per addon and separate lexical
chunks. Dependency addons keep distinct tables. Path prefixes are file identity,
not namespace semantics. `AllowAddOnTableAccess` only controls retrieval through
`C_AddOns.GetAddOnLocalTable`; it does not control table injection.

The exact selected unit controls context:

- the reviewed donor models package `UseSecureEnvironment` as the default;
- the reviewed donor models per-file `LoadIntoEnvironment` as an override;
- `SuppressLocalTableRef` is selected restricted/internal metadata with
  unsupported current semantics. The donor passes a `nil` second value, but that
  is not client authority; do not apply the ordinary helper or fabricate a table,
  and keep ellipsis-dependent analysis limited;
- bootstrap/load-first/deferred phases remain distinct;
- free inline UI scripts, reviewed UI handler adapters and reviewed binding
  callbacks are not external addon chunks;
- `Secure` is not assigned any of those meanings without reviewed evidence.

Unsupported context is a scoped limitation, never a fabricated global/private-
table/security value.

SavedVariables names are declarations. Support account, character and machine
scope from the selected TOC, but never read user SavedVariables data, infer a
schema, or assume non-nil contents before addon initialization.

## 3. Preserve the Blizzard type contract

| Blizzard fact | Required treatment |
| --- | --- |
| Ordered arguments/returns | Preserve order, receiver style and Lua multiple-result behavior. |
| `Type`, `InnerType`, `Nilable`, `NilableContents` | Keep aliases, arrays/unions, container nilability and element/slot nilability separate. |
| `Default` | Preserve false, zero and empty string as present values. |
| `MayReturnNothing` | Keep zero-result alternative; do not collapse into container nilability. |
| `StrideIndex` | Preserve fixed prefix and each ordered repeated position. |
| Callback/event payload | Preserve ordered parameters/returns; a callback name is not a signature. |
| Structure `Mixin` | Preserve data fields and explicit mixin relation without guessed inheritance. |
| Widgets/script objects | Use reviewed explicit class/receiver/base relations. |
| Enums/literals | Keep closed domains closed and extensible strings open. |
| Member environment/restrictions | Carry beside types; annotations cannot express every game constraint. |

Concrete reviewed losses include repeated triples in
`C_ArtifactUI.GetMetaPowerInfo` and separate zero-result/table/element nilability
in `C_EquipmentSet.GetItemIDs`. Unsupported exact correlations remain visible;
independent valid declarations survive.

<a id="4-toc-and-xml-form-the-load-model"></a>

## 4. TOC, triggers, XML and bindings form the load model

### TOC selection, Interface and dependencies

Use one explicit or unambiguous TOC and assess compatibility separately. Preserve
original spans and exact declared order. Sorting for identity must not replace
load semantics.

Finite dependency aliases:

- required: `Dep`, `Dependencies`, `DependsOn`, `RequiredDep`, `RequiredDeps`;
- optional: `OptionalDep`, `OptionalDeps`.

Absent optional dependency is normal. Missing required dependency limits dependent
availability. A positive Interface mismatch is a compatibility diagnostic, not
permission to switch TOC/flavor or abandon independent analysis.

Official Blizzard source contains `Interface: 0`. Treat zero as an explicit
unknown/sentinel compatibility claim, not malformed syntax and not a positive
target. It never proves compatibility and does not outrank a positive compatible
candidate during automatic selection without a documented product rule. Explicit
source analysis survives with a scoped compatibility limitation.

### Product channel and conditions

Model Stable/PTR/Beta separately from flavor, Interface and game/glue location.
The reviewed donor model treats `OnlyBetaAndPTR: 1` as PTR/Beta included, Stable
excluded and missing channel unresolved. Use that as an explicit provisional
adapter rule and emit a scoped limitation wherever a result depends on unverified
client parity. Never infer channel from repository name, ref or path.

Reduce predicates by boolean form:

- allow-list: any known true includes; all known false excludes; otherwise
  unresolved;
- exclude-list: any known true excludes; all known false includes; otherwise
  unresolved;
- conjunctive gates: known exclusion dominates unknown.

Retain unknown source evidence even when another gate determines exclusion. A
reachable unresolved unit remains a limitation.

### `LoadWith` is not a dependency

`LoadWith` is modeled as a reverse load trigger from reviewed donor behavior.
Preserve source package, named trigger, resolution state, order and span. The
Blizzard TOC proves the directive exists, not the client's complete lifecycle
timing. It does not prove that the trigger package can use symbols declared later
by the `LoadWith` package and must not enter required/optional closure. Explicitly
selected source remains analyzable when automatic reachability is unresolved.

### Security directives are not interchangeable

`UseSecureEnvironment`, per-unit `LoadIntoEnvironment`, API member `SecureOnly`,
Secret-value metadata and TOC `Secure` are different inputs.

The reviewed Blizzard source contains `Secure: 1`, but reviewed Project/Wowless
paths do not establish its exact meaning. Preserve raw value/span and mark its
semantic state unsupported. Do not alias it to package environment, signing,
taint or private-table policy by name. Checks requiring that context remain
limited while independent syntax/type diagnostics survive.

### Known lifecycle and persistence metadata

`DefaultState` is known bounded lifecycle metadata. It does not suppress static
analysis of an explicitly selected root and must not create global partial.
`LoadSavedVariablesFirst` is known load-order metadata: preserve it, but never
invent SavedVariables contents or types. SavedVariables account/character/machine
declarations are metadata, not data-file access. `EscalateErrorDuringLoad` is a
known restricted error-policy flag and does not make independent language
analysis partial.

Retain localized display metadata, icon/category/group fields and `X-*` custom
metadata without limiting language work. An unrecognized package metadata key is
reported and limits only checks whose selection/context could depend on it; it
does not invalidate already enumerated files. Unknown inline/file directives can
change selection and therefore remain located unit limitations.

### Exact selected order and context

`LuaWorkspaceSnapshot` is path-sorted/content-addressed for identity; it is not
load order. Pass a separate immutable validated context containing exact package/
dependency/TOC/XML/callback order/phase, snapshot role/path/digest, originating
record/unit, addon identity, environment/private-table policy, product channel,
trigger evidence, source Interface/compatibility state and reviewed or unsupported
security metadata. Reject missing, foreign, duplicate or mismatched entries.

### UI XML

Use existing quick-xml parsing and exact source maps. Resolve only selected
Include/external Script, template inheritance, widget/script association and
explicit mixins. XML containment, inheritance and runtime parentage are distinct.

Analyze XML-only addons without dummy Lua. One malformed included XML unit is a
located failure of dependent scope and preserves independent units; unsafe paths
and invalid selected TOC are hard refusal.

Free inline scripts and reviewed handler adapters need different context. Supply
`self` and parameters only where selected source or an explicitly reviewed donor
supports that handler signature. If the adapter is donor-derived rather than
client-established, report the corresponding scoped limitation. Never replace a
handler's `...` with addon chunk arguments. Synthetic wrappers require exact maps
and never produce invented user locations.

### Implicit `Bindings*.xml`

A selected addon may have `Bindings<suffix>.xml` not listed in its TOC. Discover
at most one exact candidate under the held root using finite selected flavor/
product precedence. Absence is normal; case/precedence ambiguity is explicit.

Parse `<Binding>` Lua bodies with existing XML/source maps as mapped callback
units. Supply the reviewed source-and-donor-backed `keystate` parameter. A binding
body is not a top-level TOC chunk and receives no addon ellipsis/private-table
arguments. Keep a separate callback phase; donor after-files ordering is not
client-authoritative and cannot justify exact timing or negative availability.
Static analysis cannot prove that the binding event fires.

Malformed selected bindings XML or unsupported constructs limit binding analysis
while preserving independent diagnostics. Synthetic callback wrappers stay out
of user locations/API/flow facts.

Detailed traces:
[TOC_LOAD_CONTEXT_AUDIT.md](TOC_LOAD_CONTEXT_AUDIT.md) and
[TOC_TRIGGER_BINDINGS_AUDIT.md](TOC_TRIGGER_BINDINGS_AUDIT.md).

## 5. Checks to deliver

| Check | Reliable input | Boundary |
| --- | --- | --- |
| Lua syntax/types | configured Emmy owner and selected Library | preserve upstream code/message/severity/location |
| TOC/XML/bindings integrity | exact selected paths, conditions, triggers, compatibility/context and maps | static facts do not prove runtime execution |
| API availability/context | resolved receiver/symbol plus selected API metadata | missing declaration is not proof of removal |
| Widgets/templates/mixins | known object type and explicit relations | dynamic creation/dispatch remains unknown |
| Events/callbacks/bindings | known registration or reviewed callback unit plus any scoped semantic limitation | event-like strings do not connect systems |
| Deprecation/replacement | explicit selected metadata/reviewed correction | never infer by name similarity |
| Secret/restricted use | exact metadata and supported local operation | advisory where execution state is unknown |

Start with types, load integrity and API context. Select checks before work runs.
An unrequested rule or unavailable optional feature cannot make type checking
partial. Report exactly which selected check/unit lacks evidence.

Current active rule providers cover only API existence and a narrow local Secret
operation. Event, broad callback, widget and binding checks remain work.

API existence requires original Emmy receiver binding and selected API origin.
Exact catalog absence cannot repair a misidentified local receiver. Secret
checking distinguishes coherent unresolved producer semantics from malformed or
stale identity. Unresolved TOC `Secure` context is separate from both.

<a id="6-secret-values-and-protected-behavior"></a>

## 6. Secret/protected behavior

Preserve `SecretReturns`, conditional flags, field `SecretValue`,
`SecretArguments`, `HasRestrictions` and `SecureHooksAllowed` with exact origin.
A generic restriction flag does not prove one current call is forbidden.

Reuse narrow local flow only when exact producer/value/operation and dominating
guard are established. Copying, formatting, `pcall`, similar spelling or a later
guard does not prove permission. Unknown alias/control flow remains unknown.
Absence of a finding is never a runtime-safety certificate.

Keep Secret metadata separate from primitive types and TOC execution-security
metadata. Do not build another whole-program dataflow engine or auto-insert guards.

<a id="7-useful-output-and-updates"></a>

## 7. Diagnostics, completeness and updates

A diagnostic has stable code, severity, message, exact file/range and optional
related location. Keep findings separate from limitations such as partial Library,
unknown channel/trigger/compatibility/security context, malformed binding XML or
cancellation. Known findings survive partial input; unexamined scope never reports
clean.

Deduplicate only same code/cause/location. Library/STD declarations are related
locations, never fabricated Main spans.

Internally use UTF-8 half-open bytes tied to exact text; convert once to negotiated
editor encoding/version. A changed/deleted file, TOC, XML, bindings document,
dependency, trigger, source Interface/security metadata, channel or Library
invalidates dependent facts. Physical delta is safe only while selected context
remains exact; otherwise rebuild cold. Never serve old results as current after
failed/stale/cancelled update.
