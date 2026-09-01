# E4-A coverage, authority, and E4-B lineage boundary

**Status:** normative.

## Authority classes

Search preserves, but does not redefine:

```text
owner project/source fact
Blizzard UI implementation-source fact
Reference Pack platform-contract fact
graph assertion/path/conflict
explicit owner alias/deprecation metadata
existing finding metadata carried by an owner view
query-relative retrieval signal
```

Query-relative signals never become owner facts.

## Coverage dimensions

Keep separate:

```text
owner entity enumeration coverage
owner field/source/detail coverage
document projection coverage
field/origin projection coverage
exact identity index coverage
canonical-name coverage
explicit-alias coverage
namespace/member/receiver coverage
prefix/similarity feature coverage
FTS content/index/query coverage
shape-feature coverage
graph seed/edge/path/query coverage
lane execution coverage
fusion/explanation coverage
result-set/pagination/continuation coverage
privacy/license/snippet coverage
SearchStore integrity/read coverage
evaluation corpus coverage
```

No single “index complete” flag collapses these dimensions.

## Entity authority versus query match

A candidate carries both:

```text
entity authority
    what exact owner evidence supports about the entity

query match class
    why this query retrieved the entity
```

Examples:

- A `ReferenceApiEntity` can have strong platform-contract evidence while its text match remains approximate.
- An exact source function name can match exactly while the source entity still does not establish public API support.
- A graph path can be proven as static structure while remaining weak evidence of query intent.

## Exact-name nonclaim

An exact canonical-name match establishes:

```text
query normalized name == indexed canonical name
```

It does not establish:

- that the user intended this entity;
- that same-name entities in other universes are identical;
- that a current API replaces an old one;
- that source implementation is public contract;
- that an operation is runtime safe.

## Explicit alias nonclaim

An exact alias signal establishes only the exact owner-recorded alias relation in its profile/generation. It does not infer:

- cross-generation lineage;
- replacement equivalence;
- behavioral compatibility;
- migration safety;
- intent certainty.

## Approximate lanes

Text, similarity, shape, and graph signals are candidate evidence only. Their results cannot support authoritative absence, alias creation, lineage, replacement, impact, or remediation.

## Search miss authority

Search may report exact scoped absence only when the owning domain already supports negative authority and all relevant owner/document/index/lane/privacy/conflict/truncation gates are complete.

Search itself cannot create domain negative authority from an empty index.

## Conflicts

Preserve conflicts such as:

```text
multiple exact same-scope canonical entities where uniqueness was expected
explicit alias with conflicting targets
owner entity or field conflict
Reference/source profile mismatch
graph endpoint/path conflict
index manifest versus owner manifest conflict
privacy/license conflict
ranking explanation inconsistency
```

Do not pick first/last/most popular or highest text score to resolve them.

## E4-B handoff

E4-A can emit `LineageCandidateInput` only as a query-relative candidate package:

```text
source exact entity/generation
target exact entity/generation
all retrieval signals and field origins
shape differences
explicit owner deprecation/alias facts if present
graph paths
coverage/conflicts
query and ranking profile
candidate-only status
```

This is not a lineage assertion.

E4-B owns:

```text
same entity across generations
renamed or moved
replaced by / superseded by
introduced / removed
migration compatibility
patch impact
proof ceilings and reviewed resolution
```

E4-B must independently evaluate candidates against exact before/after generations, owner facts, graph/source/reference evidence, and coverage. It cannot accept E4-A rank as proof.

## Search-to-context boundary

```text
query
-> wow-service
-> wow-search ranked candidates and explanations
-> explicit selected exact entity ID
-> wow-service
-> wow-context exact root
```

`wow-context` never receives a fuzzy string as a root. `wow-search` never invokes context or hides candidate selection.

## Existing replacement/deprecation facts

If the exact current ReferenceView contains a deprecation or replacement relation, E4-A may display it as an exact owner fact attached to a candidate. It still cannot infer missing links, cross-generation identity, compatibility, or impact.

## Complete result nonclaim

`Complete` means complete for the exact search request/profile/bound shards and executed lanes. It does not mean:

- all repository/source/reference text was indexed;
- all possible user intents were considered;
- all runtime behavior is known;
- no candidate exists in an omitted universe;
- lineage or migration is decided;
- search output is sufficient context for code modification.

## Validation

- every candidate keeps entity and match authority separate;
- no exact/approximate lane text implies lineage/replacement;
- all coverage axes are present or explicitly unavailable;
- exact misses cite owner negative-authority support;
- conflicts cannot be hidden by ranking;
- E4-B handoff is typed Candidate-only;
- service/context boundaries remain explicit.
