# AGENTS.md — `wow-core`

These instructions apply to `crates/wow-core/**` and refine the repository and `crates/` agent rules.

## Scope

The assigned work package is **E0-A**: pure identity, generation, source-handle, evidence, conflict, coverage-record, capability-summary, evaluation, finding, warning, budget, error, canonicalization, and result-envelope primitives.

No WoW API lookup, Secret rule, parser, graph, database, search, project indexing, transport, editor, network, filesystem, clock, random ID, or runtime behavior belongs here.

## Start with the selected work

Follow the repository [contributor protocol](../../AGENTS.md) and
[execution model](../../docs/EXECUTION_MODEL.md). Read the selected Issue or
[`docs/REFACTOR_QUEUE.md`](../../docs/REFACTOR_QUEUE.md) task's exact reading
list, then the relevant sections of this crate's contract, decisions, canonical
format, and consumer guide. Read the full [`TEST_MATRIX.md`](TEST_MATRIX.md)
or implementation plan when that task changes their scope or reaches the named
acceptance milestone; they are not mandatory pre-reading for every slice.

Consult a pinned dependency's existing API only when the selected change depends
on that API. Do not upgrade dependencies or run a source/release campaign for a
refactor unless its contract requires current external semantics or Tier X.

## Implementation constraints

- Follow the relevant `IMPLEMENTATION_PLAN.md` sequence when the selected task
  depends on it; do not treat unrelated plan stages as blockers.
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
- Preserve the `TEST_MATRIX.md` hash, determinism, and mutation requirements
  when changing those behaviors or completing their acceptance milestone; they
  are not a full-suite gate for unrelated slices.
- Tests added or changed must prove the intended path executed; randomized-order
  failures record the seed, and authority mutations must prove the guard can fail.
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

Report the selected operation/owner change, compatibility or canonical-byte
effect, relevant tests and checks run or skipped, known gaps, and any dependency
or sibling-owner boundary affected. Use the full contract inventory at its
acceptance milestone, not for every bounded slice.
