# AGENTS.md — `apps/wow`

These instructions route every `apps/wow` work package.

## Required reading

1. [`../../AGENTS.md`](../../AGENTS.md)
2. [`../../crates/AGENTS.md`](../../crates/AGENTS.md)
3. [`../../crates/wow-service/AGENTS.md`](../../crates/wow-service/AGENTS.md)
4. [`README.md`](README.md)
5. the selected work-package agent file and machine contract.

Routes:

```text
E0-F -> E0_F_AGENTS.md + CONTRACT.json
E3-C -> e3/AGENTS.md + e3/CONTRACT.json
```

## Common dependency rule

The only framework dependency is `wow-service`. Never import lower crates, upstream analyzer types, context types directly, source readers, graph/store/reference/project handles, or application-internal copies of service records.

Request a service seam when necessary; do not bypass it.

## Common request rules

- Parse explicit typed commands/options only.
- Pass `current` as a symbolic service selector; never resolve it.
- Preserve exact IDs/profiles/guards/continuations/artifact bytes.
- Do not search or guess roots.
- Do not read project source, repositories, directories, globs, editor state, WoW installation, SavedVariables, or logs.
- Artifact input for context validate/render is explicit bounded transport data only.
- No plugin/config/script/shell execution.

## Common output rules

- Invoke service once.
- JSON output is exact canonical service JSON plus the defined final newline.
- Direct artifact output is exact validated artifact bytes with no wrapper or newline change.
- Text output derives only from service records and cannot hide partial/truncated/NotEvaluated/conflict/omission/continuation state.
- Parser/startup/artifact transport errors use stderr only.
- No banner/progress/log on canonical stdout.
- No double output/reinvoke after cancellation or broken pipe.

## Security

Never leak source outside validated artifacts, Secret-capable values, private absolute paths, credentials, tokens, private URLs, raw continuation internals, lower handles, or config input paths into semantic output.

## Documentation phase

No Cargo/Rust/workflow/CI/placeholder implementation. Null app/service vectors remain blocking.

## Completion report

```text
work package and command surface
only wow-service dependency
request/selector/root/profile mapping
output modes and exact byte behavior
exit codes and service-state mapping
artifact/config input limits
cancellation/broken-pipe/single-invocation behavior
security and dependency tests
all executed and skipped gates
```
