# AGENTS.md — `apps/wow` E5-C

- Only framework dependency: `wow-service`.
- Parse strict explicit bounded commands/options/JSON inputs.
- Unknown fields and commands fail before service invocation.
- Pass exact submission/artifact/publication/cohort/plan/current/LKG/rollback/revocation IDs and digest guards mechanically.
- Never select newest, best, previous, highest version, sole, default, or same-name target.
- Never infer authorization from GitHub, OS, terminal, file, repository, or commit identity.
- Never accept private signing keys, KMS/HSM/vault credentials, bearer tokens, deployment secrets, private cohort data, or source bodies in ordinary flags/config/fixtures.
- Exactly one service call per valid command.
- No local signing, publication, canary assignment, rollout logic, activation CAS, LKG inference, rollback, reindex, partition deletion, or public distribution.
- JSON output is exact service bytes plus one LF; artifact output is exact eligible bytes; text preserves scoped states, blockers, `OutcomeUnknown`, signatures, canary limitations, LKG qualification, rollback/closure, and nonclaims.
- Broken pipe/output failure never causes a second service call.
- No Cargo/Rust/workflow during documentation phase.