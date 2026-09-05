# `wow-core` E0-A implementation plan

**Status:** implementation-ready plan; no Rust code or Cargo workspace yet.

This plan is the handoff for the first coding agent. It turns the contracts into the smallest coherent E0-A slice without activating unrelated crates.

## 1. Work package boundary

Primary owner:

```text
E0-A / wow-core
```

Allowed writes during the implementation package:

```text
root Cargo workspace files required to activate E0-A
crates/wow-core/**
crate-local or explicitly assigned cross-crate fixtures under tests/**
crates/MANIFEST.json status fields
crates/WORKSTREAMS.md only when the package state changes
```

Do not edit another production crate's API to make `wow-core` easier. Submit a seam request when another owner is required.

## 2. Required reading before code

1. [`../../AGENTS.md`](../../AGENTS.md)
2. [`../AGENTS.md`](../AGENTS.md)
3. [`README.md`](README.md)
4. [`DECISIONS.md`](DECISIONS.md)
5. [`DATA_MODEL.md`](DATA_MODEL.md)
6. [`OPERATIONS.md`](OPERATIONS.md)
7. [`CANONICALIZATION.md`](CANONICALIZATION.md)
8. [`ERROR_MODEL.md`](ERROR_MODEL.md)
9. [`TEST_MATRIX.md`](TEST_MATRIX.md)
10. [`CONSUMER_GUIDE.md`](CONSUMER_GUIDE.md)
11. [`CONTRACT.json`](CONTRACT.json)
12. [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md)
13. [`../WORKSTREAMS.md`](../WORKSTREAMS.md)
14. [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
15. [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)
16. Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.

`wow-core` must not copy patch-specific KB content into code. The KB read is required to preserve authority/evidence terminology and avoid contradicting repository policy.

## 3. Decisions to freeze before the first code commit

The coding agent must restate and accept these decisions in its implementation notes:

- canonical ID grammars and reserved segments;
- SHA-256-only E0 digest policy;
- UTF-8 repository-relative path contract;
- zero-based end-exclusive byte spans;
- fixture/release profile split;
- strict/default generation merge behavior;
- source location versus evidence-authority separation;
- provenance/confidence/claim-scope separation and acyclic derivation;
- conflict as an orthogonal record over evidence;
- exact coverage-record versus derived capability-summary separation;
- coverage precedence and explicit NotEvaluated records;
- negative-authority denial reasons;
- finding fingerprint versus context-bound finding ID;
- canonical JSON profile and hash domains;
- strict unknown-field policy;
- no clock/random/IO/async/domain algorithms.

If one of these is unimplementable or contradictory, stop before creating dependent public APIs and propose the smallest contract correction.

## 4. Suggested internal module topology

Names are recommendations, not mandatory public API:

```text
ids
    typed identifiers, parsing, percent encoding

digest
    SHA-256 value parsing, purpose/domain separation, typed IDs

profile
    ProfileIdentity validation/comparison

generation
    GenerationContext validation/merge/context ID

source
    path normalization, spans, SourceHandle and handle ID

evidence
    provenance/confidence/claim scope, EvidenceRecord validation/ID, derivation DAG

conflict
    ConflictRecord validation/ID and affected capability/partition scope

coverage
    CoverageRecord/ID, conservative combination, CapabilitySummary,
    NotEvaluatedRecord, NegativeAuthorityDecision

finding
    message args, root-cause key, fingerprint, context binding, ordering

warning
    operation warning validation/ID

budget
    limits, checked usage, truncation state

error
    stable CoreError catalog and safe structured context

canonical
    canonical projection, ordering, JSON bytes, result digest

envelope
    E0 check-envelope validation and finalization
```

Do not expose every internal helper publicly. Public exports should correspond to cross-crate operations in `OPERATIONS.md`.

## 5. Implementation sequence

### Step A — crate activation and test harness

- Create the smallest workspace and `wow-core` package only.
- Set an explicit Rust edition and MSRV policy in the root/package manifest.
- Add crate-level denial/allow policy intentionally; do not copy a giant lint list without testing it.
- Load `examples/HASH_VECTORS.json` and verify the test harness can detect an intentionally changed digest.

Gate:

```text
workspace compiles
hash-vector test demonstrably fails under mutation
no sibling production crate activated
```

### Step B — identifiers and digest primitives

Implement:

```text
ProfileId
RuleId
ProducerId
CapabilityId
OperationId
EntityKey
CoveragePartitionId
ContentDigest
ConflictId / CoverageId / WarningId
family-tagged derived IDs
```

Do not implement profiles, handles, or findings until family parsing and domain-separated hashing pass.

Gate: identifier/digest cases in `TEST_MATRIX.md`.

### Step C — profile, path, span, and source handle

Implement structural `ProfileIdentity`, path normalization, span validation, source handle construction/verification/comparison.

Gate:

```text
fixture/release split proven
path traversal/absolute/non-UTF8 behavior proven
presentation-only line/column data excluded from the canonical span and handle identity
handle hash vector passes
```

### Step D — generation context

Implement validation, context ID, explicit merge modes, and strict same-generation guard.

Gate: full merge matrix and randomized ordering.

This gate freezes the boundary needed by every later E0 crate.

### Step E — evidence and coverage

Implement evidence validation/identity and an acyclic derivation graph; separate conflict records; exact coverage records/IDs; conservative capability summaries; availability/`NotEvaluated`; and negative-authority decisions.

Gate:

```text
candidate-only authority violations rejected
Derived requires inputs and the derivation graph is acyclic
conflict records resolve exact evidence and affected scopes
coverage records and capability summaries reconcile
coverage truth table passes
conflict denies negative authority
NotEvaluated contains exact coverage/conflict blockers
```

### Step F — finding, budget, and envelope

Implement structured message arguments, finding fingerprint/context binding, warning identity, deterministic ordering, budget/truncation, full reference-graph validation, envelope validation, canonical serialization/digest/finalization.

Gate:

```text
all committed E0 examples validate
exact canonical bytes match golden files
randomized input order is byte-identical
status/truncation/conflict/NotEvaluated/reference-resolution invariants pass
```

### Step G — API minimization review

Before declaring E0-A complete:

- list every public item and owning consumer;
- make internal any item without a cross-crate consumer in E0;
- remove speculative plugin/registry/config abstractions;
- confirm no `serde_json::Value`/unbounded metadata escape hatch exists;
- confirm no public API accepts raw `String` where a validated type is required;
- confirm errors remain structured and safe.

## 6. External dependency qualification

No dependency is mandated by this documentation. Candidate responsibilities may justify small crates such as:

```text
serialization derive/JSON encoding
SHA-256
structured error derivation
Semantic Version parsing
property-based/randomized test generation (dev-only)
```

Before adding a dependency, record:

- exact responsibility and why `std` is insufficient;
- license and current maintenance state;
- enabled features and default-feature decision;
- MSRV impact;
- transitive dependency/security impact;
- whether it enters the public API;
- removal/rollback path.

Avoid in E0 unless a demonstrated requirement exists:

```text
async runtime
UUID/random ID generator
clock/date-time library
URL/network client
filesystem/path canonicalization library that touches host state
database crate
logging/tracing framework in the public contract
arbitrary JSON/value framework exposed publicly
global interning/singleton registry
```

`anyhow`-style erased errors may be used only at an application boundary later; they are not the `wow-core` public error contract.

## 7. Public API design rules

- Constructors validate; invalid values cannot be created through the normal public API.
- Parsing and construction are distinct where raw text versus structured fields matter.
- Prefer private fields with validated constructors over public mutable records.
- Preserve enough accessors for serialization and downstream equality without exposing mutation.
- Implement total ordering only where the contract defines semantic/canonical order.
- Hash-map iteration order is never public order.
- Do not implement implicit conversions that erase ID family, digest purpose, fixture/release kind, or coverage status.
- Do not implement `Default` for identity/evidence/context values when an empty/default value is semantically invalid.
- Do not use panics for external invalid input; reserve panics for proven internal impossibilities and test them as such.

## 8. Serialization boundary

E0 needs deterministic internal JSON/golden output, not a permanently frozen public schema.

- Keep semantic types independent of `serde_json::Value`.
- Use explicit versioned DTO/projection types when needed.
- Reject unknown fields in E0 DTOs.
- Ensure deserialization runs the same invariants as constructors.
- Do not derive serialization blindly for private/internal fields that should be excluded from canonical output.
- `canonical_digest` must be verified after deserialization.

## 9. Review checklist

### Responsibility

- [ ] No parser, search, graph, rule, Reference Pack, project, or transport workflow leaked into core.
- [ ] No IO/clock/random/global state.
- [ ] No domain-specific current WoW baseline embedded.

### Identity

- [ ] No floating profile/revision alias.
- [ ] Full digests only.
- [ ] Fixture cannot masquerade as release.
- [ ] Context/generation mismatch fails.

### Evidence

- [ ] Provenance/confidence/claim scope separate.
- [ ] Candidate-only sources cannot claim stronger authority.
- [ ] Derived evidence has explicit inputs.
- [ ] Conflicts preserved.
- [ ] Project finding locations and platform/reference evidence remain separate records/handles.

### Coverage

- [ ] Complete/Partial/Unknown/Failed/N/A truth table passes.
- [ ] NotEvaluated is typed and never a pass.
- [ ] Negative authority returns reasons.

### Determinism

- [ ] Hash vectors pass.
- [ ] Randomized-order output byte-identical.
- [ ] No volatile fields.
- [ ] Path/span rules cross-platform deterministic.

### Safety

- [ ] Errors do not leak host paths/source/credentials.
- [ ] Checked arithmetic for budgets/offsets.
- [ ] Strict unknown-field handling.

## 10. Handoff contract to E0-B/E0-C

The normative [`CONSUMER_GUIDE.md`](CONSUMER_GUIDE.md) defines the maximum E0 seam. After E0-A implementation merges, update it to the exact public names that survived API minimization. The principal consumers are:

### `wow-reference`

```text
ProfileIdentity
ReferenceGenerationId
ContentDigest
EntityKey
CapabilityId
CoveragePartitionId
CoverageRecord
EvidenceRecord
ConflictRecord
SourceHandle
NegativeAuthorityDecision
```

### `wow-emmy`

```text
ProjectGenerationId/context binding
SourceHandle/span
Producer/Rule/Finding IDs
Evidence/Coverage records
CapabilitySummary / NotEvaluatedRecord
structured finding/message arguments
```

No downstream crate begins implementation against unmerged draft names. If public names differ from this documentation, update all references and `CONTRACT.json` in the same commit.

## 11. Definition of done

E0-A is implementation-complete only when:

- the package exists and contains real implementation, not placeholders;
- every E0 operation in `CONTRACT.json` is implemented or explicitly removed by contract update;
- all `TEST_MATRIX.md` E0 cases relevant to active operations are executable and pass;
- all JSON examples and hash vectors pass exactly;
- public API review finds no speculative surface;
- fresh `fmt`, `clippy`, tests, and any dependency-policy checks are reported;
- `crates/MANIFEST.json` changes `wow-core` from contract-ready to implementation-complete only after those gates;
- E0-B and E0-C can consume the stable public boundary without unstructured identity strings.
