# E5-A calibration test matrix

**Status:** normative executable acceptance, leakage, mutation, security, and determinism matrix.

Every implementation case binds exact pack/corpus/split/run/profile identities. Missing prerequisite implementation, corpus materialization, reviewer authorization, holdout infrastructure, benchmark, threshold, or checksum is `NotEvaluated`/blocked, never pass.

## Configuration and dependency boundaries

| ID | Case | Expected |
|---|---|---|
| `CAL-CONFIG-001` | Valid E5-A profile bundle over frozen E2-B matcher | accepted |
| `CAL-CONFIG-002` | Direct dependency remains core/emmy/graph only | architecture pass |
| `CAL-CONFIG-003` | Direct project/store/search/context/rules/service/app import | rejected |
| `CAL-CONFIG-004` | Floating `main`, tag, current, or latest in canonical corpus identity | rejected |
| `CAL-CONFIG-005` | Mixed fact/graph/profile/generation identities | rejected |
| `CAL-CONFIG-006` | E5-A candidate has `core` trust or default rollout | rejected |
| `CAL-CONFIG-007` | Null prerequisite/profile/checksum at first Rust commit | freeze failure |
| `CAL-CONFIG-008` | Missing E2-B operation represented as successful stub | rejected |
| `CAL-CONFIG-009` | E5-B reviewer authorization simulated inside E5-A | rejected/NotEvaluated |
| `CAL-CONFIG-010` | E5-C publication/rollback fields in E5-A effect path | rejected |
| `CAL-CONFIG-012` | Unknown optional profile field | preserve/reject per schema; never ignore silently |

## Candidate source and materialization

| ID | Case | Expected |
|---|---|---|
| `CAL-SOURCE-001` | Exact repository commit/tree and bounded roots | candidate validation succeeds |
| `CAL-SOURCE-002` | Branch/tag resolves now but no exact commit/tree in identity | reject |
| `CAL-SOURCE-003` | Commit exists but source tree/content manifest absent | admission blocked |
| `CAL-SOURCE-004` | Exact project/analyzer/graph/fact publication closure | eligible for further admission |
| `CAL-SOURCE-005` | Publication/fact generation mismatch | reject |
| `CAL-SOURCE-007` | Unsupported source files silently omitted | reject |
| `CAL-SOURCE-009` | Repository hook/filter/workflow/build/test executed | hard failure |
| `CAL-SOURCE-012` | Source handle digest/span mismatch | reject |
| `CAL-SOURCE-013` | New repository commit reuses old candidate-source identity | reject/new identity required |
| `CAL-SOURCE-017` | Raw source used as matcher fallback | reject |

## Corpus, provenance, license, and privacy

| ID | Case | Expected |
|---|---|---|
| `CAL-CORPUS-001` | Valid immutable corpus manifest with exact members | accepted |
| `CAL-CORPUS-002` | Corpus identity includes pack output/result | reject identity cycle |
| `CAL-CORPUS-004` | Quarantined member included as admitted | reject |
| `CAL-CORPUS-005` | Quarantined member used as clean Negative | reject |
| `CAL-CORPUS-006` | Same repository files counted as independent observations | reject independence claim |
| `CAL-CORPUS-009` | Provenance independence unknown | explicit Unknown; generalization scope reduced |
| `CAL-CORPUS-010` | Current common owner used as proof of same origin | reject inference |
| `CAL-CORPUS-011` | Different names used as proof of independence | reject inference |
| `CAL-CORPUS-014` | Corpus update silently mutates old bytes/labels | reject |
| `CAL-CORPUS-017` | Synthetic closed fixture has explicit synthetic class | valid; not ecosystem generalization evidence |

## Labels and independent review

| ID | Case | Expected |
|---|---|---|
| `CAL-LABEL-001` | Positive universal entity/relation with exact evidence | accepted |
| `CAL-LABEL-002` | Negative with complete closed scope | accepted |
| `CAL-LABEL-003` | Negative with Partial/Unknown/Failed/Truncated scope | reject or NotEvaluated |
| `CAL-LABEL-004` | Possible dynamic target | only Possible acceptable |
| `CAL-LABEL-005` | Required capability absent | `NotEvaluated` label/outcome |
| `CAL-LABEL-006` | Truth cannot be established | `Unknown`, excluded from TP/FP/FN/TN |
| `CAL-LABEL-007` | Reviewers support incompatible labels | explicit Conflict |
| `CAL-LABEL-008` | Majority/last reviewer silently wins | reject |
| `CAL-LABEL-010` | Label copied from current recognizer output | hard failure |
| `CAL-LABEL-020` | Missing durable reviewer authorization in E5-A | blocked/NotEvaluated, not fabricated |

## Splits, leakage, and holdout

| ID | Case | Expected |
|---|---|---|
| `CAL-SPLIT-001` | Valid explicit group assignment | accepted |
| `CAL-SPLIT-002` | Random per-file percentage split | reject canonical split |
| `CAL-SPLIT-003` | Fork/upstream component crosses Train/Test | hard leakage failure |
| `CAL-SPLIT-006` | Mutation child crosses split from parent | reject unless explicit challenge profile |
| `CAL-SPLIT-007` | Multiple versions of one project cross ordinary splits | reject/one provenance group |
| `CAL-SPLIT-009` | Unknown duplicate candidate ignored | independence claim blocked |
| `CAL-SPLIT-012` | Consumed test rerun described as untouched | reject |
| `CAL-SPLIT-013` | Holdout membership/labels visible before candidate freeze | hard failure |
| `CAL-SPLIT-016` | Split membership changes under same ID | reject |
| `CAL-SPLIT-022` | Sealed holdout infrastructure unavailable | NotEvaluated/blocked, not pass |

## Calibration-pack schema and universal semantics

| ID | Case | Expected |
|---|---|---|
| `CAL-PACK-001` | Valid E2-B declarative pack with calibration/shadow metadata | accepted |
| `CAL-PACK-002` | New operator language in E5-A | reject |
| `CAL-PACK-003` | Regex/glob/expression/script/template/include/plugin | reject |
| `CAL-PACK-004` | Repository owner/name/URL condition | hard failure |
| `CAL-PACK-005` | Addon display name used as donor identity condition | hard failure |
| `CAL-PACK-006` | Incidental path substring condition | hard failure |
| `CAL-PACK-010` | Exact public convention literal justified and mutated | accepted |
| `CAL-PACK-013` | Existing registered universal module role | graph validation may accept |
| `CAL-PACK-018` | DonorLocal candidate marked promotion eligible | reject |
| `CAL-PACK-021` | Pack modifies E2 core pack/graph registry | reject |

## Shadow execution and case results

| ID | Case | Expected |
|---|---|---|
| `CAL-RUN-001` | Valid visible split and exact fact snapshots | shadow run starts |
| `CAL-RUN-002` | Hidden holdout labels supplied to matcher | hard failure |
| `CAL-RUN-003` | One valid Positive | TruePositive case result |
| `CAL-RUN-006` | No proposal in meaningful closed Negative scope | TrueNegative |
| `CAL-RUN-007` | Expected Possible emitted Possible | ExpectedPossible |
| `CAL-RUN-008` | Expected NotEvaluated remains NotEvaluated | ExpectedNotEvaluated |
| `CAL-RUN-009` | Unknown case counted as Negative | reject report |
| `CAL-RUN-011` | Greedy first match hides duplicate/ambiguity | reject |
| `CAL-RUN-012` | Partial/truncated lane marked complete | reject |
| `CAL-RUN-014` | Late work after cancellation | hard failure |

## Independent graph validation

| ID | Case | Expected |
|---|---|---|
| `CAL-GRAPH-001` | Valid registered universal proposal | accepted receipt |
| `CAL-GRAPH-002` | Unknown graph kind/relation/attribute | rejected proposal |
| `CAL-GRAPH-003` | Invalid endpoint/direction/key ingredients | rejected proposal |
| `CAL-GRAPH-004` | Proposal constructs final graph ID/generation | reject architecture |
| `CAL-GRAPH-005` | Graph rejection omitted from metrics | invalid report |
| `CAL-GRAPH-006` | Graph validator unavailable | NotEvaluated/blocked |
| `CAL-GRAPH-007` | Repetition across donors upgrades confidence | reject |
| `CAL-GRAPH-008` | Candidate metadata appears in semantic key | reject |
| `CAL-GRAPH-009` | Possible output relabeled Derived by graph adapter | reject |
| `CAL-GRAPH-010` | Core and calibration proposals overlap | retain separate producer partitions/report conflict |
| `CAL-GRAPH-011` | Graph registry changes without pack/version/profile update | reject incompatibility |

## Mutation and anti-overfitting

| ID | Case | Expected |
|---|---|---|
| `CAL-MUT-001` | Rename repository | identical semantic outputs |
| `CAL-MUT-002` | Transfer/change owner metadata | identical semantic outputs |
| `CAL-MUT-005` | Move/rename source paths preserving facts | identical semantic outputs |
| `CAL-MUT-007` | Rename irrelevant local variables/helpers | identical semantic outputs |
| `CAL-MUT-011` | Change justified convention literal | exact match/output delta |
| `CAL-MUT-012` | Remove resolved callee/receiver join | match disappears/downgrades |
| `CAL-MUT-014` | Exact target becomes dynamic/ambiguous | Derived -> Possible/NotEvaluated |
| `CAL-MUT-015` | Complete coverage becomes Partial | no clean Negative/complete no-match |
| `CAL-MUT-019` | Swap provenance metadata while facts fixed | outputs follow facts |
| `CAL-MUT-023` | Adversarial fanout exceeds profile | deterministic Partial/failure |

## Metrics, gates, baselines, and comparison

| ID | Case | Expected |
|---|---|---|
| `CAL-METRIC-001` | Immutable per-case results precede aggregate | valid |
| `CAL-METRIC-002` | Aggregate-only report | reject |
| `CAL-METRIC-003` | Precision/recall without denominator/profile | reject |
| `CAL-METRIC-004` | Unknown/NotEvaluated/Conflict exclusions hidden | reject |
| `CAL-METRIC-005` | Files/calls/mutations counted as independent repositories | reject |
| `CAL-METRIC-006` | Large donor weighting hides smaller-group FP | hard failure remains |
| `CAL-METRIC-009` | Graph rejection hidden or nonzero where zero required | candidate blocked |
| `CAL-METRIC-011` | Threshold selected after observing candidate to admit it | reject contamination |
| `CAL-METRIC-012` | Threshold/label/split changes under same run ID | reject |
| `CAL-METRIC-018` | All hard gates pass but E5-B review absent | at most PromotionEligibleByMetrics |

## Candidate artifact, partitions, supersession, and deactivation

| ID | Case | Expected |
|---|---|---|
| `CAL-PART-001` | Exact pack/rule/input/profile shadow partition key | valid |
| `CAL-PART-002` | Wall clock/path/row ID in partition identity | reject |
| `CAL-PART-005` | Newest/first/last output selected implicitly | reject |
| `CAL-PART-006` | Disable candidate enumerates exact owned active partitions | valid plan |
| `CAL-PART-007` | Disable deletes core/foreign partition | hard failure |
| `CAL-PART-008` | Disable leaves stale active candidate output | hard failure |
| `CAL-PART-009` | Historical case/decision evidence destroyed | reject unless allowed tombstone |
| `CAL-PART-010` | Coverage downgrade omitted or overclaims absence | reject |
| `CAL-PART-011` | Core/foreign partition digests unchanged | required pass |
| `CAL-PART-012` | Candidate artifact built with unresolved hard gate | reject |
| `CAL-PART-014` | E5-A artifact presented as promoted core pack | reject |

## Security and resource limits

| ID | Case | Expected |
|---|---|---|
| `CAL-SEC-001` | Lua/JS/Wasm/native/shell/SQL payload | reject/never execute |
| `CAL-SEC-002` | Include/URL/environment/template/plugin/callback | reject |
| `CAL-SEC-004` | Huge provenance graph | bounded Partial/failure |
| `CAL-SEC-005` | Huge/deep pack/clause graph | bounded reject |
| `CAL-SEC-009` | Oversized source/label/reviewer/report strings | reject/redact |
| `CAL-SEC-010` | Raw private source/path/token in error | reject/redact |
| `CAL-SEC-011` | Credential/private key/access token in fixture | reject |
| `CAL-SEC-012` | Sealed holdout labels/results accessible before freeze | hard failure |
| `CAL-SEC-013` | Prompt-like note attempts clause control | inert/reject data flow |
| `CAL-SEC-014` | Duplicate evidence amplifies confidence/weight | hard failure |

## Determinism and freeze

| ID | Case | Expected |
|---|---|---|
| `CAL-DET-001` | 1/2/N workers | byte-identical semantic artifacts |
| `CAL-DET-002` | Shuffled source/example/label/fact/evidence order | byte-identical semantic artifacts |
| `CAL-DET-003` | Cold/warm cache | byte-identical semantic artifacts |
| `CAL-DET-004` | Different host/temp roots | byte-identical semantic artifacts |
| `CAL-DET-005` | Independent materialization histories, identical logical inputs | byte-identical semantic artifacts |
| `CAL-DET-014` | Physical storage IDs enter canonical bytes | reject |
| `CAL-FREEZE-006` | Quantitative thresholds/benchmarks required but missing | fail gate |
| `CAL-FREEZE-008` | Tests rewrite checksum fixtures automatically | reject process |
| `CAL-FREEZE-009` | `Cargo.toml`, `.rs`, or CI added before gate | reject documentation phase |
| `CAL-FREEZE-010` | Runtime/client validation claimed without record | reject claim |

## Fresh validation commands when implementation exists

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo test --workspace --doc

E5-A fixture/schema/checksum validator
E5-A provenance/split/leakage validator
E5-A label/graph-validation validator
E5-A mutation/anti-overfitting suite
E5-A deterministic 1/2/N-worker and shuffled-order suite
E5-A ordinary/adversarial benchmark suite
```

Until the workspace and validators exist, these checks are `skipped`/blocked, not pass.
