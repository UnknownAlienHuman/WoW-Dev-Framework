# E4-A test matrix

**Status:** normative executable acceptance and mutation matrix.

Every implementation change must identify the affected cases and add a failing mutation for any repaired authority, generation, privacy, ranking, continuation, or resource boundary.

## Profiles and universe binding

| ID | Case | Expected |
|---|---|---|
| `SEARCH-PROFILE-001` | Valid complete E4-A profile set | pass |
| `SEARCH-PROFILE-002` | Unknown profile version | reject |
| `SEARCH-PROFILE-003` | Executable callback/expression field in profile | reject |
| `SEARCH-PROFILE-004` | Incompatible document and ranking profiles | reject |
| `SEARCH-PROFILE-005` | Exact lane configured with case folding | reject |
| `SEARCH-PROFILE-006` | Raw cross-shard FTS score feature configured | reject |
| `SEARCH-PROFILE-007` | Unbounded limit in profile | reject |
| `SEARCH-PROFILE-008` | Source-controlled profile payload | reject |
| `SEARCH-PROFILE-009` | Exact compatible user/reference/Blizzard shards | pass |
| `SEARCH-PROFILE-010` | Same-name entity across universes | remain distinct |
| `SEARCH-PROFILE-011` | Incompatible client flavor/build/ProfileId | reject |
| `SEARCH-PROFILE-012` | Missing optional Blizzard shard | explicit omission |
| `SEARCH-PROFILE-013` | Missing required Reference shard | NotEvaluated/fail |
| `SEARCH-PROFILE-014` | Symbolic current in core universe set | reject |
| `SEARCH-PROFILE-015` | Shuffled profile object order | same canonical ID |

## Documents, fields, and origins

| ID | Case | Expected |
|---|---|---|
| `SEARCH-DOC-001` | Valid project entity document | pass |
| `SEARCH-DOC-002` | Valid Reference entity document | pass |
| `SEARCH-DOC-003` | Valid Blizzard UI source entity document | pass |
| `SEARCH-DOC-004` | Owner entity missing | reject |
| `SEARCH-DOC-005` | Cross-generation entity handle | reject |
| `SEARCH-DOC-006` | Field not allowed for document kind | reject |
| `SEARCH-DOC-007` | Arbitrary JSON property bag | reject |
| `SEARCH-DOC-008` | Full source body in default document | reject |
| `SEARCH-DOC-009` | Private absolute path field | reject/redact by exact policy |
| `SEARCH-DOC-010` | Explicit alias with exact owner record | pass |
| `SEARCH-DOC-011` | Alias inferred from fuzzy spelling | reject |
| `SEARCH-DOC-012` | Alias target conflict | conflict/partial |
| `SEARCH-DOC-013` | Canonical identifier case preserved | pass |
| `SEARCH-DOC-014` | Case-folded identifier classified exact | mutation fails |
| `SEARCH-DOC-015` | Unknown/nilable/optional shape states | remain distinct |
| `SEARCH-DOC-016` | Model-generated keywords/summary | reject |
| `SEARCH-DOC-017` | Source comment instruction changes field schema | no effect |
| `SEARCH-DOC-018` | Duplicate value with distinct origins | origins retained |
| `SEARCH-DOC-019` | Same text in different universes/spans | remain distinct |
| `SEARCH-DOC-020` | Projection loss without loss record | reject |
| `SEARCH-DOC-021` | Removed owner partition | all stale documents/fields removed |
| `SEARCH-DOC-022` | Random owner record order | same documents/IDs |

## Shard build and publication

| ID | Case | Expected |
|---|---|---|
| `SEARCH-SHARD-001` | Initial complete shard build | Validated/SealedReadOnly |
| `SEARCH-SHARD-002` | Exact base with unchanged partitions | reuse |
| `SEARCH-SHARD-003` | Changed partition | new immutable version |
| `SEARCH-SHARD-004` | Removed partition | absent from complete target membership |
| `SEARCH-SHARD-005` | Recursive delta-only target membership | reject |
| `SEARCH-SHARD-006` | Stale/incompatible base shard | reject |
| `SEARCH-SHARD-007` | Mixed owner generations in one shard | reject |
| `SEARCH-SHARD-008` | User and Reference records in one shard | reject |
| `SEARCH-SHARD-009` | Build failure before commit | prior shards unchanged |
| `SEARCH-SHARD-010` | Cancellation during projection/index build | no sealed target |
| `SEARCH-SHARD-011` | Physical commit but read-back validation failure | quarantine/fail |
| `SEARCH-SHARD-012` | Writable open after seal | reject |
| `SEARCH-SHARD-013` | Corrupt SQLite artifact | integrity failure, not empty result |
| `SEARCH-SHARD-014` | Golden exact query mismatch | validation fail |
| `SEARCH-SHARD-015` | FTS rowid/document mapping mismatch | validation fail |
| `SEARCH-SHARD-016` | Stale FTS/index entry after removal | validation fail |
| `SEARCH-SHARD-017` | No-change classified as new semantic shard | mutation fails |
| `SEARCH-SHARD-018` | Different build histories same final inputs | same logical shard |
| `SEARCH-SHARD-019` | Physical bytes differ under logical profile | same logical ID, classified |
| `SEARCH-SHARD-020` | Response loss and exact idempotent recovery | return existing state |

## Request and normalization

| ID | Case | Expected |
|---|---|---|
| `SEARCH-QUERY-001` | Valid exact-entity request | pass |
| `SEARCH-QUERY-002` | Valid structured mixed request | pass |
| `SEARCH-QUERY-003` | Unknown request field | reject |
| `SEARCH-QUERY-004` | Fuzzy/natural-language intent inferred by hidden model | reject |
| `SEARCH-QUERY-005` | Raw FTS MATCH string | reject |
| `SEARCH-QUERY-006` | SQL/regex/expression/callback input | reject |
| `SEARCH-QUERY-007` | Invalid UTF-8/NUL/control input | reject per profile |
| `SEARCH-QUERY-008` | Oversized text/identifier | bounded reject |
| `SEARCH-QUERY-009` | Exact identifier preserves case | pass |
| `SEARCH-QUERY-010` | Approximate case fold stored separately | pass |
| `SEARCH-QUERY-011` | Hidden stopword/stemming/synonym expansion | mutation fails |
| `SEARCH-QUERY-012` | Safe AST quotes FTS operators as literals | pass |
| `SEARCH-QUERY-013` | Unknown column/field name | reject |
| `SEARCH-QUERY-014` | Required lane unavailable | NotEvaluated/fail |
| `SEARCH-QUERY-015` | Optional lane unavailable | explicit partial |
| `SEARCH-QUERY-016` | Fallback changes generation/universe | reject |
| `SEARCH-QUERY-017` | Fallback resets budget | reject |
| `SEARCH-QUERY-018` | Equivalent reordered request JSON | same normalized query/plan |

## Exact, alias, member, and prefix lanes

| ID | Case | Expected |
|---|---|---|
| `SEARCH-EXACT-001` | Exact EntityKey hit | highest band |
| `SEARCH-EXACT-002` | Exact key miss with full authority | ExactNotFoundWithAuthority |
| `SEARCH-EXACT-003` | Exact key miss with partial coverage | ExactNotFoundPartial |
| `SEARCH-EXACT-004` | Qualified canonical name hit | exact qualified band |
| `SEARCH-EXACT-005` | Short canonical name collision | all exact candidates retained |
| `SEARCH-EXACT-006` | Same name different kind/universe | separate candidates |
| `SEARCH-EXACT-007` | Case-only mismatch in exact name | no exact hit |
| `SEARCH-EXACT-008` | Explicit alias exact hit | exact alias band |
| `SEARCH-EXACT-009` | Fuzzy spelling treated as alias | mutation fails |
| `SEARCH-EXACT-010` | Deprecated-name text without alias record | not alias |
| `SEARCH-EXACT-011` | Alias loop/conflicting targets | conflict |
| `SEARCH-EXACT-012` | Exact namespace/member hit | pass |
| `SEARCH-EXACT-013` | Exact receiver/method hit | pass |
| `SEARCH-EXACT-014` | Receiver guessed from opaque text | reject/no signal |
| `SEARCH-EXACT-015` | Case-sensitive prefix | candidate prefix signal |
| `SEARCH-EXACT-016` | Folded prefix | lower approximate signal |
| `SEARCH-EXACT-017` | Prefix shorter than minimum | reject/skip |
| `SEARCH-EXACT-018` | Prefix expansion over cap | deterministic truncation |
| `SEARCH-EXACT-019` | First-row ambiguity resolution | mutation fails |
| `SEARCH-EXACT-020` | Multi-lane same entity | one candidate, all signals |

## Text, identifier-similarity, and shape lanes

| ID | Case | Expected |
|---|---|---|
| `SEARCH-APPROX-001` | FTS term match in one shard | local ordinal signal |
| `SEARCH-APPROX-002` | FTS phrase match | pass |
| `SEARCH-APPROX-003` | FTS prefix term under profile | pass |
| `SEARCH-APPROX-004` | Raw BM25 compared across shards | mutation fails |
| `SEARCH-APPROX-005` | Retained old generation changes current FTS rank | mutation fails |
| `SEARCH-APPROX-006` | Source comment text match | source-doc authority retained |
| `SEARCH-APPROX-007` | Reference documentation match | reference-doc authority retained |
| `SEARCH-APPROX-008` | Snippet rendered as source evidence | mutation fails |
| `SEARCH-APPROX-009` | Snippet contains hostile Markdown/HTML | escaped bounded data |
| `SEARCH-APPROX-010` | No FTS candidates | nonauthoritative empty |
| `SEARCH-APPROX-011` | Identifier trigram candidate | candidate-only |
| `SEARCH-APPROX-012` | Bounded edit-distance candidate | candidate-only |
| `SEARCH-APPROX-013` | Similarity becomes alias/lineage | mutation fails |
| `SEARCH-APPROX-014` | Case/separator-only similarity | labeled approximate |
| `SEARCH-APPROX-015` | Similarity matrix exceeds budget | bounded partial/fail |
| `SEARCH-APPROX-016` | Exact structured shape | ExactShape candidate |
| `SEARCH-APPROX-017` | Compatible partial shape | labeled partial |
| `SEARCH-APPROX-018` | Unknown optional/nil/type collapsed | mutation fails |
| `SEARCH-APPROX-019` | Reference restriction shape | exact profile-bound field |
| `SEARCH-APPROX-020` | Source implementation supplies restriction authority | mutation fails |
| `SEARCH-APPROX-021` | Shape produces replacement conclusion | mutation fails |
| `SEARCH-APPROX-022` | Required shape capability missing | NotEvaluated/partial |

## Graph-assisted retrieval

| ID | Case | Expected |
|---|---|---|
| `SEARCH-GRAPH-001` | Exact seed direct neighbor | pass |
| `SEARCH-GRAPH-002` | Exact seed bounded multi-hop path | reason path signal |
| `SEARCH-GRAPH-003` | Candidate seed allowed by profile | authority capped |
| `SEARCH-GRAPH-004` | Candidate seed not allowed | skip/reject |
| `SEARCH-GRAPH-005` | Same target via multiple paths | one candidate, paths retained |
| `SEARCH-GRAPH-006` | Path rendered as direct edge | mutation fails |
| `SEARCH-GRAPH-007` | Possible edge | path remains possible |
| `SEARCH-GRAPH-008` | Conflicted edge | conflict retained |
| `SEARCH-GRAPH-009` | Explicit project-to-reference bridge | pass without identity merge |
| `SEARCH-GRAPH-010` | Same-name cross-universe without bridge | no link |
| `SEARCH-GRAPH-011` | Cyclic call/state graph | cycle-safe bounded |
| `SEARCH-GRAPH-012` | High fanout | deterministic truncation |
| `SEARCH-GRAPH-013` | Graph no-hit partial coverage | nonauthoritative |
| `SEARCH-GRAPH-014` | Graph-wide no-seed scan | reject |
| `SEARCH-GRAPH-015` | Caller relation expression/query language | reject |
| `SEARCH-GRAPH-016` | Different traversal order | same candidate/path ordering |
| `SEARCH-GRAPH-017` | Graph snapshot changes mid-request | reject/no switch |

## Fusion, ranking, and explanations

| ID | Case | Expected |
|---|---|---|
| `SEARCH-RANK-001` | Exact ID versus many fuzzy/text signals | exact ID remains above |
| `SEARCH-RANK-002` | Qualified versus short canonical | profile band order |
| `SEARCH-RANK-003` | Explicit alias versus approximate total | alias remains above |
| `SEARCH-RANK-004` | Exact constraints conflict | candidate capped/excluded |
| `SEARCH-RANK-005` | Same entity signals fused | all signals retained |
| `SEARCH-RANK-006` | Same name cross-universe fused | mutation fails |
| `SEARCH-RANK-007` | Integer reciprocal-rank arithmetic | exact expected vector |
| `SEARCH-RANK-008` | Feature integer overflow | reject |
| `SEARCH-RANK-009` | Floating platform aggregate in canonical rank | reject |
| `SEARCH-RANK-010` | Raw FTS value enters cross-shard total | mutation fails |
| `SEARCH-RANK-011` | Repeated text occurrences exceed cap | bounded contribution |
| `SEARCH-RANK-012` | Repeated graph paths exceed cap | bounded contribution |
| `SEARCH-RANK-013` | Required filter failure | exclude/cap per profile |
| `SEARCH-RANK-014` | Conflict/coverage penalty | explicit |
| `SEARCH-RANK-015` | Universe priority for API query | Reference preferred as retrieval policy |
| `SEARCH-RANK-016` | Universe priority changes entity authority | mutation fails |
| `SEARCH-RANK-017` | Stable exact tie key | deterministic total order |
| `SEARCH-RANK-018` | Rowid/insertion/thread tie breaker | mutation fails |
| `SEARCH-RANK-019` | Explanation reconstructs rank tuple | pass |
| `SEARCH-RANK-020` | Missing skipped/failed lane in explanation | reject |
| `SEARCH-RANK-021` | Opaque relevance score only | reject |
| `SEARCH-RANK-022` | High rank states user intent proven | mutation fails |
| `SEARCH-RANK-023` | High rank emits replacement/lineage | mutation fails |
| `SEARCH-RANK-024` | 1/2/N workers and shuffled lane results | same manifest |

## Miss and negative authority

| ID | Case | Expected |
|---|---|---|
| `SEARCH-MISS-001` | Exact complete key miss with owner authority | authoritative miss |
| `SEARCH-MISS-002` | Exact miss with incomplete owner enumeration | partial |
| `SEARCH-MISS-003` | Exact alias miss with incomplete alias coverage | partial |
| `SEARCH-MISS-004` | Privacy-hidden relevant document | no authoritative miss |
| `SEARCH-MISS-005` | Conflict affects exact value | ConflictBlocked |
| `SEARCH-MISS-006` | Required exact lane failed | LaneUnavailable/Failed |
| `SEARCH-MISS-007` | Exact lane truncated | Truncated |
| `SEARCH-MISS-008` | Text-only no hit | NoCandidatesUnderExecutedLanes |
| `SEARCH-MISS-009` | Fuzzy/shape/graph no hit | nonauthoritative |
| `SEARCH-MISS-010` | Approximate candidates only | CandidateOnly |
| `SEARCH-MISS-011` | Empty SQL result treated as domain absence | mutation fails |
| `SEARCH-MISS-012` | Complete SearchStore but partial owner facts | no authority upgrade |
| `SEARCH-MISS-013` | Exact found plus optional lane failed | found + partial metadata per profile |
| `SEARCH-MISS-014` | Miss explanation omits decisive coverage | reject |
| `SEARCH-MISS-015` | Current/nearest shard fallback for miss | reject |

## Result sets, pagination, and continuation

| ID | Case | Expected |
|---|---|---|
| `SEARCH-PAGE-001` | Materialize ordered result-set manifest | pass |
| `SEARCH-PAGE-002` | Candidate cap distinct from page size | pass |
| `SEARCH-PAGE-003` | Page contains whole candidates | pass |
| `SEARCH-PAGE-004` | Mandatory explanation split/pruned | reject |
| `SEARCH-PAGE-005` | Pages 1..N have no duplicate/gap | pass |
| `SEARCH-PAGE-006` | Final empty continuation page | typed end state |
| `SEARCH-PAGE-007` | Cursor binds exact result manifest | pass |
| `SEARCH-PAGE-008` | Cursor against another shard/request/profile | reject |
| `SEARCH-PAGE-009` | Current advances between pages | continue old exact result |
| `SEARCH-PAGE-010` | Shard/result set GCed | exact unavailable failure |
| `SEARCH-PAGE-011` | Cumulative budget preserved | pass |
| `SEARCH-PAGE-012` | Budget reset mutation | reject |
| `SEARCH-PAGE-013` | Privacy/tokenizer/ranking change on continuation | reject |
| `SEARCH-PAGE-014` | Tampered cursor | reject |
| `SEARCH-PAGE-015` | Replay exact retained shards | same manifest or fail |
| `SEARCH-PAGE-016` | Replay mismatch | fail, no substituted result |
| `SEARCH-PAGE-017` | Cancellation during page/detail render | cancelled/partial only |
| `SEARCH-PAGE-018` | Page size/order changes through hidden default | mutation fails |

## Persistence and FTS5 profile

| ID | Case | Expected |
|---|---|---|
| `SEARCH-STORE-001` | Exact built-in FTS5 capability probe | pass before activation |
| `SEARCH-STORE-002` | FTS5 missing | blocking, no extension fallback |
| `SEARCH-STORE-003` | Loadable extension enabled | reject |
| `SEARCH-STORE-004` | Source-controlled tokenizer/rank function | reject |
| `SEARCH-STORE-005` | Private rowid mapped one-to-one to document ID | pass |
| `SEARCH-STORE-006` | Rowid enters public ID/cursor | mutation fails |
| `SEARCH-STORE-007` | Read-only/query-only query open | pass |
| `SEARCH-STORE-008` | Writable query open | reject |
| `SEARCH-STORE-009` | ATTACH/DETACH/raw SQL/PRAGMA exposed | reject |
| `SEARCH-STORE-010` | Unknown external SQLite adopted as shard | reject |
| `SEARCH-STORE-011` | Integrity check failure | fail/quarantine |
| `SEARCH-STORE-012` | Physical page/WAL variation | no logical rank/ID change |
| `SEARCH-STORE-013` | Tokenizer/compile-option change | new profile/shard |
| `SEARCH-STORE-014` | Snippet/rank behavior probe mismatch | blocking |
| `SEARCH-STORE-015` | Result-set object retained by cursor | GC blocked |
| `SEARCH-STORE-016` | Unreferenced derived shard GC | safe after closure |

## Security, privacy, and limits

| ID | Case | Expected |
|---|---|---|
| `SEARCH-SEC-001` | FTS keyword/operator injection | literal/reject |
| `SEARCH-SEC-002` | SQL injection text | literal/reject |
| `SEARCH-SEC-003` | Regex/callback/expression payload | reject |
| `SEARCH-SEC-004` | Column/table/tokenizer name injection | reject |
| `SEARCH-SEC-005` | Leading wildcard/prefix explosion | reject/bounded |
| `SEARCH-SEC-006` | Very long repeated terms | bounded |
| `SEARCH-SEC-007` | Unicode confusable/combining mutation | deterministic |
| `SEARCH-SEC-008` | Malicious source comment changes policy | no effect |
| `SEARCH-SEC-009` | Private path/token/credential field | excluded/redacted |
| `SEARCH-SEC-010` | Local-private shard used for external consumer | reject |
| `SEARCH-SEC-011` | Snippet source boundary escape | escaped |
| `SEARCH-SEC-012` | Corrupt/tampered manifest | reject |
| `SEARCH-SEC-013` | Cross-generation source/document substitution | reject |
| `SEARCH-SEC-014` | Arbitrary filesystem path | reject |
| `SEARCH-SEC-015` | Network/process/editor/WoW-client/model call | absent |
| `SEARCH-SEC-016` | Unbounded graph/text/result request | reject |
| `SEARCH-SEC-017` | Cancellation in every expensive stage | bounded stop |
| `SEARCH-SEC-018` | Error/log contains full source/private query | mutation fails |
| `SEARCH-SEC-019` | Mandatory explanation exceeds budget | fail honestly |
| `SEARCH-SEC-020` | Cancelled result cached as complete | reject |

## Evaluation and determinism

| ID | Case | Expected |
|---|---|---|
| `SEARCH-EVAL-001` | Exact complete corpus exact-key/name recall | 100% |
| `SEARCH-EVAL-002` | Explicit alias recall | 100% for complete alias corpus |
| `SEARCH-EVAL-003` | False exact/alias classification | zero |
| `SEARCH-EVAL-004` | False authoritative miss | zero |
| `SEARCH-EVAL-005` | False lineage/replacement/migration/impact claim | zero |
| `SEARCH-EVAL-006` | Top-k approximate retrieval metrics | reported by corpus/lane |
| `SEARCH-EVAL-007` | Collision corpus | no identity collapse |
| `SEARCH-EVAL-008` | Leave-one-repository/package-out | reported |
| `SEARCH-EVAL-009` | Rename/path/name popularity mutation | production ordering stable where semantics unchanged |
| `SEARCH-EVAL-010` | Feature ablation | reported |
| `SEARCH-EVAL-011` | Cold/warm cache | same canonical result |
| `SEARCH-EVAL-012` | 1/2/N workers | same canonical result |
| `SEARCH-EVAL-013` | Different physical SQLite layout/checkpoint | same logical result |
| `SEARCH-EVAL-014` | Different update histories same final state | same shard/results |
| `SEARCH-EVAL-015` | Missing benchmark/evaluation harness | NotEvaluated, not pass |
| `SEARCH-EVAL-016` | Threshold changed without profile version | reject |
| `SEARCH-EVAL-017` | Recall gain with hard authority violation | reject |
| `SEARCH-EVAL-018` | Explanation bytes differ without profile change | fail |

## Freeze gate

| ID | Case | Expected |
|---|---|---|
| `SEARCH-FREEZE-001` | All prerequisite implementation/fixture pins null before implementation | allowed while not-started |
| `SEARCH-FREEZE-002` | First Rust commit with required null pins | reject |
| `SEARCH-FREEZE-003` | SQLite/FTS/tokenizer probe absent | reject |
| `SEARCH-FREEZE-004` | Corpus thresholds not measured/frozen | reject |
| `SEARCH-FREEZE-005` | Member or bundle SHA-256 missing | reject |
| `SEARCH-FREEZE-006` | Tests rewrite fixtures automatically | reject |
| `SEARCH-FREEZE-007` | All pins/vectors/checksums frozen | implementation gate pass |

## Acceptance

E4-A is incomplete until all nondeferred cases execute against the exact frozen implementation/profile/corpora and prove:

- owner facts and query signals remain separate;
- one shard binds one exact owner generation;
- exact bands cannot be outranked by approximate repetition;
- raw FTS values never cross shard boundaries;
- exact miss authority is complete and scoped;
- no lineage/replacement/migration/impact inference exists;
- all rankings/explanations/pages are deterministic;
- query/source/storage inputs cannot execute or escape;
- privacy and resource boundaries hold;
- zero false-authority hard gates pass;
- all fixture/checksum/profile values are frozen.
