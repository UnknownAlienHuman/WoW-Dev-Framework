# `wow-emmy` E0-C implementation plan

**Status:** ordered handoff plan for a future coding agent. This documentation change adds no Rust code.

Implementation must follow this order. A later phase may not compensate for an unproven earlier boundary.

## Phase 0 — prerequisites

1. Confirm E0-A `wow-core` implementation is merged and its examples/hash vectors pass.
2. Confirm E0-B contract/fixture is merged; E0-C does not depend on its Rust crate directly, but cross-crate fixture names must remain coherent.
3. Read all files listed in [`AGENTS.md`](AGENTS.md).
4. Inspect current official upstream source.
5. Confirm no competing adapter implementation exists.
6. Confirm the Cargo workspace activates only current E0 crates.

**Gate:** do not code against draft/missing `wow-core` types or operations.

## Phase 1 — upstream pin research

Create a candidate pin record from exact current upstream source.

Record:

```text
repo/commit/date
workspace/crate versions
features
license
Rust edition/MSRV/toolchain
public API symbols to be used
relevant dependency changes
```

Do not add the dependency before confirming that required public APIs are available without private/internal access.

**Gate:** candidate pin record complete and reviewable.

## Phase 2 — compatibility probe harness

Implement fixture-only probe code before the production adapter surface.

Probe sections P1–P12 in [`PIN_AND_PROBE.md`](PIN_AND_PROBE.md).

Freeze:

- selected generic diagnostic upstream family/code;
- normalization category;
- source coordinate behavior;
- main/library workspace behavior;
- required semantic fact extraction path;
- incremental update semantics;
- observed new diagnostic families;
- performance observations;
- rollback candidate.

**Gate:** accepted probe report. If rejected, select another pin or revise the contract explicitly; do not patch around a missing mandatory capability in downstream crates.

## Phase 3 — crate skeleton and adapter isolation

Create the smallest crate with internal responsibilities such as:

```text
pin
configuration
session
workspace
file
source_coordinates
upstream_adapter
diagnostics
facts
coverage
fixture
error
```

Rules:

- only `wow-core` as direct framework dependency;
- no `wow-reference`, `wow-project`, `wow-rules`, service, transport, storage, TOC/XML, or search dependency;
- no empty future modules/traits;
- upstream types private to adapter module;
- no async runtime unless the accepted upstream API/current E0 actor requires it and the contract is updated.

**Gate:** crate compiles with no placeholder success paths and public API contains no upstream types.

## Phase 4 — source identity and coordinates

Implement first:

```text
validate_utf8_source
normalize_workspace_relative_path
build_line_index
map_upstream_file_identity
convert_upstream_range_to_byte_span
convert_byte_span_to_positions
validate_source_span
build_project_source_handle
```

Run `EMMY-FILE-*` and `EMMY-SPAN-*` before diagnostics/facts.

**Gate:** exact LF/CRLF/multibyte/EOF/update spans and no host path leakage.

## Phase 5 — configuration/workspace registry

Implement:

```text
build_analyzer_configuration
validate_analyzer_configuration
register Main workspace
register Library workspace
register initial file manifests
canonical configuration/workspace digests
```

Run `EMMY-CONFIG-*`.

No editor config read/write is allowed for correctness.

**Gate:** closed fixture workspace validates independently of temporary root/order.

## Phase 6 — session actor and file lifecycle

Implement:

```text
create_analyzer_session
add_file
update_file
remove_file
apply_update_batch
refresh_analyzer_index
close_analyzer_session
```

Use one mutable owner/actor. Do not publish intermediate state.

Run `EMMY-SESSION-*` and basic `EMMY-INC-*`.

**Gate:** coherent atomic update behavior, stale-digest/generation rejection, fatal session corruption handling.

## Phase 7 — annotation library health

Load only `library/C_E0Fixture.lua`.

Implement/report:

```text
library registration
library parse/index health
library capability coverage
resolution dependency invalidation
library-root-cause record
```

Run `EMMY-LIB-*`.

**Gate:** KnownApi/SecretText resolve; broken library blocks dependent facts without project-finding contamination.

## Phase 8 — snapshot publication

Implement:

```text
derive AnalyzerSnapshotId
build file manifest
build capability/coverage records
validate snapshot references/digests
publish immutable snapshot
snapshot-bound read view
```

Run snapshot/session/coverage validation cases.

**Gate:** one supplied project generation, deterministic snapshot digest, no old/new mix.

## Phase 9 — generic diagnostics

Implement the one probe-selected upstream diagnostic mapping:

```text
inventory diagnostic families
normalize observation
convert/validate span
map severity and rollout metadata
build wow-core Finding
canonicalize/validate findings
```

Run `EMMY-DIAG-*`.

New families remain unclassified/shadowed or reject activation.

**Gate:** clean counterpart clean; one expected generic finding exact; no message-text identity; no platform evidence.

## Phase 10 — reference and call facts

Implement only E0 fact kinds:

```text
FileFact
ReferenceFact
CallFact
FunctionScopeFact as required
AnalyzerFactSet
```

Run `EMMY-REF-*` and fact validation.

**Gate:** KnownApi resolved; RemovedApi unresolved without platform-absence conclusion; exact spans.

## Phase 11 — local binding/use/operation facts

Implement:

```text
LocalBindingFact
LocalUseFact
LocalFlowEdge for initializer/copy/conversion required by fixture
OperationFact for selected E0 operation
```

Run `EMMY-LOCAL-*`.

**Gate:** exact binding identity under shadowing, no Secret verdict/declassification semantics.

## Phase 12 — guard/control-flow slice

Implement the smallest proven function-local model for fixture cases:

```text
ControlFlowRegion
GuardFact
ControlFlowRelation
dominates when proven
precedes_without_dominance
```

Run `EMMY-GUARD-*`.

If the selected upstream public API cannot support exact dominance facts without private internals, reject/revise the pin/contract rather than guessing.

**Gate:** guarded/after-use/different-value/unrelated-branch cases distinguish correctly.

## Phase 13 — capability and failure isolation

Implement exact coverage records per session/library/file/capability.

Run `EMMY-COVER-*`, parse failure, broken library, budget, and corruption cases.

**Gate:** empty output under failed capability cannot become clean; no half snapshot after fatal failure.

## Phase 14 — incrementality

Implement only selective reuse/invalidation proven by the compatibility probe.

Run all `EMMY-INC-*` with varied update sequences.

**Gate:** identical final contents yield identical canonical outputs; removed/stale facts gone; library dependencies recomputed.

## Phase 15 — fixture/checksum freeze

Using implemented E0-A canonicalization:

1. canonicalize all E0-C example files;
2. write real SHA-256 member/bundle digests into `examples/CHECKSUMS.json`;
3. freeze upstream pin/probe report identity;
4. derive exact fixture snapshot/generation IDs;
5. change `crates/MANIFEST.json` implementation state;
6. reject any null digest after code activation.

Do not auto-rewrite expected files during normal tests.

## Phase 16 — public seam review

Expose only operations/data required by:

- E0-D `wow-project` session/snapshot/file/fact integration;
- E0-E `wow-rules` fact consumption through project snapshots;
- E0-F service generic diagnostics.

Review:

- no raw upstream types;
- no WoW authority;
- no project-generation ownership;
- no transports/editor mutation;
- no deferred operations.

Run `EMMY-SEAM-*`.

## Phase 17 — full deterministic/security run

Vary:

```text
file/update order
temp root
worker/test scheduling
diagnostic return order
hash-map insertion
LF/CRLF/multibyte sources
malicious comments/paths/large inputs
```

Require canonical identity/outputs and all mutation checks.

## Phase 18 — completion report

Report:

```text
exact upstream pin and probe decision
public adapter API
crate dependencies/features/toolchain/license
fixture workspace and checksums
all applicable test IDs/results
source coordinate report
diagnostic mapping and rollout
fact capability inventory
incremental invalidation report
security/no-execution/no-editor-mutation checks
known unavailable capabilities
rollback pin
```

## Forbidden shortcuts

Do not:

- pin a floating branch;
- treat compile success as compatibility;
- expose upstream types publicly;
- use rendered messages as diagnostic identity;
- use line/column only as source identity;
- leak absolute URIs/paths;
- infer WoW API absence or Secret status;
- own project generation;
- write a second parser to avoid upstream facts;
- load full Blizzard UI into fixture library;
- mutate editor settings;
- execute source/repository code;
- catch session corruption and publish uncertain partial data;
- reuse stale facts without invalidation proof;
- activate LSP/MCP/plugin/persistence stubs;
- change normative fixtures solely to match an easier implementation.

## Completion boundary

E0-C ends at an immutable, deterministic analyzer snapshot that exposes one generic diagnostic and the normalized E0 fact slice.

No WoW-specific rule has run yet.
