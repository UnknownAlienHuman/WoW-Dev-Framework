# `apps/wow` E6-B external candidate CLI contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `apps/wow/e6-b/external-candidate-cli`

`apps/wow` is a thin transport over `wow-service`. Its only framework dependency is `wow-service`.

## Command families

```text
wow external status
wow external provider validate
wow external generation validate
wow external query
wow external continue
wow external result get
wow external result list
wow external result validate
wow external explain
wow external artifact build
wow external mapping validate
wow external map
wow external selection validate
wow external select
wow external context
wow external operation get
wow external operation reconcile
wow external cache validate
```

Each valid command constructs one typed request and invokes exactly one E6-B service operation.

## Canonical reading order

1. [`AGENTS.md`](AGENTS.md)
2. [`CLI_COMMANDS.md`](CLI_COMMANDS.md)
3. [`OUTPUT_EXIT_AND_STREAMS.md`](OUTPUT_EXIT_AND_STREAMS.md)
4. [`SECURITY_AND_INPUTS.md`](SECURITY_AND_INPUTS.md)
5. [`TEST_MATRIX.md`](TEST_MATRIX.md)
6. [`CONTRACT.json`](CONTRACT.json) and [`examples/`](examples/README.md)
7. [`../../../crates/wow-service/e6/`](../../../crates/wow-service/e6/README.md)

## Responsibilities

- parse strict bounded command/config/artifact inputs;
- mechanically transport exact IDs, digests, profiles, selectors, and operation identities;
- pass only service-supported symbolic selectors unchanged;
- project signals into typed cancellation;
- invoke service exactly once;
- emit canonical envelope JSON, faithful text, or one exact eligible artifact;
- map statuses to frozen exit codes;
- enforce stdout/stderr, broken-pipe, and atomic file-output behavior.

## Forbidden responsibilities

- opening provider sessions or credential stores;
- accepting raw provider credentials, private endpoints, session handles, or cursor bytes;
- generic MCP/tool invocation;
- resolving provider/current/catalog state locally;
- querying `wow-cbm` or owner crates directly;
- following provider paths/URLs;
- mapping by name/path/snippet;
- selecting top/first/best/highest-score/sole candidates;
- building context directly;
- inserting provider prose/rank/score into framework facts;
- provider install/start/stop/configure/index/delete;
- source edit, tool execution, runtime validation, or background retry.

## Output modes

```text
envelope-json
text
artifact
```

Machine output never hides Candidate authority, partial/conflict/truncation, external-state class, `OutcomeUnknown`, mapping ambiguity, selection origin, privacy/license omission, or mandatory nonclaims.

## Current state

```text
documentation frontier: E6-B
implementation frontier: not-started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```
