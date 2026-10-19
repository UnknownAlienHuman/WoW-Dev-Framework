# `wow-service`

`wow-service` is useful for application use cases that combine multiple owners.
The active crate includes a local project/check path plus graph and search
orchestration. Uncalled annotation/reference publication journals are removed.
The target is simple coordination: load the selected source
and configuration, call the existing analysis/search/context owners, and return
typed results or clear errors. A single-owner operation may be called directly
by the application; Service is not a required forwarding layer.

## Where to look

- Public API: [`src/lib.rs`](src/lib.rs)
- Local setup/check: `src/local/{disk_input,native_input,input,backend}.rs`;
  generated Lua already enters as Library files.
- Graph-only Library regeneration: `src/local/native_library_input.rs`; replace
  its `rebuild_for_project` callers with the existing Library and needed origins.
- Status/check result construction: `src/service.rs` (`derive_status`,
  `StatusResult`, and `CheckResult`); these are plain typed values.
- Project publication/update: [`src/live_project/`](src/live_project/)
- Graph and search use cases: `src/graph/`, `src/search.rs`

Local status/check results are ordinary typed values serialized once at the
application boundary. [#194](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194)
continues only through other result families that still self-identify or journal
ordinary reads. Builder pack/materialization, its distribution manifest option
and the second generated-Lua copy are removed; supported generation writes an
ordinary annotation directory. The broader store lifecycle removal remains
[#111](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/111).

Keep parsers, Emmy analysis, Ketho normalization, ranking, and SQL in their
owners. Preserve bounded non-executing source reads, cancellation, source spans,
flavor/config selection, and real failures. Rebuild generated annotations and
indexes from retained source/configuration rather than migrating old derived
formats.

For active scope and order, see root [`AGENTS.md`](../../AGENTS.md),
[implementation](../../docs/IMPLEMENTATION.md), and
[reduction queue](../../docs/REFACTOR_QUEUE.md). Read only the implementation
and actual callers for the selected operation; legacy E0–E7 routes are not
acceptance requirements for the current product.
