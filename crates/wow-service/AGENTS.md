# AGENTS.md — `wow-service`

These instructions route all service work packages. Read the target package before changing code or contracts.

## Required routing

1. Read repository and [`../AGENTS.md`](../AGENTS.md) instructions.
2. Read [`README.md`](README.md), [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md), and [`../WORKSTREAMS.md`](../WORKSTREAMS.md).
3. Select exactly one active service work package.
4. Read the current external WoW engineering KB router for patch-sensitive work.
5. Read actual target addon repository instructions for addon-facing operations.

Work-package routes:

```text
E0-F -> E0_F_AGENTS.md + root E0 contract files
E1-D -> e1/AGENTS.md + e1 contract files
E3-C -> e3/AGENTS.md + e3 contract files
```

## Common ownership

`wow-service` coordinates public owner contracts. It may validate identities, acquire exact views, sequence operations, aggregate outcomes, and build service envelopes.

It must not implement parser, analyzer, recognizer, rule, graph, reference, context, search, storage, renderer, or application algorithms.

## Exact-generation rule

- A symbolic selector may be resolved only at the service boundary.
- Record the exact resolved identity immediately.
- Never reread `current` during the operation.
- Never substitute last-known-good, nearby, compatible-looking, or newer generations silently.
- Never merge owner records from different exact bindings.
- Continuation reopens exact retained generations and never resolves current.

## Port discipline

- Use narrow public owner ports and typed requests/results.
- No raw SQL, SQLite handle, actor/session, parser object, filesystem scanner, source reader, or mutable graph/project handle.
- If a needed operation is missing, file a seam request to the owner; do not reproduce it inside service.
- Validate every returned record's owner, universe, profile, generation, schema, capability, and coverage.

## Status and evidence discipline

- Preserve `Partial`, `Truncated`, `NotEvaluated`, conflict, failure, and cancellation.
- Component readiness is not proof an operation passed.
- Empty arrays are not clean/complete without owner coverage records.
- Service ordering/render choice cannot upgrade evidence, confidence, or authority.
- Raw owner artifacts remain unchanged; service envelopes reference or embed them according to exact schemas.

## Lifecycle discipline

- Acquire resources in the package-defined global order.
- Check cancellation before and after each acquisition/owner call.
- Release in reverse order on success, failure, panic boundary, serialization failure, and cancellation.
- No public success envelope before mandatory resource closure succeeds.
- No background work or leaked continuation task.
- Operational lease/timing fields never enter semantic result identity.

## Application boundary

Applications depend on `wow-service` only. Service does not parse CLI flags, choose terminal formatting, write stdout/stderr, map process exit codes, or inspect current directory/environment for semantic configuration.

## Documentation phase

Do not add `Cargo.toml`, `.rs` files, workflows, CI, placeholder modules, or fake successful fixtures. Null implementation pins remain blocking until the target work package's freeze gate is satisfied.

## Completion report

```text
work package and operations
exact owner ports and dependency slice
selector resolution and acquired generation IDs
capability/coverage/conflict state
resource acquisition/release results
service envelope/status/error behavior
budgets/cancellation/continuation behavior
all executed tests and skipped/NotEvaluated gates
known deferred operations
```
