# E5-A evaluation, metrics, and promotion-eligibility gates

**Status:** normative.

## Evaluation layers

```text
1 pack/schema/security validation
2 corpus admission/provenance/license/label closure
3 split and leakage validation
4 per-example shadow matching
5 independent graph proposal validation
6 mutation invariance/sensitivity
7 per-rule/role/corpus/split metrics
8 determinism and resource benchmarks
9 anti-overfitting/generalization-scope assessment
10 candidate-artifact and deactivation-plan validation
```

Every lower layer must complete before a higher-layer pass can be claimed.

## Case classification

For each expected item and observed proposal:

```text
TruePositive
FalsePositive
FalseNegative
TrueNegative when the closed evaluation scope makes it meaningful
ExpectedPossible / UnexpectedPossible
ExpectedNotEvaluated / UnexpectedNotEvaluated
UnknownExcluded
ConflictBlocked
PartialOrTruncated
GraphRejected
AuthorityUpgradeViolation
```

The report records matching policy for multiple proposals/labels; no greedy first match that hides duplicates or ambiguity.

## Denominators

Precision/recall reports explicitly state:

- which label classes are included;
- split/provenance/role/rule/universe filters;
- whether counts are examples, expected outputs, proposals or independent groups;
- Unknown/NotEvaluated/Conflict/Partial/Truncated exclusions;
- duplicate/copy grouping;
- weights and unweighted counts.

A percentage without its denominator/profile is invalid.

## Mandatory hard gates

Regardless of aggregate thresholds:

```text
zero repository/addon/owner/path/popularity semantic conditions
zero forbidden confidence/provenance/negative-authority upgrades
zero graph-schema-invalid accepted proposals
zero hidden mandatory false positives
zero label/split/holdout leakage
zero source/model/executable-control violations
zero nondeterministic semantic outputs
all repository/path/local-name invariance cases pass
all decisive literal/structural/coverage sensitivity cases pass
all required near-miss families execute
all partial/conflict/Unknown/NotEvaluated states remain honest
deactivation plan proves partition-local removal
```

## Quantitative thresholds

Before implementation, exact thresholds remain `null`. They are selected only after baseline measurements on frozen corpora and cannot be set after observing a candidate solely to admit it.

Threshold profiles may include:

```text
minimum mandatory-label precision by role/rule/split
minimum recall by role/rule/split
maximum unexpected Possible/NotEvaluated rate
maximum graph rejection rate (normally zero for eligible proposals)
maximum per-partition latency and memory
maximum proposal/fact amplification
minimum independent provenance-group coverage
maximum degradation versus E2 core baseline
```

Thresholds cannot override hard gates.

## Split interpretation

- Train: diagnostic, not promotion evidence.
- Dev: tuning evidence, reported separately.
- Test: fixed candidate evaluation; consumed after influencing changes.
- SealedHoldout: final untouched evidence for the declared candidate generation.
- Challenge: mutation/security/resource behavior; not natural generalization count by default.
- Quarantine: excluded with exact reasons, never silently negative.

## Per-group reporting

Report results by:

- repository revision for reproducibility;
- upstream/fork/copy provenance group for independence;
- structural-shape group;
- role/relation/rule;
- fact adapter/schema/profile;
- ordinary versus mutation/challenge cases.

A large donor cannot dominate summaries without visible per-group failures.

## Baselines

Compare against:

```text
NoCalibrationPack
CurrentFrozenCorePack
PreviousCandidateVersion if any
MinimalDecisiveRule baseline where defined
```

A candidate must not reduce core outputs or modify core partitions. Shadow overlap/conflict is measured explicitly.

## False-positive investigation

Every mandatory FP records one of:

```text
implementation defect
rule contract defect
fact adapter/parser coverage defect
graph schema/seam defect
corpus label defect
expected ambiguity/Unknown
profile incompatibility
security/resource failure
```

The case remains failed/blocked until a versioned resolution. Do not relabel or drop it in the same run.

## Promotion eligibility states

```text
NotEvaluated
BlockedByAdmission
BlockedByLeakage
BlockedByLabels
BlockedByHardGate
BlockedByMetrics
BlockedByResources
BlockedByLicensePrivacy
ShadowValidated
PromotionEligibleByMetrics
```

`PromotionEligibleByMetrics` means the frozen E5-A criteria passed. It is not an approved promotion, core pack, default rollout or runtime proof.

## Determinism

Equivalent exact inputs under:

```text
1/2/N workers
shuffled facts/examples/partitions/evidence
cold/warm evaluator cache
different host/temp roots
independent materialization/build histories reaching identical logical facts
```

must produce identical pack compilation, case results, proposal partitions, metrics, gate decisions, candidate artifact and deactivation plan. Timings/memory are noncanonical benchmark fields.

## Resource evaluation

Measure ordinary and adversarial:

- parse/validation/compile;
- facts scanned and joins/captures;
- matches/proposals/evidence amplification;
- per-partition/corpus time and memory;
- cancellation latency;
- report size.

Missing implementation/benchmark is blocked, never pass.

## Generalization conclusion

The anti-overfitting report can conclude only the scope supported by independent admitted groups. Initial user repositories can establish useful project breadth, but until upstream lineage independence is reviewed they cannot alone establish ecosystem-wide generalization.

## Candidate artifact gate

Build `CalibrationCandidateArtifact` only when:

- all required layers executed;
- hard gates pass;
- quantitative thresholds pass or are explicitly not required for a non-promotion shadow artifact;
- generalization scope is honest;
- license/privacy allows the artifact class;
- deactivation plan validates;
- all IDs, bytes and checksums freeze.
