# `wow-rules` implementation status

## Implemented

- Closed E0-E input model bound to one project snapshot/generation, one Reference view, and one target profile.
- Global uniqueness, path/digest/span, relationship, target-profile, coverage-state, and aggregate-budget validation.
- Complete-Reference API-absence rule; analyzer `unresolved` alone is never absence evidence.
- Exact-binding Secret concatenation rule using complete Reference presence/restriction evidence and explicit dominance relations.
- Shadowing-safe guard behavior and fail-closed handling of dangling/cross-file/contradictory observations.
- Deterministic canonical input, diagnostic, and report identities with stable sorting and no host paths or rendered analyzer prose.

## Deliberately outside this crate

- Source acquisition, parsing, semantic analysis, project publication, Reference persistence, and transport.
- Secret inference from names, types, analyzer messages, or incomplete coverage.
- Runtime/client/combat/taint/secure-execution verdicts.
- Automatic fixes, replacement selection, or editor mutation.

## Next integration boundary

`wow-service` E0-F must build the closed `RuleEvaluationInput` from one immutable `wow-project` read view plus one compatible `wow-reference` view, expose the result through versioned operation envelopes, and preserve all `NotEvaluated` and authority boundaries.
