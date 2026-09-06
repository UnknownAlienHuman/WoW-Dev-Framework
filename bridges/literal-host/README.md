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
