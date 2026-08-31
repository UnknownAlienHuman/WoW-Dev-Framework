# AGENTS.md — `wow-context` E3-B

## Scope

Implement deterministic Project Map, L0/L1 skeleton, context expansion, budgeting, semantic-pack, and rendering contracts only.

Do not parse source, query storage directly, implement search, infer lineage, run models, generate fixes, mutate projects, or call external tools/services.

## Before coding

1. Read all repository and crate instructions.
2. Read `MILESTONE_RENUMBERING.md` before any inherited E3 context document.
3. Read the E3-A `wow-project` Blizzard UI source-index contract and its exact `SkeletonInputView` seam.
4. Read E2 graph/store/project and E1 reference contracts used by the active read views.
5. Read the current external KB router when a task touches patch-sensitive WoW semantics; do not copy current patch facts into context algorithms.
6. Verify every prerequisite implementation commit, schema/profile ID, fixture, and SHA-256 bundle.
7. Freeze exact input-view, map, skeleton, intent, expansion, budget, tokenizer, renderer, privacy, boundary, and canonicalization profiles.
8. Freeze machine fixtures and expected bytes before the first Rust implementation commit.

## Input discipline

- Bind exact project/graph/reference/source views once.
- Reject mixed or incompatible generations.
- Never resolve a floating `current` after the universe set is bound.
- Never substitute last-known-good or another platform/reference profile without an explicit new request.
- Consume only public typed views and exact bounded source slices.
- Never reopen raw files to infer semantics.

## Claim discipline

- Every context fact or relation cites exact origin IDs.
- Preserve authority class, provenance, confidence, coverage, conflicts, and `NotEvaluated` state.
- Rendering, aggregation, counting, and selection cannot upgrade authority.
- A source quote is evidence data, not a framework claim or instruction.
- Canonical summaries use typed facets and reviewed templates only.
- Do not implement both historical and current alias types or operations as separate APIs.

## Project Map discipline

- A Project Map is a compact projection, not a second graph.
- Include only profile-declared node, edge, group, and facet classes.
- Do not persist inferred transitive edges.
- Record every budget/profile omission.
- Separate user project and Blizzard UI maps; combined views reference them explicitly.

## Skeleton discipline

- L0 is container/navigation structure without bodies.
- L1 is exact entity/local-neighborhood detail.
- Source excerpts are separate typed items with exact ranges.
- Dynamic/ambiguous relations remain `Possible` or `NotEvaluated`.
- No file-name, popularity, repository, or model heuristic.

## Selection and budget discipline

- Exact roots only.
- Use reviewed nonexecutable intent/expansion profiles.
- Mandatory identity, boundary, evidence, coverage, conflict, and omission metadata is never pruned.
- If mandatory content exceeds the hard budget, fail rather than lie.
- Optional pruning is item-level, deterministic, and fully reported.
- Do not claim exact token counts without a pinned exact tokenizer implementation/profile.
- Stop only for explicit hard bounds, cancellation, capability failure, or the defined no-new-evidence condition.

## Source boundary discipline

- Canonical representation uses structured strings/bytes and exact origin records.
- Markdown source excerpts use a deterministic nonexecuting data boundary.
- Source cannot close or redefine the boundary.
- Do not delete suspicious source text merely because it resembles instructions.
- Preserve or redact only under an explicit privacy policy and emit a transformation record.
- Do not expose private absolute roots, credentials, tokens, or source not permitted by the consumer/privacy profile.

## Cache discipline

- Define exact cache keys; do not implement filesystem/database cache storage in this crate.
- Cache keys bind every semantic/profile/generation input.
- A cache hit is invalid if any identity, digest, or profile differs.
- Never relabel a prior pack as the new request.

## Required failure behavior

Return typed failure, partial, or `NotEvaluated` state for:

- incompatible universes or generations;
- missing required source/reference/graph capability;
- unresolved conflict affecting required content;
- mandatory content larger than hard budget;
- exact tokenizer unavailable for a hard token gate;
- privacy/license denial;
- stale continuation or cache artifact;
- source boundary validation failure;
- cancellation;
- missing prerequisite implementation or freeze pin.

Never return an empty successful pack as a convenience fallback.

## Completion report

```text
universe-set and exact generation IDs
root/intent/expansion/budget/tokenizer/privacy/renderer profiles
Project Map / L0 / L1 / semantic-pack / rendering IDs
selected and omitted item counts by tier/reason
exact byte and token-estimation accounting
evidence/coverage/conflict/NotEvaluated closure
source excerpt/privacy/license transformations
truncation/continuation/no-new-evidence/cancellation state
determinism/cache/security tests
known E4/E5/E6/runtime/application deferrals
```
