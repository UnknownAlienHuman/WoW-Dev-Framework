# AGENTS.md — `wow-reference-builder`

- Read repository instructions and `crates/wow-service/e1/` first.
- Depend only on `wow-service` among framework crates.
- Keep domain policy in service.
- Require explicit request/source/output/scratch paths.
- Constrain all path access to validated roots.
- Execute only typed materialization/finalization/probe adapter requests.
- Never run source/repository scripts or generic shell commands.
- Never mutate editor/user/workspace configuration.
- Never download, upload, sign, publish, or activate a release in E1.
- Preserve prior destinations on every failure/cancellation.
- JSON output is authoritative; text is a projection.
- Tests must cover arguments, exit codes, path attacks, cancellation, atomic failure, no-network/no-shell, and direct-dependency prohibition.
