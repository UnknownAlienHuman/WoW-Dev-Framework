# `wow-reference` E0-B implementation plan

**Status:** ordered handoff plan for a future coding agent. No Rust code is added by this contract change.

The implementation must follow this order. Later steps may not invent workarounds for an incomplete earlier boundary.

## Phase 0 — preflight

1. Read all required files in [`AGENTS.md`](AGENTS.md).
2. Confirm `wow-core` E0-A implementation and public seam are merged.
3. Run `wow-core` examples/hash vectors before consuming its API.
4. Confirm the active workspace contains only E0 crates; do not activate deferred crates.
5. Record the exact fixture profile and bundle digests.
6. Confirm there is no pre-existing competing reference implementation.

**Gate:** no code starts while `wow-core` operations/types used by [`CONTRACT.json`](CONTRACT.json) are missing or semantically inconsistent.

## Phase 1 — crate skeleton

Create the smallest Rust crate only when E0-A implementation exists.

Suggested internal responsibility modules, not mandatory filenames:

```text
profile
inventory
raw_value
evaluator
lowering
conflict
coverage
model
view
fixture
error
```

Rules:

- no `store`, `sqlite`, `download`, `annotations`, `lineage`, or `search` module;
- no empty future traits;
- no public module exported merely because it exists;
- no asynchronous runtime unless a proven current operation requires it (E0-B does not);
- dependency set limited to `wow-core` plus minimal serialization/hash/parser support approved by E0 workspace policy.

**Gate:** crate compiles with no placeholder successful operations.

## Phase 2 — fixture/profile/inventory

Implement:

```text
validate_fixture_profile
derive_fixture_reference_generation
require_fixture_profile
inventory_fixture_inputs
validate_reference_input
canonicalize_input_inventory
```

Tests first:

```text
REF-PROFILE-001..007
REF-INPUT-001..011
```

Do not load arbitrary host paths. Tests supply fixture bytes/descriptors explicitly.

**Gate:** profile and inventory canonical bytes/digests are deterministic under shuffled discovery order.

## Phase 3 — raw canonical value model

Implement only value forms used by the closed fixture and required unknown-field round-trip.

Implement:

```text
parse_raw_value
validate_raw_value
canonicalize_raw_value
```

Tests:

```text
REF-RAW-001..007
```

Avoid generic Lua runtime/value abstractions. The raw model is declarative data, not an interpreter object model.

**Gate:** canonical raw-value vectors are byte-stable and bounded.

## Phase 4 — restricted evaluator

Implement the allow-listed fixture evaluator over supplied normalized syntax facts or fixture registration records.

Implement:

```text
restricted_evaluate_fixture
evaluate_registration_record
classify_unsupported_construct
```

Security requirements:

- no execution;
- no IO/network/environment;
- strict budgets;
- source span and partition impact retained;
- one bad isolated record does not erase unrelated records.

Tests:

```text
REF-EVAL-001..010
REF-SEC-001..006
```

**Gate:** malicious/unsupported fixtures cannot cause side effects and produce exact typed gaps.

## Phase 5 — unknown-field preservation

Implement:

```text
preserve_unknown_fields
classify_unknown_field_impact
round_trip_unknown_fields
```

Tests:

```text
REF-UNKNOWN-001..006
```

**Gate:** deleting an unknown raw field or omitting capability impact causes deterministic test failure.

## Phase 6 — typed lowering

Implement:

```text
lower_system_record
lower_function_record
lower_restriction_facets
validate_lowered_fact
```

Only support:

```text
C_E0Fixture
KnownApi
SecretText
secret.return on SecretText return 1
```

Do not create extensibility layers for unimplemented APIDocumentation systems.

Tests:

```text
REF-LOWER-001..009
```

**Gate:** raw/source/evidence links and unknown fields remain intact after lowering.

## Phase 7 — duplicates and conflicts

Implement:

```text
classify_duplicate_registration
merge_equivalent_duplicate_provenance
build_registration_conflict
```

Tests:

```text
REF-DUP-001..007
```

The implementation must process reversed/permuted observations identically.

**Gate:** incompatible duplicates cannot produce a selected winner.

## Phase 8 — coverage and model

Implement:

```text
build_reference_coverage_records
coverage_for_exact_symbol_lookup
coverage_for_restriction_lookup
evaluate_lookup_negative_authority
assemble_fixture_reference_model
validate_reference_model
canonicalize_fixture_model
apply_fixture_variant
```

Tests:

```text
REF-COVER-001..007
REF-MODEL-001..010
REF-AUTH-001..007
```

Use `wow-core` coverage/authority operations. Do not create a local coverage summary substitute.

**Gate:** complete, partial, and conflict variants validate with distinct canonical generations/digests and expected blockers.

## Phase 9 — immutable ReferenceView

Implement:

```text
open_reference_view
reference_view_identity
lookup_symbol_exact
lookup_restriction_facets
resolve_reference_source_handle
```

Tests:

```text
REF-LOOKUP-001..012
REF-FACET-001..007
REF-SOURCE-001..006
```

The exact lookup index may be an ordered map or another deterministic structure. Its internal representation is private.

**Gate:** all outcomes match [`examples/lookup-cases.json`](examples/lookup-cases.json), including coverage and conflict IDs.

## Phase 10 — closed fixture harness

Implement:

```text
validate_lookup_case
execute_lookup_case
canonicalize_lookup_cases
validate_fixture_bundle
verify_fixture_checksums
fixture_bundle_digest
```

Tests:

```text
REF-FIXTURE-001..006
```

Do not regenerate expected values from implementation output during tests. Expected fixture files are independent inputs.

**Gate:** any one-byte semantic fixture change without coordinated contract/checksum update fails.

## Phase 11 — E0 integration seam

Expose only the maximum public surface required by:

- `wow-rules` exact API and restriction lookups;
- `wow-service` fixture view acquisition/status;
- E0 cross-crate integration tests.

Run:

```text
REF-SEAM-001..005
```

Review the public API against [`../wow-core/CONSUMER_GUIDE.md`](../wow-core/CONSUMER_GUIDE.md).

**Gate:** `wow-reference` cannot create findings, project handles, operation envelopes, aliases, candidates, or replacement advice.

## Phase 12 — determinism and mutation review

Run tests repeatedly with:

- shuffled input discovery;
- reversed duplicate observations;
- randomized internal insertion order;
- different test worker counts;
- clean temporary roots;
- deliberate broken mutations from `TEST_MATRIX.md`.

Required result:

```text
canonical profile/inventory/raw/model/lookup bytes identical
all mutations fail for the intended reason
no volatile path/time data in canonical outputs
```

## Phase 13 — completion report

The implementing agent reports:

```text
crate/version/dependencies
wow-core seam consumed
operations implemented
public API inventory
fixture profile/generation/digests
all test IDs and pass/fail/skipped
security tests
canonical byte/digest comparisons
unsupported E1 operations
known NotEvaluated capabilities
gaps requiring a contract change
```

## Forbidden implementation shortcuts

Do not:

- parse source with regex as the canonical E1 design;
- invoke a Lua interpreter for the fixture;
- return `Option<Entity>` as the public lookup contract;
- infer absence from an empty map;
- implement `current_profile()`;
- expose mutable model indexes;
- discard raw records after lowering;
- store unknown fields in debug-only data;
- choose a conflict winner by order;
- reuse project source handles;
- add SQLite because E1 will need it later;
- activate `wow-annotations`, `wow-search`, or `wow-graph`;
- add fake stubs for deferred operations;
- change normative fixtures solely to make implementation tests pass.

## Completion boundary

E0-B implementation ends at a deterministic fixture-backed exact `ReferenceView`.

The next package may consume it only after:

```text
E0-A and E0-B tests pass
public seam is reviewed
fixture checksums are stable
no deferred capability leaks into the API
```
