# `wow-graph` implementation status

## Implemented E2-A boundary

- Exact graph universe and generation identities.
- Domain-separated content-addressed node, edge, and snapshot identities.
- Immutable canonical node, edge, relation-coverage, and snapshot records.
- Independent relation families for calls, state, native events, custom signals, CVar callbacks, scripts/hooks, load, ownership, inheritance, mixins, factories, dependencies, and API use.
- Explicit `Proven`, `Derived`, `Possible`, and `Candidate` confidence without automatic promotion.
- Strict duplicate, endpoint, self-edge, generation, universe, evidence, canonical-order, and snapshot-digest validation.
- Bounded direct incoming/outgoing/both neighbor queries.
- Separate `Complete`, `Partial`, `NotEvaluated`, and `Truncated` query states.
- Authoritative empty result only when every requested relation has complete negative-authority coverage and the result is not truncated.
- Immutable snapshot storage and exact current publication through `wow-store` catalog compare-and-swap.
- Exact read-back validation, universe-bound publication keys, retention leases, bounded GC, integrity validation, and logical manifests.

## Explicit nonclaims

- Direct edges are not transitive paths.
- Static edges do not prove runtime delivery, execution, readiness, performance, taint, combat, protected, or Secret Value behavior.
- Same names or owner keys across generations or universes do not merge identities.
- Missing coverage, partial work, failed work, conflicts, and truncation never prove absence.
- E2-A does not implement recognizers, project indexing, lineage, migration, impact, search, context, service orchestration, LSP, MCP, or source edits.

## Next package

E2-B may emit typed recognizer assertions and evidence. E2-C may bind exact project generations to complete graph publication inputs. Neither package may bypass graph validation or manufacture direct relations from reason paths.
