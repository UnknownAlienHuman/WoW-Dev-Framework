## Task and actual state

<!-- Queue ID, concrete defect/gap, audited base and branch. Pick the actual state:
specified | coding | code-written-unverified | focused-checks-passed |
reviewed/merged | package-accepted. Specification-only and unverified PRs stay draft.
A task document, published commit or unrelated green CI is not implementation proof. -->

## What to implement

<!-- Observable deliverable; existing behavior to preserve; explicit exclusions.
Name what already exists so the next agent does not recreate it. -->

## How and ownership

<!-- Concrete file/module/symbol boundaries, implementation sequence, owner seams,
source acquisition, failure/cancellation/budget behavior and compatibility plan.
Avoid generic instructions such as 'implement correctly' without a path. -->

## Exact normative sections

<!-- Link each requirement to an existing document #section, preferably at the
reviewed revision. Include relevant contract IDs/ADRs and explain any proposed
change. Recheck current code and moving upstream sources during implementation. -->

## Dependencies and parallel work

<!-- Existing PR links; distinguish integration prerequisites, conditional profile
requirements and acceptance gates. Name shared-file ownership conflicts.
Do not gate independent functional coding on broad fixture expansion. -->

## Acceptance criteria

<!-- Replace with concrete observable positive, negative and compatibility cases.
Write functional code first, then build and run focused checks. Preserve existing
tests; full package/launch acceptance is a separate explicitly evidenced state. -->

- [ ] The intended real owner/application path executes, not a stub or fake adapter.
- [ ] Exact identities, evidence, coverage, conflicts and negative authority are preserved.
- [ ] Invalid/stale/partial input and cancellation/budgets have specified safe outcomes.
- [ ] Public format/storage/profile compatibility and migration are accounted for.
- [ ] Required focused checks and review have actual head/platform evidence.

## Verification ledger

```text
commit / source-profile / platform:
command — PASS | FAIL | NOT-RUN | NotEvaluated
retained evidence / exact failure or unavailable prerequisite:
```

<!-- Never tick a criterion from a test name, written test, successful write or
source comment alone. Missing tooling/runtime/credentials is not a pass. -->

## Remaining work and unclosed gates

<!-- Exact unsupported capabilities, unfinished code, deferred lanes and package/
launch gates. Link concrete follow-up PRs when splitting a task; do not close a
parent with hidden unfinished implementation. Update docs/WORK_QUEUE.md and JSON
from observed PR/code/check state, without auto-blessing fixtures. -->

## Publication and safety

- [ ] The diff reuses existing owners and preserves architecture or includes a reviewed ADR.
- [ ] No private source/provenance/credentials, unlicensed data or unauthorized effects are added.
- [ ] Remote branch and changed blobs are read back; no force-push or overwritten contributor work.
- [ ] The PR is not represented as feature-complete while it contains only a specification.
