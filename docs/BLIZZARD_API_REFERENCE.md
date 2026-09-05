# Native Blizzard API input and retained wire import

The current source path is the Rust Ketho loader and projection documented in
[KETHO_RUST_PORT.md](KETHO_RUST_PORT.md#native-source-to-library-path).
It consumes one selected local Git revision and generated-API TOC, evaluates a
bounded declarative subset through the existing EmmyLua AST without executing
Lua, and retains raw values, metadata, owners, source spans and projection issues.

```sh
cargo run -p wow-annotations --example native_library -- \
  /path/to/wow-ui-source HEAD \
  Interface/AddOns/Blizzard_APIDocumentationGenerated/Blizzard_APIDocumentationGenerated.toc \
  Mainline /path/to/new-output
cargo xtask verify-library /path/to/new-output --require-input-complete
```

The legacy v1 JSON producer and its standalone build/verify commands are retired.
There is no interpreter fallback. The following Rust commands still validate
**existing v1 wire artifacts**, not newly generated native-library reports:

```sh
cargo run -p wow-reference --bin wow-reference-api -- verify /path/to/api-reference.json
cargo run -p wow-reference --bin wow-reference-source -- verify /path/to/api-reference.json /path/to/ui-topology.json
```

These are distinct schemas. Native report v3 cannot be passed to the v1 importer.
Current native generation does not assert negative authority; partial, unsupported,
conflicted or excluded data remains explicit. Retained v1 import acceptance is
wire-contract compatibility, not proof of current source freshness, semantic
classification of every legacy producer record or runtime availability.

Native source/model/constant/projection regressions test the current pipeline.
Rust CLI fixtures separately test retained importers, tamper rejection, exact
source binding and no-clobber publication. Full persistent ReferenceView, corrected
widget/type closure and real EmmyLua/LuaLS semantic probes remain incomplete.
