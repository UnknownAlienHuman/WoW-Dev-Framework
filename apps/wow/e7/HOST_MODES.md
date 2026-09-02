# E7-A `wow` host modes

**Status:** normative.

## Single executable

All E7-A modes ship from one exact `wow` build and compatibility manifest. There are no separately versioned LSP/MCP/daemon binaries in the baseline distribution.

```text
wow <command>              one-shot CLI
wow daemon serve           local multi-client daemon
wow lsp serve --stdio      single-client LSP server
wow mcp serve --stdio      single-client MCP server
```

This is packaging unification, not semantic coupling. Each mode has a separate transport profile and static mapping table.

## Common startup

Every mode validates before accepting semantic requests:

```text
exact executable/build manifest
service operation registry and schema digests
explicit public host configuration
explicit data-root/store profile
selected transport/exposure/privacy/resource profiles
required owner/store recovery state
compatibility with existing store schemas and retained artifacts
```

Missing, corrupt, incompatible, or unimplemented required state blocks startup or narrows advertised capabilities. Startup never repairs or migrates silently.

## One-shot CLI

The CLI:

- parses one explicit command and strict inputs;
- constructs one service request;
- invokes service exactly once;
- writes canonical envelope JSON, faithful text, or one exact artifact;
- maps exact status to a frozen exit code;
- closes all process-owned resources before exit.

It does not auto-connect to or auto-start the daemon. A later explicit `--transport local-daemon` client profile would be a separately documented mode and cannot silently fall back to direct execution.

## Local daemon

`wow daemon serve` hosts one service runtime and multiple isolated local sessions. It:

- opens one explicit local named pipe or Unix domain socket;
- validates peer transport access;
- performs exact compatibility handshake;
- maintains bounded per-client sessions, queues and streams;
- persists only the session/request state permitted by the daemon profile;
- supports exact reconnect and operation reconciliation;
- exposes no network listener or generic operation.

Daemon lifetime does not create a floating semantic current. Every request captures exact session/project/profile/overlay generations.

## LSP stdio

`wow lsp serve --stdio` hosts one service runtime for one LSP client over stdin/stdout. It:

- performs protocol initialization and advertises exact capability intersection;
- opens one ephemeral stdio session;
- handles workspace/project registration explicitly;
- maps supported document synchronization and editor requests to one service operation each;
- emits protocol diagnostics/results/progress without semantic upgrades;
- closes the session and process resources on shutdown/exit or transport loss according to the profile.

It does not listen on a socket or connect to the local daemon.

## MCP stdio

`wow mcp serve --stdio` hosts one service runtime for one MCP client over stdin/stdout. It:

- performs exact protocol initialization/capability negotiation;
- exposes a static tool/resource registry derived from the service operation registry;
- opens one ephemeral stdio session;
- maps each tool call to one service operation;
- serves only exact retained resources permitted by policy;
- does not request sampling/elicitation or call client tools/models;
- closes synchronously on shutdown or transport loss.

It does not connect to the daemon or discover provider/MCP tools.

## Configuration separation

The public host configuration selects paths, profiles, limits, logging and allowed exposures. Sensitive provider/signing/release adapter material remains in external protected configuration owned by those adapters; E7-A host configuration contains references only.

No mode reads implicit cwd/home/environment/Git/editor/WoW/provider state. The executable path or launch directory does not define the project.

## Feature exposure

A build may include implementations beyond the selected exposure profile. Exposure is the intersection of:

```text
implemented service registry
transport mapping table
build/release compatibility manifest
host configuration
client negotiation
service authorization and session scope
```

No layer may widen a lower restriction.

## Process exits

One-shot CLI uses frozen exit codes. Long-running modes use process exit only for host lifecycle, not individual semantic outcomes.

Graceful shutdown exits zero only after request admission stops, required effects are reconciled, sessions/streams close, and store/owner resources close. Protocol/config/startup/corruption/internal/forced-shutdown states use explicit nonzero categories frozen by the release profile.

## No hidden mode transition

The host never:

- starts the daemon because LSP/MCP failed;
- switches from daemon to in-process service;
- changes project/profile because a session expired;
- retries an operation through another mode;
- reconnects to another endpoint;
- enables admin tools because the client requested them;
- exposes a remote listener from a local profile.

A different mode/profile requires a new explicit invocation and compatibility check.