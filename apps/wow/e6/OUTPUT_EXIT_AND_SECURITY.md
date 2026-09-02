# E6-B CLI output, exit codes, and security

**Status:** normative.

## Output modes

- `envelope-json`: exact canonical service bytes plus one LF.
- `text`: faithful bounded projection preserving external/local lane states, Candidate authority, zero/coverage/loss, mapping, selection, context sidecar separation, blockers, reconciliation, and nonclaims.
- `artifact`: one exact service-approved result set, candidate artifact, mapping record, selection receipt, context sidecar, combined manifest, or validation report without wrapper/newline mutation.

Text must not call a provider result verified source, a score confidence, zero result absence, mapping semantic proof, selection acceptance, or combined context validation of provider interpretation.

## Exit codes

```text
0   Complete, NoChange, CandidateOnly, ExactMapped, SelectionRecorded, ContextReady,
    or completed validation/decision with the requested effect recorded
1   completed Invalid validation or explicit authorization denial where declared
2   Partial, Truncated, Blocked, ConflictBlocked, NotEvaluated
3   structured configuration/query/result/mapping/selection/context/domain failure
4   OutcomeUnknown or internal owner/service/closure/serialization/post-service output failure
64  CLI/config/input/output-path failure before service invocation
130 Cancelled
```

A zero-candidate result can exit 0 only when the service completed the exact query and output states explicitly that negative authority is unavailable. A partial/truncated zero exits 2.

`SelectionRecorded` exits 0 for `Selected`, `Rejected`, or `Deferred` because the requested recording effect completed; the decision remains explicit in payload/text.

## Security

Only explicit config/input/output paths are accessed. The CLI does not discover provider processes, endpoints, tools, sessions, credential stores, local repositories, editor/client state, or source roots.

Sensitive material, private endpoints, unrestricted provider cursor bytes, provider database paths, process/client handles, private source, and confidential notes are rejected or redacted according to the service contract.

stdout contains requested output only. stderr contains bounded redacted diagnostics. Broken pipe or output failure never redispatches the service operation. Explicit file output uses path policy, staging, bounded write, durability/atomic replacement where required, and final digest verification.