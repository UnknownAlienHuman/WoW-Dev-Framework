# Reference/source and project/source bridge contracts

**Status:** normative E3-B cross-universe boundary.

## Purpose

Bridge assertions connect exact entities from different universes while preserving endpoint identity, authority, profile and evidence. They never copy or merge one universe into another.

## Bridge classes

```text
ReferenceUiBridge
    reference_api <-> blizzard_ui_source
    may be built and published in E3-B

ProjectUiBridge
    first_party_project <-> blizzard_ui_source
    requires an exact user ProjectSnapshot and source publication
    endpoint/profile contract defined here; actual project-specific publication is deferred

RuntimeUiBridge
    runtime <-> blizzard_ui_source
    deferred; requires runtime evidence

HistoricalUiBridge
    source generation/build lineage
    deferred to E4 lineage
```

## Bridge profile

```text
UiSourceBridgeProfile
    bridge_profile_id/version
    allowed source/reference/project profiles
    allowed universe pairs and directions
    allowed relation kinds
    endpoint kind compatibility
    exact resolution rules and required facts
    confidence/provenance ceilings
    coverage/negative-clause requirements
    ambiguity/conflict policy
    budgets/cancellation
    canonical digest
```

The profile is repository-owned and nonexecutable. No user-provided regex, Lua, callbacks, model prompt or source script.

## Reference/source bridge inputs

- exact `BlizzardUiSourceGeneration` and source GraphView;
- exact `ReferenceProfile`/`ReferenceGeneration` and reference entity/graph view;
- exact bridge/graph registry versions;
- source analyzer facts and source handles;
- reference canonical symbol/type/event/entity keys and evidence;
- profile compatibility/build-binding state;
- coverage/conflict records for all decisive inputs.

## Allowed initial reference/source relations

### `uses_api`

```text
blizzard_ui_source function/method
    --uses_api-->
reference_api API symbol
```

Requirements:

- exact source call/reference fact from analyzer/project adapter;
- exact reference entity key resolution under the compatible reference profile;
- no unresolved lexical/member ambiguity;
- source and reference evidence refs retained.

This relation says the exact source snapshot statically references the exact API entity. It does not say the API is public for addons, unrestricted, safe, callable in all contexts or implemented only there.

### `handles_event` / `registers_event`

A source registration/handler may bridge to an exact reference event entity only when the reference profile models that event and the source registration facts match the registered relation rule. Native frame events, EventRegistry native bridges, custom EventRegistry events and CVar callbacks remain separate relation families.

A same-looking string is insufficient.

### type/widget/XML reference bridges

Exact source declarations/usages may bridge to reference types/widgets/events when both models expose compatible stable keys and the bridge profile defines the semantics. No generic same-name edge.

## Source-internal relations are not bridges

The following remain inside `blizzard_ui_source`:

```text
contains / defines / declares / loads / calls
XML parent / inherits / references_template
mixin / factory / registration / hook / state relations
```

They do not target `reference_api` merely because names look public.

## Resolution states

```text
ResolvedExact
    one exact compatible endpoint and complete decisive coverage

ResolvedPossible
    structure permits one or more possibilities; emitted only when profile allows and remains Possible

Ambiguous
    multiple compatible endpoints; retain ambiguity group, no arbitrary selection

NotFoundWithAuthority
    reference endpoint absent under exact complete negative authority

NotFoundWithoutAuthority
    no endpoint in bounded view but coverage is insufficient

IncompatibleProfile
    source/reference profiles cannot be joined

Conflict
    decisive evidence disagrees

NotEvaluated / Failed / Cancelled
```

`NotFoundWithAuthority` for a bridge endpoint does not prove the source call is invalid or that no runtime/global symbol exists outside the reference capability. It applies only to the exact reference lookup capability.

## Name resolution

Allowed resolution ingredients are explicit and typed:

- analyzer-resolved global/member/callee identity;
- exact namespace/member facts;
- exact reference canonical key and alias records under the reference profile;
- explicit XML/type/event identifiers from normalized source/reference facts;
- exact source lexical/scope and call/member access facts;
- bridge rule version.

Forbidden shortcuts:

- case-insensitive nearest name;
- suffix/prefix similarity;
- source path/directory convention;
- popularity or frequency;
- first/last matching candidate;
- documentation/comment text alone;
- model/embedding similarity;
- current/latest profile fallback.

## Confidence ceiling

- A direct exact analyzer/reference resolution may yield `Derived` bridge confidence.
- `Possible` inputs or ambiguity cannot yield stronger than `Possible`.
- Bridge builders do not emit `Proven`; source/reference direct evidence remains proven independently while the cross-universe conclusion is deterministic derived structure.
- Candidate/model results remain `Candidate` in later optional lanes and are not E3-B canonical bridges.

## Project/source bridge contract

A later project integration receives:

```text
exact user ProjectSnapshot/GraphSnapshot
exact current-or-selected BlizzardUiSourcePublication/GraphSnapshot
exact compatible ReferenceProfile/Generation
ProjectUiBridgeProfile
```

Potential relation families include:

```text
project hook --hooks--> UI source function/method
project XML object/template --inherits/references_template--> UI source XML entity
project override/replacement candidate --explicit registered relation--> UI source entity
project copied/analogous implementation --candidate lineage relation in E4 only
project API use --uses_api--> reference entity, not UI source implementation by convenience
```

The actual source facts must establish each relation. Same method/function/template name or matching source text does not automatically create a project bridge.

## Hook bridges

A project `hooksecurefunc`, `HookScript`, `SetScript` or other hook structure can target a UI source entity only when static target resolution is exact under both snapshots.

The bridge records structure only. It does not establish:

- taint safety;
- combat safety;
- protected/forbidden/managed legality;
- receiver runtime identity beyond exact static facts;
- ordering or performance;
- that the target exists in every build/profile.

## XML bridges

Project XML inheritance/template references require exact project XML facts, exact UI source XML entity identity, compatible source/profile and graph relation schemas. Duplicate or dynamically selected names remain ambiguous/possible.

## Source/reference/project graph publication

Producer partitions stay separate:

```text
ui-source-direct:<source-generation>:<partition>
ui-source-recognizer:<pack/rule/version>:<partition>
ui-reference-bridge:<source-generation>:<reference-generation>:<bridge-rule>
project-ui-bridge:<project-generation>:<source-generation>:<bridge-rule>   [later]
```

Updating one partition removes only its prior assertions and downgrades its coverage. Graph views retain exact supporting producer/evidence IDs.

## Invalidation

Invalidate a bridge partition when any decisive source/reference/project endpoint, profile, registry, rule, fact, alias, coverage or conflict record changes.

Source entity rename/move may preserve lineage later but still invalidates the exact generation bridge. No stale endpoint remapping by display name.

## Bridge explanation

Every bridge result can explain:

- exact endpoint keys/universes/generations;
- relation schema and direction;
- resolution rule/version;
- source/reference/project input facts;
- evidence/source handles;
- profile compatibility/build-binding;
- confidence/coverage/conflicts/ambiguity;
- producer partition and graph validation result.

No generated narrative is required for correctness.
