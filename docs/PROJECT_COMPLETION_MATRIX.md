# Product completion

Current scope: [ARCHITECTURE.md](ARCHITECTURE.md), accepted 2026-10-18. This page
records useful capability, not E0–E7 certification. Status reviewed against
`db1a62c9` on 2026-10-21; this is not full product acceptance. Implementation tasks live
in [REFACTOR_QUEUE.md](REFACTOR_QUEUE.md) and their Issues; code/donor routes are
in [IMPLEMENTATION.md](IMPLEMENTATION.md).

## What exists and what must become usable

| Workflow | Existing code | Remaining product work |
| --- | --- | --- |
| Native Ketho annotations | Supported `wow-reference-builder generate`, native literals, one final emission, aliases, selected corrections/inheritance and maps (#196/#203/#204 delivered). | Verify selected Ketho/Blizzard semantics and the generator-to-addon consumer path (#125/#201); reuse produced Library/origins instead of graph requalification (#107). |
| EmmyLua addon analysis | One Main + Library analysis supplies diagnostics, facts and flow; `param-type-mismatch` reaches Main locations (#193 delivered). | #200 normal root/TOC/library configuration; #201 real addon flow. The complete user workflow still needs its own verification. |
| Blizzard UI search | Source/TOC/XML owners and native graphs; FTS over file labels/recorded names; separate source-excerpt reader. | #131 reader-ready index, #169 source-content/FTS tables, #106 real body/symbol search, #117 XML links, #122 hit-to-excerpt handoff. |
| Updating source | Physical updates retain syntax trees in the shared engine but reindex remaining files; ordinary live update still rejects TOC/package load plans. | #109 connect reload/rebuild and verify diagnostics/search against the final source. |
| Editor/agent access | Selected stdio LSP/MCP, overlay, definition/reference/symbol/context routes exist. | #198/#199 standards fixes; #125 useful Search exposure and upstream Emmy setup. No required reimplementation of all Lua language features. |
| Build/distribution | Cargo workspace and CI exist. | A normal build and real workflow on each advertised platform; ordinary artifact/notices. No custom release engine. |

Project replay now has one strict DTO and one store record (#231/#232); its
numbered/feature catalog matrix is removed. Ordinary retained reads still rebuild
Emmy/Project and recapture replay (#131). CLI configuration, publication/lifecycle
machinery and other domains' formats remain reduction work, not completed scope.
Raw metadata preservation and synthetic type probes already exist.

## Verification boundary

Quick for #232 passed policy, formatting, workspace compilation and strict Clippy.
It did not execute the advertised end-to-end workflows or establish release
readiness. Historical test counts are not validation of the current revision.

## Completion checks

- Generate a library from the selected source/flavor and verify the relevant
  annotation semantics in a real consumer using existing representative probes.
- Configure a real addon once and obtain actual Emmy/WoW diagnostics with correct
  locations. A call violating a Library annotation must produce the relevant
  upstream diagnostic; a valid counterpart stays clean. Generator-only external
  consumer probes do not establish this production integration.
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
