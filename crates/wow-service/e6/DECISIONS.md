# E6-B decisions

**Status:** normative.

## E6B-001 — Service owns orchestration, not provider semantics

`wow-cbm` E6-A remains the only owner of descriptor/state/query validation and Candidate normalization.

## E6B-002 — Provider configuration is exact and repository-controlled

A query references one exact provider configuration/profile. Floating `default`, environment discovery, provider tool discovery, and source-supplied configuration are forbidden.

## E6B-003 — Credentials never enter public requests

Only stable credential references and authorization receipts cross the service boundary. Raw secret material stays inside the credential/session adapter.

## E6B-004 — Session acquisition is a narrow port

The host adapter returns one exact session binding plus an E6-A allow-listed transport. Service does not spawn or configure arbitrary processes.

## E6B-005 — Query dispatch is durable

Register operation/request identity before dispatch. Provider calls may consume quotas or observe mutable state; response loss becomes `OutcomeUnknown`, not safe retry.

## E6B-006 — Results are immutable artifacts

Raw bounded provider receipts, normalized result sets, explanations, and artifacts receive immutable identities and explicit validation/retention state.

## E6B-007 — External authority never rises in service

Service cannot upgrade `semantic_candidate + Candidate` through mapping, selection, repetition, provider reputation, or context inclusion.

## E6B-008 — Mapping belongs to source owners

`wow-project` and `wow-reference` validate locator identity under exact retained generations. Service cannot inspect source or choose among ambiguous mappings.

## E6B-009 — Mapping proof is narrow

`ExactMapped` proves only locator-to-owner-record identity. Provider summaries, traces, relationships, scores, and interpretations remain unverified Candidate evidence.

## E6B-010 — No-mapping authority is owner-scoped

Only an owner with complete relevant coverage may return `NoMappingWithOwnerAuthority`; otherwise the state is partial, conflict, or `NotEvaluated`.

## E6B-011 — Selection is explicit and auditable

The caller supplies exact candidate and mapping IDs. Service never selects by rank, score, count, name, path, or position.

## E6B-012 — Selection is not acceptance

A selection receipt records intent to inspect/use a candidate. It is not semantic proof, edit authorization, migration approval, or production promotion.

## E6B-013 — Context uses exact local roots

After mapping and selection, service invokes the existing exact-root context path. Provider metadata remains an external sidecar and never becomes a framework fact.

## E6B-014 — No recursive public service call

E6-B reuses internal owner-acquisition/use-case primitives; it does not call E3-C through its public service API.

## E6B-015 — Local capability remains independent

Provider/session/query failure cannot lower exact local capability states. The external lane is reported separately.

## E6B-016 — No hidden fallback

Service never switches provider, external generation, query scope, stale cache, model, web, or local search without a new explicit request.

## E6B-017 — Continuation binds cumulative budgets

Continuation cannot reset limits, hide prior truncation, or move to a new state/session unless the exact stable-generation/session policy authorizes reacquisition.

## E6B-018 — Cache is validated, not owned by `wow-cbm`

`wow-store` owns physical cache/catalog retention. E6-A validates logical entries; service coordinates exact retrieval and publication.

## E6B-019 — Every durable boundary has its own receipt

Provider dispatch, result publication, mapping, selection, context publication, retention, and audit are not collapsed into one inferred success.

## E6B-020 — Close-before-success applies

Provider session, owner views, leases, store handles, and audit resources close in reverse order. Mandatory close failure changes the public outcome.

## E6B-021 — Provider text is structurally isolated

Snippets, summaries, labels, paths, and errors are data. They cannot select profiles, authorize tools, alter mappings, or instruct agents.

## E6B-022 — E7 owns transports and public integration

LSP/MCP/HTTP/daemon command surfaces, provider setup UX, public distribution, and release/update channels remain E7.