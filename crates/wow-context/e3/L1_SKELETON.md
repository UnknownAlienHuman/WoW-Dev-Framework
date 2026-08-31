# E3-B L1 entity and local-neighborhood skeleton

**Status:** normative focused structural layer.

## Purpose

L1 gives exact, bounded detail for one or more already-resolved entities and their directly relevant structural neighborhood without dumping a full file, graph, or transitive closure.

Supported root kinds include:

```text
function
method
callback
API symbol
event/signal
hook/script handler
frame/object/region
template
mixin/prototype
factory/registry
state root/path
file declaration/source span
recognized module/service/library entity
```

## L1 root record

```text
exact entity key and universe/generation
entity kind and canonical labels
signature/parameters/returns/types where available
source declaration/body span handles
owner/container/package/file/load roles
confidence/provenance/coverage/conflict state
exact graph assertions and source/reference origins
```

No same-name entity substitution.

## Local facets

The intent/profile may request bounded facets:

```text
DeclarationAndType
OwnershipAndContainment
LoadAndLifecycle
CallersAndCallees
EventsAndCallbacks
HooksAndScripts
StateReadsAndWrites
ObjectParentAndInheritance
TemplatesAndMixins
FactoriesAndRegistries
ApiUsageAndReferenceFacts
ExistingFindingEvidence
DocumentationAndSourceExcerptCandidates
ConflictsCoverageAndOmissions
```

Each facet has its own capability, depth, fanout, confidence, evidence, and budget rules.

## Relations

L1 distinguishes:

- direct accepted relation assertions;
- `Possible` relations;
- bounded reason paths;
- rejected/conflicted proposals;
- relation absence under complete scope;
- no-new-evidence/partial/NotEvaluated.

A call path, load path, ownership path, or inheritance chain is never rendered as a direct edge.

## API/reference facts

When an exact project/source relation resolves to a ReferenceView entity, L1 may include exact bounded facts such as:

- API/type/event identity and signature;
- profile availability/coverage;
- restriction/Secret/protected facets already present in ReferenceView;
- correction/conflict/source/evidence records;
- exact negative-authority state.

`wow-context` does not derive API legality or restrictions from source implementation and does not generate replacement/migration advice.

## Events, callbacks, and hooks

Keep distinct:

```text
native frame event registration
EventRegistry native frame-event bridge
custom EventRegistry producer/subscriber
CVar callback
SetScript
HookScript
hooksecurefunc
other exact registered signal/hook kinds
```

L1 preserves recognizer/source confidence and does not collapse them into a generic event bus. Hook structure never becomes a taint/combat/protected/runtime-safety claim.

## State

State facets include exact roots and literal paths plus dynamic prefixes/possible paths under original confidence. A same-named global is not a SavedVariables root without the exact TOC fact. No SavedVariables contents are read.

## Source excerpt candidates

L1 records exact candidate source spans/handles and selection reasons. Actual bytes are fetched only after context selection and privacy/license/budget approval.

Default L1 does not include a complete function body. Profiles may request bounded declaration or local body slices for selected roots.

## Neighborhood expansion

A profile defines:

```text
allowed axes/relation kinds/directions
maximum depth
per-kind fanout
confidence/provenance policy
path limits
required vs optional facets
source/reference enrichment
```

Default E3-B behavior is direct one-hop detail. Deeper expansion is explicit, bounded, and path-preserving.

## Build algorithm

```text
validate exact root keys and universe set
-> fetch exact entity/assertion/source/reference records
-> fetch mandatory owner/source/load context
-> evaluate profile-required direct facets
-> enumerate optional bounded neighbors and paths
-> deduplicate semantic/evidence items
-> retain conflicts/rejections/coverage
-> estimate costs and prune optional candidates
-> emit source excerpt candidates, omissions, and expansion frontier
-> canonicalize and validate
```

## Multiple roots

Multiple roots are allowed only under explicit count and combined-budget limits. Shared evidence/items are deduplicated, but each root retains its own inclusion reason and relation path.

Roots are never merged because they share a signature/name/source file.

## Mandatory L1 closure

- exact root identity and source declaration context;
- exact bound universe/profile generations;
- origin/evidence for every included facet;
- direct-vs-path and confidence classification;
- conflicts/coverage/NotEvaluated/omissions;
- selection/budget state;
- expansion frontier/continuation where relevant.

If this minimum exceeds budget, fail rather than drop the root's identity/evidence.

## L1 nonclaims

- no runtime values or execution order beyond static evidence;
- no architectural importance or intent guessed from source names;
- no whole-program call completeness without exact coverage;
- no Secret/taint/combat/protected safety upgrade;
- no cross-build lineage or impact;
- no diagnosis, fix, code generation, or edit plan;
- no model-generated summary.

## Validation

- exact root/entity/source/reference closure;
- no cross-universe/generation substitution;
- no direct relation fabricated from a path;
- no event/hook/state class collapse;
- dynamic relations retain `Possible`/NotEvaluated;
- excerpt candidates resolve exact source spans;
- optional pruning and omissions reconcile;
- deterministic output under shuffled query order.
