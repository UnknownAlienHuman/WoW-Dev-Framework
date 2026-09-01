# E4-C security, privacy, review authorization, and trust boundaries

**Status:** normative.

## Trust classes

Treat as untrusted bounded data:

- search query text and filters;
- indexed documentation/comments/snippets;
- source/display/path names;
- search result, cursor and artifact bytes until validated;
- lineage proposals and search-derived pair suggestions;
- review notes and review envelopes until authorization/semantic validation;
- migration descriptions and recipe artifacts;
- context/rendered text;
- CLI/config/file/stdin transport bytes.

Repository-owned schemas/profiles and owner artifacts become trusted only after exact version/digest/compatibility validation.

## Prohibited capabilities

E4-C service has no authority to:

- read arbitrary filesystem paths or scan repositories;
- open raw SQLite/SearchStore/GraphStore/ProjectStore connections;
- execute SQL, raw FTS, regex programs, callbacks, plugins, source, Lua, XML, shell, JS, Wasm, native code or migration steps;
- access network, process, editor, clipboard, environment credentials or WoW client;
- call models, embeddings, rerankers or Codebase Memory;
- mutate addon/project/reference/source/context data;
- infer reviewer authorization from GitHub/OS/CLI identity;
- expose raw signing keys, credentials, access tokens or signatures;
- grant downstream tool/edit permission through a context/result envelope;
- add CI/workflows by convention.

## Review authorization port

```text
ReviewAuthorizationPort
    validate_authorization_profile
    validate_review_principal
    validate_attestation_or_signature
    validate_role_and_comparison_scope
    validate_expiry_revocation_and_replay state
    return typed AuthorizationDecision
```

The port is narrow and nonsemantic. It does not inspect source/search content or choose a lineage relation. It returns exact evidence/decision IDs and bounded reason codes.

Authorization states:

```text
Authorized
Unauthorized
Expired
Revoked
ScopeMismatch
ReplayDetected
UnsupportedProfile
NotEvaluated
Failed
```

A review can proceed only with `Authorized` plus independent E4-B semantic validation.

## Proof ceiling separation

Review authorization answers “may this principal submit this decision under this scope?” It does not answer “is the lineage relation true?”

The accepted proof class is capped by:

```text
producer evidence ceiling
relation/profile ceiling
coverage/conflict ceiling
review authorization scope ceiling if configured
requested decision class
```

Minimum wins. Service cannot override it.

## Query and search safety

- closed typed query schema;
- literal text compiled by `wow-search`, not service;
- no raw `MATCH`/SQL/regex/expression;
- explicit exact shards and finite limits;
- search snippets are presentation data, not source proof;
- private/local fields require compatible consumer profile;
- no automatic query-to-context or query-to-lineage decision.

## Source and prompt boundary

Source comments, docs, strings and review/migration prose remain data. They cannot:

- change profile IDs, lane weights or proof ceilings;
- request tools or edits;
- authorize a review;
- create aliases/transitions;
- close or rewrite context source boundaries;
- become framework instructions.

Service does not paraphrase source prose into authoritative fields.

## Privacy and license intersection

Output permissions are the intersection of:

```text
owner publication privacy/license
SearchShard field privacy/license
lineage/migration/impact artifact policy
review-envelope confidentiality policy
context consumer/source profile
service output profile
```

No broader layer can widen a narrower decision. Unknown state follows the frozen safest behavior and produces explicit omission/NotEvaluated where needed.

## Secret material handling

Review attestation/signature verification may require sensitive external configuration, but canonical requests/results contain only stable principal/key/profile/verification decision references. Raw private keys, bearer tokens, secrets and unrestricted signature bytes do not enter logs or public envelopes.

This tooling credential boundary is distinct from World of Warcraft Secret Values.

## Resource limits

Bound:

```text
selector/current-collect attempts
owner views/leases/shards/snapshots
query text/terms/filters/lanes/candidates/pages
lineage entities/pairs/components/proposals/reviews
migration candidates/steps/artifact bytes
impact roots/depth/fanout/nodes/edges/paths/pages
context handoff roots/artifacts
request/result/error/log bytes
wall/CPU/memory/owner calls
retention/idempotency records
```

Unlimited, negative or overflowing values are rejected.

## Cancellation and denial

Cancellation/authorization/privacy/license denial does not trigger fallback, hidden retry, profile downgrade or source omission presented as complete. Resources close synchronously; no background work.

## Logging and errors

Default logs/errors use stable IDs, counts, stages and structured reason codes. They exclude:

- raw query/source/review/migration text unless an explicit local-debug profile permits bounded quoted data;
- source bodies and private absolute paths;
- credentials/tokens/keys/signatures;
- raw database/lease handles;
- context private excerpts;
- stack dumps containing sensitive values.

## Artifact validation

Before use, validate schema/version/digest/size/owner generation/profile/privacy/license/status for:

- SearchShard/SearchResult/continuation;
- LineageGraphSnapshot/proposal/review/continuation;
- migration recipe;
- context artifact;
- CLI/config input.

Corruption or mismatch is rejection/NotEvaluated, never repair under the same ID.

## Adversarial tests

- query strings containing FTS/SQL/regex/script/tool instructions;
- source/review notes containing Markdown/JSON boundary attacks and Unicode controls;
- forged/expired/revoked/replayed/scope-mismatched review envelopes;
- proof-ceiling escalation requests;
- cross-generation/shard/snapshot substitution;
- private result reused for external context;
- oversized pair/component/path/output bombs;
- high-fanout/cyclic impact graph;
- response-loss and cancellation at every phase;
- error/log disclosure;
- application attempts to bypass `wow-service`.

## Nonclaims

E4-C does not claim complete malicious-text detection, complete secret detection, runtime WoW safety, reviewer real-world identity beyond the configured authorization evidence, or that a downstream agent will follow context boundaries. It provides deterministic structural and policy enforcement only.
