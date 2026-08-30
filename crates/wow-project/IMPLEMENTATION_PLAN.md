# `wow-project` E0-D implementation plan

**Status:** ordered handoff plan for a future coding agent. This contract change adds no Rust code.

## Phase 0 — prerequisites

1. Confirm E0-A `wow-core` implementation is merged and canonical vectors pass.
2. Confirm E0-C `wow-emmy` implementation has an accepted exact upstream pin/probe and frozen fixture bytes.
3. Read all files listed in [`AGENTS.md`](AGENTS.md).
4. Confirm E0-B fixture profile/reference generation identity selected by integration harness.
5. Confirm no competing project-generation implementation exists.
6. Confirm E0 Cargo workspace has not activated store/graph/recognizers.

**Gate:** no code starts while core/emmy public seams or fixture identities remain draft/null.

## Phase 1 — crate skeleton

Create the smallest crate with internal responsibilities such as:

```text
configuration
input_inventory
source_registry
generation
update
analyzer_binding
snapshot
publication
view
fixture
error
```

Rules:

- direct framework dependencies exactly `wow-core`, `wow-emmy`;
- no TOC/XML/store/graph/recognizer/search/service/transport modules;
- no empty future traits/APIs;
- no async runtime/background watcher unless contract explicitly changes;
- public API contains only normalized framework-owned types.

**Gate:** crate compiles with no placeholder successful operations.

## Phase 2 — project configuration and budgets

Implement:

```text
validate_project_configuration
validate_project_capability_policy
validate_project_budget_policy
canonicalize_project_configuration
```

Run `PROJECT-CONFIG-*`.

**Gate:** explicit fixture profile/reference/analyzer identities and deterministic configuration digest.

## Phase 3 — source identity and input inventory

Implement:

```text
register_project_source_origin
normalize_project_file
inventory_project_inputs
build_project_file_manifest
validate_project_source_registry
file_by_id
file_by_path
validate_project_source_handle
```

Use source bytes/digests frozen by the E0-C/E0-D fixture binding.

Run `PROJECT-INPUT-*` and `PROJECT-SOURCE-*`.

**Gate:** exact four-file Main inventory, no path/role/origin leak, temp-root independence.

## Phase 4 — project generation derivation

Implement:

```text
build_project_generation_derivation_input
derive_project_generation_id
validate_generation_candidate
classify_no_change
```

Use E0-A domain-separated canonical hash rules.

Run `PROJECT-GEN-*`.

**Gate:** equivalent final inputs -> same ID; every semantic input mutation -> different ID; volatile mutations -> unchanged.

## Phase 5 — update request and final-state builder

Implement:

```text
validate_project_update_request
validate_project_file_operation
apply_operations_to_candidate_manifest
canonicalize_final_project_state
```

Run `PROJECT-UPDATE-*` without invoking analyzer first.

**Gate:** stale/conflicting/path/budget/no-op cases resolved deterministically before analyzer mutation.

## Phase 6 — analyzer update binding

Implement only the normalized E0-C seam:

```text
build_analyzer_update_batch
apply through wow-emmy session actor
receive AnalyzerSnapshot
validate analyzer update effects
```

No raw upstream type may enter project public/internal boundary outside the `wow-emmy` public types.

Run `PROJECT-ANALYZER-*` update-batch cases.

**Gate:** exact target project generation and final Main file manifest reach analyzer.

## Phase 7 — analyzer snapshot validation

Implement:

```text
validate_analyzer_snapshot_for_project
validate_analyzer_main_file_mapping
validate_analyzer_project_source_handles
validate_analyzer_capability_binding
```

Check profile/reference/project generation, pin/probe/config, workspaces, files/digests/lengths, source origins, coverage, facts/findings IDs.

Run full `PROJECT-ANALYZER-*`.

**Gate:** every mismatch rejects publication.

## Phase 8 — project coverage and deferred capabilities

Implement exact project-owned coverage records:

```text
configuration
file inventory
source registry
generation coherence
analyzer snapshot binding
per-file analyzer fact/generic finding availability
```

Implement typed deferred E2 records:

```text
TOC/XML/load/state/event/hook/graph/store = NotEvaluated/unavailable
```

Run `PROJECT-PUB-005..009`, `PROJECT-DEFER-*`.

**Gate:** empty failed/unimplemented capability cannot look complete.

## Phase 9 — snapshot assembly and validation

Implement:

```text
assemble_project_snapshot
validate_project_snapshot
canonicalize_project_snapshot
```

Run `PROJECT-PUB-*` candidate/digest/reference tests.

**Gate:** one immutable candidate fully validates before publication.

## Phase 10 — atomic publication/current pointer

Implement:

```text
publish_project_snapshot
require_current_project_generation
retain_last_known_good_snapshot
abort_project_publication
```

Current pointer/registry implementation remains private and deterministic.

Run `PROJECT-PUB-*`, `PROJECT-LKG-*`, `PROJECT-FAIL-*`.

**Gate:** no half/current-pointer mutation on failure/cancellation; old snapshot never relabeled.

## Phase 11 — immutable ProjectView

Implement narrow read surface:

```text
open_project_view
project_snapshot_identity
project_configuration
file_manifest/source registry
file_by_id/path
analyzer_snapshot_identity
analyzer facts for file/capability
analyzer generic findings
project coverage/deferred capabilities
```

Run `PROJECT-VIEW-*` and seam tests.

**Gate:** no mutable/raw analyzer/project state, no diagnostics/platform conclusions.

## Phase 12 — successful update effects

Implement/verify:

- update generic-error to clean;
- update missing-api to KnownApi;
- add/remove one optional synthetic Main Lua file;
- no-op;
- independent operation-order permutations.

Run `PROJECT-EFFECT-*`, remaining update/incremental tests.

**Gate:** exact new generation/snapshot effects and old-snapshot isolation.

## Phase 13 — fixture/pin/checksum freeze

Before or with first implementation commit:

1. import accepted E0-C upstream pin/probe/config identity;
2. freeze selected E0-B profile/reference generation;
3. freeze Main file canonical bytes/digests/lengths shared with E0-C fixture;
4. derive exact baseline/update project-generation IDs;
5. canonicalize all E0-D example files;
6. write actual SHA-256 member/bundle digests;
7. update `CONTRACT.json` and manifest implementation state;
8. reject null freeze fields after activation.

Do not auto-rewrite expected files in normal tests.

## Phase 14 — deterministic/security/mutation review

Vary:

```text
input/update order ending in same state
temporary root
worker scheduling
hash-map insertion
analyzer fact/finding order
stale generation/digest
path/source-role attacks
large/budget inputs
analyzer failure/mismatch/cancellation
```

Deliberately break:

```text
generation derivation input coverage
source-handle generation validation
analyzer file-manifest match
atomic publication ordering
last-known-good identity
failed capability reporting
deferred capability status
```

All mutation tests must fail for the intended reason.

## Phase 15 — public seam review

Consumers:

- `wow-rules`: immutable project view/facts/source/generation only;
- `wow-service`: current/explicit project snapshot acquisition/status/check inputs;
- `apps/wow`: only through service, never direct project internals.

Review:

- no reference/platform authority;
- no rule/diagnostic algorithm;
- no raw upstream types;
- no store/graph/TOC/XML capability;
- no implicit current/latest project/profile;
- no source execution/editor mutation.

Run `PROJECT-SEAM-*`.

## Phase 16 — completion report

Report:

```text
crate dependencies/public API
project fixture/config/source registry identity
selected profile/reference generation
accepted analyzer pin/probe/config/snapshot
ProjectGenerationId derivation inputs/vectors
baseline and update publication results
all applicable test IDs/results
coverage/deferred capability states
source security/privacy tests
atomic publication/last-known-good behavior
canonical byte/digest determinism
known NotEvaluated capabilities
```

## Forbidden shortcuts

Do not:

- derive generation from timestamp/counter/debug representation;
- let analyzer own project generation;
- publish before analyzer validation;
- expose candidate as current;
- relabel last-known-good;
- merge old/new facts/files;
- scan/watch filesystem;
- add TOC/XML/store/graph/recognizers;
- execute source/repository code;
- mutate editor settings;
- expose raw analyzer internals;
- create API/Secret/rule conclusions;
- return empty success for deferred capabilities;
- change fixtures merely to match an easier implementation.

## Completion boundary

E0-D ends at one coherent immutable `ProjectSnapshot` and one deterministic update/publication path. No TOC, XML, load graph, project graph, or WoW rule has been implemented here.
