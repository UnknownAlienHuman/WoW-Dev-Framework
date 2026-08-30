# `wow-service` E0-F normative examples

These files define the closed `status`/`check` orchestration and result-envelope contract for the future E0-F implementation.

## Files

- [`status-result.json`](status-result.json) — healthy/deferred component state with exact configured identities and no false check/test/runtime claim.
- [`check-clean-result.json`](check-clean-result.json) — complete exact scope with explicit rule clean records, no raw findings/blockers/truncation, semantic `clean`.
- [`check-findings-result.json`](check-findings-result.json) — full E0 scope with accepted generic findings, fixed API/Secret rule outcomes, raw preservation, presentation roots/children, semantic `findings`.
- [`check-partial-result.json`](check-partial-result.json) — coherent result with findings and exact `NotEvaluated` blockers, semantic `partial`.
- [`check-context-error.json`](check-context-error.json) — exact generation/profile/context failure; no malformed check envelope.
- [`check-cancelled-result.json`](check-cancelled-result.json) — cancellation before publication; no late result.
- [`CHECKSUMS.json`](CHECKSUMS.json) — E0-A–E prerequisite, context, result, graph, CLI, and byte-freeze gate.

## Current state

The repository still contains no E0 Rust implementation. E0-A through E0-E prerequisite identities and canonical outputs are not frozen. Therefore these fields remain null:

```text
implementation IDs and prerequisite bundle SHA-256 values
ReferenceGenerationId / ProjectGenerationId
ReferenceView / ProjectSnapshot / ProjectView / AnalyzerSnapshot / RuleRegistry IDs
service configuration/component registry/context lease IDs
source/fact/evidence/coverage/conflict/finding/outcome/root/remediation IDs
presentation graph node/edge/root/selection IDs
status/check/failure/cancelled result IDs and digests
CLI canonical stdout digests
member and bundle SHA-256 values
```

Nulls are valid only while `crates/MANIFEST.json` reports `wow-service.implementation_state = not-started` and `apps/wow` has no implementation.

Before the first `wow-service` or `apps/wow` Rust commit, the implementation agent must:

1. freeze and verify every E0-A–E implementation/fixture bundle identity;
2. freeze service configuration/component registry and coherent context lease IDs;
3. freeze accepted generic finding families/counts from E0-C;
4. freeze all E0-E rule outcome/finding/clean/NotEvaluated/root/remediation vectors;
5. freeze raw check sets and presentation graphs;
6. freeze status/clean/findings/partial/failure/cancelled envelopes;
7. freeze CLI JSON stdout and exit-code vectors;
8. canonicalize every example through E0-A canonicalization;
9. write all member and bundle SHA-256 values;
10. update `CONTRACT.json`, `apps/wow/CONTRACT.json`, and manifest implementation states;
11. execute every applicable `TEST_MATRIX.md` and CLI contract test.

Tests verify frozen files and never rewrite them automatically.

## Status boundary

`status-result.json` reports component readiness/degradation/deferred operations. It contains no fields/claims for:

```text
project clean
check passed
tests passed
runtime verified
safe
working
release ready
```

## Check boundary

Every check example contains an exact selected context and one primary semantic outcome:

```text
clean
findings
partial
failure result
cancelled result
```

`check-findings-result.json` and `check-partial-result.json` preserve all raw findings/outcomes even when the presentation graph groups roots/children.

## Folding boundary

Presentation graph:

- references raw/problem record IDs;
- never mutates/deletes them;
- uses structured causal/blocker/duplicate evidence only;
- is acyclic;
- has at most one primary parent per child;
- does not determine semantic status from root count.

## CLI boundary

Canonical JSON examples are service outputs and expected `apps/wow --format json` stdout bytes after freeze. Human text output is noncanonical and may change without changing service result identity.

## Change protocol

Any semantic change must update:

- owning service/status/check/folding/envelope/CLI documents;
- both machine contracts;
- affected examples and tests;
- all prerequisite/context/result/checksum vectors after implementation starts.

Do not weaken generation coherence, raw preservation, `NotEvaluated`, status precedence, or deferred-operation behavior merely to make an implementation pass.
