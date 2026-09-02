# E7-A MCP tools and resources

**Status:** normative static allow-list. Exact MCP specification revision freezes before implementation.

## Descriptor model

Every tool/resource descriptor is repository-owned immutable data:

```text
stable descriptor ID/version
MCP profile/spec compatibility
static title/description
closed input/output schema
one mapped service operation or retained artifact class
authorization/privacy/license requirements
side-effect classification
message/output budgets
canonical digest
```

Descriptors cannot be supplied or modified by project source, provider output, repository metadata, MCP client, model, prompt, or tool arguments.

## Initial tools

Canonical names are frozen in `CONTRACT.json`. The conceptual surface includes:

```text
wow.protocol.status
wow.session.initialize
wow.session.capabilities
wow.session.rebind_exact
wow.session.shutdown
wow.workspace.bind
wow.workspace.status
wow.analysis.diagnostics
wow.analysis.hover
wow.analysis.definition
wow.analysis.references
wow.analysis.symbols
wow.analysis.code_actions
wow.analysis.resolve_action
wow.context.request
wow.search.request
wow.external.query
wow.external.continue
wow.external.result_validate
wow.external.explain
wow.external.map
wow.external.select
wow.external.context
wow.operation.status
wow.operation.cancel
```

Each tool invokes exactly one `wow-service` operation. Tool arguments contain exact IDs/profiles/guards or explicitly permitted outer selectors consumed by that one service operation.

## Tool mapping

| Tool family | Service operation |
|---|---|
| protocol status/profile | `protocol_status` or `protocol_profile_validate` |
| session/workspace | corresponding `session_*` / `workspace_*` operation |
| diagnostics/navigation/actions | corresponding `analysis_*` operation |
| context/search | `context_request` / `search_request` |
| external Candidate | exact E6-B service operation |
| operation control | `operation_status` / `operation_cancel` |

No tool composes search+selection+context, external query+mapping+selection, diagnostics+apply, or publication workflows locally. Callers invoke explicit steps and receive exact receipts.

## Tool side effects

E7-A tools are classified:

```text
ReadOnly
SessionLocalOverlayEffect
ExistingDurableServiceEffect (only when an explicitly mapped preexisting operation is later exposed by a reviewed profile)
```

The initial MCP profile is read-oriented and does not expose source edits, pack/release publication, activation, rollback, shell, filesystem writes, or arbitrary owner effects.

Tool metadata side-effect hints are descriptive protocol projection, not authorization or proof. Service enforces exact operation contracts.

## External Candidate tools

E6-B semantics remain unchanged:

```text
provider query -> Candidate result
provider locator -> exact owner mapping receipt
explicit candidate + mapping -> selection receipt
selection receipt + exact mapped root -> context
```

The MCP adapter cannot automatically choose top/first/best/highest-score/sole candidate. Provider rank, score, snippet, summary, and “exact” labels remain Candidate metadata. Zero result is not negative authority.

## Resources

Initial resource classes:

```text
ProtocolStatusResource
SessionStatusResource
WorkspaceStatusResource
DiagnosticResultResource
NavigationResultResource
SearchResultResource
ContextSemanticPackResource
RenderedContextArtifactResource
ExternalCandidateResultResource
ExternalCandidateArtifactResource
OperationStatusResource
EvidenceCoverageConflictResource
```

Every resource is exact, retained, authorized, bounded, and immutable for its URI identity.

## Resource URIs

Conceptual opaque scheme:

```text
wowdev://<resource-class>/<opaque-exact-id>?profile=<exact-profile-id>
```

The exact grammar and encoding freeze before implementation. URI components are opaque IDs, not filesystem paths, repository URLs, provider locators, query strings, or current/latest selectors.

Resource lookup validates:

- descriptor/resource class;
- exact artifact/result/session/generation IDs;
- authorization and consumer profile;
- privacy/license/source-disclosure policy;
- retention and digest;
- output budget and renderer/source-boundary profile.

## Resource listing

`resources/list` is bounded and snapshot-bound. It returns only authorized descriptors/resources in stable ordering and may use exact continuation. It does not enumerate the filesystem, repositories, arbitrary stores, providers, or hidden holdout/review data.

An empty list is not proof no framework artifacts exist outside the authorized/profiled snapshot.

## Resource read

`resources/read` returns one exact eligible representation. Source/context content uses existing structural source-data boundaries. It cannot follow links/URIs embedded in source/provider text.

Raw databases, WAL files, source roots, credentials, authorization tokens, hidden holdout labels, review signatures, and lower owner handles are never resources.

## Prompts

E7-A does not expose MCP prompts. Repository/source/provider text cannot register prompts. Static future prompt templates would require a separate versioned contract and cannot grant tool or authority changes.

## Sampling and elicitation

Not supported. The server does not ask a client/model to reason, choose candidates, generate summaries, authorize actions, supply credentials, or execute follow-up tools.

## Dynamic list changes

The initial profile uses a static descriptor set for the process/profile. Session authorization may make descriptors unavailable, but source/provider/project changes do not dynamically create tools/resources. List-changed notifications are disabled unless a later exact profile requires them.

## Result schemas

Tool results preserve:

```text
exact service result ID/status
session/view/overlay/root IDs
candidate/authority/provenance
coverage/conflicts/blockers/omissions
partial/truncated/NotEvaluated/OutcomeUnknown
privacy/license/source boundary
nonclaims
```

Plain text summaries are secondary projections and cannot hide mandatory structured state.

## Errors

Malformed tool/resource arguments or unknown descriptors are protocol errors with no service call. Valid service blockers/partial/candidate states remain structured tool results. The adapter does not convert them to success text that implies proof.
