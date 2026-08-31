# E2-B recognizer mutation and evaluation contract

**Status:** normative quality gate.

## Purpose

A recognizer can appear correct while merely memorizing repository names, paths, identifiers, or positive examples. Mutation evaluation must demonstrate that each rule responds to structural semantics, not accidental corpus labels.

## Corpus classes

### Closed synthetic fixtures

Small source-independent fact bundles for every clause/output/error path.

### Cross-shape fixtures

At least two independently shaped examples for a universal convention where practical.

### Near-miss negatives

Structurally similar facts missing one decisive relation, symbol resolution, producer, literal, or coverage requirement.

### Pinned implementation evidence

Later calibration/evaluation may use pinned repositories from the current external KB map and user repositories. They remain implementation evidence, not WoW API authority.

## Required mutations

### Repository identity

Rename owner/repository/addon metadata. Output must remain identical.

### Path topology

Move/rename source directories/files while preserving semantic fact IDs/ownership as the project contract allows. Rules that change only because of path substrings fail.

### Local identifier names

Rename local variables/functions/frames/tables when the rule does not declare the name as a public convention. Output semantics must remain.

### Convention literal

Change `CreateFrame`, `RegisterEvent`, `TriggerEvent`, script name, library key, or other decisive exact literal. The relevant rule must stop/change exactly as declared.

### Structural edge

Remove/change a TOC dependency, XML parent/inherits link, call receiver, callback producer, state root, or literal path segment. Expected graph proposal changes must occur.

### Resolution/confidence

Convert exact target to dynamic/ambiguous/unresolved. Derived output must become Possible/NotEvaluated or disappear according to policy.

### Coverage

Change Complete to Partial/Failed/Unknown or truncate the searched partition. No `EvaluatedNoMatch` or negative clause success may survive incorrectly.

### Duplicate/shuffle

Duplicate exact facts/evidence and randomize serialization, enumeration, partition, and worker order. Canonical matches/proposals remain stable while supporting evidence is retained.

### Producer version/replacement

Change rule/pack version, remove a rule, or replace input partition. Stale producer assertions must be removed only from the owned partition.

### Adversarial scale

Increase fanout/captures/joins/outputs to profile limits and beyond. Matching truncates/fails deterministically without graph explosion.

## Labels

```text
Positive      expected role/relation exists
Negative      expected role/relation must not exist
Possible      exact ambiguity is expected
NotEvaluated  required capability intentionally unavailable
Unknown       corpus cannot authoritatively label result
```

Unknown is never coerced to negative.

## Metrics

Per rule/role/relation:

```text
TP, FP, FN, TN when meaningful
Possible expected/observed
Unknown/unlabeled
NotEvaluated/Partial/Truncated
precision/recall with explicit denominator
proposal graph-validation rejection count
mutation invariance/sensitivity results
determinism and resource usage
```

A precision report includes the exact corpus/profile/revision and every excluded category.

## Promotion gates

Before default rollout:

- all closed fixtures and mutations pass;
- no repository/addon/path branch condition exists;
- near-miss negatives exercise every decisive clause;
- exact literals are justified and mutation-sensitive;
- ambiguity/partial coverage remains honest;
- graph proposal validation passes;
- frozen metric thresholds pass on the frozen corpus;
- no unexplained false positive/negative in mandatory labels;
- output remains deterministic and bounded.

## Calibration-pack rules

E5 named packs must additionally prove:

- the pattern generalizes beyond the named source where claimed;
- removing the pack reduces coverage only;
- core graph semantics remain unchanged;
- repository rename/path mutation does not change output;
- pack metadata/provenance does not enter match conditions;
- licenses permit fixture/pattern use.

## Review workflow

A failed corpus case results in one of:

```text
implementation defect
rule contract defect
fact adapter/parser coverage gap
graph registry/seam defect
corpus label defect
expected ambiguity/unknown
profile incompatibility
```

Do not change a label or weaken a negative fixture merely to make metrics green.

## Reproducibility

Evaluation report records:

- exact commits and fixture digests;
- pack/rule/fact/graph profiles;
- worker count/order seeds or canonical shuffle set;
- budgets;
- output/report digests;
- pass/fail/skipped reason.

Missing tooling is skipped, never pass. No in-client behavior is claimed by offline recognizer evaluation.
