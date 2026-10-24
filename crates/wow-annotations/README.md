# Annotation generation

Current status: [projection and manifest handoff](../../docs/CURRENT_STATUS.md).

Reuse `native.rs`, `ketho.rs`, `ketho/`, `aliases/`, `literals.rs`, `navigation/`
and `sidecars.rs`. Keep one renderer and exact source maps. Extend reviewed
Blizzard semantics through the existing model; do not add a second parser,
generator, manifest family or certification layer.

The builder emits ordinary Lua plus `source-report.json`, but the ordinary direct
check does not yet consume that manifest. The old native requalification path
rebuilds typed sidecars from retained source and is not the target reader. Plain
Lua Libraries remain valid for typing.

See [Implementation](../../docs/IMPLEMENTATION.md#native-annotation-generation),
[Ketho/native projection](../../docs/KETHO_RUST_PORT.md), and preserve
[third-party notices](THIRD_PARTY_NOTICES.md).
