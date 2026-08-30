# Agent workstreams and integration order

**Status: operational**

This file prevents a swarm from implementing every planned component at once or inventing incompatible seams in parallel.

## Ownership model

- One agent owns one work package and one primary crate.
- A work package may read all contracts but writes only its assigned crate plus explicitly listed fixtures.
- Shared contract changes are proposed before dependent implementation begins.
- A separate integration/review agent validates seams after each wave.
- No agent may silently redesign another crate to make local code easier.

## E0 work packages

### E0-A — `wow-core` result primitives

Owns:

- profile and generation identity;
- source handles;
- provenance, confidence, capability, and coverage state;
- normalized findings and deterministic result envelopes.

Blocks all other E0 packages. Merge first.

### E0-B — `wow-reference` fixture view

Owns:

- one explicit fixture profile;
- one APIDocumentation-derived normalized symbol set;
- exact lookup and capability reporting;
- no full builder, SQLite, corrections engine, or profile downloader.

May proceed after the `wow-core` contract is frozen.

### E0-C — `wow-emmy` adapter

Owns:

- pinned upstream dependency;
- analyzer session lifecycle;
- annotation-library loading;
- one generic diagnostic normalization path;
- semantic/source span extraction required by the E0 rule.

May proceed in parallel with E0-B after `wow-core`.

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
- clean negative and partial-coverage cases.

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

## E0 merge gates

Each package must pass its local tests before integration. The integration agent then verifies:

```text
one profile identity across all components
one project generation across all findings
valid fixture API resolves
unknown fixture API produces the WoW finding
generic analyzer finding is preserved
one direct Secret-local misuse is detected
missing capability produces NotEvaluated
a clean fixture stays clean
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
