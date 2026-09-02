# E7-A authentication, authorization, privacy, and trust boundaries

**Status:** normative.

## Separate decisions

```text
transport authentication
operation authorization
workspace/source authorization
artifact/source disclosure
effect authorization
semantic evidence/proof
```

These decisions are independent. Authentication or client approval never creates semantic proof.

## Authorization port

```text
ProtocolOperationAuthorizationPort
    validate_session_principal
    authorize_workspace_binding
    authorize_operation
    authorize_source_or_artifact_disclosure
    authorize_effect_scope
    validate_expiry_revocation_replay
```

The port returns stable nonsecret decision/evidence references. Raw credentials, private keys, tokens, cookies, KMS/HSM/vault handles, and transport secrets do not enter canonical service requests/results.

## Forbidden identity shortcuts

Never infer authorization from:

- LSP client name/version;
- editor “trusted workspace” flag alone;
- MCP host/client name or approval UI;
- OS user, process owner, terminal, pipe/socket owner;
- GitHub login, repository ownership, commit author, PR role;
- file owner/permissions alone;
- project/addon path or source comments;
- possession of a result ID without scope validation.

## Scope

Authorization decisions bind exact:

```text
principal/session/protocol profile
workspace/project/source universe
operation kind and effect class
artifact/result/source classes
privacy/consumer trust profile
exact or bounded root/document scope
issuance/expiry/revocation/replay evidence
```

A grant for hover does not authorize source edits, search, context excerpts, external provider use, pack publication, or another workspace.

## Operation classes

Initial E7-A classes:

```text
SessionControl
WorkspaceMetadataRead
DocumentOverlayWriteSessionLocal
AnalysisRead
SearchCandidateRead
ContextMetadataRead
ContextSourceExcerptRead
AdvisoryActionRead
EffectingOwnerOperation (only existing explicitly exposed service operations)
```

E7-A protocol apps do not expose core publication, source-file mutation, shell, model, arbitrary network, or generic owner effects.

## Client capabilities

Protocol capability negotiation affects message shape and supported projections only. It does not authorize an operation or expand privacy/source scope.

MCP tool availability and LSP server capabilities are computed from implementation + profile + owner capability + authorization + privacy state. A client cannot enable an unavailable or denied capability by declaring support.

## Privacy intersection

Permitted output is the intersection of:

```text
owner artifact/source policy
workspace/project publication policy
Reference/Blizzard UI/provider source license and privacy state
session consumer trust profile
operation authorization decision
protocol capability descriptor
result/renderer/source-boundary profile
```

A higher layer can narrow but never widen a lower restriction.

## Source classes

At minimum distinguish:

```text
StableMetadataOnly
PublicOrRedistributableSource
LocalProjectSource
PrivateSource
GeneratedAnnotationSource
BlizzardImplementationSource
ExternalCandidateSnippet
HiddenHoldoutOrReviewMaterial
SensitiveOperationalMetadata
Unknown
Denied
```

Unknown defaults to the safest explicit policy. Local analysis permission is not external disclosure or redistribution permission.

## LSP local source

An editor may already possess a document’s text, but the server still validates whether derived context/source excerpts can be returned. Private absolute roots and unrelated project files are not exposed merely because one document is open.

Document overlay text remains session-local unless an explicit retained artifact profile permits storage. Closing/shutdown discards it according to policy without implying a disk save.

## MCP source and resources

MCP resource reads use exact opaque resource IDs/URIs and authorization checks. They do not accept arbitrary file paths, URLs, provider locators, glob patterns, or repository names as resource authority.

Tool/resource output follows the source-data boundary from `wow-context`; source and provider text remain structurally labeled untrusted data.

## External candidates

Invoking E6-B through an MCP tool requires exact provider authorization/session profiles. Provider results remain `Candidate` and cannot be inserted into framework facts or context without exact owner mapping and explicit selection receipt.

Credential availability is operational; it does not enter result confidence.

## Effect authorization

Effecting operations already exposed by service retain their own durable operation, authorization, review, signing, publication, or rollback contracts. E7-A does not collapse them into a generic protocol permission.

An MCP client’s confirmation or LSP execute-command request is not sufficient. Initial E7-A adapters do not expose generic effect execution.

## Result reuse

A result/artifact/continuation handle can be reused only when:

- exact session/consumer authorization scope permits it;
- bound generations/profiles remain retained;
- privacy/license policy matches;
- result is not cross-session private unless explicitly shareable;
- continuation/request identity matches;
- revocation/expiry state permits disclosure.

Cache hits never bypass authorization.

## Revocation

Revocation affects future access/operations. It does not rewrite historical semantic artifacts. Active operations follow the frozen revocation profile: cancel, block output, finish metadata-only, or reconcile an effect.

Every denial/revocation/replay/expiry is auditable without logging secrets.

## Audit

Audit records bind:

```text
session/principal/protocol profile
workspace and operation
exact authorization decision and evidence refs
requested/disclosed source/artifact classes
result/error/closure state
trusted time evidence when required
previous audit digest
```

Source bodies, credentials, confidential review/holdout material, and unrestricted client text are excluded by default.

## Nonclaims

E7-A does not prove:

- the client or model will obey source-data boundaries;
- an authenticated client is safe;
- a tool result authorizes another tool;
- a local workspace is public or redistributable;
- an advisory action is safe to apply;
- a source comment is true;
- an external provider result is verified;
- a static finding reflects runtime state.
