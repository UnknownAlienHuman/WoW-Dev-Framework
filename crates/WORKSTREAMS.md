# Agent workstreams and integration order

**Status: operational**

This file prevents a swarm from implementing every planned component at once or inventing incompatible seams in parallel.

## Ownership model

- One agent owns one work package and one primary crate.
- A work package may read all contracts but writes only its assigned crate plus explicitly listed fixtures.
- Shared contract changes are proposed before dependent implementation begins.
- A separate integration/review agent validates seams after each wave.
- No agent may silently redesign another crate to make local code easier.
- Documentation-ready does not mean implementation-ready when a prerequisite code boundary is still absent.

## E0 work packages

### E0-A — `wow-core` result primitives

Owns:

- profile and generation identity;
- source handles;
- provenance, confidence, evidence derivation, and explicit conflicts;
- exact coverage records, derived capability summaries, and `NotEvaluated` blockers;
- normalized findings, warnings, budgets, and deterministic result envelopes.

Implementation starts from the detailed contract pack under [`wow-core/`](wow-core/):

```text
AGENTS.md
DECISIONS.md
DATA_MODEL.md
OPERATIONS.md
CANONICALIZATION.md
ERROR_MODEL.md
TEST_MATRIX.md
CONSUMER_GUIDE.md
IMPLEMENTATION_PLAN.md
CONTRACT.json
examples/*.json
```

E0-A contract state is **implementation-ready / code not started**. Its committed examples and hash vectors are normative. A coding agent must not change their semantics merely to simplify serialization or APIs.

Blocks all other E0 implementation packages. Merge implementation first.

E0-A handoff gate:

```text
all required wow-core operations implemented or contract-revised in the same change
all applicable TEST_MATRIX case IDs executable
all example envelopes, including conflict-blocked evaluation, and hash vectors pass byte-exactly
all internal references resolve and evidence derivation remains acyclic
coverage records reconcile exactly with capability summaries
randomized order produces identical canonical bytes
no IO/clock/random/async/domain workflow in wow-core
public API inventory reviewed and frozen for E0 consumers
```

### E0-B — `wow-reference` fixture view

Owns:

- one explicit fixture-only profile;
- one closed project-owned APIDocumentation-shaped catalog;
- restricted declarative evaluation without arbitrary Lua execution;
- raw canonical records and unknown-field preservation;
- exact system/function/restriction lowering;
- duplicate and conflict classification;
- exact per-partition coverage;
- exact symbol and restriction-facet `ReferenceView` lookups;
- authoritative and non-authoritative exact miss behavior;
- no full builder, SQLite, corrections engine, annotations, lineage, or downloader.

Implementation starts from the detailed contract pack under [`wow-reference/`](wow-reference/):

```text
AGENTS.md
DECISIONS.md
DATA_MODEL.md
OPERATIONS.md
LOOKUP_AND_COVERAGE.md
FIXTURE_PROFILE.md
ERROR_MODEL.md
TEST_MATRIX.md
IMPLEMENTATION_PLAN.md
CONTRACT.json
examples/*.json
```

Fixture identity:

```text
fixture-retail-120100-e0-v1
catalog: C_E0Fixture
KnownApi: exact hit
SecretText: synthetic secret.return producer
RemovedApi: exact absent query key
variants: complete | partial | conflict
```

E0-B contract state is **implementation-ready / code not started**.

Important pre-implementation freeze:

- `examples/CHECKSUMS.json` currently records the normative member list with null byte digests because no canonical Rust serializer exists yet;
- before the first `wow-reference` Rust commit, canonical bytes and actual SHA-256 values must be frozen using the implemented E0-A canonicalization contract;
- null digests are invalid after `implementation_state` changes from `not-started`.

May be coded only after the E0-A implementation and consumer handoff are merged. Do not code against draft documentation names alone.

E0-B handoff gate:

```text
fixture profile validates as fixture-only and cannot masquerade as release-grade
input inventory and evaluator policy are deterministic and bounded
arbitrary calls/IO/module loading never execute
raw records and unknown fields round-trip
KnownApi resolves exactly
RemovedApi is authoritative absent only in complete coverage
partial RemovedApi lookup is absent_without_authority with exact blockers
SecretText returns first-class secret.return evidence
complete source ingestion plus facet conflict remains conflict
reference source handles never point to addon/project locations
all non-deferred TEST_MATRIX IDs execute
fixture member and bundle SHA-256 values are frozen and verified
randomized order produces byte-identical canonical model/lookup output
no E1 capability returns fake success
```

### E0-C — `wow-emmy` adapter

Owns:

- pinned upstream dependency;
- analyzer session lifecycle;
- annotation-library loading;
- one generic diagnostic normalization path;
- semantic/source span extraction required by the E0 rules.

May proceed in parallel with E0-B implementation only after the implemented `wow-core` boundary is merged. Its documentation contract may be prepared earlier, but no Rust adapter code may bind to unimplemented/draft core names.

### E0-D — `wow-project` minimal generation

Owns:

- one configured first-party workspace;
- normalized file identity and digest;
- coherent project generation publication;
- no TOC/XML graph beyond what the E0 fixture explicitly needs.

Begins after the analyzer session contract is available.

### E0-E — `wow-rules` vertical rules

Owns only:

- `wow.api.exists` against the fixture reference view;
- one direct local `wow.secret.local_operation` rule;
- required capability declarations;
- clean negative and partial/conflict coverage cases.

Begins after E0-B, E0-C, and the minimal E0-D snapshot contract.

### E0-F — `wow-service` + application integration

Owns:

- `status` and `check` use cases;
- generation coherence checks;
- provider execution and root-cause ordering;
- deterministic serialization handoff;
- minimal `apps/wow` CLI wiring;
- cross-crate golden fixture.

Begins after E0-A through E0-E. Merge last.

## E0 integration gates

Each package must pass its local tests before integration. The integration agent then verifies:

```text
one profile identity across all components
one project generation across all findings
valid fixture API resolves
unknown fixture API produces the WoW finding only under authoritative complete absence
generic analyzer finding is preserved
one direct Secret-local misuse is detected
missing or conflicted capability produces NotEvaluated
a clean fixture stays clean
reference and project evidence remain separate
repeated canonical output is byte-identical
no editor configuration is mutated
```

## Later workstream order

### E1 — Reference Pack

1. `wow-store` schema/migrations/object store
2. full `wow-reference` ingestion/corrections/pack reader-writer
3. `wow-annotations` projection/parity
4. service/app pack build and validation

### E2–E3 — Project and graph

1. `wow-graph` typed storage/query core
2. `wow-recognizers` core structural rules
3. full `wow-project` TOC/XML/incremental indexing
4. `wow-context` skeletons and Project Map
5. service integration and patch fixtures

### E4–E6 — Discovery

1. `wow-search` exact/migration/FTS/graph ranking
2. recognizer calibration packs and external manifests
3. `wow-cbm` optional candidate bridge
4. service merge with strict evidence separation

## Seam request format

When an agent cannot continue without another crate change, report:

```text
requesting crate
owning crate
required operation/data contract
current workaround rejected
why orchestration cannot solve it
proposed minimal seam
cycle/security impact
fixture that will prove the seam
```

Do not implement the seam in the wrong crate while waiting.
