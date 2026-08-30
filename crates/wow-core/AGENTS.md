# AGENTS.md — `wow-core`

These instructions apply to `crates/wow-core/**` and refine the repository and `crates/` agent rules.

## Scope

The assigned work package is **E0-A**: pure identity, generation, source-handle, evidence, conflict, coverage-record, capability-summary, evaluation, finding, warning, budget, error, canonicalization, and result-envelope primitives.

No WoW API lookup, Secret rule, parser, graph, database, search, project indexing, transport, editor, network, filesystem, clock, random ID, or runtime behavior belongs here.

## Required reading order

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
11. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
12. [`CONTRACT.json`](CONTRACT.json)
13. [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md)
14. current WoW Addon Engineering KB [`AGENTS.md`](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/AGENTS.md) and [`INDEX_MINI.md`](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/INDEX_MINI.md)

## Implementation constraints

- Implement in the order defined by `IMPLEMENTATION_PLAN.md`.
- Preserve every normative decision ID in `DECISIONS.md`.
- Do not add a public type/function merely because a later crate might need it.
- Do not expose unchecked constructors for identity-bearing values.
- Do not use `Default` for invalid-empty semantic values.
- Do not use generic string/JSON metadata to bypass a missing contract.
- Do not add implicit conversions that erase identifier family, digest purpose, profile kind, or generation context.
- Do not move another crate's behavior into core to avoid a dependency seam.
- Do not silently canonicalize unsafe input such as whitespace, traversal, controls, or mutable revisions.

## Test constraints

- Preserve `TEST_MATRIX.md` case IDs in executable test names or table data.
- Hash-vector tests merge before downstream crates consume core.
- Randomized-order tests record the failing seed.
- Every regression test proves the intended path executed.
- Mutation tests must demonstrate that profile/context/coverage/authority invariants can fail.
- No network, clock, random OS state, or WoW client is required.

## Contract changes

A semantic change requires, in the same commit:

```text
DECISIONS.md when a decision changes
DATA_MODEL.md / OPERATIONS.md / ERROR_MODEL.md as applicable
CANONICALIZATION.md and hash vectors for identity/serialization changes
TEST_MATRIX.md
CONSUMER_GUIDE.md when a public seam changes
CONTRACT.json
crates/MANIFEST.json when activation/ownership/state changes
an ADR when repository-wide architecture changes
```

## Completion report

Report:

```text
implemented operation IDs
public API inventory
external dependencies and justification
hash vectors and test case IDs executed
fmt/clippy/test results
canonical byte/digest results
known gaps
confirmation that no sibling crate or forbidden responsibility changed
```
