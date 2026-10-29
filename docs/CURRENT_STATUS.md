# Current status

**Source reviewed:** `8ab892f040c42a9877836be8bd9818b98b1455d8` (2026-10-29).
This page records delivery; the [queue](REFACTOR_QUEUE.md) orders work.

**The ordinary addon checker is not implemented yet.** The existing CLI `check`
requires the legacy project configuration and calls
`LocalProjectBackend -> generic Service`. The target `wow check --root ...`
interface in [FRONTENDS.md](FRONTENDS.md) is planned.

## What the code actually provides

| Area | Implemented | Remaining boundary |
| --- | --- | --- |
| EmmyLua | One Lua 5.1 Main/Library/virtual session, STD, upstream diagnostics and exact byte ranges | Strict diagnostic admission, global-disable handling and semantic configuration equivalence |
| Direct Service checker | Lower exact-snapshot request/result and finite located limitations | Public source-to-result orchestration and requested-scope blockers; lower request/check are still publicly exported |
| Project | Bounded file/Library readers, TOC/XML parsing, package predicates and source maps | Profile-free physical-Lua plan and safe preparation API |
| Annotation generation | Native Reference/Annotations/Ketho pipeline | Callable Environment, return groups, nilable contents and other documented projection gaps |
| CLI/LSP/MCP and source lookup | Transitional platform routes | Shared ordinary check and later direct SQLite lookup |

The direct lower checker currently adds chunk-context incompleteness
unconditionally and always returns `Partial`. Configuration reuse compares raw
serialized settings. Library health errors lose the structured caller-input
classification. These are known work items, not delivered guarantees.

Selected-file errors also need correction: a missing parent directory is fatal,
while any leaf metadata-read error is treated as absence. Global
`diagnostics.enable=false` is incorrectly mapped to native failure. The
[source traces](IMPLEMENTATION_TRACE_REGISTER.md#source-owners-behind-the-open-boundaries)
identify these paths and the output-order/buffering gaps; the code remains open.

## Next change

Implement [Slice 1A](REFACTOR_QUEUE.md#1-first-usable-ordinary-addon-check):
explicit root/TOC/retail/Libraries -> physical source -> one Service operation ->
configured Emmy diagnostics -> ordinary text/JSON output.
[Draft PR #266](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/266)
contains the TOC metadata prerequisite (T23); check its current head before
changing the same owner. Dependencies, XML/chunk injection and special loading
follow in later slices.

## Evidence and maintenance

No ordinary addon, editor or source-search workflow is recorded as executed.
The [trace register](IMPLEMENTATION_TRACE_REGISTER.md) distinguishes existing
code from actual runs. Tests remain deferred under [AGENTS.md](../AGENTS.md).
The [documentation register](AUDIT_REGISTRY.md) tracks this cleanup and links the
complete historical reconciliation; it is not required startup context.

When code changes, update the affected row and trace. Keep detailed behavior in
its owning guide rather than copying it into this status page.
