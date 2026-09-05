# Implementation status and update policy

This is the executable implementation ledger, not the planned E0-E7 completion
matrix. Ketho/vscode-wow-api is the primary annotation-service implementation
donor; [KETHO_RUST_PORT.md](KETHO_RUST_PORT.md) specifies the Rust migration route.

## Implemented executable scope

Active Cargo workspace members:

- `wow-core`: deterministic identity, evidence, coverage and result primitives.
- `wow-reference`: deterministic reference view; generated API and UI topology
  JSON validation/import; source-bound API/topology bundle and development CLIs.
- `wow-annotations`: pure Rust port of Ketho's annotation emitter and naming
  helpers. Ordered callables, fields, arrays, explicit enum types, structures,
  argument-only callbacks, varargs, defaults and ScriptObject receivers are
  implemented. Native event/CVar literal unions and typed enum/constant output
  are also implemented, with explicit ordering/formatting, escaped strings and
  bounded output. A native reference-to-renderer connection now accepts exact
  source documents through the EmmyLua parser and emits annotations plus raw
  metadata, explicit loss/error records and declaration source mappings.
  Its library remains nonexecuting and free of IO; a Rust development driver
  performs local Git/TOC reads and writes a new output directory.

The native source-to-library path is runnable without Python or a Lua runtime;
see [the exact command and limits](KETHO_RUST_PORT.md#native-source-to-library-path).
It is still a bounded in-memory E1-B/E1-C slice, not full ReferenceView persistence,
correction/type closure or consumer certification. Declaration maps are present;
literal maps are whole-file. Full fine-grained E1 maps and actual EmmyLua/LuaLS
semantic consumer probes remain incomplete. Existing donor goldens still pass.

The native scalar lane reads Constants `Values`, retains descriptor types and
resolves source-local enum labels, transitive references and exact integer `+`/`-`
expressions. It records per-value outcomes and transitive source spans/hashes in
native library report v2. Unknown global names remain unresolved data rather than
rejecting a whole documentation file. Invalid individual callable/structure
projections are excluded with their own issue and leave valid neighboring
annotations/source maps intact. This does not implement runtime-global discovery,
correction packs, widget alias mappings or general Lua evaluation.

Legacy Python source tools still provide local Git snapshot inventory,
declarative API and XML/TOC producers/verifiers. They are migration debt, not the
product architecture to extend. The Rust port must replace these paths and their
CI invocations before the product is described as Python-free. No new Python
product components are accepted.

The canonical WoW development skill and byte-identical host copies are present.
CI checks current stable Rust, locked and updated dependency resolution, Rust
and legacy Python tests, and the existing Python-to-Rust source-bridge regression.
The new emitter's unit/golden tests are entirely Rust and offline.

The existing source-bridge integration test uses synthetic Git SHA-1 and SHA-256
repositories, including null/negative/exponent values, optional child collections,
export attributes, dirty worktrees, stale selection and digest tampering. These
fixtures are not evidence of an installed WoW client or a language-server probe.

`wow-reference-source materialize` is no-clobber and idempotent for identical
bytes. It publishes a staged file by an atomic hard link; unsupported filesystems
return an error. Directory crash durability is not claimed.

## Update policy

No repository toolchain override exists. CI installs current stable Rust.
Compatible dependency requirements permit updates; Cargo.lock records one build,
not a permanent version. Moving Blizzard selectors are resolved per operation.
Exact revision/version/hashes describe the inspected input only. The scheduled
source-bundle job clones current Gethe/live and does not imply support for every
other flavor. Raw source-wire numeric lexemes remain distinct from core identity
JSON; no universal numeric normalization is claimed.

## Optional context and remaining product work

No private knowledge provider is a build or runtime dependency. No provider
endpoint, token or corpus belongs in public code. Operator-only retrieval,
managed Blizzard checkout updates and GitHub-only manifest acquisition remain
unimplemented. The current skill describes manual research rather than fictitious
commands. Removing current identifiers does not erase Git history or old copies;
link sanitization alone does not make confidential prose safe to publish.

Full I0-A/I0-B acceptance and persistent channel publication remain incomplete.
The real EmmyLua semantic adapter is not an active workspace member. The public
wow binary, service composition, project model, diagnostics, persistence, graph,
search, provider transport and release lifecycle remain subsequent work. R0,
platform/install/signing and WoW runtime gates are not complete.

## Next implementation

Port Ketho correction/type/widget mappings, complete the selected projection
capabilities, and verify resulting libraries in both annotation consumers.
Keep the separate real wow-emmy analyzer adapter on the R0 path. Do not postpone
annotation parity behind unrelated graph/governance work or extend the legacy
Python pipeline. Activate only implemented owner slices and report exact scope.

## Native declaration recovery

The native Ketho source-to-library adapter now retains valid siblings when a
literal group is malformed, reports duplicate literal members individually, and
rejects enum `Values`/mixed collection shapes. Reserved return labels have explicit
collision-checked `name_projections`; prose controls are escaped with source-linked
metadata while source annotation directives remain rejected. Library report v3
preserves v2 scalar-resolution evidence. Tests cover source links, output bounds,
name conflicts and rejection without spurious projection records.

The concurrent `native_constants` resolver remains the only scalar-resolution
implementation; no parallel resolver or Python product path was introduced.
Full corrections/widget/type closure and dual-consumer semantic certification
remain incomplete. Local or CI success is not a full product-release claim.
