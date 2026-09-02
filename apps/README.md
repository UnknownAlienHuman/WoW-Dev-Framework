# Applications and frontend hosts

Applications are thin transports and clients over `wow-service`. They never reimplement reference, analyzer, project, graph, recognizer, diagnostics, search, context, external-provider, calibration, publication, release, installation, update, or storage semantics.

## Public product — `apps/wow`

Read [`wow/README.md`](wow/README.md).

Final planned public executable:

```text
wow
    one-shot CLI
    explicit local daemon
    standalone LSP 3.18
    standalone MCP 2025-11-25
    local release verification and explicit update/rollback client
```

### Contract routes

- E0-F: `status` and `check`.
- E3-C: context commands.
- E4-C: search, lineage, migration validation, and static impact.
- E5-B/C: calibration and governed core-pack lifecycle commands.
- E6-B: optional external candidate query, mapping, selection, and context commands.
- E7-A: session, overlay, local-daemon, LSP, and MCP host.
- E7-B: release identity, bundle verification, installation validation, explicit update, rollback, and reconciliation client.

### Boundary

```text
strict command or protocol input
-> one typed wow-service request
-> exactly one service invocation
-> canonical or reviewed protocol output
```

The app does not resolve current catalogs, select providers, candidates, mappings, releases, installations, or rollback targets; open owner stores or source; build indexes, graphs, or context; authorize effects; access protected credentials; execute repository or bundle code; apply edits; migrate data; overwrite itself; publish releases; or retry unknown effects.

## Reference Pack client — `apps/wow-reference-builder`

Read [`wow-reference-builder/README.md`](wow-reference-builder/README.md).

The client maps exact build, validate, and rebuild-compare commands to `wow-service`. It does not download source, execute arbitrary Lua or repository scripts, mutate editors, sign, publish, activate, install, or update releases.

It is an internal or advanced tool and is not included in the default public `wow` bundle unless an exact administrative artifact and support profile includes it.

## Internal release client

`tools/wow-release` is routed from [`../tools/README.md`](../tools/README.md). It also depends only on `wow-service` and is excluded from the default public bundle.

It maps one strict command to one E7-B release service operation and does not run Cargo or shell directly, access private signing or distribution credentials, call raw provider APIs, create archives, or mutate installations.

## Dependency rule

```text
applications and tools -> wow-service only among framework crates
```

Host libraries may implement arguments, strict framing and serialization, bounded explicit file and stdin access, local IPC, signals, queues, protocol mapping, and output. They cannot absorb domain or release policy or expose owner adapters directly.

## E7-A host invariants

- exact immutable operation registry, no reflection or generic operation proxy;
- one semantic request to one service operation;
- missing capabilities are not advertised;
- explicit workspace, project, and profile registration;
- exact versioned project-owned document overlays;
- incremental LSP synchronization with full-document replacement and explicit resynchronization;
- fixed read-only MCP tools and resources by default;
- disconnect is not cancellation and progress is not completion;
- response replay does not reexecute service;
- bounded queues, streams, and multi-client isolation;
- protocol stdout contains protocol frames only;
- no automatic edit application, source mutation, hidden daemon fallback, or default remote listener.

## E7-B client invariants

- a compiled executable, signature, bundle, channel, installation, and update are separate states;
- local verification commands are network-free;
- update check is explicit and never downloads or installs;
- plan, apply, rollback, and reconcile are separate operations;
- no latest, newest, previous, force, ignore-signature, or skip-backup shortcut;
- no app-side extraction, installer script, helper command, self-overwrite, store migration, or backup deletion;
- `OutcomeUnknown` is visible, unsafe to retry, and reconciled by exact operation identity;
- no startup update check, telemetry, crash upload, or remote configuration.

## Common prohibitions

- no lower framework dependency;
- no hidden current, latest, default, project, profile, provider, release, installation, or rollback selection;
- no arbitrary shell, process, network, repository, tool, plugin, model, SQL, or RPC execution;
- no implicit cwd, home, environment, Git, editor, WoW, provider, or installation discovery;
- no source, project, store, configuration, or editor mutation outside exact owner operations;
- no provider, database, signing, build, distribution, or installer credential surface;
- no automatic candidate promotion or authority upgrade;
- no semantic difference caused by terminal or transport;
- no empty success for deferred operations;
- no blind retry after `OutcomeUnknown`;
- no CI or release workflow before exact implemented commands and evidence.

## Current state

```text
planned application documentation: complete through E7-B
implementation: not started
next repository package: I0-A / wow-core E0-A
first application work: I0-F after I0-A through I0-E
Cargo packages and Rust source: absent
supported public release: none
```
