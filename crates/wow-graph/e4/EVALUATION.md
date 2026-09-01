# E4-B lineage, migration, and impact evaluation

**Status:** normative. Evaluation distinguishes discovery recall from accepted-authority correctness.

## Evaluation goals

E4-B must prove:

- useful bounded lineage candidate recall;
- correct proof ceilings and promotion;
- zero false accepted identity/replacement/removal/introduction authority;
- explicit ambiguity, split/merge/copy handling;
- exact typed change classification;
- migration candidate/recipe honesty;
- bounded static-impact path correctness without runtime overclaim;
- update/removal/producer-replacement closure;
- deterministic snapshots, queries, continuations and explanations;
- privacy/license/security/resource limits.

## Corpus classes

```text
synthetic exact stable-identity continuity
synthetic exact explicit transition/deprecation/replacement
synthetic rename and move
synthetic copy versus move
synthetic split and merge
synthetic introduced/removed under complete coverage
synthetic unmatched under partial/conflicted/truncated coverage
synthetic signature/type/restriction/ownership/load/relation changes
synthetic migration candidate and validated-recipe boundaries
synthetic static-impact direct/path/possible/conflict cases
same-name and same-path unrelated hard negatives
identical body/fingerprint copied/vendor/generated hard negatives
high search-rank false-lineage hard negatives
cross-universe/project-reference-Blizzard collision corpus
pinned user-addon before/after generations
pinned Blizzard UI source before/after generations
pinned ReferenceView before/after profiles
adversarial review/privacy/license/resource/cancellation corpus
```

Real paired corpora remain null/unaccepted until exact generations, licenses, owner manifests and ground truth are reviewed and frozen.

## Ground-truth classes

```text
ProvenContinuity
DerivedContinuity
PossibleContinuity
CandidateOnly
NoLineageRelation
Rename
Move
CopyOrExtractionCandidate
Split
Merge
SignatureChange
TypeChange
RestrictionChange
OwnershipChange
LoadRoleChange
RelationSetChange
DeprecatedWithoutReplacement
ExplicitReplacement
RemovedWithAuthority
IntroducedWithAuthority
UnmatchedPartial
MigrationCandidateOnly
ValidatedMigrationRecipe
DirectStaticImpact
BoundedTransitiveStaticImpact
PossibleImpact
NoImpactWithinRequestedClosedScope
NotEvaluated
Conflict
```

Ground truth records exact evidence, profile and coverage. Annotator prose alone is not truth.

## Metrics

Report separately by universe, entity kind, producer, relation/change kind, ambiguity shape, proof class and coverage state:

- candidate-pair recall and candidate reduction ratio;
- component recall and size distribution;
- accepted lineage precision/recall by proof ceiling;
- false `Proven`/`Derived` lineage count;
- false move/rename/split/merge count;
- copy-as-move and ambiguity-collapse count;
- false replacement/deprecation/migration-recipe count;
- false removed/introduced count;
- unmatched partial honesty;
- field/relation change precision/recall;
- migration precondition/validation closure;
- impact target/path precision/recall under exact scope;
- false runtime/severity/safety/fixability statements;
- explanation/proof/evidence/coverage closure;
- deterministic ID/byte/order/page equality;
- build/query/impact latency, CPU, memory, pair counts, component size and store bytes;
- privacy/license/source leakage;
- stale proposal/assertion/change/index leakage after producer/generation update.

Unknown/partial/conflict/NotEvaluated cases remain visible in denominators/partitions.

## Hard gates

Reject regardless of average recall if any occurs:

```text
false accepted same-lineage/successor assertion above Candidate/Possible ceiling
false move/rename/split/merge acceptance
copy classified as move without evidence
false replacement/deprecation/migration recipe
false removed/introduced under incomplete coverage
same-name/path/body/fingerprint/high-rank candidate silently promoted
one-to-many or many-to-one ambiguity forced into a bijection
rejected/deferred/conflicting proposal lost
source/reference/search/review authority class collapsed
reason path flattened into a direct relation
static impact rendered as runtime breakage/severity/safety/fixability
cross-universe or cross-generation identity merge
nondeterministic snapshot/query/page bytes
unbounded all-pairs or graph traversal
private/license/source-data leak
hidden partial/conflict/truncation/failure/cancellation
```

## Candidate-generation evaluation

Measure each blocking stage:

- before/after entity coverage;
- bucket count/size;
- generated pair count;
- true-pair recall;
- hard-negative pairs;
- truncation/overflow;
- component shapes/sizes;
- work avoided versus theoretical all-pairs;
- deterministic output.

An efficiency improvement that drops qualifying true pairs without explicit coverage downgrade is rejected.

## Proof and review evaluation

For every accepted assertion recompute:

```text
relation-kind ceiling
producer/input ceiling
reviewer ceiling
coverage/conflict ceiling
effective confidence
```

Mutate each decisive input/attestation/coverage record and prove the assertion downgrades, conflicts, disappears or becomes NotEvaluated.

Review-order and reviewer-count mutations must not act as majority vote unless a specific reviewed profile explicitly requires a fixed independent-review set.

## Removal/introduction evaluation

Corpora must include:

- complete true removal/introduction;
- partial after/before source inventory;
- excluded package/root/entity kind;
- candidate-generation truncation;
- unresolved high-rank candidate;
- privacy/license-hidden entity;
- producer failure/cancellation;
- cross-profile build mismatch;
- deleted source but retained stale graph/search document mutation.

False removal/introduction maximum: zero.

## Change-classification evaluation

Test every typed state transition:

```text
Known/ExplicitNull/Missing/Unknown/Unsupported/Omitted/Conflict/NotEvaluated
```

Test compound changes, relation diffs, direct versus path, optionality/nilability/multiple returns, restriction authority separation, and endpoint-lineage ambiguity.

## Migration evaluation

Separate metrics/gates for:

- candidate relevance;
- explicit replacement accuracy;
- recipe precondition completeness;
- transformation schema validity;
- forbidden-case coverage;
- postcondition/validation-plan closure;
- runtime/client checks correctly marked pending;
- no executable edit or success claim.

A useful candidate does not count as a validated recipe.

## Impact evaluation

Freeze exact roots, graph snapshots, allowed relation profiles and expected direct/reason paths. Measure:

- exact target/path recall/precision;
- confidence/proof cap correctness;
- possible/conflict/coverage/truncation honesty;
- cross-universe bridge correctness;
- cycle/high-fanout bounds;
- continuation stability;
- zero path-to-direct-edge or runtime-breakage overclaim.

“No impact” is accepted only for the exact closed requested relation scope with complete traversal coverage.

## Anti-overfitting mutations

For named real corpora mutate irrelevant:

- repository/owner/provider names;
- package display text;
- directory layout and nonsemantic local names;
- source ordering when semantics do not depend on it;
- duplicate documentation/comments;
- popularity metadata.

Expected semantic lineage output remains unchanged except exact source identities that legitimately changed.

Mutate decisive evidence:

- stable identity removed/conflicted;
- explicit Reference transition removed/changed;
- old entity retained to turn move into copy ambiguity;
- candidate component split/expanded;
- source/profile coverage downgraded;
- exact name/signature/type/restriction/owner/load relation changed;
- review authority/decision changed;
- target dependency edge removed;
- graph/query budget truncated.

Expected assertions/changes/impact must change.

## Determinism

Run 1/2/N workers and randomized:

- entity/input partition order;
- blocking bucket order;
- search signal order;
- proposal/review order;
- graph/store row/order/layout;
- cache cold/warm;
- temp roots/host/locale/timezone.

Compare canonical proposal/component/assertion/conflict/change/migration/impact/query/page IDs and bytes. Operational timings remain noncanonical.

## Quantitative gates

Freeze after executable measurement:

```text
candidate recall minimum by corpus/relation class
maximum pairs per entity/bucket/component/request
maximum candidate reduction/work budget
accepted lineage precision/recall by proof class
zero false authority counts
change/migration/impact precision/recall thresholds
latency/CPU/memory/store-size/fanout/continuation thresholds
```

Missing implementation/corpus/ground truth/threshold/benchmark remains blocked/NotEvaluated.

## Evaluation report

```text
LineageEvaluationReport
    exact implementations/profiles/corpora/generation IDs
    ground-truth and mutation manifests
    producer/block/component metrics
    proof/review/removal/change/migration/impact metrics
    false-authority hard gates
    security/privacy/license/determinism/performance results
    unknown/partial/NotEvaluated partitions
    accepted/rejected decision
    canonical digest
```

Evaluation is read-only and never changes the relation/profile under test.
