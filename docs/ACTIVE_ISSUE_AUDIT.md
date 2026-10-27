# Active Issue instruction audit

**Audit base:** `main` after PR #268
(`28302632599bed087394dee83751815d13041579`)
**Scope:** all 21 open implementation Issues
**Source traces:** T01–T35

Issue bodies are task instructions, secondary to current status, the queue,
implementation guide and trace register. They must not copy delivery history,
widen the active slice or preserve a removed result shape.

## Register

| Issue | Status | Durable boundary |
| --- | --- | --- |
| #103 recognizer rules | confirmed | keep only typed facts with a named diagnostic/search/navigation consumer |
| #106 Blizzard UI workflow | confirmed | later direct body/symbol/search/context workflow |
| #107 graph duplication | corrected | use loaded/generated Library and direct origins; closed #196 is historical delivery, not an active dependency |
| #109 coherent updates | corrected | update the exact target/plan/snapshot owner; no second session or profile identity |
| #111 Store lifecycle | corrected | caller list is a source snapshot, not a live “after PR #250” checkpoint; recheck before deletion |
| #112 formats/documentation | corrected | small target/plan/result formats; no profile-digest or plan-version ladder in the direct route |
| #113 performance | confirmed | measure one real workflow only after it exists |
| #117 TOC/XML relationships | reconciled | Slice 1B+ context/index work; XML must not block Slice 1A physical diagnostics |
| #118 lineage/fingerprints | confirmed | remove remaining fingerprint/proof plumbing with real callers |
| #122 readable context | confirmed | direct located result over exact indexed text |
| #125 product completion | reconciled | final observed workflows, not a moving progress ledger |
| #131 direct reads | confirmed | later-process direct index read without Project/Emmy/Graph replay |
| #169 direct FTS | confirmed | body/symbol rows in the same direct current index |
| #191 umbrella | reconciled | staged queue and live status own work order/state |
| #194 result DTOs | corrected | only requested-scope blockers are limitations; nonblocking observations remain Project facts |
| #195 recognizer handoff | confirmed | remove generic pack/plan/matcher round-trip through named consumers |
| #197 CLI | corrected | exact `retail` Slice 1A arguments, root-relative paths and deterministic exit precedence |
| #198 LSP | reconciled | later adapter to the same target/plan/check owner |
| #199 MCP | reconciled | later single standard profile over the same owners |
| #200 Project input | corrected | target-bound plan identity, physical scope/empty-Main behavior, standalone limitations and exact snapshots |
| #201 end-to-end | corrected | first observation covers XML/dependency partial input, empty physical Main and actionable malformed Library input |

## Material source corrections

### Plan identity, not an argument shim

Every current TOC reader accepts `ProfileIdentity`, and `ProjectLoadPlan` stores
its mandatory Interface and canonical profile digest. `AddonLoadTarget` must own
plan identity/validation directly. An adapter that constructs a fake profile or
retains `target_profile_digest` does not complete T32.

### Physical-only means XML cannot abort it

Current `Loader::expand` recursively parses selected XML before returning Main.
Slice 1A therefore needs a distinct `physical_lua` projection. Included XML stays
source-located and makes the requested scope partial, but malformed/deferred XML
cannot erase diagnostics from direct TOC Lua.

### One-plan limitations are not package limitations

`project_package_limitations` skips parser dependency placeholders because the
package closure normally replaces them. Slice 1A has no package closure. It needs
a standalone mapper that reports unresolved required dependencies, selected
conditions, missing direct source and deferred XML while suppressing normal
optional absence and compatibility observations when no target Interface exists.

### Direct snapshots and errors

The direct route uses the compiled analyzer pin, Main universe `Project`, one
separate `BlizzardUi` snapshot per Library root and existing acquisition limits.
Paths remain canonical root-relative identities. A malformed caller Library is an
invalid input; current `LibraryHealthFailed -> ComponentUnavailable` mapping is a
known correction, not an internal outage contract. Duplicate roots/snapshot IDs
are rejected, and the error retains configured root plus relative file. An empty
physical Main with selected deferred XML uses an empty Project snapshot and a
located partial limitation rather than a dummy source.

## Maintenance rule

When an owner/result boundary changes, update current status and trace; update the
queue only if order changes; update only affected Issue bodies. Do not copy a new
PR list into README, AGENTS, skills or domain guides. Before deleting code,
recheck current production callers instead of treating an old Issue census as
live truth.
