# AGENTS.md — `wow-service` E3-C

## Scope

Implement only exact context-view acquisition, use-case sequencing, service envelopes, and the public owner ports needed by E3-C.

Do not implement context generation, graph queries, project/reference/store internals, search, rules, models, source parsing, CLI behavior, or external transports.

## Before coding

1. Read all repository/service/E3-C contracts.
2. Read the E3-A `wow-project`, E3-B `wow-context`, E2-D `wow-store`, E2-A `wow-graph`, and E1-B `wow-reference` contracts.
3. Read current KB routing for any patch-sensitive claim; service must not hard-code live WoW facts.
4. Freeze every owner-port schema/operation ID and compatibility profile.
5. Freeze exact acquisition, failure, cancellation, continuation, envelope, CLI handoff, and checksum fixtures.
6. Verify all prerequisite implementations and retained-view behavior before adding Rust.

## Orchestration discipline

- One public operation invokes one explicit orchestration plan.
- Validate request/config/profile before acquiring expensive resources.
- Resolve symbolic selectors once and record exact identities.
- Acquire in the frozen global order; close in reverse order.
- Validate `ContextUniverseSet` before invoking map/skeleton/pack operations.
- Use owner results unchanged except for typed service envelope references.
- Validate context artifacts through `wow-context` before publication.
- Finalize success only after mandatory close succeeds.

## Selector discipline

- `CurrentPublished` is resolved only here, never in app or context.
- No second current read, hidden retry, or compatibility fallback.
- Exact publication/store generation remains exact.
- Optional expected-current guards fail on mismatch.
- Reference generation comes from the exact selected publication binding or an exact guard, never floating current.
- Continuation accepts exact embedded generations only.

## Resource discipline

- Never expose raw SQLite connections, transactions, locks, snapshots, lease objects, parser/analyzer handles, or filesystem paths.
- Treat lease admission, retention, close, and release failures as typed service outcomes.
- Close partially acquired sets on every failure/cancellation path.
- Do not publish a complete result if closure is incomplete.
- No background task, daemon, or detached continuation.

## Context boundary

- Exact roots only.
- Service selects profile IDs/defaults and invokes context operations; it does not inspect source text or rank candidates.
- Do not alter ProjectMap/L0/L1/ContextSemanticPack/RenderedContextArtifact records.
- Preserve all partial/truncated/NotEvaluated/conflict/omission/budget state.
- Do not infer negative authority from empty context output.

## Error and privacy discipline

- Return stable typed codes with bounded arguments.
- Errors may include stable project/publication/profile/artifact IDs but not source bodies, private absolute paths, credentials, raw continuation internals, or unrestricted configuration.
- Source bytes may appear only inside validated context artifacts permitted by their privacy/license/consumer profiles.
- Service logs/timings are noncanonical and disabled from canonical output.

## Application seam

Expose only typed request/result interfaces required by `apps/wow/e3`. Do not parse flags, read artifact files/stdin, write streams, inspect terminal state, or choose exit codes.

## Completion report

```text
operation and request ID
symbolic selectors and exact resolved identities
acquisition order and owner-port versions
ContextUniverseSet/profile/root IDs
context semantic/render artifact IDs
status/coverage/conflict/omission/truncation/continuation
resource closure report
budgets/cancellation
all tests and missing freeze/evaluation gates
```
