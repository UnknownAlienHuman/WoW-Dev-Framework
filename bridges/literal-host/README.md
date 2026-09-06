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

Resource policy and observations are separate small modules. Limits default to
250M fuel per call, with the unchanged 500M/128MiB hard caps. Source composition
accepts explicit bounded limit overrides without recompilation. Fuel exhaustion
is distinguished from other VM traps; observational failures are sticky and do
not trigger fallback/retry. Successful receipts include limit and usage data,
not a total including failed calls or a wall-clock guarantee. Eager translation
prevents cold/warm translation charges from changing the same execution budget.

The explicit real_guest CI suite includes a large whole-inventory regression.
Only third-party interpreter packages use optimization in the test profile;
framework assertions, sandbox guards and all tests remain enabled.

`tests/fuel_boundary.rs` measures a synthetic module's actual execution fuel and
checks fresh and repeated calls at that exact budget and one unit below it.
Exhaustion must not alter other handles. This preserves the useful boundary
regression from the earlier repair without restoring its superseded API/defaults
or pinning a Wasmi/compiler-specific instruction count.
