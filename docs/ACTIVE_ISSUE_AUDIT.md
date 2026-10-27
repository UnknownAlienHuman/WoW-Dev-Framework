# Active Issue instruction audit

**Audit base:** `main` after PR #265 (`231c15065563ee8443d4353282f395aa000e1a4d`)  
**Scope:** all 21 open implementation Issues  
**Runtime/source traces used:** T01–T31

Issue bodies are implementation instructions, but they are secondary to
[CURRENT_STATUS.md](CURRENT_STATUS.md), [REFACTOR_QUEUE.md](REFACTOR_QUEUE.md),
[IMPLEMENTATION.md](IMPLEMENTATION.md) and the
[trace register](IMPLEMENTATION_TRACE_REGISTER.md). An Issue must not preserve a
stale result shape or re-add completed work.

The open draft PR #266 is not part of `main` and does not change this audit base.

## Method

For every open Issue:

- read the full body;
- checked named production producers/readers rather than tests alone;
- compared delivered/open claims with T24–T30;
- checked whether the body copied moving PR status or defined a durable outcome;
- marked material contradictions for correction.

Statuses:

- **confirmed** — the named problem and owner boundary still exist;
- **reconciled** — the body is compatible with current owners and links live
  status for moving details;
- **corrected** — the body contained a material stale/false instruction and was
  rewritten in this audit.

## Register

| Issue | Status | Current boundary |
| --- | --- | --- |
| #103 recognizer rules | confirmed | useful typed facts still need named diagnostics/search/navigation consumers; generic pack inventory is removable |
| #106 Blizzard UI workflow | confirmed | no ordinary current-source body/symbol/search/context workflow exists |
| #107 graph duplication | confirmed | useful origins/relations still pass through reconstruction/proof-heavy paths |
| #109 coherent updates | reconciled | shared owner exists; selected membership/context/Library changes still require cold rebuild and atomic publication |
| #111 Store lifecycle | confirmed | orphan administration can be removed first; retention/GC/recovery still have transitional callers |
| #112 formats/documentation | reconciled | one live status source and small direct DTOs remain the target; baseline register is not a deletion backlog |
| #113 performance | confirmed | there is no ordinary real workflow to measure yet |
| #117 TOC/XML relationships | reconciled | exact selected context/maps are useful; special-load and binding facts remain later slices |
| #118 lineage/fingerprints | confirmed | named lineage routes are gone; fingerprint/proof plumbing still has transitional callers |
| #122 readable context | confirmed | current public Context still depends on old packs/replay/source rereads |
| #125 product completion | corrected | completion contract remains valid; stale “implementation stops at #250” prose removed in favor of live status |
| #131 direct reads | confirmed | expensive reconstruction is above lazy Store membership/payload reads |
| #169 direct FTS | confirmed | source bodies are not yet queried through one direct current index |
| #191 umbrella | corrected | stale delivered-PR inventory replaced with the live status/queue boundary and first vertical slice |
| #194 result DTOs | corrected | Issue falsely required snapshot and upstream-analysis IDs removed by #260; direct result is findings/completeness/limitations only |
| #195 recognizer handoff | confirmed | fixed built-in facts still round-trip through generic pack/plan/matcher consumers |
| #197 CLI | reconciled | target remains direct ordinary config -> `addon_check`; first command is the explicit `LuaTypes` slice |
| #198 LSP | reconciled | project reuse, version gaps, stale rejection and standard empty results remain open |
| #199 MCP | reconciled | one useful profile plus metadata/unknown-notification behavior remains open |
| #200 configuration/acquisition | corrected | T30 predicates are delivered, not remaining work; first command no longer waits for every special-load feature |
| #201 end-to-end workflow | corrected | delivered T30 is separated from still-open context; first observation starts with explicit `LuaTypes` then expands |

## Material corrections

### Issue #194

The old body said `AddonCheckResult` contains Main/Library/virtual snapshot IDs and
Main/virtual analysis IDs. PR #260 removed those fields and current code contains
only scope, completeness, diagnostics and limitations. The Issue now forbids
reintroducing the removed receipts.

### Issues #200 and #201

Both bodies still listed the three-valued predicate correction as unfinished
after PR #265. They also repeated the over-scoped plan in which channel,
`LoadWith`, implicit bindings, secure context and complete XML behavior blocked the
first public command. The corrected bodies use the queue's slices:

1. explicit physical-Lua `LuaTypes` command;
2. dependency/XML/chunk context;
3. special loading;
4. WoW checks and remaining adapters.

T30 remains an observation case, not implementation work.

### Issues #125 and #191

Their product outcomes were valid, but progress paragraphs stopped at PR #250.
Moving delivery lists were removed. They now defer to current status and the queue
rather than becoming additional live checkpoints.

## Findings not converted into work

- Library caller order is not an open bug: the configured Emmy owner sorts
  Library snapshots by exact snapshot ID before registration and compares the same
  canonical set for reuse.
- A draft/open implementation PR is not delivered state.
- Compilation/Clippy and a source trace do not close an Issue whose outcome is an
  ordinary command or protocol result.

## Maintenance rule

When a merge changes an owner/result boundary:

1. update current status and the trace register;
2. update the queue only when work order changes;
3. update only affected Issue bodies;
4. do not copy the new delivery list into README, AGENTS or domain guides.

Before deleting a subsystem, distinguish a production caller from a test/fixture,
an absent route from an orphan export, and a historical baseline requirement from
current product scope.
