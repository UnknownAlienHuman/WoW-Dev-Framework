# `apps/wow` E6-B external-candidate CLI

**Status:** implementation-ready documentation; no Rust code exists.

**Contract ID:** `apps/wow/e6-b/external-candidate-cli`

The application is a thin transport adapter over `wow-service`; it imports no other framework crate.

## Commands

```text
wow external-candidate status
wow external-candidate provider validate
wow external-candidate query submit|get|list|cancel|continue
wow external-candidate operation reconcile
wow external-candidate result validate|explain|compare
wow external-candidate artifact build|get
wow external-candidate mapping resolve|get
wow external-candidate selection record|get
wow external-candidate context build|continue
wow external-candidate cache validate
```

Each valid command maps to exactly one E6-B service operation.

The CLI does not discover providers/tools, start processes, read credentials, construct arbitrary MCP requests, query databases, open provider paths, map source, select candidates, build context, invoke lower crates, or retry an unknown effect locally.

## Authority wording

Machine and text output preserve:

```text
semantic_candidate + Candidate
zero result without negative authority
UnverifiedProviderLocator until exact owner mapping
mapping validates locator identity only
selection is explicit and not verification
provider sidecar is separate from exact context
provider failure is lane-local
OutcomeUnknown is unsafe to retry
```

Read `AGENTS.md`, `CLI_COMMANDS.md`, `OUTPUT_EXIT_AND_SECURITY.md`, `TEST_MATRIX.md`, `CONTRACT.json`, and `examples/`.