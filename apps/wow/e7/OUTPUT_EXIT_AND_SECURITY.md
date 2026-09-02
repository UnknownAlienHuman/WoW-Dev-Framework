# E7-A host output, process exits, configuration, and security

**Status:** normative.

## One-shot CLI output

Existing output modes remain:

```text
envelope-json
text
artifact
```

- `envelope-json`: exact canonical service bytes plus one LF.
- `text`: faithful bounded projection preserving exact identities, authority, coverage, conflicts, omissions, cancellation/reconciliation and nonclaims.
- `artifact`: one exact service-approved artifact without wrapper/newline/content mutation.

stdout contains requested output only. Bounded redacted diagnostics use stderr. Broken pipe or output failure never causes a second service call.

## Long-running protocol stdout

For LSP and MCP stdio modes, stdout contains protocol frames only. No banner, human log, progress line, panic text, shell prompt or unrelated JSON is written to stdout.

Daemon mode writes no semantic result to process stdout unless an explicit machine log profile is selected; client responses use the local endpoint. Human diagnostics go to stderr or an explicit log sink under redaction policy.

## Process exit categories

The exact numeric codes are frozen by the E7-B release profile. E7-A defines semantic categories:

```text
SuccessGracefulClose
ConfigurationOrCompatibilityFailure
EndpointOrProtocolStartupFailure
StoreOrRecoveryFailure
InternalHostFailure
ForcedOrUncleanShutdown
```

Individual daemon/LSP/MCP request failures do not terminate the process unless the host integrity/security profile requires shutdown.

One-shot CLI retains its existing operation-specific exit mappings, including nonzero `OutcomeUnknown`, transport failure, cancellation and broken-pipe categories.

## Protocol result fidelity

A transport adapter may translate a service envelope into protocol-native fields only through a reviewed exact mapping. It must preserve a stable result/envelope reference and cannot:

- change status or proof ceiling;
- drop required partial/conflict/`NotEvaluated`/`OutcomeUnknown` state;
- turn Candidate into exact;
- turn zero into negative authority;
- turn mapping/selection/context inclusion into provider proof;
- hide document/session version;
- call progress completion success;
- retry after output failure.

## Configuration files

Public host configuration is strict versioned JSON or another exact reviewed format selected by the release profile. It may define:

```text
host mode and exposure profile
explicit data/store/log/endpoint paths
project/profile registration requests or exact selectors
session/lease/resource/progress/stream limits
protocol/capability profiles
privacy/license/output policies
nonsecret adapter configuration references
```

It may not contain secret keys/tokens/cookies/passwords, private provider/signing endpoints, arbitrary commands, environment interpolation, includes, scripts, plugins, generic MCP/RPC definitions or provider database paths.

Unknown fields and duplicate keys fail. No implicit config search in cwd/home/environment/registry/editor/Git/WoW locations unless a future exact release profile defines one; the baseline requires `--config` where configuration is needed.

## Sensitive adapter material

Provider, signing, deployment and release credentials remain inside narrow host adapters/OS-protected stores. The app receives only nonsecret references and service authorization/session receipts.

Sensitive values are never accepted in normal argv, public config, MCP arguments, LSP initialization options, daemon messages, fixtures, logs, crash reports or result envelopes.

## Paths and data roots

All explicit paths are validated against the selected platform profile. The host rejects forbidden traversal, symlink/reparse/device/UNC/alternate-stream/cross-root behavior as applicable.

A data root separates:

```text
immutable stores and objects
session/lease/recovery records
optional redacted logs
public configuration
protected adapter state outside public config
```

Source roots remain project-owner records, not app-managed arbitrary filesystem access.

## Logging

Logs use stable build/host/session/request/operation IDs, stages, status and bounded counts. They exclude:

```text
document/source bodies and snippets
raw client/provider payloads
secret material and session capability proofs
private endpoints/paths/account identifiers
owner/store/process handles
confidential review/holdout/cohort data
unbounded stack traces
```

Log level/format never changes semantic behavior or canonical output.

## Crash behavior

Panics/internal faults are caught at the host boundary where safe, converted to bounded internal errors, and trigger session/request recovery records. The host does not claim graceful close or success. Sensitive data is not dumped by default.

A process crash cannot cause automatic semantic replay on restart. The daemon recovers exact durable state and reconciles; stdio clients create a new session and may retrieve/reconcile retained effects only through exact service operations.

## Telemetry and network

Baseline E7-A performs no telemetry, update checks, remote configuration, crash upload or network listening. External provider calls, where an explicitly enabled E6 profile uses them, occur only through the provider adapter and are not transport telemetry.

Any future telemetry/update/network behavior requires an E7-B or later explicit profile with consent, schema, destination, retention, security and failure behavior.

## Security hard stops

```text
no raw secret in public inputs/outputs
no arbitrary shell/script/plugin/model/tool/RPC
no hidden project/profile/provider discovery
no app-side source read/write or owner mapping
no generic method/tool dispatcher
no remote listener in baseline
no cross-session state or output
no protocol text as instructions
no automatic edit application
no blind retry after OutcomeUnknown
no public success before close
```

## Platform tests

Each supported release target must pass argv/config/path/Unicode/stdio/pipe/socket/signals/console/broken-pipe/atomic-file/log/redaction/crash/shutdown behavior under its exact platform profile. An untested platform is not advertised by the compatibility manifest.