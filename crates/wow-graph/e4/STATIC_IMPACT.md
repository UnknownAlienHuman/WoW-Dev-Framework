# E4-B bounded static-impact contract

**Status:** normative. Static impact is an evidence-bearing path analysis over exact published graph/change inputs; it is not runtime failure prediction.

## Inputs

```text
exact LineageGraphSnapshot
exact GenerationChangeSet
exact target generation GraphSnapshot/View
optional exact source generation GraphSnapshot/View for removed/replaced roots
exact impact profile
exact change roots
finite traversal/output budgets
cancellation/continuation
```

No current selector, fuzzy root, raw source, SearchStore query, model prompt or arbitrary relation program.

## Root classes

```text
accepted LineageAssertion
ChangeRecord
GenerationAbsenceDecision
accepted replacement/deprecation relation
validated MigrationRecipe or MigrationCandidate
exact changed Reference entity/facet
```

Candidate roots are allowed only when the request explicitly permits Candidate impact and the entire result remains Candidate-limited.

## Impact categories

```text
DirectReferenceImpact
DirectCallImpact
DirectHookOrScriptImpact
DirectSignalRegistrationImpact
DirectTemplateOrInheritanceImpact
DirectFactoryOrObjectImpact
DirectStateImpact
DirectLoadOrDependencyImpact
DirectTypeOrSignatureImpact
DirectRestrictionImpact
BoundedTransitiveImpact
PossibleImpact
ConflictBlockedImpact
NotEvaluatedImpact
```

Categories are typed and profile-defined. They do not encode severity or runtime outcome.

## Traversal axes

The profile declares an exact whitelist of graph relation kinds, directions and confidence classes. Examples:

```text
uses_api / referenced_by
calls / called_by
hooks / hooked_by
sets_script / handled_by
registers_event / handles_event / subscribes_callback / triggers_callback
depends_on / optional_depends_on / loads / loads_before
references_template / inherits / mixes_in / instantiates / created_by
reads_state / writes_state
owns / contains / declares / exports
```

Each underlying relation retains source/target entity, direct assertion, producer, confidence, provenance, evidence, coverage and conflict state.

## Direction

Impact usually traverses from changed entity to dependents by reverse-use directions. The profile must name the direction explicitly; no generic “affected by” relation is assumed.

A target may be impacted through multiple independent paths. Preserve all selected paths within budgets and deduplicate only identical path identities.

## Direct versus transitive

```text
DirectImpact
    one direct accepted relation assertion from root to target

BoundedTransitiveImpact
    ordered path of two or more direct relation assertions
```

A transitive path never becomes a direct relation or a new graph edge. Rendering must show the path.

## Confidence and proof cap

Path/result confidence cannot exceed the weakest decisive element:

```text
root change/assertion confidence
relation assertion confidence on every edge
lineage endpoint mapping confidence
input coverage/conflict/truncation ceiling
impact rule/profile ceiling
```

Any Candidate/`Possible` edge or root caps the affected path accordingly. Multiple possible paths do not become Proven by count.

## Change-specific traversal

### Signature/type/restriction changes

Trace exact users/callers/type references/derived declarations according to the profile. Static use does not prove runtime incompatibility; compatibility classification comes from exact change/migration records.

### Move/rename

Trace exact source/import/export/owner/load references that name or locate the entity. A move with stable semantic references may have no affected dependents under a given profile; do not invent textual impact.

### Removed/replaced/deprecated

Trace exact uses of the old entity in the target/source graphs, preserving generation roles. Replacement candidate existence does not make every use migratable.

### Load/dependency changes

Trace exact package/load/dependency paths. Static reachability does not prove runtime loading, readiness, event delivery or combat behavior.

### Event/hook/state/template changes

Preserve native event, EventRegistry bridge/custom signal, CVar callback, hook/script, state, XML parent, inheritance, mixin and factory relation types separately. Do not collapse them into one callback/ownership path.

## Scope and universes

Ordinary E4-B impact targets entities in one exact target generation/universe. Cross-universe edges can be traversed only when the graph contains an explicit compatible bridge and the impact profile permits it.

A project use of a changed Reference entity can be traced through an exact `uses_api` bridge. The resulting impact is scoped static dependency evidence, not proof that the addon fails at runtime.

## Planning algorithm

```text
validate exact roots, snapshots and profiles
-> resolve root before/after entities and governing change/assertion records
-> derive allowed traversal starts/directions by change kind
-> plan deterministic bounded graph queries
-> reserve mandatory root/evidence/coverage closure
-> traverse direct relations and bounded paths
-> classify each path/target without authority upgrade
-> retain conflicts, skipped axes, omissions and NotEvaluated state
-> stable-sort/deduplicate exact path identities
-> emit StaticImpactResult and continuation
```

## Budgets

Bound:

```text
roots
relation kinds/axes/directions
maximum depth
visited nodes/edges
expanded frontier per relation kind
paths per root and target
returned targets/paths
source/evidence/coverage/conflict refs
serialized bytes
wall/CPU/memory
continuation pages/cursor bytes
```

No unbounded full-graph impact scan. If the mandatory root/evidence closure exceeds the hard budget, fail rather than omit proof metadata.

## Stopping states

```text
CompleteForRequestedScope
NoNewEvidence
HardBudgetReached
DepthOrFanoutReached
CoverageBoundary
ConflictBoundary
UnsupportedRelationOrChangeKind
Cancelled
Failed
ContinuationAvailable
```

`NoNewEvidence` is not proof that no runtime or out-of-scope impact exists.

## Result records

For every affected target:

- exact target generation/entity;
- root and change/assertion IDs;
- one or more ordered direct reason paths;
- impact categories;
- confidence/proof cap and decisive weakest element;
- evidence/provenance/coverage/conflicts;
- selected/omitted/skipped path state;
- budget/continuation information;
- explicit nonclaims.

## Nonclaims

E4-B static impact never states, without separate evidence:

```text
runtime breakage
user-visible failure
severity or priority
security exploitability
Secret Value access or combat legality
taint/protected/forbidden safety
performance or CPU/FPS impact
fixability or migration success
release readiness
```

## Continuation

Continuation binds exact lineage/change/graph snapshots, roots, impact profile, visited/frontier/path manifests, prior omissions/conflicts, stable ordering and cumulative budgets. It never refreshes current or switches target generation.

## Validation

- every root and path edge resolves in exact snapshots;
- direct/path distinctions preserved;
- confidence/authority does not increase along a path;
- cross-universe bridges are explicit;
- no forbidden relation or direction;
- coverage/conflict/truncation visible;
- whole path records not silently sliced;
- deterministic output under shuffled graph order and 1/2/N workers;
- hard-negative fixtures do not claim runtime breakage;
- removal/replacement roots have valid authority.
