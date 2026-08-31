# AGENTS.md — `wow-context`

## Current route

The current implementation-ready contract is E3-A:

1. [`e3/README.md`](e3/README.md)
2. [`e3/AGENTS.md`](e3/AGENTS.md)
3. all remaining normative files listed by [`e3/CONTRACT.json`](e3/CONTRACT.json)

The existing crate [`README.md`](README.md) remains the original high-level crate brief. When it conflicts with the more specific E3-A package, the E3-A contract controls the E3-A implementation.

## Dependency boundary

```text
wow-core
wow-project
wow-graph
```

No direct dependency on `wow-store`, `wow-search`, `wow-reference`, `wow-service`, applications, filesystem/network/process/editor/runtime/model SDKs, or transports.

## Required external routing

Before changing patch-sensitive WoW fields, read the current [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) `AGENTS.md`, `INDEX_MINI.md`, and exact task route. Do not copy live patch claims into stable context templates.

## Implementation gate

No Rust/Cargo activation until all E3-A prerequisites, profiles, synthetic and pinned real-addon vectors, tokenizer requirements for exact token budgeting, and SHA-256 fixtures are frozen.
