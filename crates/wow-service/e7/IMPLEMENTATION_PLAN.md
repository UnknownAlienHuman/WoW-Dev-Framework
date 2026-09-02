# E7-A implementation plan

**Status:** normative order; implementation has not started.

## Phase 0 — prerequisite and protocol freeze

Before E7-A Rust source:

- implement/freeze all E0–E6 prerequisites used by active operations;
- freeze exact official LSP and MCP specification revisions and conformance vectors;
- freeze stdio/framing/JSON-RPC/profile compatibility, position encoding, capability, authorization, privacy, budget, cancellation, progress, backpressure, and canonicalization profiles;
- freeze owner service ports and exact request/result/error schemas;
- freeze session/view/overlay identities and all machine fixtures/checksums;
- record unsupported/deferred methods/tools/resources explicitly.

No placeholder server, fake owner, fake authorization, fake conformance result, or “latest spec” dependency.

## Phase 1 — protocol-neutral types

Implement profile, capability, session, workspace binding, exact view set, operation, result/error, progress/cancellation, and projection types in the E7-A `wow-service` slice.

Tests: `S7A-PROTO-*`, schema/canonicalization/security subsets.

## Phase 2 — session and exact view lifecycle

Implement initialization, exact workspace/current-once resolution, owner acquisition, compatibility validation, retention, explicit rebind, shutdown, reverse closure, and abrupt-disconnect handling.

Tests: `S7A-SESSION-*`, response-loss/retention/closure cases.

## Phase 3 — document overlays

Implement exact document identity, full open, ordered incremental/full change, encoding/range validation, complete overlay generations, save observation, close, overlay owner-analysis port, and invalidation.

Tests: `S7A-DOC-*`, Unicode/edit/fuzz/resource cases.

## Phase 4 — read/analysis operations

Implement diagnostics, hover, definition, references, symbols, advisory actions, context, and search wrappers through exact owner ports. Preserve candidate/partial/coverage/conflict states.

Tests: `S7A-AN-*` and E0–E6 regressions.

## Phase 5 — operation control

Implement exact request-to-operation registry, cancellation, progress, partial results, continuation, bounded queues, backpressure, shutdown barriers, and effect reconciliation.

Tests: `S7A-CTRL-*` and scheduling/fault injection.

## Phase 6 — authorization/privacy/security

Implement narrow authorization-port integration, privacy/license/source-boundary enforcement, session isolation, redaction, URI/path policy, and security limits.

Tests: `S7A-SEC-*` plus fuzz/adversarial corpora.

## Phase 7 — LSP adapter

Activate `apps/wow-lsp` after service types/bytes freeze. Implement only the approved method subset, exact framing, capability negotiation, document sync, position conversion, request/result/error mapping, cancellation/progress, and shutdown.

No direct lower-crate import, edit, formatting, rename, execute-command, settings mutation, or hidden discovery.

## Phase 8 — MCP adapter

Activate `apps/wow-mcp` after service types/bytes freeze. Implement only fixed tools/resources, exact stdio framing/profile, schema validation, authorization projection, cancellation/progress, and result/resource mapping.

No sampling, elicitation, dynamic tools, arbitrary file/URL/tool passthrough, or provider/database access.

## Phase 9 — conformance and cross-adapter parity

Run official/repository protocol vectors and verify:

- equivalent LSP/MCP operations reach the same service operation/result where semantics overlap;
- transport IDs/client metadata do not alter semantic IDs;
- all projection loss is declared;
- source/privacy/authorization state is consistent;
- no protocol bypasses service/owner boundaries.

## Phase 10 — fault and resource evaluation

Inject malformed framing/JSON/URI/edits, duplicate IDs, stale versions, disconnects, slow peers, queue floods, cancellation, owner failure, response loss, output failure, shutdown races, cross-session substitution, and credential/source injection.

Measure and freeze message, document, queue, owner-call, latency, memory, throughput, and shutdown/closure thresholds only after implementation.

## Phase 11 — deterministic freeze

Run 1/2/N workers, shuffled owner completions, different request IDs/client names/progress cadence/cache/storage layout/platform adapters. Freeze service semantic bytes and protocol projection bytes required by the profiles.

## Phase 12 — implementation evidence

Populate exact implementation commits, lock/toolchain/target profiles where activated, official spec/profile digests, owner-port IDs, platform adapter reports, benchmark thresholds, fixtures, canonical bytes, and SHA-256 manifests. Update machine frontier only with fresh passing evidence.

## Deferred to E7-B

- repository Rust workspace/toolchain activation policy if not already performed by E0 implementation;
- reproducible build/package/release artifact profiles;
- SBOM, provenance attestations, signing, verification;
- release catalogs/channels/publication/update/rollback;
- release CI/workflows and protected environment policy;
- binary distribution and installer behavior.

## Other deferred scope

- remote network daemon/multi-tenant protocol transport;
- LSP completion/rename/formatting/semantic tokens/inlay hints/file operations;
- MCP sampling/elicitation/prompts/dynamic tools;
- source edits/workspace edit application;
- runtime WoW client control/verification;
- model-driven orchestration.
