# `wow-reference` E1-B normative examples

These files define the closed source/profile, evaluator, raw/normalization, correction, persistent build, exact ReferenceView, and checksum-freeze contract for the future implementation.

## Files

- [`source-snapshot.json`](source-snapshot.json) — exact provider/content/profile/partition/file manifest shape.
- [`apidoc-evaluator-cases.json`](apidoc-evaluator-cases.json) — supported declarations and no-execution/unsupported/budget cases.
- [`normalization-cases.json`](normalization-cases.json) — raw state distinctions, normalized facts, unknowns, duplicates, conflicts, restrictions, transitions.
- [`correction-cases.json`](correction-cases.json) — exact apply/expire/reject/conflict/NotApplicable and raw preservation.
- [`reference-build-plan.json`](reference-build-plan.json) — deterministic schema/operation/object/validation/store publication and manifest plan.
- [`reference-view-cases.json`](reference-view-cases.json) — Found, authoritative absence, partial, conflict, NotEvaluated, invalid, bounded raw/list cases.
- [`CHECKSUMS.json`](CHECKSUMS.json) — complete source/parser/evaluator/fact/correction/coverage/schema/store/view checksum freeze gate.

## Current state

No E1-B Rust implementation exists. Exact source snapshot/profile/parser/environment/field registry/correction/schema/store/fact/query identities and byte digests remain null.

Nulls are valid only while the E1-B implementation state is `not-started`. Before the first E1-B Rust commit, the implementation agent must:

1. freeze `wow-core` and `wow-store` prerequisite implementations/fixtures;
2. freeze exact materialized source snapshot/file/partition/profile manifests;
3. pin/probe the exact parser and evaluator environment;
4. freeze canonical number/string/table/binding/registration semantics and budgets;
5. freeze raw/unknown/unsupported/normalized/conflict/restriction/predicate/deprecation facts;
6. freeze correction set and application outcomes;
7. freeze capability/coverage/negative-authority vectors;
8. freeze persistent schema/write/read/validation/migration/build plan;
9. freeze ReferenceStore generation/manifest/open validation from `wow-store`;
10. freeze exact ReferenceView request/result variants;
11. freeze ReferenceData build report/manifest/eligibility;
12. canonicalize every JSON example and write all member/bundle SHA-256 values;
13. update `CONTRACT.json` and implementation status;
14. run all applicable `TEST_MATRIX.md` cases.

Tests verify frozen fixtures and never rewrite them automatically.

## Source and runtime boundary

The examples contain declarative source snippets/AST-shaped cases as data. They are never executed. The fixture does not claim current live status beyond its exact pinned profile and never includes a permanent runtime spell whitelist.

## Storage boundary

The build plan uses registered static schema/operation/validation IDs. It contains no SQL. `wow-store` owns SQLite transactions, publication, objects, and read-only opening.

## Authority boundary

An empty exact lookup becomes `AbsentAuthoritative` only through the explicit coverage/conflict/truncation/runtime decision. Other misses remain `NotFoundPartial`, `Conflict`, or `NotEvaluated`.

## Change protocol

Any semantic change must update:

- the owning E1 document;
- `CONTRACT.json`;
- affected examples;
- `TEST_MATRIX.md`;
- all IDs/digests/checksums after implementation starts.

Do not weaken no-execution, unknown preservation, correction digest binding, profile isolation, store immutability, or negative-authority requirements to make implementation easier.
