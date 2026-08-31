# E3-B renumbering and integration test supplement

**Status:** normative. This supplements the inherited `TEST_MATRIX.md`; it does not create a second context implementation.

The inherited matrix continues to cover the full Project Map, L0/L1, control/effect, expansion, source, budget, renderer, metrics, and evaluation behavior. This supplement freezes the corrected milestone assignment, current type/operation names, exact E3-A platform-source prerequisite, semantic-pack/cache model, and final implementation gate.

## Milestone, schema, and aliases

| ID | Case | Expected |
|---|---|---|
| `CTX-REN-001` | Current E3-B contract and work-package IDs | pass |
| `CTX-REN-002` | Historical E3-A context label parsed through renumbering map | same semantic behavior; current ID |
| `CTX-REN-003` | Historical and current canonical types implemented separately | reject `context_legacy_and_current_type_both_activated` |
| `CTX-REN-004` | Historical and current operation names implemented separately | reject `context_legacy_and_current_operation_both_activated` |
| `CTX-REN-005` | Old source-producer boundary selected instead of wow-project E3-A | reject |
| `CTX-REN-006` | Unknown contract/profile schema version | reject |
| `CTX-REN-007` | Breaking profile change without version bump | reject |
| `CTX-REN-008` | Identity DAG contains semantic-pack to renderer backreference | reject |
| `CTX-REN-009` | Validation report ID included in semantic-pack hash core | reject |
| `CTX-REN-010` | Renaming historical labels only | same semantics except versioned IDs |
| `CTX-REN-011` | Current router/contract routes all exist | pass |
| `CTX-REN-012` | Directory exists but implementation state remains not-started | pass; no activation |

## Universe binding and input views

| ID | Case | Expected |
|---|---|---|
| `CTX-IN-001` | Exact primary project/graph/reference bind | pass |
| `CTX-IN-002` | Primary project and graph from different publications | reject generation mismatch |
| `CTX-IN-003` | Optional exact Blizzard UI source bind from wow-project E3-A | pass |
| `CTX-IN-004` | Required platform source omitted | reject or explicit partial by profile |
| `CTX-IN-005` | Optional platform source omitted | explicit omission and coverage impact |
| `CTX-IN-006` | Floating current resolved inside wow-context | reject |
| `CTX-IN-007` | Current advances after universe bind | existing operation remains on old exact views |
| `CTX-IN-008` | Same display name in user and platform universe | distinct identities |
| `CTX-IN-009` | Same path in two generations | distinct identities |
| `CTX-IN-010` | Name/path-only cross-universe join | reject |
| `CTX-IN-011` | ReferenceProfile incompatible with project build/flavor | reject |
| `CTX-IN-012` | Platform SkeletonInputView incompatible with project/graph generation | reject |
| `CTX-IN-013` | Owner returns record from another universe | reject |
| `CTX-IN-014` | Owner returns record from another generation | reject |
| `CTX-IN-015` | Source handle digest mismatch | reject |
| `CTX-IN-016` | Virtual source map missing | partial or reject required facet |
| `CTX-IN-017` | Raw store connection supplied | reject |
| `CTX-IN-018` | Raw analyzer session supplied | reject |
| `CTX-IN-019` | StoreImageId supplied | reject |
| `CTX-IN-020` | Required owner capability failed | fail or explicit partial by request policy |
| `CTX-IN-021` | Upstream graph view truncated | context coverage partial |
| `CTX-IN-022` | Reference conflict affects requested API fact | preserve conflict; no exact claim |
| `CTX-IN-023` | Input view contains unexpected extra raw handle | reject |
| `CTX-IN-024` | Cancellation during binding | cancelled; no background work |

## Requests and profiles

| ID | Case | Expected |
|---|---|---|
| `CTX-REQ-001` | Exact entity root with compatible intent | pass |
| `CTX-REQ-002` | Exact package/file/map/skeleton/finding roots | pass by allowed intent |
| `CTX-REQ-003` | Fuzzy symbol name as root | reject |
| `CTX-REQ-004` | Natural-language query as semantic selector | reject |
| `CTX-REQ-005` | Regex/SQL/expression/callback selector | reject |
| `CTX-REQ-006` | Root outside bound universe | reject |
| `CTX-REQ-007` | Root kind incompatible with intent | reject |
| `CTX-REQ-008` | Unknown facet/axis/relation kind | reject |
| `CTX-REQ-009` | Request asks to promote Possible to Proven | reject |
| `CTX-REQ-010` | Request asks to include denied source | reject/omission |
| `CTX-REQ-011` | Unknown profile field ignored | mutation fails; reject |
| `CTX-REQ-012` | Equivalent request field order | same normalized ID |
| `CTX-REQ-013` | Different privacy profile | different request and cache IDs |
| `CTX-REQ-014` | Different tokenizer/framing profile | different render target identity |
| `CTX-REQ-015` | Unlimited/negative/overflowing budget | reject |
| `CTX-REQ-016` | Opaque audit text changes | does not alter semantic selection ID unless audit profile explicitly includes it |
| `CTX-REQ-017` | Source-controlled profile supplied | reject |
| `CTX-REQ-018` | Renderer requests absent semantic facts | reject or explicit new semantic request |

## Semantic pack and rendering

| ID | Case | Expected |
|---|---|---|
| `CTX-PACK-001` | Valid ContextSemanticPack closure | pass |
| `CTX-PACK-002` | Historical ContextBundleCore emitted separately | reject |
| `CTX-PACK-003` | Dangling item/evidence/source/reference ref | reject |
| `CTX-PACK-004` | Mixed generation item | reject |
| `CTX-PACK-005` | Canonical item ordering | pass |
| `CTX-PACK-006` | Semantic pack includes renderer ID in hash core | reject |
| `CTX-PACK-007` | Semantic pack includes metrics/evaluation ID in hash core | reject |
| `CTX-PACK-008` | Canonical JSON roundtrip | same pack ID |
| `CTX-PACK-009` | JSON object/array order mutation | canonical bytes unchanged where semantically set-ordered |
| `CTX-PACK-010` | Markdown typed template facts | pass |
| `CTX-PACK-011` | Markdown free generated claim | reject |
| `CTX-PACK-012` | Path rendered as direct relation | reject |
| `CTX-PACK-013` | Coverage/conflict/omission labels hidden | reject |
| `CTX-PACK-014` | Source structural boundary validates | pass |
| `CTX-PACK-015` | Renderer item/range mapping closes | pass |
| `CTX-PACK-016` | Rendering loss declared and permitted | pass |
| `CTX-PACK-017` | Required trust field lost in compact renderer | reject |
| `CTX-PACK-018` | Validator repairs mismatched artifact | reject |
| `CTX-PACK-019` | Multiple renderer artifacts from same pack | distinct artifact IDs |
| `CTX-PACK-020` | Renderer-specific semantic replan without new request | reject |
| `CTX-PACK-021` | Existing finding evidence exact generation | pass |
| `CTX-PACK-022` | Rendered bytes vary by host/clock/temp root | mutation fails |

## Cache, comparison, metrics, evaluation

| ID | Case | Expected |
|---|---|---|
| `CTX-CACHE-001` | Exact semantic cache key closure | pass |
| `CTX-CACHE-002` | Floating current/path/name-only cache key | reject |
| `CTX-CACHE-003` | Exact cache hit schema/digest/profile match | pass |
| `CTX-CACHE-004` | Cross-generation relabel | reject |
| `CTX-CACHE-005` | Cross-privacy consumer reuse | reject |
| `CTX-CACHE-006` | Cross-tokenizer/renderer reuse | reject |
| `CTX-CACHE-007` | Partial/cancelled artifact used as complete | reject |
| `CTX-CACHE-008` | Corrupted cache bytes | miss/reject |
| `CTX-CACHE-009` | Physical cache I/O implemented in wow-context | architecture test fails |
| `CTX-CACHE-010` | Cold/warm cache semantic outputs | identical |
| `CTX-CACHE-011` | Content-addressed subrecord reuse with exact dependencies | pass with retained identity |
| `CTX-CACHE-012` | Same text/name heuristic reuse | reject |
| `CTX-CACHE-013` | Pack comparison equivalent semantics | classification |
| `CTX-CACHE-014` | Renderer-only difference | classification |
| `CTX-CACHE-015` | Input/profile/coverage/budget difference | classified before semantic diff |
| `CTX-CACHE-016` | Operational timing/cache hit enters semantic identity | reject |
| `CTX-CACHE-017` | Frozen evaluation hard gates execute | pass/fail recorded |
| `CTX-CACHE-018` | Evaluation score changes semantic pack selection | reject |

## Determinism and integration corpora

| ID | Case | Expected |
|---|---|---|
| `CTX-DET-001` | 1/2/N workers | same semantic and rendered bytes |
| `CTX-DET-002` | Shuffled owner query batches | same outputs |
| `CTX-DET-003` | Different SQLite physical layout/checkpoint state | same logical outputs |
| `CTX-DET-004` | Different host/temp/absolute roots | same semantic outputs; private roots absent |
| `CTX-DET-005` | Cold versus warm external cache | same outputs |
| `CTX-DET-006` | Equivalent request JSON ordering | same request/pack IDs |
| `CTX-DET-007` | Repeated duplicate evidence/text | stable dedup with all reasons |
| `CTX-DET-008` | Tiny synthetic corpus | golden pass |
| `CTX-DET-009` | High-fanout/cycle synthetic corpus | golden pass |
| `CTX-DET-010` | Pinned roth-ui publication fixture | golden pass |
| `CTX-DET-011` | Pinned E3-A Blizzard UI source fixture | golden pass |
| `CTX-DET-012` | Repository/package/path/name mutations | universal semantics stable |
| `CTX-DET-013` | Privacy/license/boundary corpus | golden pass |
| `CTX-DET-014` | Canonical JSON and Markdown rebuild comparison | pass |

## Freeze and implementation gate

| ID | Case | Expected |
|---|---|---|
| `CTX-FIX-001` | Null pins while implementation_state=not-started | allowed |
| `CTX-FIX-002` | First Rust commit with required nulls | reject |
| `CTX-FIX-003` | Prerequisite implementation commit missing | reject |
| `CTX-FIX-004` | E3-A platform-source fixture missing for platform profile | reject/explicit user-only slice |
| `CTX-FIX-005` | Profile/schema/catalog IDs not frozen | reject |
| `CTX-FIX-006` | Expected artifact/error/continuation vectors not frozen | reject |
| `CTX-FIX-007` | Tokenizer exact vectors missing for hard token profile | reject |
| `CTX-FIX-008` | Benchmarks/evaluation not run but reported pass | reject |
| `CTX-FIX-009` | Member/bundle checksums incomplete | reject |
| `CTX-FIX-010` | Cargo.toml or .rs placeholder before gate | repository test fails |
| `CTX-FIX-011` | CI/workflow introduced in documentation-only package | repository test fails |
| `CTX-FIX-012` | Tests rewrite fixtures automatically | reject |

## Fixture bridge cases

| ID | Case | Expected |
|---|---|---|
| `CTX-EXP-021` | Continuation changes budget, tokenizer, or privacy profile | reject |
| `CTX-COV-001` | Every material claim has origin closure | pass |
| `CTX-COV-004` | Possible/Candidate authority upgrade | reject |
| `CTX-COV-009` | Conflict majority or last-write resolution | reject |
| `CTX-COV-013` | Budget-pruned candidate | explicit omission record |
| `CTX-COV-014` | Unenumerated query region | distinct from pruned |
| `CTX-SRC-010` | License denies source redistribution | omit bytes; preserve allowed metadata |
| `CTX-SRC-016` | Source contains framework-like delimiter text | remains quoted source data |
| `CTX-BUD-013` | Only a model display name identifies tokenization | token count is not exact |

## Acceptance

All cases here and in `TEST_MATRIX.md` and `IDENTITY_DAG_TESTS.md` must pass before E3-B can be complete. Historical aliases may be decoded through one migration layer, but no duplicate type, schema, operation, or runtime path is permitted.
