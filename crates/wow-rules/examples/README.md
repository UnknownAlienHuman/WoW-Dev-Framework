# `wow-rules` E0-E normative examples

These files define the closed two-rule input/output contract for the future E0-E implementation.

## Files

- [`rule-fixture.json`](rule-fixture.json) — registry descriptors, fixture profile policy, prerequisite contract identities, capabilities, budgets, and source/fact/query bindings.
- [`api-exists-cases.json`](api-exists-cases.json) — exact found, authoritative absent, partial/conflict/profile/library/dynamic, evidence, root-cause, and remediation cases.
- [`secret-local-cases.json`](secret-local-cases.json) — unsafe, guarded, after-use, different-value, non-dominating, copy/conversion, facet/control-flow unavailable, evidence, and remediation cases.
- [`execution-cases.json`](execution-cases.json) — provider selection, context mismatch, outcome exclusivity, budgets/cancellation, deterministic aggregation, and deferred rule cases.
- [`CHECKSUMS.json`](CHECKSUMS.json) — prerequisite/input/output/checksum freeze gate.

## Current state

No E0 rule implementation exists. The prerequisite implementations and canonical IDs from E0-A through E0-D are not frozen yet. Therefore the examples intentionally contain null fields for:

```text
ReferenceGenerationId
ProjectGenerationId
AnalyzerSnapshotId
ReferenceView/ProjectSnapshot/ProjectView IDs
project/reference/analyzer source/evidence/fact/coverage/conflict IDs
exact lookup/facet/result/authority IDs
RuleRegistry/context/evaluation/finding/clean/NotEvaluated/root-cause/remediation IDs
source spans/content digests
member and bundle SHA-256 values
```

These nulls are valid only while `crates/MANIFEST.json` reports `wow-rules.implementation_state = not-started`.

Before the first `wow-rules` Rust commit, the implementation agent must:

1. freeze and verify the implemented E0-B profile/reference lookup/facet/coverage/evidence vectors;
2. freeze and verify the implemented E0-C project analyzer fact/source/generic finding vectors;
3. freeze and verify the implemented E0-D project generation/snapshot/source registry vectors;
4. derive the exact E0 rule registry, fixture policy, execution context, evaluation, finding, clean, NotEvaluated, root-cause, causal-hint, and remediation IDs;
5. canonicalize all example files using E0-A canonicalization;
6. write actual member and bundle SHA-256 values;
7. update `CONTRACT.json` and manifest implementation state;
8. execute all applicable `TEST_MATRIX.md` cases.

Tests verify frozen examples and never rewrite them automatically.

## Evidence boundary

Examples preserve three independent layers:

```text
project source/analyzer evidence
reference facts or authority inputs
rule derivation evidence
```

Do not replace them with one status string.

## API absence boundary

`api-exists-cases.json` never treats an analyzer unresolved fact or empty lookup as platform absence. Only the exact ReferenceView `authoritative_absent` outcome can produce the E0 finding.

No absent reference source handle, replacement candidate, or source edit exists.

## Secret boundary

`secret-local-cases.json` requires:

```text
exact producer/return-slot facet
exact local value identity
exact supported operation
exact fixture guard policy
proven dominance when clean
```

Annotations/names/types alone are insufficient. Copy/conversion does not declassify. The fixture guard policy is synthetic and not production/runtime authority.

## Output boundary

For one canonical rule evaluation scope exactly one primary outcome:

```text
Findings
EvaluatedClean
NotEvaluated
Failed
Cancelled
```

Every E0 remediation is `plan_only`; no edit or replacement is included.

## Change protocol

A semantic change must update:

- owning rule/provider/capability/finding documents;
- `CONTRACT.json`;
- affected example cases;
- `TEST_MATRIX.md`;
- prerequisite/output/checksum vectors after implementation starts.

Do not weaken coverage, authority, guard, evidence, or publication expectations merely to make an implementation pass.
