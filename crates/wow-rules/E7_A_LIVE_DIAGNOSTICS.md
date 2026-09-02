# E7-A overlay-aware live diagnostics seam

**Status:** normative cross-crate seam; implementation has not started.

`wow-rules` evaluates existing diagnostic providers over one exact saved or overlay-aware owner input. It does not own sessions, transports, document synchronization, LSP diagnostics, editor code actions, or source mutation.

## Operations

```text
rules_live_document_diagnostics
rules_live_workspace_diagnostics
rules_live_code_action_candidates
```

## Input

```text
exact saved ProjectGeneration or overlay-aware project/analyzer view
exact ReferenceView and graph capability snapshots
requested diagnostic provider set/profile
exact document/workspace scope
overlay freshness classification
budgets and cancellation
```

Every provider declares required capabilities. Missing/stale/partial capabilities produce `NotEvaluated`, partial coverage, or disabled remediation—not clean results.

## Diagnostic output

```text
LiveDiagnosticResult
    exact input/profile/provider IDs
    root-cause and downstream finding relationships
    canonical UTF-8 source ranges and stable source handles
    severity/code/message/related evidence
    provenance/confidence/coverage/conflicts/omissions
    saved-versus-overlay freshness
    remediation tier and exact guards
    result ID/digest
```

Transport-specific severity numbers, tags, related-information objects, result IDs, and diagnostic `data` fields are E7-A service/application projections.

## Code action candidates

A code action candidate is one of:

```text
ExactGuardedEdit
PlanOnly
Disabled
NotAvailable
```

`ExactGuardedEdit` requires exact URI/source handle, document version/overlay ID/content digest, expected old bytes/range, replacement bytes, edit ordering, profile, and owner authorization. Any missing guard downgrades to `PlanOnly` or `Disabled`.

The rule owner never applies the edit. The initial E7-A LSP profile never autonomously invokes `workspace/applyEdit`.

## Diagnostic result identity

An unchanged result requires identical exact project/overlay/reference/graph/provider/profile inputs and provider outputs. Same document version, path, or visible messages alone is insufficient.

## Hard boundaries

- no direct dependency on service/application/LSP/MCP/editor crates;
- no transport lifecycle or document buffer ownership;
- no source parsing outside approved owner facts;
- no editor command or setting mutation;
- no autofix without exact mechanical guards;
- no Candidate/fuzzy/external result promoted to a diagnostic fact;
- no clean result when required overlay or published dependency coverage is stale/partial;
- no duplicate downstream noise represented as independent root cause.

## Tests

Cover saved and overlay-local diagnostics, stale published dependencies, `NotEvaluated` capability gates, root-cause folding, exact edit guards, stale edit rejection, non-BMP ranges, cancellation, partial workspace diagnostics, unchanged-result identity, deterministic ordering, and faithful LSP/MCP/CLI projections.