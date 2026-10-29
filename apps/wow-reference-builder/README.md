# Native annotation generator

Current status: [generation and ordinary-reader handoff](../../docs/CURRENT_STATUS.md).

Current supported command:

```text
wow-reference-builder generate <checkout> <ref> <GeneratedApi.toc> <environment> <new-output-directory>
  [--corrections <reviewed.json>]
  [--alias-catalog <checkout> <ref> <resource.lua>]...
```

It reads local committed Git blobs, parses restricted APIDocumentation data
through Reference, and emits ordinary `.lua` files plus `source-report.json`. The
output directory must not exist. Source and generated stubs are never executed.
The builder writes directly into that directory and does not remove it on
failure. A failed or cancelled run may leave partial or complete output there;
use a fresh destination for retries and inspect any existing destination before
deciding what to do with it.

The selected TOC must provide a nonempty flat list of unique relative `.lua`
paths; paths are resolved beneath the TOC's directory. Blank and `#` lines,
including ordinary `##metadata` lines, are ignored rather than interpreted.
Non-Lua entries are rejected. This command does not expand dependencies, XML, or
conditional TOC directives and does not determine addon load selection.

Current exits: 0 projected, 3 partial output, 2 arguments, 5 generation failure,
6 observed cancellation, 8 signal-handler setup failure. Inspect input failures
and omissions; exit 0 does not prove API coverage or a working addon.

**Known selection defect:** `environment` is compared only with a system-level API
field. Old `Mainline` examples conflate client flavor with API environment, and
member `SecureOnly` overrides are missed. The ordinary direct check also does not
yet consume the generated report/maps. See
[Implementation](../../docs/IMPLEMENTATION.md#native-annotation-generation) and
[Ketho/native projection](../../docs/KETHO_RUST_PORT.md).

## Resource and cancellation boundaries

Input/file/output limits do not establish a process-wide peak-memory limit.
The current [report producer](src/lib.rs) constructs a JSON value and serializes
it completely before checking its 64 MiB size. That check refuses oversized
output after allocation. Correct this in the producer: serialize a borrowed typed
report through a bounded writer, enforcing the limit while encoding rather than
cloning the complete report into an intermediate JSON value.

Cancellation is cooperative. The bounded [Git reader](src/io.rs) performs blocking
reads and waits without a stop token; Reference's
[Lua parse](../../crates/wow-reference/src/native.rs) is also synchronous.
Checkpoints around these stages do not interrupt an active read/parse or establish
a hard timeout. Exit 6 means cancellation was observed, not immediate termination
or rollback of output. Keep these limits explicit; do not add a new runtime or
resource-management platform to describe guarantees the existing owners lack.

Application source-bundle workflows use this executable, but their existence is
not an observed builder-to-addon result. The copied producer and `/1` verifier
are removed. The eventual `wow library build` reuses this pipeline rather than
creating another generator.
