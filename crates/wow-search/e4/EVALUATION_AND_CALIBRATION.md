# E4-A retrieval evaluation and ranking calibration

**Status:** normative.

## Purpose

Evaluate whether E4-A retrieves and orders useful exact entities while preserving authority, determinism, privacy, and resource limits.

Evaluation cannot change owner facts or waive hard gates.

## Corpus classes

```text
SyntheticExact
SyntheticCollision
SyntheticAdversarial
PinnedUserAddon
PinnedBlizzardUiSource
PinnedReferenceView
CrossUniverse
UpdateHistory
HighFanoutGraph
PrivacyAndLicense
ContinuationAndCancellation
```

Initial real fixtures:

```text
UnknownAlienHuman/roth-ui
    exact pinned commit from wow-project E2-C fixture

Gethe/wow-ui-source or selected materialization
    exact E3-A pinned source snapshot only after its freeze gate

Reference Pack
    exact frozen ReferenceGeneration/Profile
```

Repository names are corpus provenance only, never production ranking features.

## Labeled query record

```text
SearchEvaluationCase
    case ID
    exact corpus/universe/shard generations
    structured SearchRequest
    expected relevant exact entity IDs by graded relevance
    expected exact/alias/match class constraints
    forbidden false-authority outcomes
    required lane/coverage/miss behavior
    privacy/snippet expectations
    result/budget/latency class
    rationale/evidence
```

Labels are reviewed, versioned, and evidence-backed.

## Metrics

Per lane and fused result:

```text
exact-key/name/alias recall
top-1/top-3/top-5 recall
MRR
nDCG where graded relevance is justified
candidate set size
rank displacement by lane
query coverage and lane availability
authoritative exact-miss precision/recall
partial-miss honesty
explanation completeness
deterministic repeat equality
latency CPU memory database/output bytes
continuation stability
privacy/license violations
```

## Hard zero-tolerance metrics

Accepted profiles require zero:

```text
false ExactEntityIdentity
false ExactCanonicalName
false ExactExplicitAlias
false authoritative exact miss
false lineage/replacement/migration/impact claim
cross-universe or cross-generation identity collapse
raw cross-shard FTS comparison
unexplained rank difference
private/disallowed source leakage
silent lane failure/truncation/coverage loss
```

A recall gain cannot offset any hard violation.

## Baselines

Compare against explicit baselines:

- exact-only;
- exact + explicit alias;
- exact + prefix;
- per-lane standalone;
- fused without graph;
- fused without text;
- previous accepted RankingProfile.

Do not compare against vague “search quality.”

## Calibration

Ranking profile changes require:

1. reviewed corpus version and train/tune/test split or leave-one-repository-out design;
2. frozen pre-change results;
3. proposed integer/ordinal feature change;
4. full hard-gate pass;
5. improvements on target metrics without unacceptable regressions;
6. per-universe and collision analysis;
7. deterministic 1/2/N worker replay;
8. documented accepted/rejected decision and new profile ID.

No online learning, user-click feedback, or hidden mutable weights in E4-A.

## Overfitting controls

- repository/addon/provider/path names removed or mutated;
- package/file/local identifiers renamed where semantics permit;
- same-name collision corpora;
- leave-one-project/source-package-out evaluation;
- synthetic counterexamples;
- query text paraphrase only for documentation-text cases, never to create labels;
- feature ablation;
- repeated-string and popularity removal tests.

## Miss evaluation

Authoritative exact miss is evaluated separately from approximate no-candidate. Cases include:

- complete exact key space;
- partial owner enumeration;
- missing alias coverage;
- privacy-hidden document;
- conflict;
- truncated index/query;
- required shard absent;
- corrupt index.

Any false authoritative miss fails the profile.

## Explanation evaluation

Every returned candidate must reconstruct:

- match class and authority band;
- all relevant lane signals;
- feature tuple/arithmetic;
- stable tie key;
- owner origins;
- skipped/failed/partial lanes;
- conflicts/coverage/omissions.

Explanation bytes are deterministic and match the result manifest.

## Performance thresholds

Do not invent values in documentation. Before implementation, run the frozen corpus on declared target platforms and then freeze:

```text
shard build wall/CPU/memory/database size
incremental partition reuse/rebuild
exact/name/alias/prefix query latency
text/similarity/shape query latency
graph expansion latency
fusion/explanation/page latency
cold/warm read behavior
high-fanout and adversarial limits
```

Report distribution and worst-case bounds, not averages only.

## External/model evaluation

No model or external service participates in canonical ranking. A later consumer-utility study may evaluate whether context selection improves after explicit candidate choice, but model preference cannot alter E4-A hard truth/security gates.

## Promotion state

```text
Proposed
Evaluated
Rejected
AcceptedForFixture
AcceptedForImplementation
Superseded
```

Only a fully frozen and hard-gate-passing profile becomes `AcceptedForImplementation`.

## Reports

Evaluation reports retain exact:

- implementation and dependency commits;
- shard/profile/corpus IDs;
- platform/runtime profile;
- query/result manifests;
- metrics and failures;
- excluded/NotEvaluated cases;
- decision and reviewer evidence.

Operational timings are noncanonical; semantic results remain canonical.
