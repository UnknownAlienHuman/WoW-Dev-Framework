# `apps/wow` contract router

**Status:** planned public application documentation is complete through E7-B; no Rust code exists.

`apps/wow` is the public product transport and host over `wow-service`. Its only framework dependency is `wow-service`.

## Final product modes

```text
wow <one-shot command>
wow daemon run|status|shutdown
wow lsp --transport stdio
wow mcp --transport stdio
wow mcp --transport streamable-http-local   # explicit, loopback-only, disabled by default
```

The same exact product build also provides local release identity, verification, and explicit update and rollback commands.

## Contract routes

- **E0-F:** root contract — `wow status`, `wow check`.
- **E3-C:** [`e3/README.md`](e3/README.md) — context commands.
- **E4-C:** [`e4/README.md`](e4/README.md) — search, lineage, migration validation, and impact.
- **E5-B:** [`e5/README.md`](e5/README.md) — calibration source, corpus, run, review, holdout, and submission commands.
- **E5-C:** [`e5c/README.md`](e5c/README.md) — core-pack artifact, signing, publication, canary, rollout, activation, LKG, rollback, revocation, and closure.
- **E6-B:** [`e6/README.md`](e6/README.md) — external provider, query, result, mapping, selection, and context commands.
- **E7-A:** [`e7/README.md`](e7/README.md) — one-shot CLI, local daemon, LSP 3.18, MCP 2025-11-25, sessions, overlays, progress, backpressure, reconnect, and isolation.
- **E7-B:** [`e7b/README.md`](e7b/README.md) — release identity and bundle verification plus explicit update, rollback, and reconciliation client operations.

## Application ownership

The app may:

- parse exact strict command, configuration, and protocol input;
- validate transport framing, known fields, and bounded sizes;
- initialize one exact build, registry, protocol, exposure, and resource profile;
- construct one typed service request per semantic command, method, tool, resource, or state-changing notification;
- pass exact IDs, digests, versions, profiles, guards, and continuations mechanically;
- project cancellation and host lifecycle signals;
- emit exact canonical service JSON, faithful text, exact approved artifacts, or reviewed protocol projections;
- host local client connections, queues, progress, and streams within the exact E7-A transport profile;
- close app-owned transport resources.

The app may not:

- resolve current, latest, default, newest, previous, best, first, sole, nearest, or same-name project, profile, provider, candidate, release, installation, or rollback targets;
- inspect lower owner stores, source, graph, analyzer, project, session, release, or installation internals;
- run owner algorithms, map locators, build context, perform release validation, or migrate data locally;
- infer authorization from client, editor, GitHub, OS, CI, repository, tag, file, or process identity;
- access private provider, signing, build, distribution, or installation credentials or endpoints;
- execute shell, PowerShell, cmd, scripts, plugins, models, repository code, bundle scripts, or arbitrary helpers;
- apply or save code actions automatically;
- overwrite its running executable;
- compose multiple service effects into an undocumented hidden workflow;
- retry an `OutcomeUnknown` or repeat a service operation after output failure;
- perform hidden network, update, telemetry, crash-upload, or remote-configuration behavior.

## E7-A host boundary

The canonical contracts are:

```text
wow-service/e7-a/frontend-session-operation-registry
apps/wow/e7-a/frontend-transports
```

- one immutable content-addressed frontend operation registry;
- `wow-local-jsonrpc/1` over current-user Windows named pipe or Unix-domain socket;
- standalone LSP 3.18 and MCP 2025-11-25 modes;
- no hidden daemon auto-start or fallback;
- immutable service sessions and project-owned document overlays;
- incremental LSP `textDocument/didChange`, with full-document change as exact replacement;
- exact document versions, position encodings, and resynchronization state;
- diagnostics, completion, signature help, hover, definitions, references, symbols, call hierarchy, and guarded code actions only when implemented and advertised;
- fixed read-only MCP tools and resources by default, without prompts, sampling, elicitation, tasks, generic tools, provider effects, governance effects, edits, or release operations;
- bounded progress, streams, queues, backpressure, cancellation, disconnect, reconnect, and response replay;
- protocol stdout purity and client, workspace, overlay, authorization, source, result, and journal isolation.

## E7-B update client boundary

Commands:

```text
wow version
wow release status
wow release verify bundle --input <PATH>
wow installation validate
wow update check
wow update plan
wow update apply
wow update rollback
wow update reconcile
```

Local identity, bundle verification, and installation validation do not access the network. `update check` is explicit and does not download or install. Plan, apply, rollback, and reconcile remain separate exact service operations.

The app never selects a bundle, helper, migration, or rollback target by convenience; extracts or executes bundle content; edits store or configuration files; deletes backups or user data; or overwrites its running binary. The exact installation owner and verified Windows helper perform staged replacement and migration according to the frozen service plan.

A helper handoff is not final `Updated`. `OutcomeUnknown` remains nonzero, visible, unsafe to retry, and reconcilable only through the exact existing operation identity.

## Output

```text
one-shot CLI JSON     exact service bytes plus one LF
one-shot text          faithful bounded projection
artifact               exact service-approved bytes
LSP/MCP stdout         protocol frames only
daemon response        exact framed service result
```

Progress is nonsemantic. Broken pipe, disconnect, serialization, or output failure never causes a second service invocation.

## Exposure

A compiled operation is exposed only by the intersection of:

```text
implemented service registry
transport mapping
build compatibility manifest
host exposure profile
client capability negotiation
service authorization and session scope
```

Each layer can narrow but never widen. Default LSP and MCP developer exposure is read-only and excludes E5 governance, E6 provider effects, source mutation, and E7-B release publication or installation administration.

## Current state

```text
planned application documentation: complete through E7-B
implementation frontier: not-started
first application implementation: I0-F status/check after I0-A through I0-E
repository next package: I0-A / wow-core E0-A
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
supported public release: none
```
