# Product operations

Delivery state: [CURRENT_STATUS.md](../../docs/CURRENT_STATUS.md). Owner boundaries:
[IMPLEMENTATION.md](../../docs/IMPLEMENTATION.md#code-ownership).

`addon_check` composes exact already-acquired Main, Library and optional virtual
snapshots through `ConfiguredAnalyzerSession` and returns scope, completeness,
exact diagnostics and scoped limitations. The result contains no snapshot,
analyzer, Project, report, proof, component or presentation receipts.

Slice 1A still needs:

- a public standalone `ProjectLoadPlan -> AddonCheckLimitation` mapper for
  `LuaTypes`; package-plan mapping is not a substitute;
- only requested-scope blockers in `input_limitations`; Project observations such
  as unused Interface/cosmetic metadata remain outside the direct result;
- completeness derived from those blockers, not unconditional chunk context or
  global Project completeness;
- `LibraryHealthFailed` classified as caller invalid input with configured root +
  relative file rather than internal `ComponentUnavailable`; duplicate roots/IDs
  remain invalid input.

Project supplies canonical root-relative paths and exact source locations. CLI may
join its configured root for text rendering; Service does not embed ambient
absolute paths. CLI/LSP/MCP remain thin adapters to the same owners.
