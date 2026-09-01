# E4-A search to E4-B lineage Candidate handoff

**Status:** normative cross-package boundary. E4-A search remains retrieval-only; E4-B lineage remains graph-owned.

## Purpose

E4-A can help discover potential cross-generation endpoint pairs. Its output is query-relative Candidate evidence only.

```text
exact old endpoint/name/shape facts
-> E4-A query against one exact target SearchUniverseSet
-> ranked exact target entities with lane signals and explanations
-> LineageSearchCandidateBundle
-> E4-B candidate validation/matching/review
```

Search never publishes lineage, replacement, removal, migration, or impact assertions.

## Required bundle fields

```text
LineageSearchCandidateBundle
    exact from/to GenerationEndpoint IDs
    exact SearchUniverseSet/shard IDs and owner generations
    exact SearchProfileSet and E4-A contract/profile IDs
    exact SearchRequest/NormalizedSearchQuery/SearchResult IDs
    source endpoint or query-origin record
    ordered candidate target entity IDs
    every SearchCandidateSignal and ranking explanation
    matched field-origin records
    authority band and canonical integer/ordinal contributions
    skipped/failed/partial/truncated lanes
    search owner/index/query coverage and conflicts
    privacy/license/consumer policy
    budget/cancellation/continuation state
    canonical digest
```

Raw FTS/BM25/floating values may be retained as noncanonical diagnostics when the E4-A profile permits them. They are not lineage features unless converted to a frozen Candidate-only ordinal record.

## Proof ceiling

Every imported E4-A signal has:

```text
maximum_lineage_proof_ceiling = Candidate
```

This includes:

- exact canonical name match;
- exact explicit alias match;
- prefix/member match;
- text/FTS match;
- identifier similarity;
- structured shape match;
- seeded graph expansion;
- combined top rank.

An exact name/alias result proves only the E4-A owner-record string relation within its shard. It does not prove continuity or user migration intent across generations.

## Forbidden conversions

```text
search top-1 -> same_stable_identity
same name -> renamed/moved continuity
similar shape -> replacement
no search hit -> removed/no replacement
many matching lanes -> Derived/Explicit
search graph path -> direct lineage edge
snippet text -> platform/source evidence
rank score -> proof confidence
```

## Endpoint mapping

Every candidate must resolve to an exact E4-B target endpoint in the intended generation pair. Search detail handles are validated against exact owner views. Same-name entities in another universe/profile/generation are not silently remapped.

## Query construction

E4-C service may generate one or more explicit structured E4-A requests from exact old endpoint fields, for example:

```text
canonical identifier/name
reviewed explicit alias
entity kind/namespace/receiver
signature/type/restriction shape
bounded owner/relation neighborhood
```

The request construction profile is frozen and non-model-based. Source prose is not converted to search intent.

## Bidirectional discovery

Profiles may query:

```text
from endpoint -> target generation candidates
and/or
to endpoint -> source generation candidates
```

Mutual retrieval can be a Candidate feature, not proof. Both directions preserve their exact query/result identities and coverage.

## Search miss

A search miss remains an E4-A miss classification. It cannot authorize `removed_in`, `introduced_in`, or “no replacement” without E4-B owner inventory, candidate-lane, ambiguity, and negative-authority gates.

## Truncation and continuation

A truncated search result cannot support closed candidate enumeration. E4-B records the component/lane as partial and blocks authoritative absence.

Continuation must retain exact E4-A shards/query/profiles/budgets and exact E4-B generation pair. It cannot rerun against current/new shards.

## Privacy and source

Search snippets/highlights are presentation data. E4-B uses exact field-origin/source/reference/graph handles for evidence. Source bytes remain governed by owner privacy/license policies.

## Evaluation

The joint corpus measures:

```text
candidate recall at bounded K by lane
false accepted lineage from search-only evidence = 0
false proven replacement from search-only evidence = 0
candidate diversity across ambiguous/split/merge cases
coverage/truncation honesty
stable candidate order and explanations
```

Recall cannot override a proof-ceiling violation.

## Dependency rule

`wow-graph` does not import or call `wow-search`. E4-C orchestration obtains the exact bundle and submits it through an E4-B typed input seam. E4-A remains independently usable without E4-B.
