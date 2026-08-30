# `wow-annotations` E1-C normative examples

These files define the closed semantic projection, type lowering, rendering, sanitization, generated source-map/loss, Ketho parity, EmmyLua/LuaLS consumer probe, artifact manifest, and checksum-freeze contract for the future implementation.

## Files

- [`semantic-model.json`](semantic-model.json) — exact ReferenceView input and consumer-neutral modules/declarations/members/types/statuses.
- [`type-lowering-cases.json`](type-lowering-cases.json) — primitives, optional/nil, collections, tuples, unions, callbacks, enums, Secret facets, and explicit loss cases.
- [`rendered-artifact.json`](rendered-artifact.json) — deterministic layout profile, rendered files/fragments, artifact/file/semantic manifests.
- [`sanitization-cases.json`](sanitization-cases.json) — directive/comment/string/code/path/identifier/privacy/resource injection corpus.
- [`source-map-loss-cases.json`](source-map-loss-cases.json) — final-byte spans, exact reference links, projection coverage/loss and eligibility.
- [`parity-cases.json`](parity-cases.json) — Ketho semantic baseline and all parity classifications.
- [`consumer-probe-cases.json`](consumer-probe-cases.json) — pinned EmmyLua/LuaLS positive/negative/config-mutation/diagnostic/source-span probes.
- [`CHECKSUMS.json`](CHECKSUMS.json) — complete prerequisite/profile/model/file/map/loss/parity/probe/artifact freeze gate.

## Current state

No E1-C Rust implementation exists. Reference input, renderer, oracle, consumer, semantic, file, source-map, loss, parity, probe, artifact, and SHA-256 values remain null.

Nulls are valid only while implementation state is `not-started`. Before the first E1-C Rust commit, the implementation agent must:

1. freeze `wow-core` and `wow-reference` implementations/fixtures;
2. freeze exact ReferenceView/ProfileId/ReferenceGenerationId/ReferenceDataManifest inputs;
3. pin current reviewed Ketho oracle and exact source/config baseline;
4. pin/probe exact EmmyLua and LuaLS consumer profiles/configurations;
5. freeze semantic/type/layout/dialect/docs/source-map/loss/consumer profiles;
6. freeze semantic declarations/members/types/restrictions/reference links/statuses;
7. freeze rendered file bytes/paths/digests and final-byte source maps;
8. freeze sanitization/injection outputs and all projection loss records;
9. freeze parity records/report and consumer probe positive/negative/mutation results;
10. freeze annotation artifact/build/eligibility manifests;
11. canonicalize every JSON and generated Lua file and write member/bundle SHA-256 values;
12. update `CONTRACT.json` and implementation state;
13. run every applicable case in `TEST_MATRIX.md`.

Tests verify frozen fixtures and never rewrite them automatically.

## Authority boundary

The exact ReferenceView is the platform input. Ketho is a differential oracle; EmmyLua/LuaLS are consumers. None can silently rewrite ReferenceData facts.

## Runtime/editor boundary

Generated files are analysis-only inert stubs. They are not addon runtime code and must not enter an addon TOC/package. The library crate does not write output roots, spawn analyzers, mutate editor settings, add globals/libraries, install extensions, or suppress diagnostics.

## Change protocol

Any semantic change must update:

- the owning E1-C contract;
- `CONTRACT.json`;
- affected examples;
- `TEST_MATRIX.md`;
- all IDs/digests/checksums after implementation activation.

Do not weaken exact reference closure, no-silent-loss, sanitization, source-map, oracle classification, consumer negative probes, or no-editor-mutation gates to make output easier.
