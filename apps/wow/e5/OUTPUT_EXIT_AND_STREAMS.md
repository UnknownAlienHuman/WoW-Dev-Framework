# E5-B CLI output and exit codes

**Status:** normative.

Output modes:

- `envelope-json`: exact canonical service envelope plus one LF; no banner/progress on stdout.
- `text`: faithful projection preserving exact IDs, blockers, authorization, holdout disclosure/consumption, partial/conflict/truncated/`NotEvaluated`/`OutcomeUnknown`, retention/audit/recovery, and promotion nonclaims.
- `artifact`: exactly one service-returned eligible artifact without wrapper or newline modification.

Exit codes:

```text
0   Complete, NoChange, validation Valid, authorized recorded review, or unblocked prepared/validated submission
1   completed validation Invalid or checked unauthorized/rejected decision where the command profile declares it
2   Partial, CandidateOnly, Blocked, ConflictBlocked, Truncated, NotEvaluated
3   structured request/selector/artifact/run/review/holdout/submission/retention/authorization/domain failure
4   OutcomeUnknown or internal owner/service/closure/serialization/post-service output failure
64  CLI/config/input/output-path failure before service invocation
130 Cancelled
```

stdout contains requested output only; stderr contains bounded diagnostics. Broken pipe stops promptly, never reinvokes service, and never claims successful delivery. File output uses explicit path validation, staging, bounded write, required durability, atomic replacement where supported, and optional final digest verification.