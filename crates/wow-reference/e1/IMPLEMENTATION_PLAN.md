# `wow-reference` E1-B implementation plan

**Status:** ordered handoff for a future coding agent. This documentation change adds no Rust implementation.

## Phase 0 — prerequisites and pin review

1. Confirm `wow-core` E0 implementation and canonicalization/ID/evidence/coverage contracts are frozen.
2. Confirm `wow-store` E1-A implementation/runtime/schema/publication/object contracts and fixtures are frozen.
3. Read every file in [`AGENTS.md`](AGENTS.md).
4. Audit/select exact external parser crate/revision/version/features/license/API.
5. Materialize the exact E1 source snapshot/manifest fixture.
6. Audit current KB/source routes but pin the fixture independently.
7. Confirm no duplicate APIDoc parser/reference store implementation exists.

**Gate:** no code without exact parser candidate and source/profile fixture.

## Phase 1 — minimal crate/module boundary

Create only responsibilities such as:

```text
snapshot
profile
partition
parser_adapter
evaluator
raw_value
raw_observation
field_registry
normalizer
entity_fact
restriction_predicate
correction
conflict
coverage
store_schema
store_adapter
build
view
error
fixture
```

Rules:

- direct framework dependencies exactly core/store;
- no annotations/UI graph/search/project/service/application/network;
- no generalized Lua interpreter/plugin system;
- no raw SQL/connection exposure;
- no placeholder successful deferred capability.

## Phase 2 — source snapshot/profile

Implement:

```text
validate snapshot root/path/file manifest
verify content digests/length/encoding/licenses
provider/content identity separation
build/validate ReferenceProfile
partition policy and semantic input order
fixture/candidate/release eligibility preflight
```

Freeze source/profile/partition/file IDs/digests and run `SOURCE-*`.

**Gate:** exact profile/source identity and deterministic ordered input set.

## Phase 3 — parser compatibility adapter

Implement one narrow adapter over pinned parser:

```text
parse verified file
normalize parser source identity/spans/diagnostics
expose exact syntax facts required by evaluator
apply node/depth/byte budgets
```

Freeze parser compatibility report. Run `PARSER-*`.

**Gate:** no parser recovery/upgrade/span/literal ambiguity in accepted fixture.

## Phase 4 — restricted evaluator foundation

Implement only frozen E1 subset:

```text
literals
table constructors
local bindings
known references/constants
frozen constant operators/helpers
known registration calls
unsupported construct records
budgets/cancellation
```

Run all `EVAL-*`/`EVAL-SEC-*` with side-effect markers.

**Gate:** supported values exact; every unsupported/security construct diagnosed without execution.

## Phase 5 — raw canonical model

Implement:

```text
canonical numbers/strings/tables/keys/field paths
raw value IDs/digests
registration observations
raw observations
unknown/unsupported records
source handles/evidence
raw manifests/counts/digests
```

Run `RAW-*`.

**Gate:** lossless raw round-trip, duplicate occurrence preservation, distinct missing/null/unknown/unsupported/default.

## Phase 6 — field registry and normalized entities/facts

Implement supported E1 registry and fact types incrementally from fixture/corpus:

```text
systems/callables/signatures
tables/structures/fields
events/payloads
enums/CVars
widgets/script objects/methods as supported
predicates/restriction facets
deprecations/explicit transitions
exact reference links
```

No abstraction before repeated shapes. Run `NORM-*` after each family.

**Gate:** every fact raw-linked, stable exact identity, profile isolation, conflicts explicit.

## Phase 7 — corrections

Implement:

```text
correction record/set validation
dependency graph
exact target/digest/applicability matching
supported normalized projection operations
application status/before-after/evidence
conflict/coverage propagation
review report
```

Freeze correction fixture/set IDs. Run `CORR-*`.

**Gate:** raw unchanged; mismatch expires; conflict does not first/last win.

## Phase 8 — coverage/conflict/authority

Implement:

```text
capability registry
partition construction
stage dependency graph
coverage records/summaries
unknown/unsupported/conflict/correction/truncation/runtime blockers
release capability manifest
negative-authority request/decision
```

Run `COV-*`, including mutation that removes a decisive input.

**Gate:** authoritative negative fails under every required blocker mutation.

## Phase 9 — persistent schema/operation/validation bundle

Define static registered bundle:

```text
identity/source/raw/normalized/correction/conflict/coverage/manifests
write/read prepared operation catalogs
validation closure checks
schema digest/migration edge
```

Coordinate exact bundle IDs with `wow-store` freeze. No store/domain dependency inversion.

Run `REFSTORE-SCHEMA-*`.

**Gate:** all domain records persist/read through registered operations; no raw SQL/store handle.

## Phase 10 — deterministic ReferenceStore build plan

Implement:

```text
phase-ordered record encoders/operation invocations
object write/reference plan
expected counts/digests/manifests
validation plan
budgets/cancellation/durability
```

Randomize input/order/temp/worker scheduling. Freeze build plan vectors.

**Gate:** same logical inputs -> same plan/manifests; no hidden callbacks/SQL.

## Phase 11 — `wow-store` integration

Invoke E1-A:

```text
staging schema/migration
registered writes/objects
validation/seal/publication
final-path read-only reopen
```

Validate returned store generation/manifest/profile/reference/schema/object identities. Run store publication failure/cancellation integration cases.

**Gate:** prior active unchanged on any failure; no completed ReferenceData manifest yet.

## Phase 12 — exact ReferenceView

Implement typed read adapters/results:

```text
open exact view
exact entity/callable/event/table/enum/CVar/widget/restriction/deprecation lookup
raw metadata/source handle
bounded exact listing
negative-authority decision
```

No fuzzy/search/external fallback. Run `VIEW-*`.

**Gate:** every result variant/context/evidence/coverage/truncation deterministic.

## Phase 13 — build report and ReferenceData manifest

Implement stage/count/digest/capability/store/object/license/checksum report/manifest. Validate reference closure and eligibility.

Run `BUILD-*`.

**Gate:** manifest only after store/read validation; no annotation/UI graph/search/runtime claim.

## Phase 14 — security/resource corpus

Run all `REF-SEC-*` plus evaluator/source/store/object tests:

```text
path/link/source execution/network/process/editor/client/prompt injection
huge/deep/malformed source/value/object/query
corrupt/tampered source/store/manifest/object/correction
private path/token/raw payload leak
```

**Gate:** no side effect/escape/unbounded allocation/repair/leak.

## Phase 15 — determinism and upgrade probes

Repeat 1/2/N workers, randomized file/task/record/SQL insertion/order/temp roots. Freeze canonical outputs.

Parser/evaluator/normalizer/schema/correction dependency updates rerun compatibility fixtures and retain last-known-good on mandatory regression.

## Phase 16 — deferred capability enforcement

Run `REF-DEFER-*`. Ensure no:

```text
annotation generation
full UI graph/skeleton
fuzzy search/lineage/replacement
runtime probe/current spell whitelist
final pack signing/distribution
```

Each request is typed unavailable, never empty success.

## Phase 17 — fixture/checksum freeze

Before/with first E1-B Rust commit:

1. freeze core/store prerequisite implementations/bundles;
2. freeze source/profile/parser/evaluator environment/field registry;
3. freeze raw/normalized/unknown/unsupported/conflict/correction/coverage vectors;
4. freeze reference schema/operation/validation/migration/build plan;
5. freeze store generation/manifest/open validation;
6. freeze ReferenceView positive/negative/partial/conflict/query vectors;
7. freeze build report/ReferenceData manifest/eligibility;
8. canonicalize all examples;
9. write member/bundle SHA-256 values;
10. update `CONTRACT.json` and manifest implementation state;
11. reject required null values after activation.

Tests verify fixtures; never rewrite automatically.

## Phase 18 — completion report

Report:

```text
source snapshot/profile/parser/evaluator/field/correction pins
partition/file/registration/raw/fact/unknown/unsupported/conflict counts
coverage/capability/release eligibility state
schema/operation/validation/build plan IDs
ReferenceStore generation/manifest/open IDs
ReferenceView result vectors and authority decisions
all tests/commands: pass | fail | skipped
security/no-execution/no-network/no-unknown-loss/no-store-bypass checks
deferred capabilities
```

## Forbidden shortcuts

Do not:

- execute Lua/general helpers/source code;
- use regex/model inference for unsupported source;
- drop unknown/raw/duplicates/conflicts;
- coerce unresolved type to any;
- apply correction best effort/update digest automatically;
- infer replacement/runtime safety;
- rely on filesystem/SQL/thread order;
- expose raw SQL/store connection;
- publish before store/read/manifest gates;
- weaken coverage to make negatives pass;
- activate annotations/UI graph/search/runtime/release assembly;
- change fixtures to fit easier implementation.

## Completion boundary

E1-B ends with one persistent exact ReferenceData generation and read-only ReferenceView. Annotation generation/parity, complete UI graph/skeleton/lineage/search, project indexing, runtime evidence, pack assembly/signing/distribution, and CI remain outside this work package.
