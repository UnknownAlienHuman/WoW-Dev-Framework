# E5-C CLI output, exit codes, and security

**Status:** normative.

Output modes:

- `envelope-json`: exact canonical service bytes plus one LF.
- `text`: faithful scoped projection preserving publication/validation/signature/canary/rollout/activation/LKG/rollback/revocation/closure state, blockers, partial/`NotEvaluated`/`OutcomeUnknown`, and nonclaims.
- `artifact`: one exact service-approved artifact, attestation, signature envelope, publication manifest, or report without wrapper/newline mutation.

Text must not call `PublishedInactive` active, a canary globally safe, a rollout future-proof, a signature semantic proof, or a rollback history erasure.

Exit codes:

```text
0   Complete/NoChange, completed Valid validation, or exact successful requested state transition
1   completed Invalid validation or checked authorization denial where declared
2   Partial/PublishedInactive/ValidatedInactive/CanaryOnly/RolloutPaused/Blocked/ConflictBlocked/Truncated/NotEvaluated
3   structured request/selector/artifact/signature/publication/canary/rollout/activation/rollback/revocation/domain failure
4   OutcomeUnknown or internal owner/service/closure/serialization/post-service output failure
64  CLI/config/input/output-path failure before service invocation
130 Cancelled
```

`Active`, `RolledBack`, `Revoked`, and `Deactivated` return 0 only when that exact requested effect and required closure completed. A partial closure maps to 2 or 3 according to the service payload.

Only explicit config/input paths are read. Private signing/authorization/deployment credentials, source bodies, private cohort membership, raw observations, and owner handles are never ordinary CLI input/output. GitHub/OS/CLI identity is not authorization.

stdout contains requested output only; stderr contains bounded redacted diagnostics. Broken pipe/output failure never repeats service. File output uses explicit path policy, staging, bounded write, durability/atomic replace where required, and final digest verification.