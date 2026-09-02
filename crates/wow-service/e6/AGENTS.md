# AGENTS.md — `wow-service` E6-B

## Scope

Implement orchestration only. E6-A owns provider descriptor/state/query normalization; project/reference owners map locators; graph/project/reference/context owners build exact local context; store owns persistence; host adapters own credentials and provider sessions.

## Before coding

1. Read repository/crate/service instructions and the complete E6-A/E6-B packages.
2. Read E3-C exact context acquisition, E2-D store publication, project/reference owner contracts, and current external KB routes.
3. Freeze implementation commits, owner-port IDs, provider/session/credential profiles, fixtures, measured limits, and checksums.
4. Do not create Rust placeholders until every freeze-gate field required by the first implementation commit is populated.

## Query discipline

- Register `OperationId + CanonicalRequestDigest` before session acquisition or provider dispatch.
- Resolve provider configuration and any permitted current selectors exactly once.
- Never repeat a provider query while the prior effect is `OutcomeUnknown`.
- Same operation ID/different digest fails.
- Continuation preserves exact provider/state/query/profile and cumulative budgets.
- Provider failure remains lane-local; no hidden fallback.

## Credential/session discipline

- Requests contain only nonsecret configuration, credential-reference, authorization, and session-policy IDs.
- Never expose tokens, keys, passwords, cookies, private endpoints, commands, environment variables, or provider database paths.
- Never install/start/update/configure/index/import/delete a provider from service.
- Acquired sessions expose only the E6-A allow-listed typed transport.
- Close session and owner resources in package-defined reverse order before public success.

## Mapping discipline

- Submit each locator to the exact selected owner generation.
- Do not open paths/URLs, inspect source, or reproduce owner mapping logic.
- Preserve `MultipleMappings`, partial, conflict, and `NotEvaluated` states.
- A clean no-mapping result requires explicit owner negative authority.
- Mapping proves locator identity only, not provider semantics.

## Selection discipline

- Record only caller-supplied exact candidate and mapping IDs.
- Never choose top, sole, highest-score, first, newest, nearest, same-name, or repeated candidates.
- Selection is not verification, acceptance, lineage, replacement, impact, or edit authorization.

## Context discipline

- Require exact mapping plus explicit selection.
- Reacquire exact retained local views and validate generation/profile closure.
- Invoke one `wow-context` operation with the exact mapped root.
- Keep provider snippets/summaries/ranks outside `ContextSemanticPack`; expose them only in a separate Candidate sidecar under privacy/license policy.
- Do not call a public service operation recursively.

## Durable effects

Result publication, mapping publication, selection recording, context artifact publication, retention, and audit are distinct effects with exact receipts. Response loss never proves no effect. Cancellation stops new work but preserves durable records and does not spawn background cleanup.

## Security

No raw SQL, generic MCP/tool call, script, plugin, model, shell, arbitrary filesystem/network/process/editor/client access, source execution, or provider database handle crosses the service seam.

## Completion report

Report changed files, exact operations/ports, implementation/profile/generation IDs, fixtures/tests, commands with pass/fail/skipped, response-loss and closure results, privacy/license state, and all remaining `NotEvaluated` gates.