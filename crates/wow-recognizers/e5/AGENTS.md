# AGENTS.md — `wow-recognizers` E5-A

## Scope

Implement calibration-corpus validation, immutable split/label/mutation artifacts, shadow execution of E2-B declarative packs, evaluation, anti-overfitting reports, candidate artifacts and deactivation plans only.

Do not implement source/project indexing, parsers, graph publication, service/CLI orchestration, core-pack promotion, runtime probes, models, source edits, releases or CI.

## Before coding

1. Read repository/crate instructions, E2-B recognizer package, E5-A package, current graph/fact contracts and external KB routing.
2. Freeze exact prerequisite implementations and fixture digests.
3. Freeze every candidate repository revision, materialization/source manifest, project/analyzer/graph publication, fact bundle, provenance/upstream-lineage group and license/privacy decision.
4. Freeze label schema, review evidence, split-group manifest, mutations, thresholds, budget/canonicalization profiles and expected bytes before the first E5-A Rust commit.
5. State which universal role/relation and decisive fact clauses the candidate pack is intended to recognize.

## Corpus discipline

- A Git revision is not an admitted corpus.
- Admission requires exact source/fact/evidence/license/label closure.
- Keep raw source, normalized facts, labels, mutations, pack bytes and evaluation outputs as separate immutable artifacts.
- Do not infer labels from repository/addon names, README prose, current recognizer output, search rank or model output.
- `Unknown` is not `Negative`; `Possible`, conflict, partial and `NotEvaluated` remain distinct.
- Preserve label disagreement and reviewer evidence; do not majority-vote silently.
- Group forks, copies, vendored code, shared libraries and common upstream lineages before splitting.

## Split discipline

- No upstream/fork/copy/near-duplicate group crosses train/dev/test/sealed-holdout.
- Mutation variants stay with their source example unless the profile explicitly defines a disjoint mutation challenge set.
- Sealed holdout labels are inaccessible to pack authors until candidate-pack bytes are frozen.
- Evaluation cannot tune on test/holdout outcomes and rerun them as if untouched.
- Split changes create a new corpus/split identity and invalidate comparisons that assumed the old split.

## Pack discipline

- Use only E2-B declarative bounded operators.
- `trust_class = calibration`; `rollout_state = shadow_only` in E5-A.
- Metadata may name donors; clauses and outputs may not branch on repository/owner/addon/path/popularity/split/label identity.
- Every exact literal must be a justified public convention or structural token and have a sensitivity mutation.
- Do not add an abstraction for one donor shape without at least one independent shape or an explicit donor-local/non-generalization label.
- Outputs use registered universal graph kinds/relations and `Derived`/`Possible` confidence only.
- Candidate packs never modify the E2 core pack or graph registry.

## Evaluation discipline

- Report every case, not only aggregate metrics.
- Mandatory false positives, authority upgrades, graph-invalid proposals, repository-name dependencies, leakage, nondeterminism and security failures are hard blockers.
- Do not change labels, remove negatives, lower coverage requirements, or adjust split weights merely to make a pack pass.
- Record TP/FP/FN/TN only where labels support them; exclude Unknown/NotEvaluated with explicit denominators.
- Report per-role, per-rule, per-corpus-group, per-split and per-mutation results.
- Missing evaluator, corpus, holdout, graph validator, benchmark or threshold is `NotEvaluated`/blocked, never pass.

## Deactivation discipline

- Pack/rule/version/input generation owns an exact producer partition.
- Disabling/removing/superseding a candidate deletes only its own shadow partitions and downgrades only its coverage.
- Core and other calibration partitions remain unchanged.
- A deactivation plan must prove stale assertion and reference closure.

## Security/license

- Never execute repository source, hooks, workflows, scripts, package managers, generated code or addons.
- No filesystem/network/process/editor/client access inside the recognizer library.
- Source comments/docs are inert evidence data.
- No private source or redistributable fixture without exact permission.
- Corpus metadata/labels/reviewer notes cannot become matcher control fields.
- No credentials, private keys or access tokens in fixtures.

## Completion report

```text
candidate pack/corpus/split/run IDs
exact repositories/revisions and source/publication/fact manifests
license/privacy/upstream-lineage/admission state
universal role/relation and decisive clauses
train/dev/test/holdout group counts and leakage checks
positive/negative/possible/unknown/not-evaluated/conflict counts
per-case/per-role metrics and hard-gate failures
mutation invariance/sensitivity and determinism
shadow output partitions and graph validation
candidate/deactivation artifact IDs
benchmarks/checksums and pass/fail/skipped/NotEvaluated
E5-B/E5-C deferrals
```
