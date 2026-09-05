# Implementation status and update policy

This ledger describes executable scope, not completion of the planned E0-E7
architecture. Ketho is the annotation-service donor; the
[port map](KETHO_RUST_PORT.md) defines the native migration route.

## Active workspace

- `wow-core`: deterministic identity, evidence, coverage and result primitives.
- `wow-reference`: deterministic fixture reference view; native EmmyLua-AST
  documentation loading/normalization and exact scalar resolution; retained
  v1 API/topology wire importers and development CLIs.
- `wow-annotations`: Rust Ketho callable, structure, callback and literal
  emitters connected to native source documents. The Git/TOC development driver
  creates a new library with raw metadata, errors and declaration source maps.
- `tools/xtask`: internal repository/source maintenance with no framework crate
  dependencies. Policy/JSON/skill checks, explicit skill synchronization, public
  HTTPS remote-head comparison, exact Git manifest build/verify and native
  artifact consistency verification. Not the public product/service CLI.

There are no interpreter source files or interpreter invocations in the build,
tests or CI. `cargo xtask check` enforces the native-only repository policy and
skill consistency. See [xtask commands and limits](../tools/xtask/README.md).
No replacement script is hidden in Rust, CI heredocs or generated payloads.

## Native source and annotations

The [native command](KETHO_RUST_PORT.md#native-source-to-library-path) reads one
materialized local Git revision and selected documentation TOC. Source worktree
changes are ignored; source Lua and generated stubs are never executed.
Report `wow-native-annotation-library/3` retains raw metadata, scalar-resolution
outcomes/evidence, explicit return-name transformations and escaped prose links.
Constants use `Values`; enums use `Fields`. Exact integer additive expressions,
enum labels and transitive same-corpus references resolve without runtime globals.
Invalid declarations/groups do not erase valid siblings; conflicts remain errors.

Declaration maps are source-bound; literal maps are whole-file. Named type/widget
closure, the remaining correction operations, fine-grained maps, persistent ReferenceView integration and
real EmmyLua/LuaLS semantic consumer probes remain incomplete. Native projection
is partial when data is omitted or unsupported; it never issues negative authority.

## Retired source path and compatibility boundary

The former interpreter-based source producers, their tests and setup/workflow
calls are removed, not retained as migration fallback. Native Ketho generation
replaces the annotation input/output path, not every old wire command.

- Source manifests now build/verify through native `cargo xtask` commands.
- Skill maintenance now uses native `cargo xtask sync-skill`.
- v1 API/topology JSON readers remain for existing artifacts. Native Rust CLI
  fixtures cover lookup, digest tampering, source mismatch, partial authority and
  idempotent/no-clobber bundle publication. These fixtures do not regenerate
  current Gethe topology or prove a real source inventory.
- A full native XML/TOC topology producer and v1 API wire producer are not
  implemented. The annotation TOC reader is not a full topology replacement.
- The old upstream source-manager/public-symbol-report command family is retired.
  `check-source` is a read-only replacement for remote-head comparison only, not
  managed cloning, auto-update or analyzer semantic compatibility.

Rust manifest regressions use synthetic SHA-1/SHA-256 Git repositories and cover
raw blobs, export attributes, dirty worktrees, digest tampering, source movement,
path rejection and new-only output. Native Ketho source/model/renderer tests
remain active. No language-server, installation or client result is fabricated.

## CI and update policy

CI checks Linux/Windows, current stable Rust, fmt/check/strict Clippy, debug/release
workspace tests, rustdoc, repository policy and exact skill copies. Separate
updated-dependency and rolling-parser lanes exercise the reference/annotation
consumers. No permanent toolchain override exists; compatible requirements and
Cargo.lock describe a reproducible build without forbidding updates.

The current-source workflow resolves the selected Gethe branch, builds/rebuilds
its Git manifest and generates annotations through Rust only. Final output bytes,
hashes, source counts and mapping ranges are checked. Source admission failures
fail the job; explicit projection omissions remain partial, not a semantic pass.
The default branch/environment can be changed for an explicit dispatched run;
this does not certify every flavor. The source check and generator share one
resolved revision, never a permanently embedded client build.

## Remaining product scope

Managed Blizzard checkout updates, GitHub-only acquisition and operator-only
knowledge retrieval remain unimplemented. No private provider, endpoint, token
or corpus is a public build/runtime dependency. Source-head checks use only an
explicit public HTTPS origin; offline freshness is unverified.

Full I0-A/I0-B acceptance and persistent channel publication remain incomplete.
The real semantic `wow-emmy` adapter and public `wow` binary are not active.
Service composition, project model, diagnostics, persistence, graph, search,
transport, installation and release gates remain subsequent work.

Next annotation work: remaining Ketho type resources, widget inheritance and actual
dual-consumer probes. Keep the analyzer adapter on the R0 path, without delaying
annotation parity behind unrelated future subsystems or restoring a parallel
interpreter implementation.

## Reviewed Ketho corrections

`wow-reference::native_corrections` now applies independent reviewed Type/Nilable
and widget-receiver corrections to a normalized copy of exact source documents.
The native driver accepts `--corrections <pack.json>` explicitly. Guards bind the
source revision/environment, normalizer, file hash, raw observation hash and old
value; mismatch never refreshes itself. Duplicate targets and receiver collisions
remain conflicts, including collisions exposed by another alias being rejected.
Raw source is unchanged. Every record has an outcome and blocked corrections keep
the result partial. Report v4 adds the canonical correction set/digest and outcomes;
uncorrected builds still emit v3. The artifact checker accepts both and rejects
false-clean correction reports. Named/primitive unions are now rendered with
correct array grouping. No source acquisition, dependency or interpreter was added.
See [usage and remaining correction scope](KETHO_NATIVE_CORRECTIONS.md).
