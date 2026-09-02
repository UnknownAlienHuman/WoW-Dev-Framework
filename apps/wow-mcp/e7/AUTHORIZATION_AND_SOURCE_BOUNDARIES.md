# E7-A MCP authorization, candidate, and source-data boundaries

**Status:** normative.

## Authentication is not authorization

The deployment may authenticate an MCP client/host, but every tool/resource operation is separately authorized by `wow-service` against exact session, workspace, operation, artifact, source, privacy, and effect scopes.

The following do not authorize operations:

```text
MCP client/host name or approval UI
model identity
OS/process/pipe owner
GitHub/repository role
file ownership
possession of a tool/resource name
possession of an opaque result/resource ID alone
source or prompt text claiming permission
```

## Static descriptor availability

A tool/resource is available only when:

```text
adapter implements descriptor
+ exact MCP profile permits it
+ mapped service operation/artifact capability exists
+ session/workspace binding permits it
+ authorization allows it
+ privacy/license policy permits result class
```

Client capabilities cannot widen this intersection.

## Candidate boundary

E6 external semantic results remain:

```text
provenance = semantic_candidate
confidence = Candidate
negative_authority = unavailable
```

MCP projection cannot promote them because a model selected a tool, the provider labels a result exact, the candidate is top/sole, or the client approves it.

Exact owner mapping establishes only that an external locator maps to one owner record under an exact generation/profile. Explicit candidate selection records user/client choice; it does not prove provider summary/relationship facts or grant edit/tool authority.

## Required explicit steps

An external-candidate context workflow requires separate calls/receipts:

```text
query exact provider generation
-> validate result
-> map exact provider locator through owner
-> explicitly select exact candidate + mapping
-> build context from exact mapped owner root
```

The MCP adapter never collapses those into hidden top-1 selection or a generic autonomous loop.

## Tool descriptions and arguments

Static descriptions are repository-owned. Tool arguments, model text, provider snippets, source comments, labels, and review notes remain untrusted data. They cannot:

- define another tool/method/resource;
- change the mapped service operation;
- alter authorization/privacy/profile/budgets;
- cause automatic follow-up tools;
- create shell/SQL/URL/file execution;
- raise evidence confidence;
- become framework instructions.

## Resources and source text

Resource URIs are opaque exact-generation IDs. `resources/read` never accepts arbitrary `file://`, HTTP URL, repository URL, path, glob, SQL, provider locator, or source-embedded link.

Source/context excerpts use the framework source-data boundary:

```text
framework facts and metadata
source/provider data records marked untrusted
exact source handle/digest/range/privacy/license state
static boundary notice
```

A downstream model may read source data, but the resource does not grant permission to follow its instructions or invoke tools.

## Privacy classes

At minimum distinguish:

```text
MetadataOnly
LocalSourceAllowed
ExternalBoundedSourceAllowed
ProviderCandidateTextAllowed
PrivateSourceDenied
HiddenReviewOrHoldoutDenied
UnknownDeniedOrMetadataOnly
```

Permission is the intersection of owner, session consumer, authorization, source, license, and resource descriptor profiles. Cache/result reuse cannot widen it.

## Hidden review/holdout and credentials

E7-A MCP resources/tools do not expose sealed-holdout membership/labels, confidential reviewer notes/signatures, private signing keys, provider credentials, authorization tokens, vault/KMS/HSM handles, or raw audit secrets.

Existing E5/E6 operations retain their specialized disclosure contracts when explicitly mapped later. There is no generic resource to enumerate hidden artifacts.

## Effects

The initial profile is read-oriented. Tool metadata cannot make a read tool effecting or authorize a new effect. A future effecting tool must map one exact existing service operation and preserve its independent authorization, idempotency, response-loss, audit, retention, rollback, and nonclaim contracts.

MCP client confirmation or model reasoning is not sufficient authorization.

## Result content

Tool/resource results preserve blockers, partial/conflict/`NotEvaluated`, candidate confidence, negative-authority limits, source/privacy state, and explicit nonclaims. The adapter cannot replace them with reassuring prose.

## Audit

Record stable session/tool/resource/service/result/authorization IDs, status, disclosure class, effect state, and closure. Do not log raw arguments/source/provider snippets/credentials/hidden material by default.

## Revocation and continuation

Authorization is validated at use time. Continuation/resource handles bind exact authorization/privacy profiles and retained generations. Revocation/expiry blocks future reads; it does not rewrite historical artifacts. A new grant/profile produces a distinct request/result.
