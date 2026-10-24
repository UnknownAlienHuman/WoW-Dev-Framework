# Active Issue caller audit

**Checked:** 2026-10-24  
**Source revision:** `281b3504b827a4d87b2d3ec9a0ce0a53179a557d`  
**Scope:** all 21 open implementation Issues after PR #253

This register checks the Issue bodies as an instruction surface against current
production callers. It complements the baseline
[AUDIT_REGISTRY.md](AUDIT_REGISTRY.md) and the post-merge
[implementation trace register](IMPLEMENTATION_TRACE_REGISTER.md). It does not
replace the ordered [queue](REFACTOR_QUEUE.md) and does not claim runtime
execution.

## Method and status

- Read every open Issue body after the PR #246–#253 merges.
- Followed the named current producer, handoff and production reader; tests and
  fixtures were treated as evidence, not automatic product consumers.
- **confirmed** means the named current boundary still exists.
- **corrected** means the body contained a current-source contradiction and was
  rewritten immediately.
- **reconciled** means its body was already rewritten around the delivered
  configured owner/direct Service split and no new contradiction was found.

## Register

| Issue | Status | Source result / current boundary |
| --- | --- | --- |
| #103 Recognizer rules | confirmed | `RecognizerRegistry::e2_default` remains internal to fixed recognizer adapters; built-in typed producers feed Service graph-build consumers. The six `e2/examples/*.json` files have no production reader and remain test inventory. |
| #106 Blizzard UI workflow | confirmed | No ordinary current-source index/search/context command exists. Existing FTS lanes cover labels/recorded names, not Lua/XML body text; later-process direct context remains dependent on #131/#169/#122. |
| #107 Graph duplication | confirmed | `graph/build/native_api.rs::publish` still calls native Library reconstruction/verification, which loads Reference, renders annotations and builds a second Library snapshot before source-call publication. |
| #109 updates | reconciled | Exact immutable `ConfiguredAnalyzerSession` and direct `addon_check` exist; physical delta and selection/config rebuild boundaries remain correctly separated. |
| #111 Store lifecycle | corrected | Backup/replacement/quarantine/source-authority and several report/release types have no production caller outside `wow-store`; GC/recovery/retention still reach Service/application callers and cannot be deleted as one block. |
| #112 wire formats | reconciled | Current body distinguishes delivered direct DTOs from legacy replay/Search/Graph formats and preserves semantic admission after decoding. |
| #113 performance | confirmed | There is no ordinary Blizzard UI workflow to measure yet. Performance remains correctly gated on #106 rather than inferred from parser reuse or failed preconditions. |
| #117 TOC/XML relationships | reconciled | Existing XML virtual units/maps are reusable; pure direct preparation, handler context and persisted direct relations remain open. |
| #118 lineage/fingerprints | corrected | The named lineage/source-comparison application, Service and Graph modules are already absent. Remaining scope is function/XML fingerprint CLI options, Project models, Service candidate manifests and old Search hold/proof plumbing. |
| #122 readable context | confirmed | The old Context path still depends on Project/Graph reports, packs/continuations and source rereads. Exact body-backed plain context is not implemented. |
| #125 product completion | reconciled | Completion requires observed ordinary generation/check/editor/search workflows; build/Clippy and lower owner construction are explicitly insufficient. |
| #131 direct reads | corrected | `ReadSnapshot::acquire` validates generation membership and reads payloads lazily through `record`; it does not decode every partition. The expensive current path is `AcquiredProjectPair::read -> ProjectReplay::hydrate_owner -> recapture` in Search/Context/Graph readers. |
| #169 direct FTS | confirmed | `StoredSearchArtifact.database_hex` still embeds an inner SQLite image as hex in canonical JSON; current FTS rows index `document_id`/metadata labels or recorded names, not source bodies. |
| #191 umbrella | reconciled | Delivery order now starts from ordinary Project acquisition into the existing configured owner/direct Service result and treats old platform removal as consumer migration. |
| #194 result DTOs | reconciled | `AddonCheckResult` is the retained direct check result; generic component/capability/presentation/result identities remain transitional caller cleanup. |
| #195 recognizer handoff | confirmed | Public built-in producers have Service graph-build consumers. `run_recognizers`/`adapt_emmy_direct_calls` have no external production caller; generic pack/plan use outside `wow-recognizers` is concentrated in `graph/build/recognizer_handoff.rs`. |
| #197 CLI | reconciled | The target CLI must call Project acquisition and existing `wow-service::addon_check` directly, not adapt the old `LocalCommand`/generic Service route. |
| #198 LSP | reconciled | Diagnostic payload is delivered, but overlay input remains one-file; project reuse, version gaps, stale-result rejection and standard empty results remain open. |
| #199 MCP | reconciled | Direct lower owners exist; one useful MCP profile, `_meta` extensibility and unknown-notification behavior remain open. |
| #200 configuration/acquisition | reconciled | Small load target, selected dependency closure, safe exact inputs and pure XML preparation remain the next owner boundary; configured Emmy/Service owners must be reused. |
| #201 end-to-end workflow | reconciled | Builder/Library/check/index/context observations remain open; the issue no longer assigns construction of PR #246–#250 owners. |

## Corrections published during this audit

- #111 now separates confirmed orphan Store surfaces from GC/recovery/retention
  types with current production consumers.
- #118 no longer assigns deletion of modules/routes already absent from `main` and
  names the remaining fingerprint/proof scope.
- #131 preserves the useful lazy Store record boundary and targets the real
  Project replay/hydration reconstruction above it.

No Issue was closed from source inspection alone. No workflow was promoted from
`unexecuted` to `observed`.

## Maintenance rule

Issue prose is secondary to current code, `CURRENT_STATUS.md`, the queue and the
trace register. When a merge changes an owner boundary, update those documents
and the affected Issue body in the same work. Before deleting a subsystem, search
production callers separately from tests/fixtures and distinguish an absent route,
an orphan export and a transitional consumed API.