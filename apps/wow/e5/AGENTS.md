# AGENTS.md — `apps/wow` E5-B

Implement strict command parsing, explicit bounded input, one service call, output framing, cancellation, and exit mapping only.

- Only framework dependency: `wow-service`.
- Unknown commands/options/config fields fail.
- Never select by score, chronology, position, display name, repository, or uniqueness.
- At most one stdin consumer.
- No cwd/home/environment/Git/editor/WoW/network discovery.
- No include, interpolation, shell expansion, plugin, script, archive extraction, or execution.
- Do not accept private keys, bearer tokens, vault secrets, or signing material in ordinary flags/config/fixtures.
- GitHub/OS/CLI/file/commit identity is not review or holdout authorization.
- Default output never exposes holdout membership, hidden labels/source, confidential notes, credentials, or vault handles.
- `promotion prepare` creates only a submission; no publish/activate/canary/rollout/rollback command exists.
- JSON output is exact service bytes plus one LF; artifact output is exact eligible bytes; text preserves blockers, authorization, consumption, `NotEvaluated`, `OutcomeUnknown`, and nonclaims.
- Broken pipe/output failure never causes a second service call.
- No Cargo/Rust/workflow during documentation phase.