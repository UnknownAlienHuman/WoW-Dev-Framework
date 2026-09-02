# E7-A editor use cases

**Status:** normative service orchestration.

All editor operations consume one exact `SessionSnapshot`. Source URI/position/range resolution belongs to `wow-project`; platform symbols/contracts belong to `wow-reference`; analyzer facts belong to `wow-emmy`; graph relations belong to `wow-graph`; diagnostics belong to `wow-rules`; retrieval belongs to `wow-search`; context artifacts belong to `wow-context`.

Service coordinates these owners and never reproduces their algorithms.

## Common admission

Every editor request validates:

```text
operation descriptor and frontend exposure
session/client/generation/snapshot
project/profile/reference/overlay compatibility
document URI, version and exact owner file handle
position/range coordinate profile
required owner capabilities
privacy/license/source disclosure
budgets/cancellation/continuation
OperationId + CanonicalRequestDigest when effecting
```

Missing mandatory capability yields `NotEvaluated`. Partial or conflicted coverage remains explicit.

## Diagnostics

`editor_diagnostics` invokes the exact diagnostic service/rule path against the captured session snapshot.

Results preserve:

```text
root-cause and downstream finding distinction
rule/provider/version/profile
exact source range and document version
severity/tags/related evidence
capability and coverage state
conflicts/omissions/NotEvaluated
remediation tier and edit guards
```

A clean result requires complete relevant coverage. Open overlays are analyzed as part of the session-private project view; diagnostics for a stale document version are not returned as current.

Both document and workspace diagnostic scopes are allowed by exact request profile. Pagination/continuation binds the same snapshot and cumulative budgets.

## Completion

`editor_completion` returns deterministic candidates from exact project scope and exact ReferenceView facts. Completion order follows explicit authority lanes and stable tie keys:

```text
exact lexical/project symbols
exact members/namespaces/types
explicit aliases and compatible platform records
bounded additional candidates
```

Completion items carry source/owner identity, kind, replacement range, insert-text form, evidence/confidence, deprecation/restriction state, and required resolve data.

No model/provider result, repository popularity, global frequency, or fuzzy score becomes exact completion authority. External semantic candidates are excluded from the baseline completion profile.

Completion resolve is represented as another exact service operation only when separately registered; the baseline result must contain enough stable data to avoid arbitrary client callbacks.

## Signature help

`editor_signature_help` resolves one exact call site through analyzer/project facts and returns compatible ReferenceView/project callable signatures with active parameter evidence.

Unknown overload resolution, partial analyzer state, dynamic calls, or profile conflicts remain `Possible`/`Candidate`/`NotEvaluated`; service does not choose a convenient signature.

## Hover

`editor_hover` resolves one exact source position to zero/one/many owner entities, then builds a bounded context/annotation projection.

Hover may include:

```text
exact symbol/entity identity and kind
project or platform definition summary
signature/type and ownership/load context
restriction/deprecation/profile state
selected source links under policy
coverage/conflicts/nonclaims
```

Multiple entities remain multiple. Search/provider summaries are not used to fill missing exact hover facts.

## Definition

`editor_definition` returns exact owner source locations only when project/reference/analyzer/graph evidence establishes the definition relation under the bound generation.

Possible/fuzzy/external candidates may be returned only in a separately tagged candidate section if the frontend profile permits. They are never encoded as exact LSP definition locations.

No same-name, same-path, first, nearest, top-ranked, or sole candidate shortcut.

## References

`editor_references` queries exact project/graph/analyzer relation partitions and returns locations with relation kind, producer, confidence, coverage, and conflict state.

A missing reference result is not globally negative unless every required partition has owner negative authority. Dynamic calls/hooks/registries with incomplete capability produce partial/`NotEvaluated`, not clean zero.

Declaration inclusion is an explicit request field. Pagination binds exact snapshot/filter/order/budgets.

## Document symbols

`editor_document_symbols` uses exact project source/entity facts and preserves multiple structural axes rather than inventing one universal parent. The selected frontend hierarchy profile explicitly chooses lexical, object, ownership, or another allowed axis.

An unsupported axis yields `NotEvaluated`; it is not silently replaced by lexical nesting.

## Workspace symbols

`editor_workspace_symbols` invokes exact-generation search over the bound project/reference universe. Results expose lane, score semantics, authority, source identity, coverage, and continuation.

Search rank does not establish definition, replacement, lineage, or edit authority. Symbol resolve, if exposed, is a separate exact operation mapping.

## Call hierarchy

`editor_prepare_call_hierarchy` resolves the source position to exact callable entities or explicit candidates. `editor_incoming_calls` and `editor_outgoing_calls` query the exact graph call axis.

Call evidence remains distinct from event registration, hooks, callbacks, XML script handlers, ownership, and load order. The result may expose these as separate relation classes; it never merges them into calls.

Partial analyzer/recognizer/graph coverage is explicit. A call hierarchy item carries exact session snapshot and entity identity so subsequent requests cannot switch generations.

## Code actions

`editor_code_actions` consumes exact findings and source/session state. It returns:

```text
MechanicallySafeEditCandidate
PlanOnlyAction
DisabledAction
```

A mechanically safe candidate requires:

```text
exact project/session/overlay/document generation
exact source digest and editor version
exact finding/remediation implementation/profile
deterministic nonoverlapping edits
precondition and postcondition validation
no unresolved conflict or unsupported restriction
privacy/license permission
```

The application may translate it to a protocol-native workspace edit carrying version/digest guards. The framework does not apply the edit, execute commands, save files, or refresh the project automatically.

A stale document version disables the action. Uncertain fixes remain plans/candidates.

## Unsupported baseline editor methods

The E7-A baseline does not advertise:

```text
rename
formatting
semantic tokens
inlay hints
folding ranges
notebook synchronization
executeCommand
workspace file operations
automatic applyEdit
```

A later package may add an operation only with owner semantics, exact effects/guards, fixtures, security analysis, and registry/version updates.

## Determinism

Equivalent exact snapshots/requests/owner artifacts yield identical canonical results independent of transport, editor, client, worker count, request arrival order, host path, locale, terminal, cache state, or network/pipe timing.