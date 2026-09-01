# `wow-graph` E4-B lineage, migration, and static-impact contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-graph/e4-b/lineage-migration-static-impact`

## Mission

Represent cross-generation relationships and change consequences explicitly, without rewriting generation-local entity identity or converting search similarity into proof.

```text
exact LineageUniverseSet / GenerationComparisonSet
+ project-owned stable identity, source fingerprint and structural change facts
+ ReferenceView explicit transition/deprecation/correction facts
+ E4-A SearchCandidateSignals capped at Candidate
+ reviewed lineage/replacement/change/impact profiles
-> producer-separated LineageProposal partitions
-> exact schema, scope, evidence, coverage and proof-ceiling validation
-> ambiguity components, conflicts and review decisions
-> immutable LineageGraphSnapshot
-> explicit lineage/change/migration records
-> bounded static impact reason paths
```

## Core distinctions

```text
GenerationEntity
    exact entity in one immutable source/reference/project/graph generation

LineageProposal
    one producer's evidence-bearing proposed cross-generation relation

LineageAssertion
    accepted relation under an exact proof ceiling and comparison profile

ChangeRecord
    observed difference between exact entities/generations

ReplacementRelation
    old entity is superseded by a distinct target; not identity continuity

MigrationCandidateOrRecipe
    bounded guidance with explicit evidence, preconditions and validation state

StaticImpactResult
    bounded path from a changed entity/record to exact dependent entities
```

## Active E4-B relation families

```text
lineage_successor_of
same_lineage_as
moved_from
renamed_from
split_from
merged_from
introduced_in
removed_after
signature_changed_from
type_changed_from
restriction_changed_from
ownership_changed_from
load_role_changed_from
relation_set_changed_from
deprecated_by
replaced_by
migration_candidate_to
```

The registry defines direction, symmetry, multiplicity, one-to-one/one-to-many/many-to-one policy, evidence requirements, proof ceilings, inverse relations, conflict behavior and query axes.

## Active input producers

```text
project_stable_identity
project_source_fingerprint
project_structural_change
reference_explicit_transition
reference_deprecation_or_replacement
search_lineage_candidate
review_decision
```

Each producer owns an independently replaceable partition. No producer overwrites another producer's proposal or assertion.

## Proof ceilings

```text
Proven
    exact owner-stable identity continuity or explicit authoritative transition under the selected profile

Derived
    deterministic conclusion from exact complete inputs under an accepted unique rule

Possible
    structure permits a relation but ambiguity or dynamic/incomplete facts remain

Candidate
    retrieval, similarity, fingerprint, same-name/path, graph-neighborhood or other discovery evidence only
```

Search rank, multiple approximate signals, same name/path, content similarity, model output or popularity never exceeds `Candidate` by itself.

## Comparison scope

Lineage exists only between exact compatible generations of the same universe class:

```text
user project generation A -> user project generation B
Blizzard UI source generation A -> Blizzard UI source generation B
ReferenceProfile/ReferenceGeneration A -> B
```

Project-to-reference and project-to-Blizzard relationships remain generation-local bridge/usage relations, not lineage.

## Active operations

```text
validate_lineage_profiles
bind_lineage_universe_set
validate_lineage_input_partitions
generate_lineage_candidate_components
validate_lineage_proposals
apply_lineage_review_decisions
plan_lineage_graph_publication
publish_lineage_graph_snapshot
open_lineage_graph_view
compare_entity_generations
classify_generation_changes
trace_lineage
explain_lineage_assertion
propose_migration_candidates
validate_migration_recipe
plan_static_impact
run_static_impact
explain_static_impact
validate_lineage_graph_snapshot
```

## Direct dependencies

```text
wow-core
wow-store
```

`wow-project`, `wow-reference`, and `wow-search` own their E4-B producer inputs. E4-C service orchestration supplies them to `wow-graph`; no reverse dependency is created.

## Hard boundaries

- generation-local E2 entity keys and GraphSnapshots are immutable;
- no cross-generation entity merge or ID rewrite;
- no greedy first/nearest/best-ranked candidate promotion;
- no forced one-to-one matching when split/merge/ambiguity exists;
- no `Removed`/`Introduced` claim without complete exact coverage;
- no replacement relation inferred from same lineage alone;
- no migration recipe without explicit preconditions, transformation and validation state;
- no static impact path rendered as runtime breakage, severity or fixability;
- no path flattened into a direct dependency;
- no raw source parsing, SearchStore querying, ReferenceView reconstruction or current resolution inside `wow-graph`;
- no LLM/model/embedding/CBM authority;
- no unbounded all-pairs candidate generation or graph traversal;
- no raw SQL/storage handles;
- no Rust/Cargo/CI during the documentation phase.

## Completion gate

E4-B implementation is complete only when exact before/after generations remain distinct; every accepted assertion preserves proposal/producer/evidence/coverage/proof-ceiling closure; ambiguous candidate components cannot silently collapse; split/merge/copy/delete/introduce/replacement hard negatives behave correctly; removal/introduction require complete scope; search candidates remain Candidate until independent evidence/review authorizes more; migration and replacement remain distinct from identity continuity; impact results preserve exact reason paths and `Possible`/partial/conflict state; publication is immutable and deterministic; and all paired-generation, mutation, security, resource, 1/2/N worker and checksum gates pass.
