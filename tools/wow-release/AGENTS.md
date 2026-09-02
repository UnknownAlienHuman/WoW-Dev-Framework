# AGENTS.md — `tools/wow-release`

## Scope

Implement strict internal CLI transport over E7-B `wow-service` operations only.

## Dependency

```text
wow-release -> wow-service
```

Any direct source/tree, Cargo/rustc, process, signing, GitHub/distribution, installer, store, provider or credential API is an architecture failure.

## Command rules

- Use only the frozen command/option/request grammar.
- Exactly one service invocation per valid command.
- Unknown fields/options fail before service.
- Pass exact source/plan/build/artifact/evidence/signature/bundle/candidate/channel/manifest IDs and operation guards mechanically.
- Never select latest/newest/highest version/tag/branch/first/sole/previous artifacts.
- Never compose build -> evidence -> sign -> bundle -> publish locally. Each command is explicit; a future composite workflow must be one documented service operation.
- `reconcile` never redispatches an effect.

## Execution boundary

- Do not spawn Cargo/rustc/linkers/shell/scripts.
- Do not construct environment blocks, sandbox plans, output paths or upload requests.
- Do not read source trees except explicit bounded transport files declared by the command.
- Do not access private signing/distribution/build credentials or protected endpoints.
- Do not use ambient GitHub/CI/OS identity as authorization.

## Input/output

- Strict bounded JSON and exact explicit paths.
- Maximum one stdin consumer.
- No include/interpolation/environment expansion/plugin/callback/arbitrary URL/API payload.
- JSON output is exact service bytes plus LF.
- Text preserves blockers, signatures, support scope, publication state and `OutcomeUnknown`.
- Broken pipe/output failure never calls service again.

## Lifecycle

Cancellation, timeout, disconnect and response loss preserve exact operation identity and owner effect state. No detached polling/upload/signing/build/cleanup after return.

## Completion report

Report command/service operation, exact input IDs/digests, authorization references, service call count, output/exit, cancellation/reconciliation, tests and every skipped/blocked/`NotEvaluated` release gate.