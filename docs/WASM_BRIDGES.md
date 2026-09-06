# Independently updateable, narrow Wasm bridges

Accepted owner direction: micromodular Rust; frequently changed algorithms may
ship as separate Wasm artifacts. This refines ADR-022/029 without admitting a
generic plugin executor, a second Lua parser or uncontrolled repository code.

## Implemented literal lane

| Package | Responsibility | Production source size |
|---|---|---|
| `crates/wow-render-contract` | Typed literal inputs, strict versioned wire and bridge trait | Small shared boundary |
| `crates/wow-ketho-literals` | The existing Ketho literal algorithm, extracted rather than duplicated | No VM or acquisition |
| `modules/ketho-literals` | Rust core-Wasm exports around that algorithm | No WASI or host imports |
| `bridges/literal-host` | Wasmi sandbox, immutable module handles and CAS selection | Independent workspace/lock |

`wow-annotations::literals` is a small compatibility bridge to the extracted
algorithm. Its existing API and golden results remain unchanged. Stable owners
never import Wasmi. The optional VM bridge imports the contract, not the source
normalizer, analyzer, service or complete annotation crate. Its native-renderer
dependency is test-only, for parity. There is exactly one literal algorithm.

The full source driver still uses the native bridge. The standalone Wasm bridge
is executable and testable, but is not automatically selected by that driver or
the unfinished public service. Do not describe the entire project as hot-swappable.

## Fixed operation and memory boundary

`wow-literals/1` uses core Wasm32 (not WASI):

```text
memory                         exported linear memory
wow_abi_version() -> i32        must equal 1
wow_request_buffer(i32) -> i32  owned guest buffer offset, -1 on failure
wow_render() -> i32             0 transport success; nonzero no output
wow_response_ptr() -> i32       result offset
wow_response_len() -> i32       result byte length
```

The host copies a strictly decoded `Request` (schema 1, output bound, one closed
operation: events, CVars or enums/constants). The guest returns `Response` with
schema and typed `Result<String, LiteralError>`. There is no arbitrary method,
script, callback, path, URI, environment, source text executor or host handle.
Guest export attributes are the only narrowly allowed unsafe attributes; no
unsafe Rust block or native FFI pointer is used. Offsets address guest memory.

Input serialization is bounded during writing; unknown/duplicate fields, bad
schema, nonfinite numeric scalars, and oversized output fail. Domain errors are
not successful partial text. Native errors keep their existing classifications.
The VM adapter's fixed runtime failures are separate from source domain errors.

Admission checks module bytes/digest, validates Wasm, denies *every* import and
start section, checks export types and ABI. Wasmi validation/strict compilation
limits are enabled. Each invocation uses a fresh store/instance, one memory and
one table, bounded memory/table/stack/recursion and fuel, and range-checked copies.
Compiled code is shared by immutable handles; guest mutable state is not shared.
No shell, WASI, filesystem, network, clocks, random, threads or credentials are
provided. Fuel bounds guest execution, not a promised wall-clock deadline for
compilation. The byte ceiling and Wasmi compilation limits are separate guards.

Hard limits: 8 MiB module, 8 MiB request, 8 MiB rendered output, bounded encoded
response; at most 128 MiB guest memory and 500 million fuel (default 100 million).
These are versioned implementation limits, not estimates of throughput or safety
certification. No transparent fallback hides a rejected/trapped guest.

## Replacement and rollback

`ModuleHandle::load` receives bytes and an explicitly approved digest; it does
not discover, download or trust a filename, remote branch or hash by itself.
The hash proves identity, **not signature/authorship/semantic correctness**.
The caller is responsible for reviewing/testing eligible module artifacts.

`ModuleSlot::snapshot()` captures a selection and immutable compiled handle for
one operation. `replace(expected, candidate)` changes only future snapshots.
The epoch participates in CAS, so A -> B -> A cannot make a stale request valid.
Rollback reselects a retained, already validated handle with a new epoch. Failed
admission/CAS leaves the current handle untouched. Each receipt binds module,
canonical encoded request and exact encoded response hashes. A retained operation
never follows a moving current pointer mid-request.

This selector is intentionally in-memory. Signed distribution, durable catalogs,
crash recovery, persistent promotion and service-owned activation remain separate
unimplemented owner work. Those gates must not be invented by a VM wrapper.

## Cheap upstream updates

Keep three independent identities and schedules:

1. **Gethe data:** re-resolve the selected branch, reuse unchanged raw blobs and
   rebuild only affected source products. A revision is a per-operation snapshot.
2. **Ketho resources:** aliases/corrections are versioned data inputs; changing
   compatible data does not rebuild either host or Wasm.
3. **Ketho-derived algorithms:** compile the changed small guest and its exact
   dependency closure, run donor/native and consumer probes, then select a new
   compatible artifact. An unchanged host loads it without recompilation.

A new ABI/schema requirement rejects an old host rather than forcing unsafe
compatibility. Updating the VM or stable owner logic itself still requires a
host release. Full managed fetch/update scheduling, affected-graph invalidation,
signed delivery and consumer certification are not implemented by this slice.

## Reproducible checks

```sh
rustup target add wasm32-unknown-unknown
cargo build --locked -p wow-ketho-literals-wasm --target wasm32-unknown-unknown --profile wasm-compact
cargo build --locked -p wow-ketho-literals-wasm --target wasm32-unknown-unknown --profile wasm-speed
cargo test --manifest-path bridges/literal-host/Cargo.toml --test runtime
WDF_WASM_A="$PWD/target/wasm32-unknown-unknown/wasm-compact/wow_ketho_literals_wasm.wasm" \
WDF_WASM_B="$PWD/target/wasm32-unknown-unknown/wasm-speed/wow_ketho_literals_wasm.wasm" \
  cargo test --manifest-path bridges/literal-host/Cargo.toml --test real_guest -- --ignored --nocapture
```

The explicit compiled-guest test requires two different real Rust-produced Wasm
artifacts. It is mandatory in the Linux/Windows Wasm CI lane; not silently skipped
when artifacts are unavailable. One unchanged host checks all three operations
against the actual native Ketho implementation, then replacement, old snapshot,
rollback, stale-epoch rejection and domain errors. Synthetic Wasm tests separately
cover ABI/import/start rejection, traps/fuel, memory growth, ranges and bad output.
These are bridge/parity checks, **not EmmyLua/LuaLS semantic certification**.

Retain third-party notices with standalone guest distributions. No source corpus,
private knowledge provider, data snapshot or client version is compiled into the
bridge. No Python is used in source, builds, tests or workflow commands.

### Memory-limit regression coverage

The host-limit probe grows from one to two pages inside a module permitting two
pages, with the host capped at one page. It must trap. A separate positive probe
permits exactly two pages and checks both the previous and resulting memory size;
repeated calls confirm that each request starts with fresh guest memory. Initial
memory exceeding the host cap rejects admission. A module-maximum rejection is a
separate case: Wasmi may return the Wasm `-1` failure value before invoking the
host limiter, and the test checks that memory did not grow. None of these probes
relaxes the host limit or disables traps on host-denied allocation.

The real-guest test executes all three operations before replacement, after
replacement, and after rollback using the same host. Snapshots captured for both
generations remain usable after rollback. The two Rust guest builds differ in
optimization profile, not in claimed algorithm behavior; native parity is checked
for both. Independently resolved host dependency inputs are retained by Linux CI
for exact offline reproduction, outside the repository and public product bundle.
