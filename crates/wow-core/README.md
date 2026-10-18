# `wow-core`

`wow-core` contains small deterministic value types shared by the framework.
It has no filesystem, database, parser, graph, search, editor, network or process behavior.

## Owns

- validated identifiers, digests, profile and generation values;
- source handles and byte spans;
- evidence, findings, coverage and budget records;
- canonical JSON and typed digest helpers used by real callers;
- typed errors for those values.

## Does not own

- CLI or transport result envelopes;
- operation replay, schema registries or compatibility readers;
- publication proofs, retention histories or lifecycle orchestration;
- WoW semantics, parsing, indexing, rendering or storage.

A caller returns its own typed result. Core validates only the values it owns.
When a shared type no longer has a real caller, delete it with its exclusive tests and documentation.

Implementation is the source of truth. Read the selected module and its direct tests; do not reconstruct the retired E0 contract platform.
