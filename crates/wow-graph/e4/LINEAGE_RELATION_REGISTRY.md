# E4-B lineage relation and change registry

**Status:** normative.

## Registry bundle

```text
LineageRegistryBundle
    relation definitions
    change-kind definitions
    migration relation/recipe definitions
    impact category/axis definitions
    producer and review authority classes
    proof-ceiling rules
    compatibility matrix
    canonical digest
```

A `LineageGraphSnapshot` pins exactly one immutable registry bundle.

## Relation definition

```text
LineageRelationDefinition
    relation kind ID/version
    source/target universe and entity-kind constraints
    direction, optional inverse and symmetry
    comparison-role constraints
    cardinality:
        one_to_one
        one_to_many
        many_to_one
        many_to_many
        boundary_to_entity
        entity_to_boundary
    identity-continuity semantics
    replacement/deprecation semantics
    required evidence classes
    maximum proof ceiling by producer/review class
    complete-coverage requirements
    ambiguity/conflict behavior
    transitivity = none | query_path_only | reviewed_derived
    axis memberships
    compatibility policy
```

## Lineage relations

### `lineage_successor_of`

Directional continuity from exact before entity to exact after entity. It does not by itself state move, rename, replacement or compatibility.

### `same_lineage_as`

Symmetric view over accepted continuity assertions when the registry permits it. It does not merge entity IDs or generations.

### `moved_from`

The accepted lineage entity's exact source/container/location identity changed under a profile that distinguishes move from copy/extraction. Requires old-location absence/continuity evidence where applicable.

### `renamed_from`

The accepted lineage entity's canonical owner name changed. It is not inferred from a fuzzy name match; it requires accepted continuity plus exact before/after name facts.

### `split_from`

One before entity has multiple after descendants under explicit qualifying evidence. Cardinality alone is insufficient.

### `merged_from`

One after entity has multiple before ancestors under explicit qualifying evidence. Cardinality alone is insufficient.

## Boundary/absence relations

### `introduced_in`

An exact after entity has no valid before counterpart in a completely covered closed scope. The source is a comparison boundary record, not a fabricated missing entity.

### `removed_after`

An exact before entity has no valid after counterpart in a completely covered closed scope. The target is a comparison boundary record.

Unmatched under partial/conflicted/truncated coverage does not qualify.

## Typed change relations

These require an accepted lineage/replacement pair plus exact typed before/after facets:

```text
signature_changed_from
type_changed_from
restriction_changed_from
ownership_changed_from
load_role_changed_from
relation_set_changed_from
```

They do not establish identity continuity independently.

## Deprecation and replacement

### `deprecated_by`

Represents an explicit deprecation relation/evidence. It may point to a distinct replacement target, a migration note, or a generation boundary according to the profile. Source comments alone are not Reference authority.

### `replaced_by`

The old entity is superseded by a distinct new entity under explicit authoritative transition/reference/review evidence. Replacement does not imply same identity, signature compatibility, safe automatic edit, or complete migration.

### `migration_candidate_to`

A Candidate/Possible relationship indicating potential migration relevance. It cannot be rendered as replacement or recipe authority.

## Copy/extraction representation

Copy/extraction remains a change/candidate classification rather than an identity relation until evidence establishes split or successor semantics. The registry may expose:

```text
copied_or_extracted_candidate
```

as a nonauthoritative proposal/change record; it is not automatically persisted as accepted lineage.

## Relation evidence classes

```text
OwnerStableIdentity
OwnerExplicitTransition
ReferenceExplicitTransition
ReferenceDeprecationReplacement
ExactSourceHistoryFact
ExactStructuralChangeFact
ReviewedDeterministicRule
ManualReviewDecision
SourceFingerprintCandidate
SearchExactStringCandidate
SearchApproximateCandidate
GraphNeighborhoodCandidate
ExternalOrModelCandidate (future only)
```

The registry maps each class to a maximum proof ceiling by relation kind and profile.

Examples:

- `OwnerStableIdentity` can support `Proven lineage_successor_of` under exact compatible generations.
- `ReferenceExplicitTransition` can support a `Proven replaced_by` only within its exact Reference comparison scope.
- `SearchExactStringCandidate` remains `Candidate` for lineage even though its query signal is exact-name.
- `SourceFingerprintCandidate` remains `Candidate` unless combined with independent owner-stable identity and complete coverage under a deterministic rule.
- `ManualReviewDecision` cannot exceed its configured reviewer authority and input proof ceiling.

## Transitivity

Default:

```text
lineage_successor_of: query_path_only
same_lineage_as: query_path_only
moved_from/renamed_from: none
split_from/merged_from: none
introduced_in/removed_after: none
changed-from relations: none
deprecated_by/replaced_by/migration_candidate_to: none
```

A multi-hop lineage path is not silently materialized as a direct A→C assertion. Any reviewed derived direct relation requires a separate producer rule, explicit profile, all intermediate evidence, conflict/coverage checks and a distinct assertion ID.

## Cardinality and exclusivity

- One-to-one continuity can coexist with multiple Candidate proposals but only accepted assertions obey the relation profile.
- Split/merge relations intentionally allow one-to-many/many-to-one.
- `introduced_in` and `removed_after` are exclusive with an accepted continuity target/source in the same closed scope unless a conflict is recorded.
- `replaced_by` can be one-to-many only when the exact authoritative transition supports it.
- A successor and replacement relation between the same pair can coexist only when both independently qualify; neither implies the other.

## Axes

```text
lineage
change
replacement
migration
impact-root
```

Axes are typed views over accepted assertions/change records, not a universal parent hierarchy.

## Registry validation

Reject:

- generic untyped `same_as`, `changed`, `parent` or `impact` relations;
- relation endpoint constraints that permit cross-universe lineage;
- a Candidate producer ceiling above Candidate;
- relation semantics that merge entity identities;
- automatic unique-candidate promotion;
- removal/introduction without coverage requirements;
- replacement implied by same lineage;
- migration recipe relation without preconditions/validation fields;
- transitive direct-edge materialization by default;
- proof ceilings based on producer count, popularity, name/path or rank;
- executable validators or source-controlled schemas;
- unversioned breaking changes.
