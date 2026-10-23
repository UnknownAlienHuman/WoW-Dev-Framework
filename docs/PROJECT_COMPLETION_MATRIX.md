# Product completion

Reviewed against `3c3d5bc526105259ac80379dd475b5565993f5c0`.
This page records exercised user behavior, not the existence of internal crates,
DTOs or green compilation. [AUDIT.md](AUDIT.md) contains the source diagnosis;
[REFACTOR_QUEUE.md](REFACTOR_QUEUE.md) owns remaining work.

| Workflow | Existing substance | Exercised public result | Status |
| --- | --- | --- | --- |
| Generate annotation Library | Native Reference/Ketho projection, aliases/corrections/literals/maps and public builder exist. | Workflow packages Lua; application probes now invoke the supported executable. Actual Library-reader and addon CLI acceptance remain unverified. | **Not complete** |
| Check addon with Library | Project owns one Main+Library Emmy analysis and parameter mismatches can exist at Main locations. | No normal root/TOC/Library CLI flow; messages/severity are lost; LSP uses isolated one-file analysis. | **Not complete** |
| Persist/search source | Project source/TOC/XML owners, SQLite/FTS code and excerpt reader exist. | No later-process body search over direct persisted text; reads still reconstruct analysis; hit-to-readable excerpt is disconnected. | **Not complete** |
| Update current source | Physical changes and conservative semantic rebuild exist. | Main Service path rejects TOC/package updates; no complete change/delete workflow proves stale diagnostics/hits disappear. | **Not complete** |
| CLI | Large internal operator command surface exists. | Three ordinary workflows still require materialized configs, IDs, profiles, reports and many flags. | **Not complete** |
| LSP/MCP | Stdio transports and many private operations exist. | They do not yet expose the configured three workflows with standard useful results. | **Not complete** |
| Build gate | Workspace Quick compiles and runs strict Clippy. | This is build evidence only. | **Available** |

## Delivered implementation components

- one Project Main+Library Emmy analysis;
- one statement-catalog parse and one final annotation emission;
- native Ketho literal path and public builder entrypoint;
- removal of Wasm, Core E0 envelopes, one-shot registries, Reference Pack
  administration, lineage/comparison and Project catalog/replay tag ladders.

These components are reusable. They do not make a workflow complete until the
public producer, real reader and visible result are connected.

## Completion observations required

1. Builder output loads directly through the real Library reader.
2. CLI valid/invalid typed calls produce clean/real diagnostic outcomes at the
   addon line with upstream message and severity.
3. After restart, a body-only term and symbol are found from the persisted index
   and return readable matching source.
4. Changed/deleted source and TOC/XML selection do not leave stale results.
5. Missing/ambiguous/partial input is explicit.
6. LSP/MCP expose the same useful operations with standard protocol behavior.
7. User source/configuration remains untouched; work/output is bounded and
   cancellation is reported at its actual granularity.

No historical schema matrix, signed pack, lineage proof, release executor or
arbitrary line/performance percentage is part of completion.
