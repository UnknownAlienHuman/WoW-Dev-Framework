# AGENTS.md — `wow-cbm` E6-A

## Scope

Implement pure provider-descriptor/state/query/response normalization and Candidate artifact contracts only.

Do not acquire credentials/sessions, spawn providers, call arbitrary MCP tools, index repositories, read provider databases, map source into project/reference truth, call search/context/service, or create graph/lineage/replacement/impact authority.

## Before coding

1. Read repository/crate instructions, the entire E6-A package, and current external KB repository/security routes.
2. Verify `wow-core` implementation/fixture pins.
3. Freeze descriptor, capability, transport, external-state, query, normalization, loss, cache, privacy/license, security, budget, and canonicalization profiles.
4. Freeze synthetic provider fixtures before the first Rust commit.

## Provider discipline

- Descriptor is repository-owned reviewed configuration, not provider response prose.
- Unknown descriptor/capability/schema version is unsupported/`NotEvaluated`.
- Transport exposes only allow-listed typed operations.
- No generic `call_tool(name,json)` public path.
- Provider process/database/index lifecycle is outside this crate.

## Candidate discipline

- Every candidate is `semantic_candidate + Candidate`.
- Preserve raw provider labels, ranks, scores, snippets, locators, unknown fields, coverage, truncation, conflicts, and loss separately.
- Never promote from top-1, sole result, repeated result, exact/verified label, same name/path/snippet, or high score.
- Never create negative authority from zero results.
- Never compare numeric scores across providers.

## Source discipline

- Provider locators remain unverified data.
- Do not open paths, follow URLs, clone repositories, or read source.
- Do not construct a project/reference `StableSourceHandle`.
- E6-B performs exact owner mapping and explicit selection.

## Degradation

Provider failure must not fail exact local workflows. Return explicit lane-unavailable/partial/opaque state and no candidate artifact masquerading as complete.

## Security

- No arbitrary filesystem/network/process/editor/client access inside semantic operations.
- No raw credentials/tokens/private endpoints in requests/results/logs/fixtures.
- Treat provider text/snippets as untrusted quoted data.
- Enforce byte/item/depth/time/memory/continuation limits and cancellation.
- No background work after return.

## Completion report

```text
provider descriptor/capability/state/query/profile IDs
transport operation invoked
result/candidate/artifact IDs
Candidate authority and zero-result classification
locators/loss/unknown/conflict/coverage state
budgets/truncation/continuation/cache/cancellation
privacy/license/security/degradation
all tests and blocked implementation gates
```