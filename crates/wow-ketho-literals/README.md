# wow-ketho-literals

The extracted Ketho event/CVar/enum/constant renderer. No source parser, VM, IO,
upstream inventory or host dependency. Both the existing annotation facade and
the separately compiled Wasm guest use this implementation. Existing golden tests
remain in wow-annotations. See [bridge routing](../../docs/WASM_BRIDGES.md) and
[third-party notices](THIRD_PARTY_NOTICES.md).
