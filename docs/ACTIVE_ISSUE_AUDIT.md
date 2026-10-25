# Active Issue caller audit

**Consistency review base:** `0784d0cb5f2c709fa7d6778f6ad7448df7a8cb77` (`main` after PR #256)
**Runtime callers reviewed through:** PR #250, `e40b64ffa1c16851fa657c0bd7fa0073e3cc33d9`
**Scope:** all 21 open implementation Issues after PR #256, including T17–T21

This register checks Issue bodies as an instruction surface against current
production callers. It complements [AUDIT_REGISTRY.md](AUDIT_REGISTRY.md), the
active [implementation trace register](IMPLEMENTATION_TRACE_REGISTER.md),
[TOC_LOAD_CONTEXT_AUDIT.md](TOC_LOAD_CONTEXT_AUDIT.md) and
[TOC_TRIGGER_BINDINGS_AUDIT.md](TOC_TRIGGER_BINDINGS_AUDIT.md). It does not
replace the ordered [queue](REFACTOR_QUEUE.md) or claim runtime execution.

## Method and status

- Read every open Issue body after PRs #246–#256 and rechecked the three bodies
  changed during T17–T21.
- Followed named producers, handoffs and production readers; tests/fixtures were
  evidence, not automatic product consumers.
- Rechecked TOC dependencies, execution context, product channel, load triggers,
  implicit bindings, official sentinel/security metadata and selected order
  against pinned Blizzard source and the pinned Wowless comparison.
- **confirmed** — the named boundary still exists.
- **corrected** — the body contradicted current source or omitted a mandatory
  handoff and was rewritten.
- **reconciled** — the body already matches current owners.

## Register

| Issue | Status | Source result / current boundary |
| --- | --- | --- |
| #103 Recognizer rules | confirmed | Built-in typed producers still feed Service graph-build consumers; example JSON files have no production reader. |
| #106 Blizzard UI workflow | confirmed | No ordinary current-source index/search/context command; FTS does not index Lua/XML body text. |
| #107 Graph duplication | confirmed | Native graph build still reconstructs/verifies Library/Reference before source-call publication. |
| #109 updates | reconciled | Configured owner/direct result exist; physical delta versus cold selection/context rebuild remains explicit. Channel, triggers, bindings and source compatibility/security metadata join cold-rebuild identity. |
| #111 Store lifecycle | corrected | Several backup/replacement/quarantine/report surfaces are orphaned inside Store; GC/recovery/retention still have production consumers. |
| #112 wire formats | reconciled | Direct DTOs are distinguished from legacy replay/Search/Graph formats; semantic admission remains required after decode. |
| #113 performance | confirmed | No ordinary Blizzard UI workflow exists to measure. |
| #117 TOC/XML relationships | corrected | Existing UI XML units/maps are useful, but persisted/direct relations must consume exact selected order/context. Binding selection/wrapping keeps its donor evidence class; zero-Interface/security uncertainty stays attached to Project identity. |
| #118 lineage/fingerprints | corrected | Named lineage/source-comparison routes are absent; remaining scope is fingerprints, Service candidate manifests and old Search proof plumbing. |
| #122 readable context | confirmed | Old Context still depends on Project/Graph reports, packs/continuations and source rereads. |
| #125 product completion | reconciled | Requires observed ordinary generation/check/editor/search behavior; build/Clippy is insufficient. |
| #131 direct reads | corrected | Store membership/payload reads are lazy; expensive reconstruction is Project replay hydration/recapture above them. |
| #169 direct FTS | confirmed | Inner SQLite remains hex in canonical JSON and FTS lacks source body text. |
| #191 umbrella | reconciled | Starts from ordinary Project acquisition into existing configured owner/direct Service result. |
| #194 result DTOs | reconciled | `AddonCheckResult` is retained; generic capability/presentation/result identities are transitional. |
| #195 recognizer handoff | confirmed | Built-in producers have real consumers; generic pack/plan use is concentrated in graph handoff. |
| #197 CLI | reconciled | Must call ordinary Project acquisition and existing `addon_check`, not adapt the generic route. Channel/trigger/binding/compatibility/security facts remain Project-owned. |
| #198 LSP | reconciled | Payload fixed; project reuse, version gaps, stale rejection and standard empty results remain open. |
| #199 MCP | reconciled | Lower owners exist; one useful profile, metadata and unknown-notification behavior remain open. |
| #200 configuration/acquisition | corrected | Ordinary target includes Stable/PTR/Beta and special-load adapters while preserving source occurrence, current-code, donor-model and unsupported-policy evidence classes; zero Interface and unresolved `Secure` remain scoped. |
| #201 end-to-end workflow | corrected | First observation must prove T17–T21 facts and their source/donor evidence class reach Service without being discarded, fabricated or promoted into stronger client guarantees. |

## Corrections completed

- #111, #118 and #131 were corrected in PR #254.
- #117, #200 and #201 were corrected after PR #255 for exact order/context.
- During the T19–T21 review, #117/#200/#201 were updated again for product
  channel, reverse triggers, implicit bindings, machine SavedVariables, official
  zero Interface and unresolved `Secure` metadata.
- The post-#256 consistency pass corrected those same bodies again where donor
  product-gating, environment/private-table, trigger or callback/discovery
  behavior had been labelled as client-authoritative source semantics. They now
  preserve source/current-code/donor/policy evidence classes and forbid stronger
  timing/availability claims without stronger evidence.
- No Issue was closed by source inspection; no workflow moved to `observed`.

## Maintenance rule

Issue prose is secondary to current code, `CURRENT_STATUS.md`, the queue and the
trace register. When a merge changes an owner boundary, update those documents
and affected Issue bodies together. Before deleting a subsystem, distinguish
production callers from tests/fixtures and an absent route from an orphan export
or transitional consumed API.
