# Rolling EmmyLua upstream

`wow-emmy` consumes the current compatible EmmyLua analyzer through one adapter. It does not declare one repository commit permanently current and does not implement a second Lua semantic parser.

The source/update/report contract is owned by [`../../docs/EMMYLUA_UPSTREAM.md`](../../docs/EMMYLUA_UPSTREAM.md) and implemented by [`../../scripts/check-emmylua-version.py`](../../scripts/check-emmylua-version.py).

## Package rule

For each implementation or analysis operation:

1. resolve/update the configured local upstream checkout according to `auto`, `prompt`, or `never` policy;
2. retain its exact commit, tree, crate manifest, public-surface digest, and compatibility result;
3. compile the adapter against the dependency recorded by the current framework build;
4. run deterministic analyzer fixtures for every adapter operation used by the framework;
5. re-resolve the moving upstream selector on the next operation.

A `Cargo.lock` revision identifies one reproducible build. It is not a permanent freshness authority. Scheduled compatibility probes detect upstream movement; dependency updates are ordinary reviewed changes.

## Separation of evidence

- Source-manager report: current checkout and public-surface evidence.
- Rust compile probe: adapter API compatibility.
- Analyzer fixture: semantic operation behavior.
- Blizzard UI/reference products: WoW platform facts and exact generation identity.
- Runtime probe: client-only behavior.

No one lane upgrades another. In particular, successful compilation does not prove analyzer semantics, and analyzer output does not prove current Blizzard API behavior without the corresponding current-source generation.

## Current implementation sequence

1. rolling source manager and compatibility report;
2. explicit in-memory workspace construction;
3. syntax and diagnostic query adapter;
4. symbol, reference, and type query adapter;
5. deterministic mapping into `wow-core` evidence, coverage, source handles, and result envelopes;
6. cancellation, budget, multi-file, malformed-input, and upstream-update fixtures.

The adapter public seam exposes framework-owned immutable values. Upstream database/session/parser handles do not cross it.
