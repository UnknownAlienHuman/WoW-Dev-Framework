# `apps/wow` E3-C context CLI contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `apps/wow/e3-c/context-cli`

## Mission

Expose the E3-C `wow-service` context operations through one bounded deterministic command-line interface without reproducing orchestration or domain logic.

```text
CLI/config/artifact transport input
-> strict parse and local transport validation
-> one typed wow-service request
-> one service invocation
-> one service result
-> envelope JSON, faithful text, or exact validated artifact output
-> frozen command-specific exit code
```

## Commands

```text
wow context status
wow context map
wow context inspect
wow context build
wow context continue
wow context validate
wow context render
```

Exact syntax is in [`CLI_COMMANDS.md`](CLI_COMMANDS.md).

## Only framework dependency

```text
wow-service
```

No direct dependency on `wow-context` or any other framework crate. Public service types may re-export opaque artifact/request payload types without granting app access to owner internals.

## Output modes

```text
envelope-json   canonical service result JSON plus one final newline
text            noncanonical faithful summary from service records only
artifact        exact bytes of one validated RenderedContextArtifact
```

`artifact` is allowed only for operations returning exactly one eligible artifact. The app does not render it.

## Current selection

The app can parse:

```text
current
store-generation:<ID>
publication-set:<ID>
```

It passes the typed selector to service. It never reads current records, store files, or project registries.

## Roots

Inspect/build roots use a closed token:

```text
<RootKind>@<base64url-no-pad(canonical UTF-8 ID bytes)>
```

The app validates grammar/size and passes the decoded exact kind/ID. It does not inspect meaning, search by name, resolve paths, or rank candidates.

## Artifact input

Validate/render may read one explicitly named file or `-` for stdin under a hard transport byte limit. The host path/stdin marker is not included in the service semantic request. Bytes are never executed.

## Required reading

1. [`AGENTS.md`](AGENTS.md)
2. [`CLI_COMMANDS.md`](CLI_COMMANDS.md)
3. [`OUTPUT_EXIT_AND_STREAMS.md`](OUTPUT_EXIT_AND_STREAMS.md)
4. [`SECURITY_AND_CONFIG.md`](SECURITY_AND_CONFIG.md)
5. [`TEST_MATRIX.md`](TEST_MATRIX.md)
6. [`CONTRACT.json`](CONTRACT.json) and [`examples/`](examples/README.md)
7. [`../../../crates/wow-service/e3/README.md`](../../../crates/wow-service/e3/README.md)
8. [`../../../crates/wow-service/e3/APPLICATION_BOUNDARY.md`](../../../crates/wow-service/e3/APPLICATION_BOUNDARY.md)

## Hard boundaries

- only `wow-service` dependency;
- no current resolution or owner acquisition;
- no lower/domain/context rendering logic;
- no fuzzy/name/path/natural-language roots;
- no implicit source/config/repository/editor/client discovery;
- no automatic service retry;
- no source/project mutation or tool authorization;
- no daemon/LSP/MCP/network transport;
- no background work;
- no Cargo/Rust/CI during documentation phase.

## Completion gate

E3-C CLI is complete only when every command maps exactly to one service request, all selector/root/profile/artifact forms are strict and bounded, only service is imported, JSON/artifact bytes and exit codes match frozen vectors, text preserves required state, cancellation/broken pipe never produce a second invocation/output, and every security/config/dependency mutation test passes.
