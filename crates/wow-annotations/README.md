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
these emitters. The development example is
`examples/native_library.rs`; #196 will expose that native path through a
supported command. The existing builder still has pack-oriented commands. Select the exact source revision and client
flavor, and retain partial/unsupported outcomes. Regenerate derived libraries
and reports when their source or projection inputs change.

The Rust projection preserves donor-specific type and signature behavior,
including callback argument/return shapes, nilability, defaults, aliases, and
explicit receiver names. Invalid or unsupported declarations stay reported;
they are not silently widened to `any`. The complete type/signature inventory
and known losses are documented in the [Ketho port map](../../docs/KETHO_RUST_PORT.md)
and [type lowering guide](e1/TYPE_LOWERING.md).

## Navigation and consumer checks

The `navigation` module provides `source_at`, `source_at_position`,
`source_for_range`, `NavigationIndex::new`, `generated_for`, and `bind_source`
for point/range, reverse, and source-buffer lookups. See the public API docs and
`e1/SOURCE_MAP_AND_LOSS.md` for map semantics and known losses.
Consumer checks in [`docs/ANNOTATION_CONSUMER_PROBES.md`](../../docs/ANNOTATION_CONSUMER_PROBES.md)
exercise signature and closed-string-enum feedback in EmmyLua and LuaLS.

The existing probes are focused semantic checks, not full-corpus or release
certification. Use one relevant positive and negative consumer case for changed
semantics; require broader qualification only when claiming that broader
coverage. Third-party notices are in [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

## Read by task

- Start with [`AGENTS.md`](../../AGENTS.md), the selected owner contract, and changed
  caller code.
- Native source, aliases, corrections, class binding, literals: [Ketho map](../../docs/KETHO_RUST_PORT.md).
- Type model and output layout: [semantic model](e1/SEMANTIC_MODEL.md),
  [type lowering](e1/TYPE_LOWERING.md), [layout](e1/LAYOUT_AND_RENDERING.md).
- Maps and losses: [source-map contract](e1/SOURCE_MAP_AND_LOSS.md).
- Consumer parity: [focused probes](../../docs/ANNOTATION_CONSUMER_PROBES.md).

Keep unknowns, conflicts, source identities, and coverage visible. Do not add
another parser, emitter, editor extension, runtime, or per-feature pipeline.
