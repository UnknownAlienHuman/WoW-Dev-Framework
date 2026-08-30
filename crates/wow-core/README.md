# `wow-core` implementation contract

**Status:** E0-A implementation-ready contract pack; no Rust code or `Cargo.toml` yet.

`wow-core` owns the smallest transport-, storage-, parser-, and product-independent contracts required to describe an exact WoW analysis result. Every production crate may eventually depend on it, so its API must remain narrow, deterministic, typed, and free of domain workflows.

## Start here

A coding agent must read this package in order:

1. [`AGENTS.md`](AGENTS.md) — local scope and change rules.
2. [`DECISIONS.md`](DECISIONS.md) — frozen E0-A design decisions.
3. [`DATA_MODEL.md`](DATA_MODEL.md) — fields, grammars, states, and invariants.
4. [`OPERATIONS.md`](OPERATIONS.md) — function-level input/output/error contracts.
5. [`CANONICALIZATION.md`](CANONICALIZATION.md) — ordering, JSON bytes, hashing, and identity projections.
6. [`ERROR_MODEL.md`](ERROR_MODEL.md) — stable error taxonomy and safety rules.
7. [`TEST_MATRIX.md`](TEST_MATRIX.md) — mandatory executable cases and mutation gates.
8. [`CONSUMER_GUIDE.md`](CONSUMER_GUIDE.md) — minimal downstream E0 seams and anti-leak rules.
9. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md) — module/order/dependency/review handoff.
10. [`CONTRACT.json`](CONTRACT.json) — machine-readable type/operation routing.
11. [`examples/`](examples/) — strict E0 envelopes and normative hash vectors.

Parent contracts still apply:

- [`../AGENTS.md`](../AGENTS.md)
- [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md)
- [`../WORKSTREAMS.md`](../WORKSTREAMS.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)

Before implementation, also read the current [`AGENTS.md`](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/AGENTS.md) and [`INDEX_MINI.md`](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/INDEX_MINI.md) in the WoW Addon Engineering Knowledge Base. Patch-sensitive facts remain there; they are not copied into this crate.

## Mission

`wow-core` defines validated values and pure operations for:

- exact profile identity;
- immutable reference/project/external generation identity;
- stable identifiers and content digests;
- source handles and canonical byte spans;
- evidence provenance, confidence, claim scope, and conflicts;
- coverage, capability availability, `NotEvaluated`, and negative authority;
- normalized findings, root-cause keys, and remediation classification;
- explicit budgets and truncation;
- deterministic ordering, hashing, and E0 result envelopes;
- structured safe boundary errors.

## Owned responsibilities

- family-specific canonical identifier grammars;
- fixture/release profile-kind separation;
- generation-context validation and explicit merge modes;
- repository-relative UTF-8 path normalization without filesystem access;
- end-exclusive UTF-8 byte-range semantics;
- SHA-256 domain-separated identity derivation for E0;
- evidence/coverage/finding/envelope invariants;
- conservative coverage aggregation;
- typed negative-authority decisions;
- byte-identical canonical JSON result generation;
- common bounded count/size contracts at crate boundaries.

## Explicit non-responsibilities

`wow-core` does not:

- parse Lua, XML, TOC, JSON source documents, or SQLite databases as domain inputs;
- access filesystem, network, environment, editor, process, clock, randomness, or WoW client;
- choose a current/latest profile;
- validate that a Blizzard Interface/build relationship is historically correct;
- know API names, addon frameworks, Secret algorithms, rule algorithms, graph semantics, search ranking, or database schemas;
- perform persistence, logging, telemetry, retries, transport serialization policy, or configuration discovery;
- resolve a source handle to host content;
- execute rules, searches, graph queries, project indexing, or Reference Pack ingestion;
- expose unbounded generic metadata/JSON extension bags.

## E0-A public value set

The implementation may refine concrete Rust names, but the semantic set is fixed by [`CONTRACT.json`](CONTRACT.json):

```text
all identifier families listed in `DATA_MODEL.md` and `CONTRACT.json`
ProfileIdentity / GenerationContext / ExternalGeneration
SourceSpan / SourceHandle
EvidenceRecord / CoverageReference / ConflictRecord
CoverageRecord / CapabilitySummary / NotEvaluatedRecord
NegativeAuthorityDecision
Finding / MessageArgument / Remediation / WarningRecord
BudgetLimits / BudgetUsage / TruncationState
CoreError
E0CheckResultEnvelope / E0OperationErrorEnvelope
```

Invalid-empty semantic values must not be constructible through the normal public API.

## E0-A operation groups

Function-level contracts are in [`OPERATIONS.md`](OPERATIONS.md). E0-A must cover:

```text
identifier and digest parsing
typed/domain-separated ID derivation
profile structural validation/comparison
path/span normalization
source-handle construction/verification/comparison
generation-context ID/validation/merge/strict guard
evidence validation/ID/acyclic derivation
conflict construction/ID/validation
coverage-record validation/ID/conservative combination
capability-summary validation, availability, and NotEvaluated construction
negative-authority evaluation
message/root-cause/finding fingerprint/context binding
finding ordering/deduplication
warning construction/ID/validation
budget/usage/truncation
result-envelope validation/order/digest/finalization
schema-version validation
```

No placeholder success result is permitted. An unavailable capability is absent, typed unavailable, or represented by `NotEvaluated` according to the owning operation.

## Core invariants

1. No floating `current`, `latest`, `live`, `head`, `default`, or implicit identity.
2. Profile labels never substitute for structured profile identity.
3. Fixture identities cannot masquerade as release profiles.
4. No mixed reference/project generations in one result.
5. External generations remain explicitly separate.
6. No absolute/escaping/non-UTF-8 E0 public source path.
7. Canonical source spans are zero-based end-exclusive UTF-8 byte ranges.
8. No confidence upgrade through merge, popularity, similarity, or model inference.
9. `Derived` evidence has explicit inputs and producer identity.
10. No authoritative negative without exact scope, complete coverage, matching context, and no affecting conflict/truncation.
11. `NotEvaluated` is not success and carries exact blockers.
12. Severity and rollout policy are separate.
13. Message prose does not define identity/dedup/root cause.
14. No volatile fields in canonical E0 results.
15. Public ordering is total and deterministic.
16. Serialization round-trip preserves semantic identity.
17. Unknown internal E0 fields are rejected, not silently dropped.
18. Budgets/truncation are explicit truth, never silent clipping.
19. The source-handle → evidence → conflict → coverage → evaluation/finding reference graph is fully resolved and acyclic.
20. Exact coverage records remain present; a capability summary never replaces or rewrites them.
21. Source location and evidence authority are separate: a project finding span cannot stand in for Reference Pack proof.

## Canonical examples

| File | Purpose |
|---|---|
| [`examples/e0-clean-result.json`](examples/e0-clean-result.json) | Complete evaluated check with no findings. |
| [`examples/e0-findings-result.json`](examples/e0-findings-result.json) | Complete evaluated check with generic, API, and Secret-local fixture findings. |
| [`examples/e0-not-evaluated-result.json`](examples/e0-not-evaluated-result.json) | Partial result with an exact missing capability and blocking partition. |
| [`examples/e0-conflict-not-evaluated-result.json`](examples/e0-conflict-not-evaluated-result.json) | Complete source coverage that remains non-authoritative because an unresolved conflict blocks evaluation. |
| [`examples/e0-generation-mismatch-error.json`](examples/e0-generation-mismatch-error.json) | Structured boundary error, separate from findings. |
| [`examples/HASH_VECTORS.json`](examples/HASH_VECTORS.json) | Normative canonical JSON and SHA-256 vectors. |

The profiles and source data in these files are synthetic fixtures. They do not claim a released Reference Pack or live-client verification.

## E0-A error taxonomy

The stable catalog is in [`ERROR_MODEL.md`](ERROR_MODEL.md). Broad families:

```text
identifier/digest validation
profile/source-handle validation
generation/context mismatch
evidence authority/reference errors
coverage/negative-authority errors
finding/envelope invariant errors
budget/schema/canonicalization errors
```

Errors, findings, and `NotEvaluated` remain separate output channels.

## E0-A tests

The executable implementation must preserve the case IDs in [`TEST_MATRIX.md`](TEST_MATRIX.md). Mandatory classes include:

- valid/invalid/noncanonical identifier cases;
- fixture/release profile separation;
- traversal/absolute/non-UTF-8 path rejection;
- byte-span state and exclusion of presentation-only line/column fields;
- complete context merge matrix;
- candidate-evidence authority rejection and derivation-cycle rejection;
- conflict-record resolution and scope validation;
- coverage-record/summary reconciliation and aggregation truth table;
- negative-authority denial reason matrix;
- finding fingerprint versus context-bound ID and warning identity;
- explicit truncation and budget overflow;
- strict schema/unknown-field behavior;
- randomized input-order byte determinism;
- committed hash vectors and exact JSON golden files;
- mutations proving the target invariant can fail.

## E0-A implementation boundary

Activate only `wow-core` first. Do not activate every planned crate in Cargo.

After core passes its contract, it hands a stable narrow boundary to:

- `wow-reference` E0-B fixture view;
- `wow-emmy` E0-C adapter;
- later E0 project/rules/service packages.

No downstream implementation should start against draft/unmerged core names.

## Definition of done

`wow-core` E0-A is implementation-complete when:

- all required operations in `CONTRACT.json` are implemented or removed through a same-change contract revision;
- every applicable `TEST_MATRIX.md` case is executable and green;
- all example envelopes and hash vectors pass byte-exactly;
- randomized-order runs produce identical canonical bytes;
- public API review finds no speculative surface or responsibility leak;
- fresh format/lint/test/dependency checks are reported;
- `wow-reference` and `wow-emmy` can consume the boundary without raw identity strings;
- `crates/MANIFEST.json` is updated from contract-ready to implementation-complete only after those gates.
