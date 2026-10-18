# Product completion

Current scope: [ARCHITECTURE.md](ARCHITECTURE.md), accepted 2026-10-18. This page
records useful capability, not E0–E7 certification. Implementation tasks live
in [REFACTOR_QUEUE.md](REFACTOR_QUEUE.md) and their Issues.

## What exists and what must become usable

| Workflow | Existing code | Remaining product work |
| --- | --- | --- |
| Native Ketho annotations | Restricted native source loader, emitter, aliases, selected corrections/inheritance, literals, maps and consumer probes. | #196 simple supported generation; #203 native-only path; close actual semantic gaps against the selected Ketho/Blizzard input (#125). |
| EmmyLua addon analysis | Pinned upstream engine, Main/Library registration, diagnostics, source/overlay analysis. | #193 one analysis; #200 normal root/TOC/library configuration; #201 real addon flow. |
| Blizzard UI search | Source/TOC/XML owners, native graphs and Search/FTS/Context implementations. | #131 current reader-ready index, #169 direct FTS, #106 usable search, #117 relevant XML links, #122 readable context. |
| Updating source | Project update and removal paths exist. | #109 prove useful final results on one changed addon without legacy epochs. |
| Editor/agent access | Selected stdio LSP/MCP, overlay, definition/reference/symbol/context routes exist. | #198/#199 standards fixes; #125 useful Search exposure and upstream Emmy setup. No required reimplementation of all Lua language features. |
| Build/distribution | Cargo workspace and CI exist. | A normal build and real workflow on each advertised platform; ordinary artifact/notices. No custom release engine. |

Current CLI inputs are still too elaborate; current caches still carry legacy
and publication machinery. Documentation changes do not complete those runtime
refactors. Raw metadata preservation and synthetic type probes already exist;
do not recreate them from stale owner status text.

## Completion checks

- Generate a library from the selected source/flavor and verify the relevant
  annotation semantics in a real consumer using existing representative probes.
- Configure a real addon once and obtain actual Emmy/WoW diagnostics with correct
  locations; a deliberate bad call must not pass as clean.
- Index the selected Blizzard checkout; find a known symbol/text and source link,
  reopen/query in a later process, and update/remove a file without stale results.
- State skipped/unsupported source accurately, keep normal limits/cancellation,
  and test only the stdio operations and platforms being advertised.

Use the actual CLI and reused artifacts for these checks. A focused patch does
not need all four before starting; a product claim must have its corresponding
workflow evidence. The current review does not claim a complete supported release.

## Explicitly removed

Old compatibility matrices/readers, graph proof/lineage promotion, recognizer
calibration/holdout/signing, external provider/CBM, Wasm algorithm plugins,
generic storage retention/backup/migration, daemon/HTTP and custom
release/install/update/rollback lifecycles are outside completion scope.
[#125](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/125) is the
completion tracker. Removed planning is in Git history, not a future backlog.
