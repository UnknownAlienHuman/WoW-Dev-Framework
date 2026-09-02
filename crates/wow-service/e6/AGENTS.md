# AGENTS.md — `wow-service` E6-B

## Scope

Implement transport-independent orchestration over exact E6-A external-candidate artifacts, provider sessions, owner mapping receipts, explicit selections, and the existing context use case.

Do not implement provider retrieval, source mapping heuristics, context construction, storage internals, or arbitrary MCP/tool transport.

## Before coding

1. Read repository/crate instructions and the entire E6-A/E6-B packages.
2. Verify every prerequisite implementation commit, public owner port, profile, fixture, benchmark, and checksum.
3. Freeze provider descriptor, external-state, session/credential, durable-operation, mapping, selection, context-handoff, privacy, retention, audit, result, error, and CLI profiles.
4. Freeze exact synthetic and real-provider vectors before the first Rust commit.
5. Record missing provider/credential/owner capabilities as blocked or `NotEvaluated`; never create a fake adapter.

## Provider discipline

- Acquire only a configured allow-listed provider adapter.
- Validate descriptor and capability negotiation before session acquisition.
- Credentials stay inside the credential/session adapter.
- Public requests contain nonsecret provider/profile references only.
- Never expose arbitrary tool names or raw MCP payloads.
- Never install, launch, configure, index, mutate, or delete provider state.
- Never fall back to another provider, model, stale result, or cache silently.

## External-state discipline

- Preserve `StableExternalGeneration`, `ObservedMutableGeneration`, or `OpaqueExternalState` exactly.
- Never call observed mutable/opaque state reproducible or current.
- Continuation/retry binds the original external-state receipt.
- Same results do not prove same generation.

## Durable-effect discipline

- Register `OperationId + CanonicalRequestDigest` before any provider/store effect.
- Persist dispatch and effect receipts.
- Response loss after dispatch becomes `OutcomeUnknown` until exact reconciliation.
- Do not blindly issue the same provider request twice.
- Same operation ID with a different request digest is rejected.
- Cancellation does not prove no provider effect.

## Mapping discipline

- Treat every provider path/URI/symbol/span/digest as `UnverifiedProviderLocator`.
- Route mapping through exactly one declared project or reference owner port.
- Never open a provider path or follow a provider URL.
- Never map by first/name/path/snippet similarity.
- Preserve multiple, partial, conflict, and `NotEvaluated` outcomes.
- `ExactMapped` validates locator-to-owner identity only; it does not validate provider claims.

## Selection discipline

- Selection requires exact result, candidate, mapping-receipt, mapped-root, selector profile, and digest guards.
- Selection origin must be explicit and auditable.
- Rank/score/top/sole/provider labels do not authorize selection.
- Selection remains a nonauthority user/orchestrator decision record.
- Selection does not grant edit, tool, publication, runtime, or platform authority.

## Context discipline

- Invoke the existing context service path with the exact mapped owner root.
- Do not pass provider prose/rank/score as framework facts.
- External candidate evidence remains separately labelled in the outer service envelope.
- A context failure cannot be replaced by provider snippets or summaries.
- Source privacy/license rules remain owned by exact project/reference/context profiles.

## Lifecycle discipline

- Acquire in frozen order and close in exact reverse order.
- Obtain retention before advertising a durable result, continuation, mapping, selection, or context handle.
- No public success before mandatory close receipts.
- No detached/background work after return.
- Cache validation never raises freshness or authority.

## Completion report

```text
operation/request/external-state/provider/session identities
exact owner publications and mapping receipts
candidate/result/artifact/selection/context identities
provider dispatch and response-loss/reconciliation state
authority/coverage/conflict/partial/truncation state
privacy/license/credential/retention/audit/closure state
commands/tests/probes/benchmarks and skipped/NotEvaluated gates
```
