# `wow-annotations` contract router

**Status:** the Rust Ketho emitter slice is implemented and active in Cargo.
The full E1-C ReferenceView-to-artifact service is not complete.

See [`src/ketho.rs`](src/ketho.rs), [`tests/ketho.rs`](tests/ketho.rs),
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) and the
[Rust port map](../../docs/KETHO_RUST_PORT.md).

This slice ports Ketho's GetType/GetField/GetFunction/GetTable/GetCallbackType/
GetSystem behavior and explicit function/method naming. It has no dependencies,
source ingestion, IO, Python/Lua runtime, editor mutation, or reference-authority
claims. Callers provide ordered declaration data, enum membership and resolved
widget aliases. Unsafe/unrepresentable input is rejected, not widened or dropped.
The complete service still needs the source-bound ReferenceView adapter, loss
sidecars, source maps, artifact publication and real language-server probes.

```text
cargo test -p wow-annotations
cargo clippy -p wow-annotations --all-targets -- -D warnings
```

`wow-annotations` projects one exact read-only `wow-reference` generation into deterministic, analysis-only LuaCATS/Emmy annotation artifacts. It owns semantic projection, type lowering, versioned layout/rendering, source maps, projection-loss reporting, Ketho semantic parity, and consumer compatibility profiles. It does not own platform truth.

## Canonical route

Read the E1-C contract package in this order:

1. [`e1/README.md`](e1/README.md) — scope, authority, dependency, input/output, and completion contract.
2. [`e1/AGENTS.md`](e1/AGENTS.md) — mandatory implementation rules.
3. [`e1/DECISIONS.md`](e1/DECISIONS.md) — accepted annotation-projection decisions.
4. [`e1/DATA_MODEL.md`](e1/DATA_MODEL.md) and [`e1/SEMANTIC_MODEL.md`](e1/SEMANTIC_MODEL.md).
5. [`e1/TYPE_LOWERING.md`](e1/TYPE_LOWERING.md).
6. [`e1/LAYOUT_AND_RENDERING.md`](e1/LAYOUT_AND_RENDERING.md).
7. [`e1/DIALECT_AND_GLOBALS.md`](e1/DIALECT_AND_GLOBALS.md).
8. [`e1/SECURITY_AND_SANITIZATION.md`](e1/SECURITY_AND_SANITIZATION.md).
9. [`e1/SOURCE_MAP_AND_LOSS.md`](e1/SOURCE_MAP_AND_LOSS.md).
10. [`e1/PARITY_AND_CONSUMER_PROBES.md`](e1/PARITY_AND_CONSUMER_PROBES.md).
11. [`e1/ERROR_MODEL.md`](e1/ERROR_MODEL.md), [`e1/TEST_MATRIX.md`](e1/TEST_MATRIX.md), and [`e1/IMPLEMENTATION_PLAN.md`](e1/IMPLEMENTATION_PLAN.md).
12. [`e1/CONTRACT.json`](e1/CONTRACT.json) and the closed [`e1/examples/`](e1/examples/README.md) fixture package.

Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.

## Direct framework dependencies

```text
wow-core
wow-reference
```

No dependency on `wow-store`, `wow-emmy`, `wow-project`, `wow-service`, applications, editors, analyzers, Ketho, or external processes is permitted in the library crate. External parity and consumer probes use reviewed test/tool adapters.

## Owned responsibilities

- exact ReferenceView/profile/reference-generation input validation;
- consumer-neutral annotation semantic model;
- explicit type and restriction lowering;
- versioned deterministic artifact topology and inert source rendering;
- WoW dialect/global projection without editor mutation;
- safe identifier, literal, and documentation rendering;
- final-byte generated source maps;
- separate reference coverage, projection coverage, and projection-loss records;
- pinned Ketho semantic comparison without authority transfer;
- versioned EmmyLua and LuaLS consumer capability/probe contracts;
- artifact manifests, eligibility, budgets, cancellation, and deterministic checksums.

## Hard boundaries

`wow-annotations` must not:

- parse or acquire Blizzard source;
- execute Lua, generated files, oracle repositories, or addon code;
- correct or replace `wow-reference` facts;
- infer current profiles, aliases, replacements, runtime Secret state, or permanent spell whitelists;
- silently widen unsupported input to `any`;
- collapse optional, nullable, missing, default, tuple, multiple-return, or restriction semantics;
- interpolate source text into directives, code, identifiers, paths, modules, or file topology;
- mutate editor/user/workspace settings, globals, libraries, extensions, or diagnostics;
- include full Blizzard implementation source or runtime addon payloads;
- write SQLite, expose raw SQL, perform network/process/shell access, or publish final releases.

## Current implementation state

The pure emitter is a bounded executable slice authorized by revised ADR-004.
The full E1 examples remain design fixtures and do not certify this slice or
pretend its source maps, artifact manifests or consumer probes already exist.
The Rust tests verify committed Ketho-derived byte vectors; they never generate
or rewrite the expected files. Historical donor revisions identify test evidence,
not a permanent client, dependency, or source-version requirement.

## Completion gate

E1-C code is complete only after one exact ReferenceView produces byte-deterministic semantic, rendered-file, source-map, loss, parity, probe, and artifact manifests; every unsupported or transformed input is explicit; EmmyLua and LuaLS positive and negative probes pass without configuration mutation or diagnostic suppression; and all [`e1/TEST_MATRIX.md`](e1/TEST_MATRIX.md) cases and [`e1/examples/CHECKSUMS.json`](e1/examples/CHECKSUMS.json) vectors pass.
