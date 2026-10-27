# TOC load-context and source-order trace

**Original review base:** `0784d0cb5f2c709fa7d6778f6ad7448df7a8cb77` (`main` after PR #256)
**Final Slice 1A source re-audit:** `28302632599bed087394dee83751815d13041579` (`main` after PR #268)
**Scope:** ordinary acquisition traces T02–T05, addon-context trace T11,
metadata trace T23 and direct first-slice traces T32–T39

This is post-baseline source evidence for the first public `wow check`. It
supplements [IMPLEMENTATION_TRACE_REGISTER.md](IMPLEMENTATION_TRACE_REGISTER.md);
it is not another product specification and does not claim an executed addon
workflow.

## Verdict

The existing Project loader contains useful bounded source acquisition, TOC/XML
records, dependency-closure machinery and exact XML maps. It is **not yet a safe
handoff to the direct addon check** because several source facts are lost or
misclassified before `wow_service::addon_check::check_addon` receives its
path-sorted snapshots:

1. reviewed dependency aliases are incomplete;
2. execution, addon-table and load-policy facts are absent or misclassified;
3. current known lifecycle/error/namespace metadata becomes globally blocking;
4. known false and unknown TOC predicates are combined too conservatively;
5. selected package/file source order is discarded at the snapshot boundary;
6. XML-only Main and malformed included XML still fail at the wrong scope.

These defects are related but must not be fixed by treating Emmy file-registration
order as a client emulator. Keep immutable path-sorted snapshots for exact source
identity and add a separate exact validated load/context handoff for WoW-specific
semantics.

### Evidence boundary

Keep the claim boundary simple:

- preserve exact Blizzard directives and source locations;
- describe current framework behavior from its actual producer and consumer;
- use reviewed Wowless behavior only as a provisional adapter rule;
- emit a scoped limitation when a result depends on donor behavior whose client
  parity is not established.

Do not add a generic proof/evidence DTO to Project or frontends. Raw source facts,
explicit adapter behavior and ordinary limitations are sufficient. Directive
names and source examples do not by themselves prove complete client execution
semantics, and Wowless is a comparison donor rather than Blizzard loader
authority.

## Evidence traced

### 1. Dependency spelling is broader than `Dep`

`load/toc.rs::parse` and `load/package_closure.rs::project_metadata` recognize:

- required: `Dependencies`, `RequiredDeps`, `DependsOn`;
- optional: `OptionalDeps`.

The pinned Blizzard source also uses all of these singular/short forms:

- [`Blizzard_ChatFrameBase.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_ChatFrameBase/Blizzard_ChatFrameBase.toc)
  contains multiple conditional `## Dep:` entries;
- [`Blizzard_PingUI.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_PingUI/Blizzard_PingUI.toc)
  contains `## RequiredDep:`;
- [`Blizzard_IME.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_IME/Blizzard_IME.toc)
  contains `## OptionalDep:`.

The missing finite aliases currently become unknown metadata and produce no
package dependency edge. Correct one shared reviewed key table used by parsing
and package projection:

| Normalized key | Meaning |
| --- | --- |
| `dep`, `dependencies`, `dependson`, `requireddep`, `requireddeps` | required dependency list |
| `optionaldep`, `optionaldeps` | optional dependency list |

Do not accept arbitrary key prefixes as dependencies. Preserve every declaration
in source order with its conditions, source span and original normalized key.

### 2. Execution, addon-table and load-policy facts are lost

`TocLoadContext.environment` and `AllowLoadEnvironment` answer whether a unit is
selected for a caller-supplied context. They do not represent the environment in
which a selected unit executes.

The pinned Blizzard source demonstrates occurrence and location of distinct
execution and namespace directives:

- [`Blizzard_PingUI.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_PingUI/Blizzard_PingUI.toc)
  declares `## UseSecureEnvironment: 1` for the addon but overrides
  `Blizzard_PingUtil.lua [LoadIntoEnvironment global]`;
- [`Blizzard_ChatFrameBase.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_ChatFrameBase/Blizzard_ChatFrameBase.toc)
  contains `Shared\ChatFrameFiltersSecure.lua [LoadIntoEnvironment secure]` and
  a conditional `## LoadFirst: 1`;
- [`Blizzard_CombatAudioAlerts.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_CombatAudioAlerts/Blizzard_CombatAudioAlerts.toc)
  declares `AllowAddOnTableAccess: 1`;
- [`Blizzard_RecruitAFriend.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_RecruitAFriend/Blizzard_RecruitAFriend.toc)
  still contains restricted/internal `SuppressLocalTableRef` metadata.

Pinned Blizzard
[`AddOnsDocumentation.lua`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_APIDocumentationGenerated/AddOnsDocumentation.lua)
explicitly states that the addon table
is passed as the second result of chunk `...` and that
`AllowAddOnTableAccess: 1` only permits later retrieval through
`C_AddOns.GetAddOnLocalTable`. This establishes the ordinary chunk contract and
keeps retrieval permission separate from injection.

The comparative Wowless loader interprets secure directives as package defaults
and per-file overrides, and uses separate bootstrap environments. For
`SuppressLocalTableRef` it sets `addonEnv` to `nil`, but that donor behavior does
not establish current client semantics. The selected source occurrence therefore
stays an unsupported special context rather than a supported suppression rule.

Current `TocConditionKind` has no `LoadIntoEnvironment`; current TOC/package
models have no typed package default environment, per-unit override, addon-table
retrieval permission, ordinary/special chunk context or load-first state. Unknown
metadata can also degrade closure completeness without preserving the semantic
fact that was lost.

Add narrow typed facts rather than string exceptions:

- package default environment: ordinary/global or secure;
- per-load-unit override: inherit/global/secure;
- ordinary addon-table injection plus independent retrieval permission;
- unsupported special chunk context for `SuppressLocalTableRef`;
- source-located phase declarations plus only the phase semantics supported by
  current code or a reviewed adapter;
- exact source span and selection conditions for every fact.

Bind resolved context values into project identity wherever they affect a result,
and emit a scoped limitation for unsupported context. Do not add a separate
adapter-version receipt or propagate a universal proof field through every fact
and frontend.

Unsupported directives remain located limitations. Cosmetic/custom metadata may
be retained without making unrelated Lua/type analysis partial. A directive that
can change selection, namespace, environment or order is not cosmetic.

### 3. Current known metadata is misclassified as blocking unknown input

Pinned selected source contains additional current directives:

- [`Blizzard_DamageMeter.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_DamageMeter/Blizzard_DamageMeter.toc)
  uses `LoadSavedVariablesFirst: 1`;
- [`Blizzard_ScriptErrors.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_ScriptErrors/Blizzard_ScriptErrors.toc)
  uses `EscalateErrorDuringLoad: 1`;
- the pinned `Blizzard_CombatAudioAlerts.toc` above uses
  `AllowAddOnTableAccess: 1`.

Current `toc.rs` recognizes a small fixed set and pushes `UnknownDirective` for
every other included metadata key. `Loader::issue` marks every such issue blocking
except unresolved optional dependency, so known lifecycle, error-policy,
namespace-access and ordinary display/custom metadata can falsely degrade file
closure.

**Decision:** classify package metadata by effect:

- `LoadSavedVariablesFirst` is typed load-order metadata. It never authorizes
  reading SavedVariables content or inventing its types/values;
- `AllowAddOnTableAccess` is typed namespace-table retrieval permission and does
  not alter ordinary chunk arguments;
- `EscalateErrorDuringLoad` is known restricted error-policy metadata and is
  nonblocking for independent static language analysis;
- localized display metadata, icon/category/group fields and `X-*` custom keys
  are retained nonblocking;
- an unrecognized package metadata key is located and limits only checks whose
  selection/context may depend on it; it does not erase already enumerated files;
- an unknown inline/file directive can alter selection and remains a blocking
  limitation for that unit.

Do not solve this with an ever-growing “ignore unknown” string list. Use one finite
semantic table plus a clear nonblocking metadata category and preserve raw keys.

### 4. Tri-state predicate reduction is over-conservative

`conditions::evaluate` returns `Unresolved` immediately when any value in an
`AllowLoad*`/`ExcludeLoadGameType` list is unknown. `LoadSelection::and` also lets
`Unresolved` dominate `Excluded`.

That loses results that are already logically determined:

- allow-list: any known match means included even when another alias is unknown;
- exclude-list: any known match means excluded even when another alias is unknown;
- conjunctive gates: one known false gate means excluded even when a different
  gate is unknown.

Use three-valued boolean reduction by operator:

| Form | Included | Excluded | Unresolved |
| --- | --- | --- | --- |
| allow-list (OR) | any known true | all values known false | no true and at least one unknown |
| exclude-list (NOT OR) | all values known false | any known true | no true and at least one unknown |
| combined gates (AND) | all included | any excluded | no excluded and at least one unresolved |

Retain the unknown predicate as source evidence even when another condition makes
the unit definitely excluded. It must not poison the selected program or produce
a false clean claim for a reachable unresolved unit.

### 5. Load order is computed, then discarded before analysis

The package loader computes dependency reachability, phases, order groups and
per-unit source ordinals. `ProjectPackageLoadInput::into_namespaced_main` then:

1. receives packages already sorted by package name;
2. creates `packages/<package>/<source>` paths;
3. sorts all resulting Main files lexicographically by project path.

`LuaWorkspaceSnapshot::build` independently sorts every source by path, and
`RegisteredMemberAnalysis::new` registers `snapshot.files()` in that order.
`check_addon` receives the exact snapshots but no `ProjectLoadPlan` or package
load/context receipt.

Therefore the current handoff cannot answer load-dependent questions such as:

- whether a declaration is available at a particular selected load unit;
- which package/private table/environment applies to that unit;
- whether a virtual body precedes or follows another selected unit;
- which dependency/load-phase relation justified availability.

Keep `LuaWorkspaceSnapshot` path-sorted and content-addressed for exact identity,
lookup and analyzer ownership. Add a separate immutable `AddonLoadContext`
(or equivalent existing-owner type) that:

- lists selected physical and virtual units in exact package/TOC/XML order;
- binds each entry to snapshot role, canonical path, content digest and source
  load-record/unit identity;
- records addon/package identity, effective environment, ordinary or unsupported
  chunk-table context, retrieval permission, bootstrap/load phase and source span;
- rejects duplicate, missing, foreign or digest-mismatched entries;
- is passed beside snapshots into Service/context-specific checks;
- never claims that Emmy registration order itself reproduces WoW runtime
  execution.

The configured Emmy owner remains the single Lua/type engine. The ordered
context is consumed only by chunk transformation and WoW checks that actually
need load semantics.

### 6. Failure scope is still wrong

Two already recorded defects remain in the same producer chain:

- `read_toc_source` rejects Main when no physical Lua source exists, before mapped
  XML units can form the language program;
- `Loader::expand` propagates `xml::parse(...)?`, so one malformed included XML
  document aborts the entire selected closure instead of preserving independent
  captured units and a located failed unit.

The direct route must admit physical-only, physical+virtual, virtual-only and a
valid empty language scope with load facts. Malformed selected TOC/root/path
admission remains a hard failure; malformed included XML is a located unit
failure whose dependent scope is incomplete.

## Final Slice 1A source re-audit

The final re-audit followed the exact `main` producer/consumer path intended by
Slice 1A. It found three additional boundaries that the earlier load-context
review did not state precisely enough.

### 7. The retained plan, not only the reader argument, is profile-bound

Every current `read_toc_project*` entry point accepts `ProfileIdentity`, but the
dependency is retained after parsing as well. `ProjectLoadPlan` stores a mandatory
positive target Interface, flavor and canonical profile digest;
`validate_profile` requires exact equality, and the plan digest includes that
profile identity. Merely changing a function parameter to `AddonLoadTarget` would
still force the direct route to fabricate publication-era identity.

Slice 1A must replace the retained identity boundary itself:

```text
AddonLoadTarget { flavor, optional positive target_interface, optional context }
+ projection scope
+ exact selected source identities
-> ProjectLoadPlan identity and admission
```

Legacy profile callers may adapt a valid profile into this target. The direct
route never creates a profile ID, source/report digest, schema registry,
correction identity or generation. A changed target or projection scope produces
a different plan identity.

### 8. Current TOC loading is not a physical-Lua-only operation

`Loader::expand` immediately captures and parses referenced XML before returning
the physical Main inventory. A malformed included XML document can therefore
abort the load before otherwise valid direct TOC Lua reaches Emmy. That
contradicts the intended first slice, which explicitly defers XML semantics.

Add an identity-bound projection scope such as `physical_lua`:

- parse the selected TOC and its line predicates/metadata;
- capture direct selected `.lua` files with current root/path/byte safety;
- retain XML entries, source spans and selection state in the plan, but do not
  open or parse XML;
- surface selected deferred XML as a located source-selection limitation for the
  requested `LuaTypes` result;
- when selected deferred XML exists without physical Lua, use the supported
  empty Project snapshot and return partial; never fabricate a dummy file or
  report clean.

Full XML traversal, mapped virtual units and XML-only admission belong to the
next context slice. This is a projection of one Project owner, not a second
loader.

### 9. Package-plan limitations cannot serve the standalone first slice

The current public Service mapper accepts only `ProjectPackageLoadPlan`. Its
private load-issue mapper deliberately drops dependency placeholders because the
package closure owns their final outcome. Slice 1A has one explicit TOC and no
package closure, so reusing this mapper would lose required-dependency and other
standalone blockers.

Add one public scope-aware mapper over `ProjectLoadPlan`:

```text
ProjectLoadPlan + requested AddonCheckScope
  -> only blockers required by that scope
  -> located AddonCheckLimitation values
```

For Slice 1A, a missing direct Lua file named by a safely parsed TOC,
missing required dependency, unresolved selected condition, recoverable unknown
syntax/file kind, selected environment/context change and deferred selected XML
can block completeness. Missing addon root/explicit TOC, unsafe paths, invalid
TOC bytes or unrecoverable syntax are pre-result invalid input. Missing optional
dependencies, repeated occurrences of one admitted Main file, source Interface
without a positive target, `LoadWith`, lifecycle/display/custom metadata and
observations unused by `LuaTypes` remain retained Project facts, not direct
limitations. Do not equate `LoadIssue::blocks_complete` or
`ProjectLoadPlan::external_files_complete()` with requested-scope completeness.

Chunk context is also scope-dependent: use a bounded collector on the same
configured Emmy owner, matching `LuaLiteralToken::Dots` only outside any enclosing
`LuaClosureExpr`, and add one located limitation per affected Main file; nested
function varargs, strings and comments do not count. Binding/injection remains the
next context slice.

The plan and Main inventory must be validated against one another before snapshot
construction. Reject duplicate Library roots/snapshot IDs. A caller-supplied
malformed Library is invalid input and identifies configured root plus relative
file; it is not an internal analyzer outage.

## Required owner changes

### `wow-project`

- replace reader and retained-plan profile identity with `AddonLoadTarget` plus projection scope;
- share the finite dependency/directive table;
- project typed package/per-unit execution context and ordinary addon-table
  retrieval/access metadata;
- classify known lifecycle/error/display metadata without false global partial;
- preserve exact selected order in a validated context receipt;
- expose `physical_lua` TOC projection for Slice 1A and pure mapped XML preparation for the next slice;
- expose a standalone scope-aware plan-to-limitation handoff;
- allow virtual-only/empty language input without a dummy physical source.

### `wow-emmy`

- keep existing path-sorted exact snapshots and `ConfiguredAnalyzerSession`;
- add no second parser/session and do not reinterpret registration order as
  client load order;
- accept only the narrow source-mapped context seam needed for chunk-owned
  ellipsis/helper injection when that later slice is implemented.

### `wow-service`

- ordinary Slice 1A acquisition passes exact snapshots plus standalone scope-aware limitations to `addon_check`;
- later context slices pass validated load/context data beside those snapshots;
- type-only checking can use snapshots while reporting unsupported context as a
  limitation;
- selected WoW checks request only the load/context facts they consume.

### CLI/frontends

- expose normal root/TOC/flavor/Library/dependency configuration only;
- render located configuration/load limitations without exposing internal
  profile/receipt identities.

## Verification boundary

This review followed source producers and consumers and checked concrete pinned
Blizzard TOCs plus the pinned Wowless comparison. It did not run the loader,
addon check, editor or game client. None of these findings is an observed runtime
failure.

Before marking the path observed, record one public command covering:

1. `Dep`, `RequiredDep` and absent `OptionalDep` resolution;
2. a known-true list predicate with another unknown alias;
3. one secure addon with a global file override and one secure file override;
4. exact dependency/TOC/XML unit order in the Service input;
5. XML-only input and malformed included XML with an independent valid Lua file;
6. preserved code/message/severity/location and explicit scoped limitations.
