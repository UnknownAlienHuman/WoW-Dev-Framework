# E6-A decisions

**Status:** normative.

## CBM-001 — Optional and replaceable

External semantic-candidate providers are optional. Exact local workflows never depend on availability.

## CBM-002 — `wow-cbm` depends only on `wow-core`

Project/reference/graph/search/context/store/service mapping and orchestration remain outside the crate.

## CBM-003 — Reviewed descriptors define the contract

Provider prose or runtime tool discovery cannot define active schemas/operations/authority.

## CBM-004 — Transport is narrow and allow-listed

No generic arbitrary MCP/tool-call API is exposed by E6-A.

## CBM-005 — Session and credentials belong to E6-B adapters

E6-A receives an already-acquired typed transport and never owns process startup, credentials, authentication, or deployment configuration.

## CBM-006 — Provider index lifecycle is outside E6-A

No install/update/configure/index/import/delete or direct database effect is owned by this crate.

## CBM-007 — External state has three explicit classes

Stable generation, observed mutable state, and opaque state have different reproducibility/cache/continuation claims.

## CBM-008 — Candidate ceiling is hard

Every normalized provider result remains `semantic_candidate + Candidate`, regardless of provider labels or rank.

## CBM-009 — Scores are provider-local

Raw score/rank may be retained and normalized for display within one provider/profile only. No cross-provider numeric comparison or fusion.

## CBM-010 — Zero result has no negative authority

It means only no returned candidates for the exact query/state/operation under reported coverage.

## CBM-011 — Locators remain unverified

Provider paths/URIs/revisions/symbols/spans are data and cannot become stable owner handles without E6-B exact mapping.

## CBM-012 — Comparison is descriptive

`compare_external_candidate_results` reports set/field/rank differences under exact compatible bindings; it does not decide truth, winner, replacement, or provider quality globally.

## CBM-013 — Continuation binds external state

Continuation cannot refresh provider state, switch provider, reset budgets, or hide prior truncation.

## CBM-014 — Cache never upgrades freshness

A cache hit preserves original descriptor/state/query/coverage/staleness/authority and is invalid for any mismatch.

## CBM-015 — Unknown fields are retained or loss-recorded

Normalization never silently drops provider semantics or coerces unknown to safe/false/complete.

## CBM-016 — Provider source text is untrusted data

Snippets/comments/summaries cannot control profiles, tools, agent instructions, authority, or selection.

## CBM-017 — Opaque state is explicitly nonreproducible

It may support bounded one-shot discovery but not exact replay/long-lived cache/authoritative comparison claims.

## CBM-018 — Provider failure is lane-local

Unavailable/malformed/timed-out provider state cannot downgrade exact local capabilities.

## CBM-019 — No hidden fallback

E6-A never silently queries another provider, stale cache, model, web service, or local search when the selected provider fails.

## CBM-020 — E6-B owns selection and context handoff

E6-A returns candidates only. It never chooses top/sole results or invokes `wow-context`.

## CBM-021 — No provider identity as truth

Provider name, vendor, repository, popularity, license marketing, or deployment owner does not change Candidate authority.

## CBM-022 — No public side effects after return

Cancellation/timeout closes the call path synchronously; late responses are discarded or separately audited by the E6-B transport owner.

## CBM-023 — Exact bytes are canonical; operational timings are not

Candidate/result/artifact identity excludes host/process/time/cache-hit/network-latency state.

## CBM-024 — Patch-sensitive WoW claims remain elsewhere

E6-A stores no current API/event/Secret/taint/runtime constants; exact owners and the external KB remain authoritative routes.