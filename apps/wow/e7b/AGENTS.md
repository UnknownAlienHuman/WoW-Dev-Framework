# AGENTS.md — `apps/wow` E7-B

## Scope

Implement only strict CLI transport for local release identity/verification and explicit update/rollback operations. All release, network, distribution, install, migration, helper and recovery semantics remain in `wow-service` and narrow owner adapters.

## Dependency

```text
apps/wow -> wow-service
```

Any direct build/signing/distribution/store/installer/provider dependency or raw filesystem/network/process helper is an architecture failure.

## Command discipline

- Parse only the frozen command/option grammar.
- Invoke exactly one service operation per valid command.
- Unknown options/config fields fail before service invocation.
- Never select `latest`, newest, previous, highest version, first, sole or same-name release/bundle/channel/install target.
- Pass exact IDs, expected-current digests, signed manifest/bundle paths and operation IDs mechanically.
- `update reconcile` never dispatches a new update/rollback.

## Network discipline

- No network for version/status/local verification/installation validation.
- Update check network access only by explicit command or exact opt-in service policy.
- Check does not download; plan does not apply; apply does not silently enable future automatic updates.
- No telemetry, crash upload, update prefetch, remote config or hidden redirects.

## Security

- Never accept secret keys/tokens/passwords/private endpoints/provider credentials in argv/config.
- Never execute shell/PowerShell/cmd, bundle scripts, arbitrary helper paths or URLs.
- Never extract before service/owner archive validation.
- Never expose local private paths, source/user data, credentials or raw installer/store handles in output/logs.
- Explicit input/output/config paths follow the platform path policy.

## Running executable

The app cannot overwrite itself. For `update apply`/`rollback`, it receives one exact service/installation-owner handoff state and exits according to the selected Windows helper protocol. It does not construct helper commands or choose files.

## Output

- JSON: exact canonical service envelope plus one LF.
- Text: faithful release/update/install/rollback status and nonclaims.
- Artifact: exact eligible public verification artifact only.
- Broken pipe/output failure never repeats service.
- `OutcomeUnknown` remains visible and unsafe to retry.

## Completion report

Report exact command/service operation, network state, release/bundle/manifest/current/LKR IDs, service call count, output/exit, helper handoff/reconciliation, tests, platform profile and every blocked/`NotEvaluated` gate.