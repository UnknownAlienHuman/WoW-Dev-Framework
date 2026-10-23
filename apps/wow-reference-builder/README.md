# Native annotation generator

Existing command at `c36a058`:

```text
wow-reference-builder generate <checkout> <ref> <GeneratedApi.toc> <environment> <new-output-directory>
  [--corrections <reviewed.json>]
  [--alias-catalog <checkout> <ref> <resource.lua>]...
```

Read local committed Git blobs, parse restricted APIDocumentation data through
Reference, and emit ordinary `.lua` files plus `source-report.json`. The output
directory must not exist. Source and generated stubs are never executed.

Current exits: 0 projected, 3 partial output, 2 arguments, 5 generation failure,
6 observed cancellation, 8 signal-handler setup failure. Inspect reported input
failures/omissions; exit 0 does not prove API coverage or a working addon.

**Known selection defect:** the current `environment` is compared only to a
system-level API field. Old `Mainline` examples conflate client flavor with API
environment, and member `SecureOnly` overrides are missed. This command is an
existing development entry point, not the finished target source-selection UX.
See [Audit](../../docs/AUDIT.md) and [Implementation](../../docs/IMPLEMENTATION.md#native-annotation-generation).

Application probe sources use this executable; their existence is not an
executed builder-to-addon result. The old copied producer and `/1` verifier are
already removed. The target `wow library build` reuses this pipeline. Emitter
semantics and donors are in [Ketho/native projection](../../docs/KETHO_RUST_PORT.md).
