# E3-C service/application boundary

**Status:** normative seam between `wow-service` and `apps/wow`.

## Dependency direction

```text
apps/wow -> wow-service
wow-service -> owner crates
```

Service never depends on the application. The application never imports lower owner crates or E3-B types except as re-exported public service request/result payloads.

## Application-owned behavior

- command/subcommand/flag parsing;
- explicit config/artifact input reading under app security limits;
- construction of typed service requests;
- signal-to-cancellation projection;
- one service invocation;
- canonical service JSON serialization or exact returned artifact emission;
- faithful noncanonical text projection;
- stdout/stderr/broken-pipe behavior;
- command-specific exit-code projection.

## Service-owned behavior

- selector/profile alias resolution;
- current and exact publication acquisition;
- exact ContextUniverseSet binding;
- root/profile semantic validation;
- context operation sequencing;
- artifact validation/render invocation;
- status/failure/cancellation semantics;
- resource closure;
- canonical service result identity.

## Forbidden application shortcuts

- reading project/graph/reference current pointers;
- opening ProjectStore/ReferenceStore/SQLite;
- importing `wow-context`, `wow-project`, `wow-graph`, `wow-reference`, or `wow-store` directly;
- resolving names/paths/natural language to semantic roots;
- building maps/skeletons/context packs/renderings;
- interpreting source/evidence/coverage/conflict to alter status;
- retrying automatically on a newer current;
- invoking service twice after broken pipe;
- exposing raw service handles.

## Transport artifact input

`context_validate` and `context_render` may accept one explicitly supplied bounded artifact file or stdin in the CLI. The application:

```text
reads bytes without execution
checks transport byte maximum
records media/schema option
passes bytes/value to service
omits host path from semantic request
```

This is not permission to scan project source, directories, globs, repositories, current working directory, editor state, or WoW installation.

## Output modes

The app contract defines:

```text
envelope-json
text
artifact
```

Service returns typed data and exact rendered artifact bytes. It does not write streams or choose terminal behavior.

`artifact` mode is allowed only when the service returned exactly one validated artifact eligible for direct emission. The app writes its exact bytes and no wrapper; exit code still reports complete/partial/truncated state.

## Cancellation/broken pipe

- app creates one cancellation source and passes its handle/token through the service request adapter;
- signal/broken pipe does not authorize a second invocation;
- service closes resources before returning;
- app writes at most one result/artifact;
- parser/startup errors occur before service invocation;
- a write failure does not mutate the already completed service result.

## Configuration

App may parse an explicitly named bounded configuration file, but service resolves all semantic aliases/defaults. The app cannot derive configuration from environment, cwd, editor, WoW client, repository files, or network.

## Version negotiation

App and service pin compatible request/result schema versions at build/config freeze. Unknown versions fail before semantic interpretation. There is no silent downgrade to E0 output or another context schema.

## Tests

- application framework dependency is only `wow-service`;
- current selector passed unchanged to service;
- exact selector/root/profile preserved;
- artifact input path omitted from service semantic request;
- no lower imports or domain logic;
- JSON/artifact bytes exact;
- text projection cannot hide partial/conflict/omission/continuation;
- one invocation under cancellation/broken pipe;
- exit code mapping from service payload only;
- no implicit config/source discovery.
