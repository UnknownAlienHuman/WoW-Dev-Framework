# Literal Wasm host bridge

An independently compiled Wasmi host for exactly the literal-render contract.
No dependency on the full annotation/source/service crates. The native renderer
is test-only for parity. Own workspace/dependency resolution permits bridge
updates independently of the source workspace.

Public API: load an explicitly approved module digest, create a ModuleSlot,
capture a Snapshot, render a typed Request, replace with expected Selection, or
roll back by reselecting a retained handle. Receipts bind module/input/output.
No generic executor, WASI, network, filesystem, credential or update service.

See [the complete contract and limits](../../docs/WASM_BRIDGES.md). Hash matching
is not signing/trust; in-memory selection is not durable publication. Compiled
guest parity tests are mandatory in the dedicated Linux/Windows CI lane.

The `source_library` example composes the shared development source driver with
one retained Snapshot. Source and annotation crates are development dependencies
only; the host library still depends only on its VM, hash and wire contract.
`tests/real_source.rs` exercises complete generation, mid-operation replacement,
retained versions and failure without output; CI runs it explicitly with real
compiled guests. `tests/real_corpus.rs` is the separately required current-source
workflow test, never silently skipped in that lane.
See [usage and limits](../../docs/WASM_BRIDGES.md#source-library-composition).
