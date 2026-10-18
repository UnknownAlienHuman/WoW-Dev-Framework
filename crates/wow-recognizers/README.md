# `wow-recognizers`

`wow-recognizers` maps normalized Emmy, TOC, and XML facts to useful structural
relationships. It does not parse source, branch on addon identity, execute Lua,
run diagnostics, or publish project generations. Source owners retain original
files and spans; the graph owner validates and stores proposals.

## Executable routes

The crate currently contains a bounded registry/matcher and source adapters for
captured calls, constructions, mixins, scripts, signals, state, TOC, and XML.
Start with the selected adapter in `src/source_*.rs`, then follow its caller in
Project/Service and the graph proposal handoff. For common CLI behavior, use
[`wow graph build`](../../apps/wow/GRAPH_BUILD.md), which runs the existing
source-to-recognizer-to-graph route. Do not add a second parser or independently
reconstruct source facts here.

The active slices include direct-call, XML script-assignment, SavedVariables
access, and selected `CreateFrame` construction recognition. These are static
observations with source bindings; dynamic dispatch, effective mixin behavior,
frame existence, lifecycle, protection, and runtime execution remain unknown.
Confidence and coverage must not be strengthened by the matcher.

## Pending reduction

The crate still carries broader registry, pack, partition, and evaluation
machinery inherited from the previous platform plan. Its reduction is pending in
the refactoring queue; these layers are not already removed, and their historical
corpus/checksum gates are not prerequisites. Preserve only matcher/adapters and
relations that a current search, navigation, or diagnostic path consumes.
