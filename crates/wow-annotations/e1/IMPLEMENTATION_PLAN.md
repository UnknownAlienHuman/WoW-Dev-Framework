# `wow-annotations` E1-C implementation plan

**Status:** ordered handoff for a future coding agent. This documentation change adds no Rust code.

## Phase 0 — prerequisites and primary-source pin review

1. Confirm `wow-core` and `wow-reference` E1 implementations/fixtures are frozen.
2. Read all E1-C contracts and current KB routing.
3. Pin exact current Ketho oracle revision/source/config/artifact generation path used for the selected fixture.
4. Pin exact EmmyLua and LuaLS consumer revisions/versions/features/config probe identities.
5. Audit licenses and current supported annotation syntax/behavior from primary sources.
6. Decide whether one shared artifact profile can satisfy both consumers or whether separate profiles are required.
7. Freeze one exact ReferenceView/profile/reference generation fixture.

**Gate:** no code without exact ReferenceView, oracle, consumer, and renderer/profile candidates.

## Phase 1 — minimal crate boundary

Create only responsibilities such as:

```text
request
semantic_profile
semantic_model
type_model
type_lowering
dialect
layout
renderer
sanitization
source_map
loss
manifest
parity
consumer_profile
error
fixture
```

Rules:

- framework dependencies exactly `wow-core`, `wow-reference`;
- no filesystem/process/network/editor/store/project/search/service/app dependency;
- no generic plugin/template language;
- no placeholder success for deferred capabilities;
- no Ketho/editor behavior copied wholesale.

## Phase 2 — build request and input closure

Implement:

```text
validate AnnotationBuildRequest
open/validate exact ReferenceView
validate all profile IDs/versions/capabilities/budgets
select exact reference facts by semantic profile
record partial/conflict/NotEvaluated inputs
```

Run `ANN-CONFIG-*`.

**Gate:** one exact profile/reference context and no fallback.

## Phase 3 — semantic profile and model foundation

Implement:

```text
module/declaration/member/type/doc/restriction/status/link identities
semantic ordering
reference projection links
semantic manifest/counts/digests
closure validation
```

Start with fixture-active API functions, named structures, events, enums, widgets, globals, and nominal Secret module only as proven.

Run `SEM-*` incrementally.

**Gate:** every selected input has status; every output has exact input/derivation.

## Phase 4 — type lowering

Implement one frozen rule at a time:

```text
primitives/literals/named types
optional/nil/default/missing
arrays/maps/tuples/unions
multiple returns
callbacks/functions/variadics
enums/widgets/script objects
restriction/Secret projections
unsupported/loss statuses
```

Do not add advanced type abstractions without source+consumer fixtures.

Run `TYPE-*` under both consumer profiles.

**Gate:** no silent any/omission/collapse; structural type graph deterministic.

## Phase 5 — dialect and global library

Implement exact profile-bound:

```text
standard allowed/removed globals
Blizzard globals/namespaces
require-like/nonstandard symbols
restricted/private globals sidecars
nominal Secret analysis types
```

Run `DIALECT-*` and config mutation audit.

**Gate:** no editor/default/installed-addon/global leakage and no profile union.

## Phase 6 — layout/rendering profile

Implement:

```text
logical module -> deterministic file paths
inert declaration templates
safe identifiers/generated names
canonical strings/literals
UTF-8 LF formatting/order/splitting
rendered fragment recording
file manifest
```

Freeze Ketho-compatible layout profile from reviewed primary-source fixture rather than memory.

Run `RENDER-*`.

**Gate:** final files parse under consumer-compatible parser, contain only allow-listed inert forms, and are byte-identical under 1/2/N workers.

## Phase 7 — documentation sanitization and security

Implement the frozen sanitization profile before enabling source docs:

```text
directive/comment/string/control/Unicode/path neutralization
per-fragment/declaration/file/artifact budgets
loss/source-map records
post-render code-shape scanner
privacy/path checks
```

Run all `SAN-*` mutation markers.

**Gate:** source docs/names/strings cannot add code/directives/files/globals or leak private data.

## Phase 8 — generated source maps

Implement spans after final bytes:

```text
file/fragment IDs
UTF-8 byte/line ranges
semantic/reference/raw/correction/evidence/source links
lowering/rendering/sanitization/status/loss links
map/file digest closure
```

Run `MAP-*` including byte mutation/stale map test.

**Gate:** every material fragment is traceable; no map drift or cross-artifact link.

## Phase 9 — projection coverage and loss

Implement:

```text
projection capability/partition registry
Exact/ExactWithSidecar/LossyDeclared/Unsupported/NotEvaluated records
loss categories/severity/policy
sidecar/loss/coverage manifests
artifact eligibility
```

Run `LOSS-*`, deliberately removing each required record to prove closure.

**Gate:** selected unknown/unsupported/conflict/partial/reference input cannot disappear.

## Phase 10 — semantic/file/artifact manifests

Build and validate:

```text
semantic manifest
file manifest
source-map manifest
projection coverage/loss sidecars
consumer manifest
artifact manifest/checksums/eligibility
```

No parity/consumer success implied yet.

**Gate:** exact counts/digests/profile/reference/profile IDs close; cancellation yields no complete manifest.

## Phase 11 — Ketho semantic baseline and comparison

External tool adapter:

1. materializes pinned equivalent Ketho source/output without executing arbitrary repo content;
2. extracts a typed oracle semantic model using a frozen parser/extractor;
3. validates baseline/source/profile equivalence;
4. feeds baseline to library comparison operations.

Implement comparison/classification/report. Run `PARITY-*`.

**Gate:** every discrepancy classified with evidence; no artifact/source auto-overwrite.

## Phase 12 — EmmyLua consumer probe

External isolated adapter runs pinned EmmyLua consumer over:

```text
artifact
positive fixtures
negative fixtures
explicit generated test config
no-mutation audit
```

Validate result via library contract. Run Emmy-related `CONS-*`.

**Gate:** types/signatures/source spans/negative diagnostics pass; no config mutation/suppression.

## Phase 13 — LuaLS consumer probe

Repeat exact process with pinned LuaLS. Compare consumer behavior.

If mandatory semantics disagree:

- fix defect if shared syntax should work;
- otherwise define separate explicit consumer profile/artifact;
- do not erase semantics globally.

**Gate:** declared shared/consumer-specific strategy proven.

## Phase 14 — security/resource/performance baseline

Run source injection, path/identifier collision, huge/deep type/docs/artifact, malicious oracle/probe output, privacy, timeout/memory/output cases. Record exact artifact size/load/index baseline with hardware/runtime/corpus.

**Gate:** bounded and no side effects/leaks; performance baseline not an unproven release gate.

## Phase 15 — determinism

Repeat build/parity validation under:

```text
1/2/N workers
shuffled semantic input serialization/store row order
changed temp roots/hosts/locales/times
changed independent probe order
```

Expect identical semantic/file/source-map/loss/artifact/parity semantic digests. Performance/timing supplemental.

## Phase 16 — deferred capability enforcement

Run `ANN-DEFER-*`. Ensure typed unavailable for:

```text
complete UI graph/FrameXML/skeletons
search/lineage/replacement
runtime spell/hotfix probes
library-owned output filesystem writes
final pack assembly/signing/distribution/CI
```

No empty artifacts/success.

## Phase 17 — fixture/checksum freeze

Before/with first E1-C Rust commit:

1. freeze core/reference prerequisite implementations/fixtures;
2. freeze exact ReferenceView/profile/reference input;
3. freeze Ketho oracle and EmmyLua/LuaLS consumer identities/configs;
4. freeze semantic/type/layout/dialect/docs/source-map/loss/consumer profiles;
5. freeze semantic model/type lowering/source-link/loss vectors;
6. freeze rendered file bytes/digests and source maps;
7. freeze parity baseline/records/report;
8. freeze consumer positive/negative/mutation results;
9. freeze artifact manifest/eligibility;
10. canonicalize examples and write member/bundle SHA-256 values;
11. update `CONTRACT.json` and implementation state;
12. reject required null values after activation.

Tests verify fixtures; never rewrite automatically.

## Phase 18 — completion report

Report:

```text
ReferenceView/profile/reference input
Ketho/EmmyLua/LuaLS pins/configs
semantic/type/layout/dialect/docs/source-map/loss/consumer profile IDs
module/declaration/member/type/status/loss/file/map counts
artifact/file/map/loss/parity/probe IDs and checksums
parity classifications/unresolved blockers
consumer positive+negative assertions and config mutation audit
all tests/commands: pass | fail | skipped
security/sanitization/resource checks
deferred capabilities
```

## Forbidden shortcuts

Do not:

- render directly from store rows/ad hoc requests;
- use `any`/omission to make consumer green;
- collapse optional/nil/unknown/restriction semantics;
- interpolate source docs/names/literals into syntax/paths;
- copy Ketho editor settings/diagnostic suppression;
- treat oracle output as source truth;
- run consumers/oracles inside library crate;
- depend on filesystem/process/network/editor/store/project/search/service;
- publish before file/map/loss/security/parity/consumer gates;
- alter fixtures to match easier output;
- activate final pack/release/CI work.

## Completion boundary

E1-C ends with deterministic profile-bound annotation artifact(s), source maps, loss/coverage manifests, reviewed Ketho semantic parity, and pinned EmmyLua/LuaLS compatibility results. Complete UI graph/search/runtime evidence/final pack assembly and release remain later work.
