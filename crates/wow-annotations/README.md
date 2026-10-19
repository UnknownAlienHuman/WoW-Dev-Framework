# `wow-annotations`

`wow-annotations` lowers admitted `wow-reference` facts into inert Emmy-style
Lua analysis libraries and source maps. `wow-reference` owns Blizzard source
admission and authority; this crate owns annotation projection, Ketho-compatible
rendering, literal generation, and generated-to-source navigation. Output is
ordinary `.lua` library data that users can add through EmmyLua, LuaLS, or
another compatible analyzer configuration. It is not addon runtime code.

## Native flow

The reusable emitter in `src/ketho.rs` consumes ordered typed declarations; it
does not parse Blizzard source or infer reference authority. `src/literals.rs`
projects explicit typed event/CVar/enum/constant data. The native connection in
`src/native.rs` composes Reference's restricted evaluator and normalization with
these emitters. The supported source-to-directory command is `wow-reference-builder generate`;
this crate remains the projection owner and has no filesystem/package lifecycle.

The Rust projection preserves donor-specific type and signature behavior,
including callback argument/return shapes, nilability, defaults, aliases, and
explicit receiver names. Invalid or unsupported declarations stay reported;
they are not silently widened to `any`. The complete type/signature inventory
and known losses are documented in the [Ketho port map](../../docs/KETHO_RUST_PORT.md)
and [type lowering guide](TYPE_LOWERING.md).

## Navigation and consumer checks

The `navigation` module provides `source_at`, `source_at_position`,
`source_for_range`, `NavigationIndex::new`, `generated_for`, and `bind_source`
for point/range, reverse, and source-buffer lookups. See the public API docs and
[source maps](SOURCE_MAPS.md) for precision and incomplete-result behavior.
Consumer checks in [`docs/ANNOTATION_CONSUMER_PROBES.md`](../../docs/ANNOTATION_CONSUMER_PROBES.md)
exercise signature and closed-string-enum feedback in EmmyLua and LuaLS.

The existing probes are focused semantic checks, not full-corpus or release
certification. Use one relevant positive and negative consumer case for changed
semantics; require broader qualification only when claiming that broader
coverage. Third-party notices are in [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

## Read by task

- Start with root [`AGENTS.md`](../../AGENTS.md), the selected Issue and changed
  caller code.
- Native source, aliases, corrections, class binding, literals: [Ketho map](../../docs/KETHO_RUST_PORT.md).
- Types, signatures and output: [lowering notes](TYPE_LOWERING.md).
- Maps and navigation: [source maps](SOURCE_MAPS.md).
- Consumer parity: [focused probes](../../docs/ANNOTATION_CONSUMER_PROBES.md).

Keep unknowns, conflicts, source identities, and coverage visible. Do not add
another parser, emitter, editor extension, runtime, or per-feature pipeline.
