# E7-B public update client security, process handoff, output, and recovery

**Status:** normative.

## Local versus network operations

Local-only:

```text
wow version
wow release status
wow release verify bundle
wow installation validate
wow update reconcile against local durable records
```

Network-capable only when explicitly invoked and admitted by service policy:

```text
wow update check
wow update plan when target materialization is explicitly included
wow update apply when its frozen plan includes materialization
```

The app itself has no generic HTTP client or URL fetch API. Distribution access occurs through a narrow service adapter and signed manifest/provider policy.

## Sensitive data

The update client never accepts or emits:

```text
GitHub/provider tokens
signing/private keys
passwords/cookies
private endpoints
arbitrary headers/URLs
KMS/HSM/vault material
helper command lines
raw store/installer/process handles
user project/source/config secrets
```

Public config contains nonsecret profile/authorization references only. Sensitive adapter state remains outside normal app inputs.

## Bundle input

For local verification, the app passes an explicit bounded path/handle to service. It does not:

- infer a bundle from Downloads/cwd/executable directory;
- extract or list unsafe archive members itself;
- trust extension/MIME/filename;
- execute scripts or binaries;
- follow manifest links;
- repair/repack the bundle.

## Process handoff

`update apply`/`rollback` can require the running `wow` process to terminate before executable replacement. The app:

```text
invokes exactly one service operation
receives exact handoff/result state
stops admitting new local host/session work
closes/drains sessions through service profile
flushes requested final output/receipt
exits with the exact handoff process category
```

It does not construct a helper command, select a helper binary, pass arbitrary paths or restart itself. The installation owner launches/validates the exact helper using the frozen plan.

A successful handoff initiation is not the final `Updated` result. The next invocation or helper-owned receipt must reconcile the installation/self-check state.

## Running modes

An update cannot proceed while the same installation has active daemon/LSP/MCP sessions unless the exact plan successfully drains/closes them or explicitly reports a blocker. The app never kills unrelated processes or treats lost stdio/socket as clean closure.

The daemon may expose status/reconcile but not silently self-update in response to startup or channel discovery.

## Output and exit

JSON is exact canonical service bytes plus one LF. Text preserves:

```text
current and target release/bundle/manifest IDs
support/compatibility/signature/revocation state
check/plan/materialize/stage/backup/migration/activation/self-check state
current/LKR/rollback identities
OutcomeUnknown and reconciliation instructions
nonclaims and network behavior
```

Artifact mode emits only exact public verification artifacts approved by service.

Exit categories reuse the service/app frozen mappings:

```text
success/no-change/update-available/planned/exact completed effect
action required/blocked/partial/NotEvaluated
validation invalid/domain failure
OutcomeUnknown/internal/output failure
pre-service CLI/input/path failure
cancelled
```

An update-available result is not an install success. A handoff-in-progress/`OutcomeUnknown` state is nonzero unless the exact platform CLI contract defines a separate successful-handoff process category while preserving final unknown status in the durable receipt.

## Cancellation

Cancellation before any materialization/install effect may be clean. After download/stage/backup/migration/helper/CAS dispatch it records intent and preserves exact effect/reconciliation state. The app never deletes staging/backups or repeats an effect on cancellation.

## Broken pipe/output failure

Failure to display a result does not undo check/download/install/rollback effects. The app never calls service again. Exact `OperationId + CanonicalRequestDigest` permits later `update reconcile`.

## Reconciliation

The app submits exact operation/request/install IDs to `release_operation_reconcile`. It displays one of the known states and required action. It does not guess from executable version, directory contents or process presence and does not convert unknown to failed/succeeded.

## Path security

Explicit bundle/config/output paths follow the Windows release platform profile, including normalization and restrictions for traversal, symlink/reparse points, devices, UNC, alternate data streams, reserved names, case/Unicode collisions and cross-root access.

The app does not own install destination/staging/backup paths; those are installation-owner records.

## Logs

Default app diagnostics expose stable IDs/status/reason codes only. No source, config contents, private paths, tokens, update URLs, raw manifests, helper internals or unbounded stack traces.

## No hidden behavior

```text
no startup update check
no background download/install
no telemetry/crash upload
no remote configuration
no automatic channel change
no implicit downgrade/rollback
no source/data deletion
no retry after OutcomeUnknown
no shell/script/helper execution by app
```

## Platform validation

The Windows target must test console/PowerShell/cmd invocation, spaces/Unicode/long paths, ACLs, antivirus/file locks, running-executable replacement, abrupt process termination, daemon/LSP/MCP drain, output failure, UAC/elevation policy where applicable, reboot/pending replacement policy if supported, and rollback/data recovery.