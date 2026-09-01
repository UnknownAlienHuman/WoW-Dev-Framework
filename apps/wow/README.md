# `apps/wow` contract router

**Status:** E0-F diagnostic CLI and E3-C context CLI contracts are implementation-ready documentation; no Rust code exists.

`apps/wow` is a thin command-line adapter over `wow-service`. Its only framework dependency is `wow-service`.

## Contract routes

### E0-F — status and check

The original CLI overview is preserved as [`E0_F_CLI_OVERVIEW.md`](E0_F_CLI_OVERVIEW.md). Original E0 agent instructions are preserved as [`E0_F_AGENTS.md`](E0_F_AGENTS.md). The root [`CONTRACT.json`](CONTRACT.json) remains the E0-F machine contract.

Active E0 commands:

```text
wow status
wow check
```

### E3-C — context operations

Read [`e3/README.md`](e3/README.md) and [`e3/CONTRACT.json`](e3/CONTRACT.json).

Active E3-C commands:

```text
wow context status
wow context map
wow context inspect
wow context build
wow context continue
wow context validate
wow context render
```

Search/fuzzy/natural-language root resolution is not active. Context roots are exact typed IDs.

## Dependency boundary

```text
apps/wow -> wow-service
```

Forbidden direct dependencies include every lower framework crate, including `wow-context`, `wow-store`, `wow-project`, `wow-graph`, and `wow-reference`.

## Application ownership

- parse bounded CLI/config/artifact transport input;
- construct typed service requests;
- pass symbolic current selectors unchanged to service;
- project signals to cancellation;
- invoke service exactly once;
- emit canonical envelope JSON, faithful text, or one exact validated artifact;
- map service outcomes to frozen command-specific exit codes;
- enforce stdout/stderr and broken-pipe behavior.

The app does not resolve current, acquire views, inspect source, perform search, interpret evidence, build maps/skeletons/packs, render context, retry on another generation, or authorize tools/edits.

## Current implementation state

```text
documentation frontier: E3-C
implementation frontier: not started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```
