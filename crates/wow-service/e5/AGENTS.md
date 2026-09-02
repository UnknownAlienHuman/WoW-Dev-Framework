# AGENTS.md — `wow-service` E5-B

## Scope

Implement E5-B transport-independent orchestration only. Do not implement E5-A recognizer algorithms or E5-C publication/activation.

## Required reading

1. repository and crate `AGENTS.md` files;
2. `crates/MANIFEST.json`, `crates/DEPENDENCY_GRAPH.md`, and `crates/WORKSTREAMS.md`;
3. this entire E5-B package;
4. `crates/wow-recognizers/e5/`;
5. relevant `wow-store`, `wow-project`, and `wow-graph` owner seams;
6. current external WoW engineering KB routes when patch-sensitive facts are involved.

## Owner discipline

- Call E5-A operations; never reproduce their logic.
- Call graph validation through its public port; never inspect graph tables.
- Acquire project/fact publications through exact retained owner views.
- Use registered `wow-store` operations; no raw SQL or connection handles.
- Missing owner capability is `NotEvaluated` or failure, not a local workaround.

## Selector discipline

- Accept exact artifact IDs or narrowly defined deterministic catalog selectors.
- Resolve symbolic current only where the service contract explicitly permits it, exactly once.
- Never select newest, first, last, highest-metric, sole candidate, or same-name artifact.
- Replace symbolic selectors with exact IDs before canonical owner calls.
- Bind continuation and retry to the original exact identities and cumulative budgets.

## Run discipline

- Every effecting operation uses `OperationId + CanonicalRequestDigest`.
- Same ID/same digest resumes or returns the recorded result.
- Same ID/different digest is rejected.
- `OutcomeUnknown` blocks repeated owner effects until reconciled.
- Response-loss recovery returns the recorded receipt; it does not rerun evaluation or create a second artifact.
- Cancellation produces no completed run or promotion submission.

## Review discipline

- Reviewer authorization is supplied by a narrow port and verified at use time.
- GitHub login, repository owner, OS user, CLI operator, file owner, or commit author is not authorization.
- Authorization cannot change labels, metrics, graph validity, confidence, proof ceilings, or candidate bytes.
- Review records are immutable; supersession references exact prior decisions.
- Bounded notes are untrusted data, not evidence.

## Holdout discipline

- Never expose sealed membership or labels before authorized access.
- Freeze candidate/run/evaluator/profile identities before requesting access.
- Holdout access authorization is separate from reviewer authorization.
- Record denied, expired, revoked, replayed, cancelled, partial, and failed access attempts.
- Mark holdout generation consumed when disclosed results can influence a descendant candidate under the contamination profile.
- Never describe a consumed holdout as untouched.

## Promotion discipline

- Build a submission only from exact immutable artifacts.
- Preserve every blocker, conflict, partial state, hard-gate failure, license/privacy restriction, and nonclaim.
- `Prepared` or `ReadyForE5CReview` is not publication or activation.
- No service operation edits the pack, source, labels, or split to make a submission pass.

## Lifecycle discipline

- Acquire in the frozen order and close in reverse order.
- No public success before mandatory closure and durable receipt publication.
- No detached/background task after return.
- Never leak raw credentials, signatures, holdout labels, private paths, source bodies, or lower owner handles.

## Completion report

```text
operation and exact request digest
resolved artifact/publication/run/submission IDs
owner ports and exact calls
review and holdout authorization states
durable/idempotency/response-loss state
coverage/conflicts/blockers/nonclaims
retention/audit/closure result
commands/tests and pass/fail/skipped/NotEvaluated
E5-C publication work explicitly deferred
```
