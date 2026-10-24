# Active Issue caller audit

**Checked:** 2026-10-24  
**Source revision:** `338dcbec597ece5e4e296425b874ecc2203d2b8f`  
**Scope:** all 21 open implementation Issues after PR #254, plus deeper T02–T05/T11 load tracing

This register checks Issue bodies as an instruction surface against current
production callers. It complements the baseline
[AUDIT_REGISTRY.md](AUDIT_REGISTRY.md), the post-merge
[implementation trace register](IMPLEMENTATION_TRACE_REGISTER.md) and the focused
[TOC/load trace](TOC_LOAD_CONTEXT_AUDIT.md). It does not replace the ordered
[queue](REFACTOR_QUEUE.md) and does not claim runtime execution.

## Method and status

- Read every open Issue body after the PR #246–#254 merges.
- Followed the named current producer, handoff and production reader; tests and
  fixtures were treated as evidence, not automatic product consumers.
- Rechecked ordinary TOC/dependency/context/order claims against pinned Blizzard
  TOCs and the reviewed Wowless loader comparison.
- **confirmed** means the named current boundary still exists.
- **corrected** means the body contained a current-source contradiction or omitted
  a mandatory source boundary and was rewritten.
- **reconciled** means its body already matched delivered owners and no new
  contradiction was found.

## Register

| Issue | Status | Source result / current boundary |
| --- | --- | --- |
| #103 Recognizer rules | confirmed | `RecognizerRegistry::e2_default` remains internal to fixed recognizer adapters; built-in typed producers feed Service graph-build consumers. The six `e2/examples/*.json` files have no production reader and remain test inventory. |
| #106 Blizzard UI workflow | confirmed | No ordinary current-source index/search/context command exists. Existing FTS lanes cover labels/recorded names, not Lua/XML body text; later-process direct context remains dependent on #131/#169/#122. |
| #107 Graph duplication | confirmed | `graph/build/native_api.rs::publish` still calls native Library reconstruction/verification, which loads Reference, renders annotations and builds a second Library snapshot before source-call publication. |
| #109 updates | reconciled | Exact immutable `ConfiguredAnalyzerSession` and direct `addon_check` exist; physical delta and selection/config/context rebuild boundaries remain correctly separated. Exact ordered load context is now part of the cold-rebuild identity. |
| #111 Store lifecycle | corrected | Backup/replacement/quarantine/source-authority and several report/release types have no production caller outside `wow-store`; GC/recovery/retention still reach Service/application callers and cannot be deleted as one block. |
| #112 wire formats | reconciled | Current body distinguishes delivered direct DTOs from legacy replay/Search/Graph formats and preserves semantic admission after decoding. |
| #113 performance | confirmed | There is no ordinary Blizzard UI workflow to measure yet. Performance remains correctly gated on #106 rather than inferred from parser reuse or failed preconditions. |
| #117 TOC/XML relationships | corrected | Existing XML virtual units/maps are reusable, but the body omitted the exact typed package/per-unit context and selected-order receipt required to relate persisted XML units to actual selected loading. Pure preparation, handler context and direct persisted relations remain open. |
| #118 lineage/fingerprints | corrected | The named lineage/source-comparison application, Service and Graph modules are already absent. Remaining scope is function/XML fingerprint CLI options, Project models, Service candidate manifests and old Search hold/proof plumbing. |
| #122 readable context | confirmed | The old Context path still depends on Project/Graph reports, packs/continuations and source rereads. Exact body-backed plain context is not implemented. |
| #125 product completion | reconciled | Completion requires observed ordinary generation/check/editor/search workflows; build/Clippy and lower owner construction are explicitly insufficient. |
| #131 direct reads | corrected | `ReadSnapshot::acquire` validates generation membership and reads payloads lazily through `record`; it does not decode every partition. The expensive current path is `AcquiredProjectPair::read -> ProjectReplay::hydrate_owner -> recapture` in Search/Context/Graph readers. |
| #169 direct FTS | confirmed | `StoredSearchArtifact.database_hex` still embeds an inner SQLite image as hex in canonical JSON; current FTS rows index `document_id`/metadata labels or recorded names, not source bodies. |
| #191 umbrella | reconciled | Delivery order starts from ordinary Project acquisition into the existing configured owner/direct Service result and treats old platform removal as consumer migration. |
| #194 result DTOs | reconciled | `AddonCheckResult` is the retained direct check result; generic component/capability/presentation/result identities remain transitional caller cleanup. |
| #195 recognizer handoff | confirmed | Public built-in producers have Service graph-build consumers. `run_recognizers`/`adapt_emmy_direct_calls` have no external production caller; generic pack/plan use outside `wow-recognizers` is concentrated in `graph/build/recognizer_handoff.rs`. |
| #197 CLI | reconciled | The target CLI must call Project acquisition and existing `wow-service::addon_check` directly, not adapt the old `LocalCommand`/generic Service route. |
| #198 LSP | reconciled | Diagnostic payload is delivered, but overlay input remains one-file; project reuse, version gaps, stale-result rejection and standard empty results remain open. |
| #199 MCP | reconciled | Direct lower owners exist; one useful MCP profile, `_meta` extensibility and unknown-notification behavior remain open. |
| #200 configuration/acquisition | corrected | The body named only `Dep` and generic load context. The real next boundary also requires `RequiredDep`/`OptionalDep`, typed `UseSecureEnvironment`/`LoadIntoEnvironment`/private-table/load-phase facts, correct tri-state predicates and a validated exact selected-order handoff beside path-sorted snapshots. |
| #201 end-to-end workflow | corrected | The ordinary-check sequence must preserve the same exact context/order input before CLI observation; builder/Library/check/index/context observations remain open and PR #246–#250 owners must not be rebuilt. |

## Corrections published during this audit

- #111 separates confirmed orphan Store surfaces from GC/recovery/retention types
  with current production consumers.
- #118 no longer assigns deletion of modules/routes already absent from `main` and
  names the remaining fingerprint/proof scope.
- #131 preserves the useful lazy Store record boundary and targets the real
  Project replay/hydration reconstruction above it.
- #117 now requires persisted/direct TOC/XML relations to consume the same
  validated selected order and package/per-unit context as diagnostics.
- #200 now names the complete finite dependency aliases, execution-context facts,
  tri-state condition semantics and separate ordered context receipt.
- #201 now carries that owner boundary into the first public CLI observation.

No Issue was closed from source inspection alone. No workflow was promoted from
`unexecuted` to `observed`.

## Maintenance rule

Issue prose is secondary to current code, `CURRENT_STATUS.md`, the queue and the
trace register. When a merge changes an owner boundary, update those documents
and the affected Issue body in the same work. Before deleting a subsystem, search
production callers separately from tests/fixtures and distinguish an absent route,
an orphan export and a transitional consumed API.