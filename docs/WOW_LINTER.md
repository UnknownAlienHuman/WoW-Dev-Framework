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
| Client flavor/game type | Retail/Mainline versus other client families and TOC game-type conditions. |
| Product channel | Stable, PTR or Beta. `OnlyBetaAndPTR` is not a flavor or Interface rule. |
| Target Interface | Compatibility target when explicitly supplied or established by compatible Library metadata. |
| Game/Glue location | Package/unit selection context. A system named `Glue` does not determine it. |
| Package/file execution context | Package secure/global default, per-unit override, private-table policy and load phase. Selection is not execution context. |
| API execution context | Ordinary addon by default. Member `SecureOnly` is distinct from flavor and package environment. |
| Main/dependencies/Library | Main is diagnosed; selected dependencies provide program context; Library provides declarations, not executable files. |

Never union flavors/channels to make diagnostics disappear. Existing plain Lua
annotation directories remain usable for types; absent metadata limits only
checks that require it.

## 2. Reuse EmmyLua and model WoW around it

Use explicit Lua 5.1 and embedded standard declarations. Stock Lua declarations
do not prove host availability in WoW. A real Blizzard syntax extension requires
a source example and narrow upstream/adapter support, not another parser.

Resolve symbols semantically. A local named like a standard global or `C_*`
namespace is not a WoW API by spelling. Unknown aliases/dynamic dispatch remain
unknown; do not globally suppress undefined names or widen missing types to `any`.

External addon chunks are selected by TOC/XML rather than an invented `require`
graph. For supported ordinary chunks, provide source-mapped addon-name/private-
table arguments with one stable private table per addon and separate lexical
chunks. Dependency addons keep distinct tables. Package path prefixes provide
file identity, not namespace semantics.

The exact selected unit controls context:

- package `UseSecureEnvironment` sets a default;
- per-file `LoadIntoEnvironment` can override it;
- `SuppressLocalTableRef` removes the private-table argument;
- bootstrap/load-first/deferred phases remain distinct;
- free inline UI scripts, verified UI handlers and binding callbacks are not
  external addon chunks.

Unsupported context is a scoped limitation, never a fabricated global/private
table value.

SavedVariables names are declarations. Support account, character and machine
scope from the selected TOC, but never read user SavedVariables data, infer a
schema, or assume a value is non-nil before addon initialization.

## 3. Preserve the Blizzard type contract

| Blizzard fact | Required treatment |
| --- | --- |
| Ordered arguments/returns | Preserve order, receiver style and Lua multiple-result behavior. |
| `Type`, `InnerType`, `Nilable`, `NilableContents` | Keep aliases, arrays/unions, container nilability and element/slot nilability separate. |
| `Default` | Preserve false, zero and empty string as present values. |
| `MayReturnNothing` | Keep a zero-result alternative; do not collapse it into container nilability. |
| `StrideIndex` | Preserve fixed prefix and each ordered repeated position; a positive index is not one boolean variadic flag. |
| Callback/event payload | Preserve ordered parameters/returns; a callback name is not a signature. |
| Structure `Mixin` | Preserve data fields and explicit mixin relation without guessed inheritance. |
| Widgets/script objects | Use reviewed explicit class/receiver/base relations. |
| Enums/literals | Keep closed domains closed and extensible strings open. |
| Member environment/restrictions | Carry beside types; ordinary annotations cannot express every game constraint. |

Concrete reviewed losses include repeated triples in
`C_ArtifactUI.GetMetaPowerInfo` and separate zero-result/table/element nilability
in `C_EquipmentSet.GetItemIDs`. Unsupported exact tuple correlations remain
visible projection limitations; independent valid declarations survive.

## 4. TOC, triggers, XML and bindings form the load model

### TOC selection and dependencies

Use one explicit or unambiguous TOC and assess compatibility separately. Preserve
original spans and exact declared order. Sorting for deterministic identity must
not replace load semantics.

Finite dependency aliases:

- required: `Dep`, `Dependencies`, `DependsOn`, `RequiredDep`, `RequiredDeps`;
- optional: `OptionalDep`, `OptionalDeps`.

Absent optional dependency is normal. Missing required dependency limits dependent
availability. Interface mismatch is a compatibility diagnostic, not permission to
switch TOC/flavor or abandon independent analysis.

### Product channel and conditions

Model Stable/PTR/Beta separately from flavor, Interface and game/glue location.
`OnlyBetaAndPTR: 1` includes PTR/Beta, excludes Stable and is unresolved when the
channel is absent. Never infer channel from repository name/ref/path.

Reduce predicates by their boolean form:

- allow-list: any known true includes; all known false excludes; otherwise
  unresolved;
- exclude-list: any known true excludes; all known false includes; otherwise
  unresolved;
- conjunctive gates: known exclusion dominates unknown.

Retain unknown source evidence even when another gate determines exclusion. A
reachable unresolved unit remains a limitation.

### `LoadWith` is not a dependency

`LoadWith` is a reverse load trigger. Preserve source package, named trigger,
resolution state, order and source span. It does not prove that the trigger
package can use symbols declared later by the `LoadWith` package and must not be
added to required/optional dependency closure. Explicitly selected source can
still be analyzed when automatic reachability is unresolved.

### Known lifecycle and persistence metadata

`DefaultState` is known bounded lifecycle metadata. It does not suppress static
analysis of an explicitly selected root and must not create a global partial
result. SavedVariables account/character/machine declarations are typed metadata,
not data-file access.

Unknown cosmetic/custom metadata may be retained without limiting language work.
Unknown directives that can alter selection, environment, namespace, order,
triggers or callbacks remain located limitations.

### Exact selected order and context

`LuaWorkspaceSnapshot` is path-sorted and content-addressed for identity; it is
not load order. Pass a separate immutable validated context containing exact
package/dependency/TOC/XML/callback order or phase, snapshot role/path/digest,
originating record/unit, addon identity, effective environment and private-table
policy. Reject missing, foreign, duplicate or mismatched entries. Do not call
Emmy registration order a client emulator.

### UI XML

Use existing quick-xml parsing and exact source maps. Resolve only selected
Include/external Script, template inheritance, widget/script association and
explicit mixins. XML containment, inheritance and runtime parentage are distinct.

Analyze XML-only addons without dummy Lua. One malformed included XML unit is a
located failure of dependent scope and preserves independent units; unsafe paths
and invalid selected TOC are hard refusal.

Free inline scripts and verified handlers need different context. Supply `self`
and callback parameters only for source-verified handler kinds. Never replace a
handler's `...` with addon chunk arguments. Synthetic wrappers require exact
maps; wrapper-only errors never point at invented user locations.

### Implicit `Bindings*.xml`

A selected addon may have `Bindings<suffix>.xml` that is not listed in its TOC.
Discover at most one exact candidate under the held addon root using finite
selected flavor/product suffix precedence. Absence is normal; case/precedence
ambiguity is explicit.

Parse `<Binding>` Lua bodies with the existing XML/source-map owner as mapped
callback units. Supply only the source-verified implicit `keystate` parameter.
A binding body:

- is not a top-level TOC chunk;
- does not receive addon ellipsis/private-table arguments;
- becomes available only after selected addon files;
- executes on a binding event, whose occurrence static analysis cannot prove.

Malformed selected bindings XML or unsupported constructs limit binding analysis
while preserving independent addon diagnostics. Synthetic callback wrappers stay
out of user locations and API/flow facts.

Detailed source traces:
[TOC_LOAD_CONTEXT_AUDIT.md](TOC_LOAD_CONTEXT_AUDIT.md) and
[TOC_TRIGGER_BINDINGS_AUDIT.md](TOC_TRIGGER_BINDINGS_AUDIT.md).

## 5. Checks to deliver

| Check | Reliable input | Boundary |
| --- | --- | --- |
| Lua syntax/types | configured Emmy owner and selected Library | preserve upstream code/message/severity/location |
| TOC/XML/bindings integrity | exact selected paths, conditions, triggers and maps | static facts do not prove runtime execution |
| API availability/context | resolved receiver/symbol plus selected API metadata | missing declaration is not proof of removal |
| Widgets/templates/mixins | known object type and explicit relations | dynamic creation/dispatch remains unknown |
| Events/callbacks/bindings | known registration or source-verified callback unit/signature | event-like strings do not connect systems |
| Deprecation/replacement | explicit selected metadata or reviewed correction | never infer by name similarity |
| Secret/restricted use | exact metadata and supported local operation | advisory where execution state is unknown |

Start with types, load integrity and API context. Select checks before work runs.
An unrequested rule or unavailable optional feature cannot make a completed type
check partial. Report exactly which selected check/unit lacks evidence.

Current active rule providers cover only API existence and a narrow local Secret
operation. Event, broad callback, widget and binding checks remain work.

API existence requires original Emmy receiver binding and selected API origin.
Exact catalog absence cannot repair a misidentified local receiver. Secret
checking distinguishes coherent unresolved producer semantics from malformed or
stale fact identity.

## 6. Secret/protected behavior

Preserve `SecretReturns`, conditional flags, field `SecretValue`,
`SecretArguments`, `HasRestrictions` and `SecureHooksAllowed` with exact origin.
A generic restriction flag does not prove one current call is forbidden.

Reuse narrow local flow only when exact producer/value/operation and dominating
guard are established. Copying, formatting, `pcall`, similar variable spelling or
a later guard does not prove permission. Unknown alias/control flow remains
unknown. Absence of a finding is never a runtime safety certificate.

Keep Secret metadata separate from ordinary primitive types; do not build another
whole-program dataflow engine for the first useful linter or auto-insert guards.

## 7. Diagnostics, completeness and updates

A diagnostic has stable code, severity, explanatory message, exact file/range and
optional related location. Keep findings separate from limitations such as
partial Library, unknown channel/trigger/context, malformed binding XML or
cancellation. Known findings survive partial input; unexamined scope never
reports clean.

Deduplicate only same code/cause/location. Do not hide independent symptoms behind
a root-cause platform. Library/STD declarations are related locations, never
fabricated Main spans.

Internally use UTF-8 half-open byte ranges tied to exact text; convert once to the
negotiated editor encoding/version. A changed/deleted file, TOC, XML, bindings
document, dependency, trigger, channel or Library invalidates dependent facts.
Physical delta is safe only while selected membership/context/config remains
exact; otherwise rebuild the containing selection cold. Never serve old results
as current after failed/stale/cancelled update.