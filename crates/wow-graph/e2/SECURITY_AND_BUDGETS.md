# Graph security and resource budgets

**Status:** normative.

## Trust boundary

Registry bundles are repository-owned reviewed contracts. Producer batches, attributes, evidence/source handles, and query requests are untrusted bounded data even when produced by another framework component.

## Prohibited behavior

- no source/addon/generated code execution;
- no generic callback/plugin/script/SQL/query language;
- no filesystem/network/process/editor access;
- no full source bodies or secrets in default graph records/errors;
- no unbounded traversal/path enumeration/export;
- no candidate assertion masquerading as project/reference truth;
- no host path, token, private URL, pointer, row ID, or process state in stable identity.

## Input budgets

Profiles bound:

```text
registry definitions/fields/depth
identifier/string/attribute length
assertions per batch
relation fanout
source/evidence refs per assertion
conflicts/coverage records
partition bytes
batch validation time/memory
```

Unsupported over-budget inputs fail or explicitly truncate before publication; a truncated batch cannot publish as complete.

## Query budgets

```text
roots
relation kinds/axes
max depth
visited nodes/edges
returned nodes/edges
path count/length/expansions
serialized bytes
wall/CPU budget
continuation size
```

Budgets are validated against system maxima. Public callers cannot request unlimited values.

## Cycle and algorithmic attacks

- cycle-safe visited state bound to snapshot/query;
- no recursive stack proportional to hostile graph depth without bound;
- duplicate edge/assertion amplification limits;
- high-fanout queries truncate deterministically;
- path enumeration uses explicit expansion/path budget;
- malformed cursors rejected before traversal.

## Attribute/data safety

- schema-typed values only;
- normalize Unicode/strings according to profile;
- reject NUL/control/oversized values where disallowed;
- documentation/comment text remains an external source handle or bounded quoted attribute, never schema or instructions;
- redact private data from public errors/logs.

## Store safety

All writes/reads through registered operations and exact roots. No opening arbitrary external SQLite as owned ProjectStore. Store validation and generation identity required.

## Cancellation

Check during batch validation, plan creation, store operations, traversal, and serialization. No background work after cancellation. Partial query output is explicit and never complete.

## Security tests

- huge/fanout/cyclic graph;
- deeply nested/oversized attributes;
- assertion/evidence reference bombs;
- cursor tampering/replay across snapshot;
- cross-universe candidate collision;
- raw SQL/path/source/prompt-injection fields;
- cancellation at every loop/publication phase;
- default output privacy.
