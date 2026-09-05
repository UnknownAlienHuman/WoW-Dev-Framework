# Workspace and build plan

> Current implementation and update policy: [status ledger](IMPLEMENTATION_STATUS.md). Earlier bootstrap schedules below are design history, not instructions to recreate the workspace or permanently pin versions.

**Status:** normative implementation handoff. The two-crate foundation and source bridge are executable; final product topology remains planned.

## Goal

Create one Rust workspace whose final default product build produces the `wow` executable described by E7-A/E7-B, while implementation begins with the smallest E0 vertical slice and never activates placeholder crates merely to satisfy the final topology.

## Final planned workspace members

```text
crates/wow-core
crates/wow-store
crates/wow-reference
crates/wow-annotations
crates/wow-emmy
crates/wow-project
crates/wow-graph
crates/wow-recognizers
crates/wow-rules
crates/wow-search
crates/wow-context
crates/wow-cbm
crates/wow-service

apps/wow
apps/wow-reference-builder

tools/wow-release
```

This is the final topology, not the first commit. A member enters the root workspace only with an implemented owned slice, focused tests, non-placeholder public API, contract/fixture pins and correct dependency edges.

## Final binary targets

```text
wow
    public product executable
    one-shot CLI
    local daemon
    LSP stdio
    MCP stdio
    local version/verification/explicit update/rollback client

wow-reference-builder
    internal/advanced exact Reference Pack build and validation tool

wow-release
    internal release-engineering client over wow-service
```

Only `wow` is in the default public release bundle. The other tools have independent administrative artifact/support profiles.

## Dependency direction

The final Cargo dependency graph must conform to [`../crates/DEPENDENCY_GRAPH.md`](../crates/DEPENDENCY_GRAPH.md). In particular:

```text
wow-core -> no framework crate
wow-store -> wow-core
wow-cbm -> wow-core
owner crates -> foundations/owners only, never wow-service/apps/tools
wow-service -> reviewed owner crates through narrow contracts
apps and tools -> wow-service only among framework crates
```

Development/test-only dependencies cannot smuggle production reverse edges or owner algorithms into apps/service.

## Initial workspace activation

### I0-A — core bootstrap

First root workspace:

```text
crates/wow-core
```

Do not add empty sibling members. Implement E0-A identities, profiles, generations, evidence, confidence, coverage, conflicts, negative authority, status/result/error, cancellation/budgets and canonical serialization first.

### I0-B/C — fixture reference and analyzer adapter

Add when implemented:

```text
crates/wow-reference
crates/wow-emmy
```

`wow-reference` initially implements only the exact E0-B frozen fixture slice. `wow-emmy` pins the selected upstream Rust analyzer behind one adapter and conformance suite.

### I0-D/E/F — first runnable vertical slice

Add:

```text
crates/wow-project
crates/wow-rules
crates/wow-service
apps/wow
```

At this point the required product commands are:

```text
wow status
wow check
```

The repository reaches launch gate R0 only when these commands run against exact frozen fixtures and all E0 acceptance/checksum gates pass.

### Later activation

Add each remaining crate/tool only at its implementation milestone:

```text
E1  wow-store, full wow-reference, wow-annotations, wow-reference-builder
E2  wow-graph, wow-recognizers, full wow-project and ProjectStore
E3  wow-context and E3 service/CLI
E4  wow-search and lineage/impact service/CLI
E5  calibration/publication owner and service slices
E6  wow-cbm and external-candidate service slice, optional
E7A session overlays and CLI/daemon/LSP/MCP host
E7B release/update service, client and wow-release
```

## Package naming

Cargo package names and library crate names use the repository directory names with normal Rust underscore imports:

```text
package = "wow-core"       crate = wow_core
package = "wow-store"      crate = wow_store
...
package = "wow-service"    crate = wow_service
package = "wow"            binary = wow
package = "wow-release"    binary = wow-release
```

No duplicate package/library names, hidden workspace members or production examples masquerading as binaries.

## Rust/toolchain freeze

Before the first Rust commit, select and record exact:

```text
Rust toolchain/channel version and components
Cargo version and resolver
Rust edition
minimum supported Rust version if a separate policy is required
Windows x86-64 MSVC target/toolchain prerequisites
rustfmt and clippy versions through the toolchain
cargo-nextest and other external tool versions when required
```

Do not write `latest`, floating stable, or an unverified MSRV claim into release profiles. The chosen toolchain is a release input and must be reproducibly materializable.

A root `rust-toolchain.toml` is added only with this exact freeze and toolchain conformance evidence.

## Workspace dependency policy

Use root workspace dependency declarations only for dependencies genuinely shared by multiple implemented packages. Every dependency requires:

```text
owner/call-site need
exact version/source/feature set
license/provenance/security review
minimal feature closure
build-script/proc-macro/native-code classification
lockfile entry and checksum/source closure
offline/reproducible materialization behavior
```

Default features are disabled when they add unused network/runtime/platform behavior. No dependency is added because it may be useful later.

Git dependencies are exact-revision pinned and require source/license/vendor/reproducibility handling. Prefer registry releases when they satisfy the exact contract and can be locked/verified.

## Feature profiles

Final planned product feature/exposure profiles:

```text
local-exact-core
    CLI + local stores + reference/project/graph/rules/search/context

daemon
    local IPC host

lsp
    standalone LSP stdio host

mcp
    standalone MCP stdio host

release-client
    local release verification and explicit update/rollback

external-cbm
    optional E6 external Candidate lane; disabled until an adapter passes

admin-governance
    E5 administrative/calibration/publication exposure; disabled in default developer profile
```

The final public `wow` target profile may compile the local transport/release-client capabilities together while exposure remains runtime-profile constrained. `external-cbm` and administrative effects remain optional/disabled by default until their complete gates pass.

Cargo feature absence must produce honest operation-registry omissions, not runtime stubs returning success.

## Network policy

Default local analysis build/runtime is offline-capable and has no implicit network behavior. Network appears only behind explicit owner operations/profiles for:

```text
source/dependency/toolchain materialization
external E6 provider calls when enabled
explicit update check/download
release signing/platform/distribution services
```

No startup update check, telemetry, remote config, source fetch or provider discovery.

## Workspace profiles

Exact Cargo profile values freeze during implementation based on correctness/performance/reproducibility measurements. Required logical profiles:

```text
dev
    fast local development with useful diagnostics

test
    deterministic test/fixture behavior

release
    public product artifact profile

release-repro
    unsigned reproducibility comparison profile
```

Do not prematurely lock optimization/LTO/panic/strip/debug settings without target/toolchain/performance/crash-diagnostic evidence. Once selected, they are exact `ReleasePlan` inputs.

## Generated artifacts

Generated service operation registries, public schemas, compatibility manifests, annotations, Reference Packs, fixtures, checksums and release manifests have explicit generators/owners and deterministic checked outputs.

Tests never rewrite normative fixtures or committed generated files. Generation is an explicit command; validation compares current generated bytes with committed expected bytes and fails on drift.

Generated build outputs never live inside source directories or enter source snapshots unless the exact source profile intentionally commits them.

## Data directories

The implementation defines exact platform-owned locations for:

```text
immutable Reference/core/data packs
ProjectStore and graph/search objects
session/lease/recovery records
logs/cache
public nonsecret configuration
protected adapter credentials outside public config
installation/current/LKR/backup records
```

Paths are platform profile inputs and owner handles, not semantic IDs. Tests use isolated temporary owner roots and never touch the user’s actual WoW/addon/editor data.

## Build commands — required final shape

The exact flags/tool versions freeze with implementation. The supported logical command surface must include:

```text
cargo build --locked --workspace
cargo test --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo nextest run --locked --workspace --all-features

cargo build --locked --release -p wow --target x86_64-pc-windows-msvc
cargo run --locked -p wow -- status
cargo run --locked -p wow -- check <exact fixture request>

cargo run --locked -p wow-release -- source validate <exact request>
cargo run --locked -p wow-release -- plan validate <exact plan>
cargo run --locked -p wow-release -- build submit <exact plan>
...
```

A command is not claimed supported until implemented and frozen in [`CONFORMANCE_COMMANDS.md`](CONFORMANCE_COMMANDS.md).

## Testing topology

Tests live with their owners plus cross-package conformance suites. Required classes:

```text
unit and property tests for invariants
canonical serialization/golden-byte tests
contract/schema/fixture/checksum validation
mutation/rejected-shortcut tests
owner-port integration tests
crash/response-loss/recovery tests
security/adversarial/path/resource tests
platform/CLI/daemon/LSP/MCP client tests
real addon evaluation with exact pins and nonexecution
reproducible build and install/update/rollback rehearsals
```

Mocks may model deterministic owner failures, but cannot substitute for mandatory real integration/platform/provider/signing/installer evidence.

## Unsafe and FFI

Default is no `unsafe`. Any `unsafe`, FFI or platform-specific native integration requires:

```text
owned module boundary and safety invariant
concrete need and alternatives
focused tests/fuzzing where applicable
platform/profile restriction
review and supply-chain impact
```

Do not add unsafe code for speculative performance.

## Error and panic policy

Domain failures use typed errors/statuses. Panics indicate invariant/programming faults and are caught only at host/process boundaries where safe. No panic content with source/secrets is emitted by default.

Exact release panic/abort/unwind behavior freezes per target after crash/recovery/support analysis.

## Documentation and public API

Public Rust APIs follow the owning package contract. Internal implementation details are not exported merely for tests or transports. Examples and docs use exact fixtures/profiles, never hidden network/current state.

Rustdoc/documentation claims must match implemented capabilities and preserve `Candidate`, coverage, `NotEvaluated`, authorization and nonclaim semantics.

## Definition of a build-ready repository

The repository is only “ready to build” when:

```text
Cargo workspace and selected members exist
rust-toolchain and Cargo.lock are exact
all workspace packages compile for the selected target
all required conformance commands pass
fixtures/checksums are populated and immutable
one complete release plan can build and verify wow
```

Current state remains documentation-only. The next action after documentation freeze is I0-A `wow-core`, not creation of all empty workspace members.