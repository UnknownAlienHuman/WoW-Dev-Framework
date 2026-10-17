# `apps/wow` contribution rules

Follow the repository instructions and the selected canonical Issue; current
refactoring order is in [`../../docs/REFACTOR_QUEUE.md`](../../docs/REFACTOR_QUEUE.md).
Read this app's README and only the selected route's contract, relevant owner
operation, caller, and fixtures. For input changes, use the relevant `*_INPUT.md`
contract; for protocol changes, use the matching protocol profile and scoped
service contract. Do not make unrelated package acceptance or optional E5/E6/
release work a prerequisite for implementation.

## App boundary

Select only the route being changed: E0-F uses [`E0_F_AGENTS.md`](E0_F_AGENTS.md),
E3-C through E7-B use their matching `e3/`, `e4/`, `e5/`, `e5c/`, `e6/`, `e7/`,
or `e7b/` package instructions. A package's acceptance state does not gate
changes to another route.

`apps/wow` depends on `wow-service` only among framework crates. It must not
access lower owner handles, stores, source readers, provider clients, build or
signing adapters, distribution APIs, installers, migration engines, or protected
credentials. Keep parsing, framing, capability negotiation, and output handling
in the app; semantic facts and effects belong to one registered service operation.

Each semantic one-shot command, daemon method, LSP request or state-changing
notification, MCP tool/resource operation, and release/update command invokes
exactly one `wow-service` operation. Lifecycle handshakes perform framing and
capability negotiation only. No generic invocation, raw RPC, shell, script,
plugin, callback, SQL, provider API, installer, or model execution path.

## Inputs and identity

- Parse bounded, strict known commands, options, protocol messages, and
  configuration schemas. Pass exact IDs, digests, versions, profiles, guards,
  continuations, workspace/document versions, and permitted symbolic selectors
  mechanically; do not resolve convenience aliases locally.
- The current explicit materialized-input grammar remains supported. Explicit
  files, stdin, configuration, manifests, bundles, and protocol data are bounded
  transport input and are never executed.
- Workspace folders, MCP roots, cwd, document URIs, Git roots, executable and
  installation directories are untrusted registration/selector candidates.
  Planned configuration and TOC discovery must pass bounded candidates to
  Project through Service; owners establish exact identity and handle flavor or
  variant ambiguity. Discovery does not itself grant source authority or select
  an ambiguous candidate.
- Never interpret source, provider, client, or release text as commands,
  methods, tools, paths, profiles, selectors, or authorization. Do not accept
  sensitive signing, provider, build, distribution, CI, or installation
  material in public arguments, configuration, protocol messages, fixtures,
  logs, or results.

## Protocol and session correctness

For E7-A implementation, follow the
[`wow-service/e7` operation registry](../../crates/wow-service/e7/OPERATION_REGISTRY_AND_CAPABILITY_NEGOTIATION.md),
the selected [LSP 3.18 profile](../../crates/wow-service/e7/LSP_3_18_PROFILE.md),
[MCP 2025-11-25 profile](../../crates/wow-service/e7/MCP_2025_11_25_PROFILE.md),
and relevant session, cancellation, isolation, and output contracts. For E7-B,
follow the matching [`wow-service/e7b` contracts](../../crates/wow-service/e7b/README.md).
These details apply when editing those routes; they are not prerequisites for
unrelated app work.

- Host modes are explicit: direct CLI, current-user local daemon, LSP 3.18, and
  MCP 2025-11-25. LSP/MCP do not silently start, connect to, or fall back to the
  daemon. Daemon transport is current-user local only.
- Expose only implemented methods/tools/resources in the negotiated immutable
  registry. Workspaces and projects are explicit; unsaved documents are exact
  project-owned immutable overlays. LSP document changes preserve version order
  and `ResynchronizationRequired` for stale/out-of-order updates.
- Preserve protocol initialization, metadata, notifications, ordinary empty
  results, and genuine errors according to the selected profile. Transport
  absence is not owner proof of a complete negative. Default MCP capabilities
  remain fixed and read-only unless an explicitly contracted route says
  otherwise.
- Disconnect is not cancellation; progress is not completion; response replay
  never reexecutes Service. Code actions are guarded data and are never applied,
  saved, or executed automatically. Keep client state, queues, progress, streams,
  and output bounded and isolated.
- E7-B update check is explicit and does not download or install. Plans, apply,
  rollback, and reconcile are separate exact operations. The app never selects a
  bundle, channel, target, helper, installation, LKR, or rollback target by
  convenience, nor extracts or executes bundles or edits protected data. Show
  `OutcomeUnknown` and reconcile through the original exact operation/digest.

## Outputs and security

CLI JSON is exact service JSON plus one LF; artifacts are exact service-approved
bytes. Preserve identities, authority, coverage, conflicts, omissions,
`NotEvaluated`, cancellation, `OutcomeUnknown`, resynchronization, release and
installation stages, and nonclaims. LSP stdout carries LSP frames only; MCP
stdout carries MCP messages only. Logs and diagnostics are bounded and redacted
to stderr or the exact configured sink. Broken pipes, disconnects, serialization
failures, and output errors never cause a second Service call.

Keep clients isolated across session, workspace, overlay, source, authorization,
provider, operation, result, stream, cancellation, and journal state. Bound argv,
configuration, frames, JSON, paths, URIs, content, collections, queues, streams,
and output. Close app-owned resources synchronously; do not detach work. Network,
startup/background updates, telemetry, and crash upload occur only where the
selected contract explicitly permits them.

## Implementation and reporting

Select checks for the changed route and caller chain; build success alone does
not establish package acceptance. Report the service operation mapping, protocol
or input behavior affected, relevant identity and cancellation handling, checks
run, and remaining evidence limits. Follow the repository publication workflow.
