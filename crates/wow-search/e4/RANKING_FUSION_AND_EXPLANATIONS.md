# E4-A ranking, fusion, and explanations

**Status:** normative.

## Goals

Ranking must be:

- deterministic;
- authority-preserving;
- explainable;
- shard-independent with respect to raw FTS statistics;
- stable under worker/storage order;
- evaluable without a model;
- incapable of converting similarity into lineage, replacement, or intent proof.

## Candidate fusion

All lane signals for the same exact entity key in the same universe/generation are grouped into one candidate.

Preserve:

- every signal ID and field origin;
- every root/query inclusion reason;
- exact versus approximate match classes;
- shard-local lane ranks;
- graph paths;
- skipped/failed/partial lanes;
- conflicts and coverage;
- caps and penalties.

Do not merge same-name entities across universes or generations.

## Authority bands

Lower ordinal is stronger:

```text
0 ExactEntityIdentity
1 ExactCanonicalQualifiedName
2 ExactCanonicalShortName
3 ExactExplicitAlias
4 ExactNamespaceMemberOrReceiverMethod
5 ExactCaseSensitivePrefix
6 StructuredMultiLaneCandidate
7 TextOrShapeCandidate
8 FoldedPrefixOrIdentifierSimilarityCandidate
9 GraphExpansionCandidate
10 PartialOrConflictLimitedCandidate
```

The accepted profile freezes the exact band set and transitions.

Approximate totals cannot cross into a stronger exact band. If an exact constraint conflicts or coverage is insufficient, a cap can move a candidate to a weaker band.

## Canonical feature vector

Within a band, use a versioned lexicographic tuple of bounded integers/enums. Example:

```text
required_filter_failures ascending
exact_constraint_matches descending
kind_match descending
universe_priority ordinal
namespace_or_receiver_match descending
shape_exact_fields descending
shape_mismatches ascending
best_lane_rank ascending
reciprocal-rank-sum descending
graph_path_cost ascending
identifier_distance ascending
conflict_penalty ascending
coverage_penalty ascending
stable entity/document key ascending
```

The concrete tuple and ranges are frozen. No floating aggregate or host math.

## Reciprocal-rank feature

Where enabled:

```text
rr_contribution = floor(K / (offset + lane_rank))
```

`K`, `offset`, maximum rank, lane caps, and sum range are fixed integers. It combines local ordinals only. Raw FTS/BM25 values do not enter cross-shard fusion.

## Lane contributions and caps

Profiles define:

- which lanes can contribute in each band;
- maximum contribution per lane;
- maximum repeated-field/path contribution;
- exact-match dominance;
- candidate-seed graph cap;
- conflict/partial/truncation penalties;
- privacy/omission handling;
- required-filter exclusion.

Repeated text occurrences or multiple similar graph paths cannot inflate a candidate without bound.

## Universe policy

Universe priority is explicit and query-dependent. It cannot be inferred from popularity or whichever shard returned first.

Examples:

- API query may prefer `ReferencePlatformContract`;
- source implementation query may prefer `BlizzardUiSource`;
- addon symbol query may prefer `PrimaryUserProject`.

Preference is retrieval ordering, not authority collapse. Exact entities from all allowed universes remain separately visible.

## Explanation

Every returned candidate explanation contains:

```text
query and exact SearchUniverseSet
entity/document and owner detail handles
authority band and why
matched exact constraints
all contributing lane signals in canonical order
all caps/penalties and arithmetic
shard-local lane rank and allowed diagnostic score
graph reason paths
stable tie key
skipped/failed/partial lanes
field origins/evidence/provenance/confidence
coverage/conflicts/omissions
explicit nonclaims
```

No opaque single “relevance score” is sufficient.

## Excluded candidates

For candidates generated but not retained in the result-set manifest, preserve bounded aggregate reasons:

```text
hard filter failure
candidate cap
privacy denial
duplicate exact entity merged
lower stable rank beyond limit
lane truncation
conflict policy
```

Debug profiles may retain full traces under stricter budgets/privacy. Production default need not expose unlimited rejected candidates.

## Tie-breaking

Final stable keys use exact canonical IDs only:

```text
universe class/order
entity kind/order
canonical owner entity key bytes
document ID
candidate ID
```

Never rowid, insertion order, source discovery order unless source/load order is an explicit semantic query feature, hash iteration, thread completion, cache history, clock, or random seed.

## Ranking identity

`RankingProfileId` binds:

- authority bands;
- feature tuple order/ranges;
- lane contribution tables;
- caps/penalties;
- reciprocal-rank constants;
- universe/kind ordering;
- tie-key schema;
- conflict/coverage policy;
- explanation schema.

Any semantic change creates a new profile and result identity.

## Validation

- recompute every candidate vector and rank from signals;
- verify exact bands dominate;
- verify no raw cross-shard FTS comparison;
- verify all signals/origins resolve;
- verify candidate dedup and signal retention;
- verify stable total order;
- verify explanation arithmetic equals rank tuple;
- verify privacy/coverage/conflict fields;
- verify no lineage/replacement/intent-authority text.

## Evaluation hard gates

- zero false exact canonical or explicit-alias classification;
- zero false lineage/replacement/migration/impact claims;
- zero unexplained rank differences;
- byte-identical ranked manifests across workers/orders/cache state;
- exact-match recall 100% in complete fixtures;
- approximate recall thresholds frozen only after corpus measurement.
