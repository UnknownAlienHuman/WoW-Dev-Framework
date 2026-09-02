# E6-B security, privacy, credentials, and audit

**Status:** normative.

## Untrusted inputs

Treat as untrusted bounded data:

- provider descriptors/capability responses until exact validation;
- provider paths, URIs, repository labels, symbols, spans, snippets, summaries, ranks, scores, and relations;
- external generation/observation labels;
- provider session responses and cursors;
- CLI/config/artifact bytes;
- cached/catalog records until digest/schema/profile validation;
- mapping and selection notes;
- source text returned only through owner/context policies.

Repository-owned schemas/profiles are trusted only after exact version/digest verification.

## Prohibited capabilities

E6-B cannot:

- execute provider tools by arbitrary name;
- accept/send generic MCP JSON-RPC or arbitrary tool schemas;
- read/write provider databases or index files;
- install/start/stop/update/configure/index/import/delete a provider;
- execute source, shell, SQL, FTS, Lua, XML scripts, JavaScript, Wasm, plugins, callbacks, expressions, or models;
- access arbitrary filesystem, network, process, editor, clipboard, environment, or WoW client outside narrow configured ports;
- follow provider URLs or open provider paths;
- invoke E4 search as hidden mapping/fallback;
- mutate project/reference/context/provider state except explicitly registered local service artifacts;
- expose credentials/session handles/provider cursors/private endpoints;
- infer authorization from repository/OS/CLI/file/commit identity;
- create edit/tool/publication/runtime authority.

## Credential boundary

Service configuration references adapter and authorization profiles, never credential bytes. `ProviderCredentialAuthorizationPort` and `ProviderSessionAcquirePort` own secret retrieval/use.

Forbidden in canonical/public surfaces:

```text
API keys and bearer tokens
OAuth access/refresh tokens
cookies and session secrets
private endpoints with credentials
SSH/private keys
KMS/HSM/vault secrets
provider DB credentials
raw Authorization headers
opaque live session/connection handles
```

This boundary concerns host/provider credentials and is distinct from WoW Secret Values.

## Network boundary

Only the configured provider adapter may perform network/IPC transport for allow-listed E6-A operations. Service itself receives no arbitrary URL/host/port. DNS/proxy/TLS/auth behavior is adapter/deployment policy and produces nonsecret receipts.

No provider network access occurs during mapping or context except through separately owned exact project/reference/context ports.

## Provider source boundary

Provider snippets/summaries are structurally isolated `semantic_candidate` data. They cannot alter profiles, choose candidates, authorize mapping, become source, define context instructions, or trigger tools.

Instruction-like provider text remains untrusted data. Canonical JSON escaping and renderer boundary profiles prevent it from changing output structure.

## Privacy/license intersection

Output permission is the intersection of:

```text
provider descriptor/result disclosure policy
external-state/session policy
project/reference owner publication policy
mapping profile
selection profile
context consumer/privacy/license policy
service output/audit profile
```

A higher layer cannot widen a lower restriction. Unknown state yields denial, omission, or `NotEvaluated` under the safest frozen policy.

## Source mapping privacy

- Public mapping receipts use stable locator/root IDs rather than private paths by default.
- Owner source bytes are not returned by mapping unless explicitly permitted.
- A provider URI can remain private even when the mapped owner record is public.
- Same text does not justify copying a private provider snippet into context.

## Audit events

Append-only events cover:

```text
provider authorization requested/authorized/denied/expired/revoked/replayed
session acquired/capabilities negotiated/state observed/closed
operation registered/dispatched/responded/reconciled/quarantined
result normalized/persisted/cataloged/retained
mapping requested/completed/ambiguous/partial/failed
selection validated/selected/rejected/deferred/superseded
context handoff/result/omission/failure
cache validation/hit/miss/rejection
retention admission/release and resource closure
cancellation and response loss
```

Each record binds previous digest, exact operation/request/artifact identities, bounded nonsecret actor/service refs, visibility, privacy/license state, and trusted time evidence where required.

## Replay and substitution defenses

Reject:

- credential/session receipt for another provider/operation/scope;
- external-state receipt from another session/descriptor/profile;
- provider result for another query/state;
- candidate/locator/mapping/selection substitution;
- mapping receipt for another owner generation;
- continuation/cache entry under a different state/privacy profile;
- provider cursor replay beyond profile;
- same-name/path/display substitution;
- response receipt from another operation;
- context request with a different mapped root.

## Resource ceilings

Bound:

```text
providers/sessions/capabilities
request/query terms and bytes
results/candidates/locators/snippets
continuation pages/cursors/cumulative budgets
catalog/list pages
mapping requests/results/evidence
selection records
context requests/attachments
retention/audit records
output/error/log bytes
wall/CPU/memory/owner/provider calls
```

Unlimited, negative, overflowing, or profile-exceeding values are invalid. Partial/truncated states remain explicit.

## Cancellation

Cancellation is checked around authorization, session/state acquisition, provider dispatch, normalization, persistence, mapping, selection, context, retention, audit, serialization, and closure. No background work remains. Effect uncertainty becomes `OutcomeUnknown`.

## Logging

Default logs include stable IDs, operation/stage/status, bounded counts, and reason codes. They exclude credentials, private endpoints, session handles, provider cursor bytes, private paths/URIs/snippets, source bodies, exact owner storage handles, and unrestricted stack dumps.

## Exact local workflows

Provider failure or denial cannot degrade unrelated local correctness. Service emits a scoped optional-feature outcome and leaves exact project/reference/graph/search/context/diagnostic workflows usable.

## Adversarial tests

- forged/expired/revoked/replayed credential/session receipt;
- arbitrary MCP/tool name or endpoint injection;
- credential in every input/output/log surface;
- malicious provider path/URI/snippet/summary/JSON control text;
- provider result/cursor/state substitution;
- cross-owner/generation mapping substitution;
- rank/top/sole candidate auto-selection;
- privacy widening through cache/context;
- high-fanout/oversized/deep result and audit bombs;
- response loss at every effect boundary;
- cancellation and close failure;
- application bypassing service/credential boundaries.
