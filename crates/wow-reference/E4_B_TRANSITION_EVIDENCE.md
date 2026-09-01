# E4-B Reference transition evidence seam

**Status:** normative supporting contract for [`wow-graph/e4`](../wow-graph/e4/README.md); implementation has not started.

`wow-reference` owns exact cross-profile ReferenceView transition, deprecation, replacement, availability, signature/type/restriction and correction evidence. It does not own project-source continuity, search ranking, accepted lineage graph publication, migration application or runtime behavior.

## Producer operations

```text
compare_reference_generations
build_reference_explicit_transition_partition
build_reference_deprecation_replacement_partition
validate_reference_lineage_input_partition
```

Outputs are exact `LineageInputPartition` records for E4-C orchestration and independent E4-B graph validation.

## Exact inputs

```text
before ReferenceProfile/ReferenceGeneration/ReferenceView
before source/raw/correction/coverage/conflict manifests

after ReferenceProfile/ReferenceGeneration/ReferenceView
after source/raw/correction/coverage/conflict manifests

exact comparison/transition/canonicalization profiles
finite budgets/cancellation
```

No `current`, latest or nearest-build fallback. Product/flavor/channel/build/Interface compatibility is explicit.

## Explicit transition classes

The producer may expose only transitions supported by exact Reference facts/corrections, such as:

```text
same exact stable reference entity identity across profiles
explicit rename or move key
explicit deprecation
explicit replacement target
explicit introduced/removed availability transition
signature changed
type changed
restriction/Secret/protected/deprecation facet changed
reviewed correction transition
```

Each record preserves:

- exact before/after Reference entity keys/IDs;
- exact profiles/generations;
- raw source observations;
- normalized fact IDs;
- correction set/item/status/digests;
- evidence/source handles;
- coverage/conflicts/truncation;
- maximum relation proof ceiling;
- canonical digest.

## Authority

Reference transition evidence can support `Proven` or `Derived` E4-B assertions only under the relation registry and exact Reference authority.

It remains scoped to the exact compared profiles. It cannot be extrapolated to:

- another product/flavor/channel/build;
- project source continuity;
- Blizzard implementation-source lineage;
- runtime behavior or current spell secrecy;
- migration success;
- a target inferred only from name/similarity.

## Explicit deprecation and replacement

Distinguish:

```text
DeprecatedNoTarget
DeprecatedWithExplicitTarget
ExplicitReplacement
TargetUnresolved
TargetConflict
CandidateRecommendationOnly
NotEvaluated
```

A deprecated API may have no replacement. A replacement target can be distinct from same-lineage identity. Source comments, user documentation, search results or external code do not become Reference replacement authority.

## Availability and absence

An API/type/event/restriction entity can be marked introduced/removed only when exact ReferenceView negative authority and complete relevant source/evaluator/correction/coverage gates pass.

Missing annotation output, missing Blizzard implementation source, an empty search result or absent name in a partial source partition is not Reference absence.

## Signature and type transitions

Compare structured Reference facts:

- callable overloads and ordering;
- parameters, names, types, optionality, nilability, defaults and variadics;
- returns, tuples and multiple returns;
- structures, enums, tables, fields and values;
- callbacks/events/payload types;
- predicates and restriction facets;
- unknown/unsupported/raw metadata.

Rendered annotation text is a projection and cannot override ReferenceData.

## Restriction transitions

Preserve exact restriction authority and context:

```text
position/producer API
predicate/access contract
Secret/protected/forbidden/deprecation facet
build/profile scope
source/correction evidence
coverage/conflicts
```

Do not infer current runtime Secret state, spell policy, combat legality or taint safety from a static transition. Current/hotfix-dependent behavior requires separate runtime evidence.

## Correction transitions

A reviewed correction can contribute only when:

- target entity/field/profile matches;
- expected raw/source/value digest matches;
- correction dependency/status is valid;
- no unresolved correction conflict;
- before/after application state is explicit.

Expired/rejected/conflicted/not-applicable corrections remain distinct. Correction history is not rewritten.

## Alias and lineage boundary

Reference explicit aliases/transitions are owner facts. Search/fuzzy/name similarity cannot create them.

If the Reference schema has no explicit stable identity or transition evidence, a same-name/same-signature pair remains Candidate/Possible under E4-B, not Proven by `wow-reference`.

## Partition ownership

```text
reference_explicit_transition:<before-profile>:<after-profile>:<entity-kind/scope>
reference_deprecation_or_replacement:<before-profile>:<after-profile>:<system/scope>
```

Updating a Reference comparison replaces only its exact partition. Raw before/after ReferenceStores remain immutable.

## Security and privacy

- no source acquisition/network in this seam;
- no arbitrary Lua/source execution;
- no search/model/embedding/CBM;
- no raw SQL/store handle;
- no editor/client mutation;
- no private source/credential/path leak;
- source/correction text remains bounded evidence data;
- all entities/facts/transitions/evidence/output/time/memory are bounded and cancellable.

## Required evaluation

- explicit stable reference identity;
- explicit rename/move;
- deprecation without target;
- explicit replacement;
- unresolved/conflicted replacement target;
- signature/type/optionality/nilability/multiple-return changes;
- restriction tightened/relaxed/unknown/conflicted;
- introduced/removed complete and partial coverage;
- correction apply/expire/reject/conflict transitions;
- same-name near-match with no explicit transition;
- annotation projection disagreement;
- cross-product/build mismatch;
- 1/2/N and shuffled source/correction order.

## Nonclaims

Reference producer output does not by itself establish:

```text
project or Blizzard source entity continuity
runtime behavior, spell secrecy, combat or taint state
automatic edit compatibility
migration execution success
whole-project static impact
```
