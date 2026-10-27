# Active Issue instruction audit

**Audit base:** `main` after PR #267 (`1e1772d1a4893bf3d2b6bb7eaf3617534dcbff7a`)  
**Scope:** all 21 open implementation Issues  
**Runtime/source traces used:** T01–T32

Issue bodies are task-specific instructions, secondary to
[CURRENT_STATUS.md](CURRENT_STATUS.md), [REFACTOR_QUEUE.md](REFACTOR_QUEUE.md),
[IMPLEMENTATION.md](IMPLEMENTATION.md) and the
[trace register](IMPLEMENTATION_TRACE_REGISTER.md). An Issue must not copy a
moving PR ledger, widen the active slice or preserve a removed result shape.

The open draft PR #266 is not delivered state and does not change this audit
base.

## Method

For every open Issue, the full body was read and compared with current production
producers/readers, the staged queue and T01–T32. Tests and comments were evidence,
not automatic product requirements.

Statuses:

- **confirmed** — the named problem and owner boundary still exist;
- **reconciled** — the body already states a durable outcome compatible with the
  canonical plan;
- **corrected** — the body contained a stale, false or over-scoped instruction
  and was rewritten.

## Register

| Issue | Status | Current boundary |
| --- | --- | --- |
| #103 recognizer rules | confirmed | useful typed facts still need named diagnostics/search/navigation consumers; generic pack inventory is removable |
| #106 Blizzard UI workflow | confirmed | no ordinary current-source body/symbol/search/context workflow exists |
| #107 graph duplication | confirmed | useful origins/relations still pass through reconstruction/proof-heavy paths; closed #196 is delivered history, not an open dependency |
| #109 coherent updates | corrected | removed the PR diary; exact physical delta versus cold selected-context rebuild remains the durable task |
| #111 Store lifecycle | confirmed | orphan administration can be removed first; retention/GC/recovery still have transitional callers |
| #112 formats/documentation | corrected | removed delivery ledgers; one live status source and small direct DTOs remain the durable boundary |
| #113 performance | confirmed | there is no ordinary real workflow to measure yet |
| #117 TOC/XML relationships | corrected | removed lower-owner PR history; exact selected context/maps remain a later slice after the physical-Lua command |
| #118 lineage/fingerprints | confirmed | named lineage routes are gone; fingerprint/proof plumbing still has transitional callers |
| #122 readable context | confirmed | current public Context still depends on old packs/replay/source rereads |
| #125 product completion | corrected | completion is staged observed behavior, not a moving implementation ledger |
| #131 direct reads | confirmed | expensive reconstruction is above lazy Store membership/payload reads |
| #169 direct FTS | confirmed | source bodies are not yet queried through one direct current index |
| #191 umbrella | corrected | live status and staged queue replace a copied delivery list |
| #194 result DTOs | corrected | direct result is scope/completeness/diagnostics/limitations, with no removed receipts |
| #195 recognizer handoff | confirmed | fixed built-in facts still round-trip through generic pack/plan/matcher consumers |
| #197 CLI | corrected | immediate command surface is Slice 1A only; auto TOC, target Interface, dependencies and WoW checks are later additions |
| #198 LSP | corrected | removed PR ledger and fixed owner reuse wording; the adapter reuses the shared project/check owner |
| #199 MCP | corrected | removed PR ledger and fixed owner reuse wording; one standard profile remains the task |
| #200 configuration/acquisition | corrected | added the profile-free `AddonLoadTarget` and T23 metadata admission to Slice A |
| #201 end-to-end workflow | corrected | first observation starts with explicit physical `LuaTypes`, then context and special loading |

## Material corrections

### Profile-free Project admission

Current TOC readers require `ProfileIdentity`, including a mandatory positive
Interface, source logical digest and schema identities. That is incompatible with
the direct ordinary route. #200 now requires a small Project-owned load target
with flavor, optional positive target Interface and optional TOC condition
context. Slice A supplies flavor only and retains source Interface without
fabricating a profile.

### First-slice metadata and completeness

T23 metadata-by-effect admission is now a Slice A prerequisite. Ordinary
display/localized/custom metadata cannot make independent `LuaTypes` partial;
actual selection/environment/callback and malformed directives remain typed or
located limitations. T31 still removes the unconditional addon-chunk limitation
and hard-coded `Partial`.

### Active frontend/update Issues

#109, #112, #117 and #197–#199 still copied old PR progress after the repository
had established one live status source. Those bodies now state durable outcomes,
current owner boundaries and staged work only.

### Earlier corrections retained

#125/#191 no longer duplicate delivery status. #194 no longer requires
snapshot/analysis receipts removed from the direct result. #200/#201 no longer
make channel, `LoadWith`, bindings, secure context and complete XML behavior
prerequisites for the first command.

## Findings not converted into work

- Library caller order is not an open bug: the configured Emmy owner sorts exact
  Library snapshot IDs before registration and compares the same canonical set
  for reuse.
- A closed Issue may remain a source-history reference; it is not an active
  dependency unless the body says unfinished work remains.
- A draft/open implementation PR is not delivered state.
- Compilation, Clippy and source inspection do not close an Issue whose outcome
  is an ordinary command or protocol result.

## Maintenance rule

When a merge changes an owner/result boundary:

1. update current status and the trace register;
2. update the queue only when work order changes;
3. update only affected Issue bodies;
4. do not copy the new delivery list into README, AGENTS, skills or domain guides.

Before deleting a subsystem, distinguish a production caller from a test/fixture,
an absent route from an orphan export, and a historical baseline requirement from
current product scope.
