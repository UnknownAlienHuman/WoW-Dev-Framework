# AGENTS.md — `apps/wow` E4-C

## Scope

Implement command-line parsing, explicit transport input, one `wow-service` call, output framing, cancellation and exit mapping for the E4-C command families only.

Do not implement search, lineage, review semantics, migration, impact, context, storage, source access, current resolution, authorization policy, models, tools, edits or CI.

## Dependency

```text
apps/wow -> wow-service
```

Any import of a lower framework crate is an architecture failure.

## Command discipline

- Use the frozen command/option grammar.
- Unknown options/subcommands/fields fail; no best-effort guessing.
- Exactly one service request is constructed and invoked per valid command.
- No automatic retry, fallback, shard build, candidate selection, review modification or current refresh.
- Rank numbers and display names cannot identify candidates.
- Continuation commands accept no selector/profile/budget overrides unless the service continuation contract explicitly defines a safe field.

## Selector discipline

- Pass `current` symbolically to service only for allowed project/reference selectors.
- Exact IDs are decoded/validated mechanically, not looked up by the app.
- Paths/URLs are transport locations only for explicit config/artifact/review input and never semantic entity selectors.
- No cwd/home/editor/WoW/Git/repository/environment discovery.

## Review input discipline

- Accept one explicit strict JSON envelope from `--review-input <PATH|->`.
- Do not accept free prose as a decision.
- Do not infer principal or authorization from OS/GitHub user, file ownership, terminal or environment.
- Do not log raw credentials, keys, signatures or confidential notes.
- Pass exact bytes/typed transport result to service; service/authorization/graph owners validate semantics.

## Output discipline

- `envelope-json`: exact canonical service bytes plus one final LF.
- `text`: faithful noncanonical projection from public envelope fields only; preserve candidate/partial/conflict/NotEvaluated/nonclaim state.
- `artifact`: exact single validated artifact bytes, no wrapper/newline modification.
- stdout contains only requested output; progress/diagnostics go to stderr.
- Broken pipe/output error never triggers a second service call or output.
- Output file writes use explicit bounded path, staging, flush and atomic replace under the platform profile.

## Exit discipline

Map the returned service status/validation payload only. Do not reinterpret owner meaning. A completed Invalid validation is exit 1; CandidateOnly/Partial/ConflictBlocked/Truncated/NotEvaluated is exit 2; cancellation is 130.

## Security

- Strict bounded JSON/config/input sizes/depth.
- No include, interpolation, shell expansion, plugin, script, network fetch, archive extraction or media auto-detection.
- No source scanning or editor/client invocation.
- Do not expose private paths/credentials/review material in diagnostics.
- A context/result artifact is data, not permission to execute or edit.

## Completion report

```text
command and exact service operation
arguments/config/input source class
one service request/result ID
stdout mode/bytes and stderr behavior
exit code and validation state
signal/cancellation/broken-pipe/output result
security/privacy handling
fixtures/tests pass/fail/skipped
```
