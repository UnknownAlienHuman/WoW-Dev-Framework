# AGENTS.md — `wow-service` E7-A

## Scope

Implement transport-independent protocol profile, session, binding, document overlay, request, cancellation, progress, response-loss, operation-registry, framing-result, transcript, and shutdown contracts only.

Do not implement owner algorithms, editor mutation, arbitrary MCP tools, source execution, or release automation.

## Before coding

1. Read repository/crate instructions and all E7-A service/application contracts.
2. Pin exact LSP, MCP, JSON-RPC, stdio framing, URI, position encoding, document sync, error, cancellation, progress, and lifecycle specification revisions.
3. Verify all delegated service operations are already implemented/frozen with exact request/result schemas.
4. Freeze the immutable operation registry and both application method/tool maps.
5. Freeze session, binding, document, request, transcript, output, resource, privacy, and security profiles.
6. Freeze canonical wire vectors before the first Rust commit.

## Protocol discipline

- Initialize exactly once per session.
- Negotiate only reviewed capabilities supported by both exact profiles.
- No unknown method/tool fallback.
- No dynamic registration in E7-A.
- LSP and MCP use separate exact framing/profile implementations.
- JSON-RPC request IDs are opaque typed identifiers; never coerce string and integer IDs together.
- Notifications receive no JSON-RPC response.
- Transport success never upgrades domain status.

## Binding discipline

- Every request names one immutable `SessionBindingGeneration`.
- Workspace/document/config changes create a new binding generation.
- In-flight requests remain on their admitted binding.
- No current/profile/source refresh inside a request.
- Rebind requires exact expected prior binding and compatibility validation.
- Mixed generations fail rather than merge.

## Document discipline

- Document URI, language ID, version, position encoding, text digest, and overlay generation are exact.
- Changes must target the expected prior version/digest.
- Out-of-order, overlapping-invalid, or coordinate-invalid changes are rejected and can require full resynchronization.
- Open buffers are session overlays; no disk or editor write.
- Save is not proof that disk/source publication changed.
- Close removes the overlay through an explicit new binding.

## Operation-registry discipline

- Every protocol method/tool maps to one exact existing service operation or one lifecycle operation.
- Registry entries pin input/output schema, authority ceiling, effect class, cancellation, progress, privacy, and permission profile.
- No arbitrary service operation name, raw owner handle, generic command, or provider method.
- `tools/list`/capabilities expose only the exact active registry snapshot.
- No code edit, command execution, source mutation, pack activation, or release effect in E7-A.

## Effect and response-loss discipline

- Effecting delegated requests use existing service `OperationId + CanonicalRequestDigest` semantics.
- Protocol request identity never replaces domain operation identity.
- Cancellation/connection loss is not proof of no effect.
- `OutcomeUnknown` remains visible and requires exact reconciliation.
- A response lost after domain completion is recovered by exact request/operation receipt, not rerun under a new binding.

## Security and permissions

- Source/context/provider/review text remains untrusted data.
- Protocol content cannot create capabilities, tools, methods, permissions, profiles, roots, or commands.
- No credentials/private endpoints/session secrets in initialize options or requests.
- No arbitrary URI schemes or workspace roots.
- No stdout logging.
- No tool/edit/runtime permission inferred from MCP client identity, LSP client capabilities, or context content.

## Lifecycle discipline

- Reject requests before successful initialize and after shutdown except exact allowed lifecycle messages.
- Cancellation and progress are request/session scoped.
- Shutdown closes request admission; exit closes all resources synchronously.
- No detached work after exit or transport EOF.
- Transcript/audit completeness is validated before final close when the profile requires it.

## Completion report

```text
protocol/spec/framing/profile IDs
session and binding generation IDs
workspace/document versions and digests
method/tool -> service operation registry entry
request/operation/cancellation/progress identities
domain status and authority/nonclaims
wire bytes/error/result/transcript IDs
resource/retention/privacy/closure state
tests/probes/benchmarks and skipped/NotEvaluated gates
```
