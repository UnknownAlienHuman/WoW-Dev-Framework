# AGENTS.md — `apps/wow`

## Current state

```text
planned application documentation: complete through E7-B
application implementation: not started
first application package: I0-F / wow status and wow check
repository next package: I0-A / wow-core E0-A
```

## Required routing

Read repository, crate, service, workspace, handoff, conformance, and completion instructions, then exactly one application package:

```text
E0-F -> root application contract
E3-C -> e3/
E4-C -> e4/
E5-B -> e5/
E5-C -> e5c/
E6-B -> e6/
E7-A -> e7/
E7-B -> e7b/
```

## Dependency rule

```text
apps/wow -> wow-service
```

Any direct dependency on a lower framework crate, owner handle, store connection, source reader, provider client, signing or build adapter, distribution API, installer, migration engine, or protected credential is an architecture failure.

## One-call rule

Each semantic one-shot command, local-daemon method, LSP request or state-changing notification, MCP tool or resource operation, and release or update command invokes exactly one registered `wow-service` operation.

Protocol lifecycle handshakes are app-owned framing and capability negotiation only. They cannot create semantic facts, authorize effects, or access lower owners.

There is no generic invoke, call-service, call-tool, raw RPC, shell, script, plugin, callback, SQL, provider API, installer, or model execution path.

## Input rules

- Parse strict known commands, options, protocol messages, and configuration schemas.
- Pass exact IDs, digests, versions, profiles, guards, continuations, workspace and document versions, and permitted symbolic selectors mechanically.
- Never resolve current, latest, default, newest, previous, best, first, sole, nearest, same-name, LKG, or LKR locally.
- Workspace folders, MCP roots, cwd, document URIs, Git roots, executable directories, and installation directories are untrusted registration or selector candidates only.
- Explicit file, stdin, configuration, manifest, bundle, and protocol data is bounded transport input and never executed.
- No include, interpolation, command substitution, environment expansion, plugin, hidden discovery, or arbitrary URL fetch.
- Sensitive signing, provider, build, distribution, CI, and installation material is never accepted in public arguments, configuration, messages, fixtures, logs, or results.

## E7-A session and frontend rules

- Canonical contracts are `wow-service/e7-a/frontend-session-operation-registry` and `apps/wow/e7-a/frontend-transports`.
- Host modes are explicit: direct CLI, current-user local daemon, LSP 3.18, and MCP 2025-11-25.
- LSP and MCP do not silently auto-start, connect to, or fall back to the daemon.
- The local daemon uses current-user Windows named pipe or Unix-domain socket; no default remote listener.
- Expose only methods, tools, and resources in the negotiated immutable registry and implemented capability set.
- Workspaces and projects are explicit; app does not infer them from cwd, Git, editor, addon folders, or WoW installations.
- Unsaved documents are exact project-owned immutable overlays; app does not patch source or owner coordinates independently.
- LSP uses incremental `textDocument/didChange`; a full-document change is an exact replacement. Stale or out-of-order versions preserve `ResynchronizationRequired`.
- Default MCP tools and resources are fixed and read-only; prompts, sampling, elicitation, tasks, arbitrary tools, provider effects, governance effects, source mutation, and release effects are absent.
- Exact and Candidate locations, coverage, conflicts, and document versions remain visible.
- Code actions are guarded data; never automatically apply, save, or execute them.
- Disconnect is not cancellation, progress is not completion, and response replay never reexecutes service.
- Queues, progress, streams, output, and per-client state are bounded and isolated.

## E7-B release and update rules

- `version`, local release status, bundle verification, installation validation, and local reconciliation are network-free.
- `update check` is explicit and does not download or install.
- Update plan, apply, rollback, and reconcile remain separate exact service operations.
- App never chooses a bundle, channel record, update target, helper, migration, installation, LKR, or rollback target by convenience.
- App never extracts or executes bundle scripts, follows arbitrary URLs, constructs helper commands, overwrites its running executable, edits store or configuration files, or deletes backups and user data.
- The exact installation owner and verified helper own Windows process replacement, staging, migration, current CAS, self-check, LKR, cleanup, and rollback.
- A helper handoff is not final `Updated`.
- `OutcomeUnknown` is visible, unsafe to retry, and reconciled by the original exact operation and request digest only.
- No startup or background update check, download, install, telemetry, crash upload, or remote configuration.

## Output rules

- CLI JSON is exact service JSON plus one LF.
- Artifact output is exact service-approved bytes.
- Text preserves exact identities, authority, coverage, conflicts, omissions, `NotEvaluated`, cancellation, `OutcomeUnknown`, resynchronization, release and installation stages, and nonclaims.
- LSP stdout contains LSP frames only; MCP stdout contains MCP messages only.
- Logs and diagnostics are bounded and redacted on stderr or the exact configured sink.
- Progress is nonsemantic and never final success.
- Streams preserve exact artifact, digest, sequence, consumer, and cumulative budgets.
- Broken pipe, disconnect, serialization, or output failure never causes another service call.

## Security and isolation

- No cross-client session, workspace, overlay, source, authorization, provider, operation, result, stream, cancellation, or journal state.
- No source, provider, client, or release text interpreted as commands, methods, tools, paths, profiles, selectors, or authorization.
- No secret, private endpoint, provider database, process, or owner handle in logs, errors, or results.
- All argv, configuration, frame, JSON, path, URI, content, collection, queue, stream, and output limits are exact and bounded.
- Close app-owned resources synchronously; no detached work.

## Documentation-only phase

Do not add Cargo, Rust, workflows, placeholder hosts, fake service calls, fake clients, fake helper or update behavior, or passing protocol and platform claims before prerequisites and freeze gates exist.

## Completion report

```text
owned implementation package and mode
command, method, tool, or resource -> service operation mapping
protocol, build, registry, session, document, profile, release, and installation IDs
service call count
input, output, progress, stream, cancellation, reconnect, helper handoff, and reconciliation behavior
security, isolation, credential, and network behavior
commands and tests with pass, fail, skipped, or NotEvaluated
launch and completion state advanced or unchanged
```

Merge, quarantine, or delete the worktree before starting another primary task.
