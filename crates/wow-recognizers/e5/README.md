# `wow-recognizers` E5-A calibration corpora and named-pack contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-recognizers/e5-a/calibration-corpora-and-named-packs`

## Mission

Create reproducible, leakage-resistant calibration corpora and declarative named calibration-pack candidates that discover universal structural roles from exact normalized facts without turning donor repository identity into production semantics.

```text
pinned candidate repository revisions
+ exact materialized source/project/analyzer/graph publications
+ independent reviewed expected universal labels
+ positive, clean-negative, near-miss, ambiguous and adversarial cases
+ repository/name/path/local-identifier/structure/coverage mutations
-> admitted immutable CalibrationCorpus
-> frozen train/dev/test/sealed-holdout split manifests
-> declarative trust_class=calibration candidate pack
-> shadow-only matching through the E2-B engine
-> graph-independent proposal validation
-> per-case/per-role metrics, mutation and determinism reports
-> immutable CalibrationCandidateArtifact
```

E5-A never promotes a candidate pack to `core`, enables it by default, publishes its proposals into the default project graph, or claims runtime behavior.

## Direct dependencies

```text
wow-core
wow-emmy
wow-graph
```

The direct dependency set is unchanged from E2-B. Exact project/TOC/XML/source facts are supplied as normalized `RecognizerFactBundle` artifacts by orchestration; `wow-recognizers` still does not depend on `wow-project`, `wow-service`, applications, storage, search, context, rules, or the external KB.

## Active inputs

- E2-B fact, pack, match, output-partition, coverage, security and mutation contracts;
- exact immutable source/project/analyzer/graph publication identities;
- repository/revision/root/file/license/provenance manifests;
- reviewed expected universal entity/relation/role labels;
- explicit unknown, ambiguous, partial, conflict and `NotEvaluated` labels;
- frozen split, mutation, evaluation, budget and canonicalization profiles;
- candidate pack bytes using only the E2-B bounded non-Turing-complete schema.

## Candidate donor set

The initial manifest pins eight user-owned addon revisions as **candidate inputs only**:

```text
UnknownAlienHuman/roth-ui@1656d4b9d33be914be2058460520e7423668d95c
UnknownAlienHuman/roth-chat@3c995183626002965043e38a837346fb290acd8a
UnknownAlienHuman/roth-tooltip@28426fef16daadc5808fec6d38b445a97f42a71a
UnknownAlienHuman/interrupt-glow@786ef9f11059b28541007af92963bc9e2234f154
UnknownAlienHuman/old-runes@9938d95759970953a7ac178a95bb5ad7aa62cb81
UnknownAlienHuman/trash-panda@f27ba9f09be0f716cb2c5f7605ed697d8aabb320
UnknownAlienHuman/gcd-optimizer@00d8bd22f03b1136841f548c0a4a5a776c1a7c71
UnknownAlienHuman/roth-blizz-plates@61de4d4d49ccf229ff3b7bff1ae1b5f97351b762
```

A Git commit pin proves only repository revision identity. Corpus admission additionally requires complete materialization, exact source/fact publication, upstream/fork lineage grouping, license/notice/privacy decisions, independent label review, and split eligibility. These eight revisions alone do not prove ecosystem-wide independence or generalization.

## Active E5-A artifacts

```text
CalibrationCandidateSource
CalibrationCorpusManifest
CalibrationCorpusExample
CalibrationExpectedLabelSet
CalibrationSplitManifest
CalibrationMutationSuite
CalibrationPackCandidate
CalibrationRun
CalibrationCaseResult
CalibrationMetricReport
CalibrationAntiOverfittingReport
CalibrationCandidateArtifact
CalibrationDeactivationPlan
```

## Universal output boundary

A named pack may carry donor/corpus names in metadata and provenance. Match clauses and output semantics cannot branch on:

```text
repository owner or repository name
addon/package display name except an exact public convention explicitly required by the universal rule
absolute or incidental relative path
Git branch/tag/URL
popularity/download/star rank
local variable/function/frame/table names not declared semantic inputs
source comments/documentation prose
model/embedding/LLM output
corpus split membership or expected label
```

Outputs remain registered universal graph kinds/relations such as `module`, `service`, `library`, `factory`, `registry`, `state_root`, ownership, lifecycle, registration, hook, state or call relations. A donor-specific role name cannot enter the graph registry.

## Trust and rollout states

```text
CandidateCorpusInput
AdmittedCorpus
CalibrationCandidate
ShadowValidated
PromotionEligibleByMetrics
PromotionSubmissionPrepared
Rejected
Quarantined
Superseded
```

`PromotionEligibleByMetrics` is not approval or rollout. E5-B owns durable calibration runs, reviewer authorization and promotion submissions. E5-C owns immutable core-pack publication, canary, rollout, rollback and last-known-good state.

## Public operations

```text
validate_calibration_candidate_source
validate_calibration_corpus_manifest
admit_calibration_corpus
validate_calibration_split_manifest
build_calibration_fact_snapshot
validate_expected_label_set
validate_calibration_pack_candidate
run_calibration_pack_shadow
run_calibration_mutation_suite
evaluate_calibration_pack
compare_calibration_runs
explain_calibration_case
build_calibration_candidate_artifact
validate_calibration_deactivation_plan
```

## Hard boundaries

- exact pinned revisions and owner publications only; no floating current/latest;
- corpus labels are independent review artifacts, never copied from recognizer output;
- no source execution, repository hooks, workflows, generators, package managers, tests or addons;
- no second Lua/TOC/XML parser or raw-source fallback;
- no model/embedding/LLM correctness, labeling or tuning path;
- no repository/addon/owner/path/popularity branch condition;
- no training/test/holdout lineage leakage;
- no hidden Unknown-to-Negative conversion or label weakening to improve metrics;
- no weighted aggregate that hides a mandatory false positive;
- no confidence/provenance/negative-authority/runtime/safety upgrade;
- no mutation of E2 core pack or graph registry;
- no default graph publication, source edit, runtime probe, service/CLI, LSP/MCP, release or CI.

## Completion gate

E5-A is complete only when each admitted corpus item has exact revision/materialization/publication/license/provenance/label closure; split groups prevent upstream/fork/copy leakage; every decisive rule clause has positive and near-miss negative coverage; repository/addon/owner/path and irrelevant-local-name mutations preserve semantic output; decisive literal/structural/coverage mutations change output exactly as declared; no required case is hidden by weighting; all candidate outputs are universal, graph-validatable, Derived/Possible only, shadow-only and producer-partition isolated; deactivation removes only the pack's partitions/coverage; independent build histories and 1/2/N workers produce identical artifacts; and every fixture, threshold, benchmark and SHA-256 gate is frozen before Rust implementation.
