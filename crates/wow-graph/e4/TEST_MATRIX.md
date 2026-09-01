# E4-B lineage acceptance and mutation matrix

**Status:** normative executable gate. Every active case needs an executable fixture or mutation that fails when the rejected shortcut is introduced. Documentation presence is not a pass.

## Profiles and exact comparison universes

| ID | Case | Required result |
|---|---|---|
| `LIN-PROF-001` | Valid frozen E4-B profile set | accepted |
| `LIN-PROF-002` | Unknown profile field/version | reject |
| `LIN-PROF-003` | Missing relation registry | reject |
| `LIN-PROF-004` | Candidate producer ceiling above Candidate | reject |
| `LIN-PROF-005` | Removal relation lacks coverage requirements | reject |
| `LIN-PROF-006` | Replacement implied by lineage in registry | reject |
| `LIN-PROF-007` | Executable/source-controlled proof rule | reject |
| `LIN-PROF-008` | Valid exact user project generation pair | accepted |
| `LIN-PROF-009` | Valid exact Blizzard UI source generation pair | accepted |
| `LIN-PROF-010` | Valid exact Reference generation pair | accepted |
| `LIN-PROF-011` | Before/after order inferred from timestamp/branch | reject |
| `LIN-PROF-012` | Symbolic current/latest in graph core | reject |
| `LIN-PROF-013` | Different universe classes compared as lineage | reject |
| `LIN-PROF-014` | Project-to-Reference bridge treated as lineage | reject |
| `LIN-PROF-015` | Project logical identity mismatch without explicit bridge | reject |
| `LIN-PROF-016` | Product/flavor/build/profile incompatibility | reject/NotEvaluated |
| `LIN-PROF-017` | Optional search shards absent | explicit reduced candidate coverage |
| `LIN-PROF-018` | Search shard generation mismatch | reject |
| `LIN-PROF-019` | Same profile definitions reordered | same profile ID/bytes |
| `LIN-PROF-020` | Reverse comparison request | distinct canonical comparison/result |

## Identity and relation registry

| ID | Case | Required result |
|---|---|---|
| `LIN-ID-001` | Same exact entity in generation A/B | distinct GenerationEntityRefs |
| `LIN-ID-002` | Accepted continuity | explicit assertion, no ID merge |
| `LIN-ID-003` | Same name/path different generation | distinct refs |
| `LIN-ID-004` | Same name cross-universe | no lineage pair |
| `LIN-ID-005` | Row ID/insertion/path used as lineage identity | mutation fails |
| `LIN-ID-006` | `lineage_successor_of` directional inverse | exact registry semantics |
| `LIN-ID-007` | `same_lineage_as` symmetric query | retains original assertion scope |
| `LIN-ID-008` | Generic `same_as`/`changed`/`impact` relation | reject |
| `LIN-ID-009` | `parent_of` used as lineage relation | reject |
| `LIN-ID-010` | One-to-many relation under one-to-one registry | conflict/reject |
| `LIN-ID-011` | Split relation under explicit one-to-many definition | allowed with evidence |
| `LIN-ID-012` | Merge relation under explicit many-to-one definition | allowed with evidence |
| `LIN-ID-013` | Multi-hop lineage path materialized as direct edge | mutation fails |
| `LIN-ID-014` | Published generation-local GraphSnapshot changed | reject |
| `LIN-ID-015` | Lineage relation crosses unrelated project IDs | reject |
| `LIN-ID-016` | Unknown relation version | reject |
| `LIN-ID-017` | Relation direction reversed silently | reject |
| `LIN-ID-018` | Registry breaking change without version | reject |

## Producer inputs and proof ceilings

| ID | Case | Required result |
|---|---|---|
| `LIN-PROD-001` | Exact project stable identity with complete coverage | eligible Proven ceiling |
| `LIN-PROD-002` | Stable identity key maps to two after entities | collision/conflict |
| `LIN-PROD-003` | Stable ID derived only from path/name | reject |
| `LIN-PROD-004` | Explainable source fingerprint | Candidate proposal |
| `LIN-PROD-005` | Identical fingerprint alone | never above Candidate |
| `LIN-PROD-006` | Opaque model embedding fingerprint | reject |
| `LIN-PROD-007` | Structural change for accepted pair | typed change input |
| `LIN-PROD-008` | Structural change used to prove pair itself | reject/candidate only |
| `LIN-PROD-009` | Exact Reference transition | relation-specific high ceiling |
| `LIN-PROD-010` | Conflicted/expired correction transition | conflict/NotEvaluated |
| `LIN-PROD-011` | Source comment claims replacement | no Reference authority |
| `LIN-PROD-012` | E4-A top exact-name candidate | Candidate lineage ceiling |
| `LIN-PROD-013` | Many approximate search signals | still Candidate |
| `LIN-PROD-014` | Search result from wrong generation | reject |
| `LIN-PROD-015` | Producer partition updated | replace own records only |
| `LIN-PROD-016` | Search producer disabled | owner/reference proposals preserved |
| `LIN-PROD-017` | Producer marks partial input complete | reject |
| `LIN-PROD-018` | Evidence/source reference missing | reject |
| `LIN-PROD-019` | Provenance class relabeled | mutation fails |
| `LIN-PROD-020` | Proposal requested ceiling above effective minimum | cap/reject |
| `LIN-PROD-021` | Input producer order shuffled | identical proposals |
| `LIN-PROD-022` | Same proposal from two producers | distinct proposal provenance |

## Candidate blocking and pair generation

| ID | Case | Required result |
|---|---|---|
| `LIN-CAND-001` | Exact stable-ID blocking | expected pair |
| `LIN-CAND-002` | Explicit Reference transition blocking | expected pair |
| `LIN-CAND-003` | Exact qualified-name block | Candidate pair only |
| `LIN-CAND-004` | Receiver/member shape block | Candidate pair only |
| `LIN-CAND-005` | Fingerprint block | Candidate pair only |
| `LIN-CAND-006` | Search-signal block | Candidate pair only |
| `LIN-CAND-007` | Graph-neighborhood block | Candidate pair only |
| `LIN-CAND-008` | Unrestricted all-pairs | reject |
| `LIN-CAND-009` | Bucket exceeds hard member budget | explicit truncation/NotEvaluated |
| `LIN-CAND-010` | Pair count exceeds per-bucket limit | explicit truncation |
| `LIN-CAND-011` | Pair count exceeds per-entity limit | explicit truncation |
| `LIN-CAND-012` | Random sampling to fit budget | reject |
| `LIN-CAND-013` | First/nearest/highest-score pair selected | reject |
| `LIN-CAND-014` | Same pair from several blocks | proposals/evidence preserved, deterministic dedup |
| `LIN-CAND-015` | Exact block skipped after weak match by hidden rule | mutation fails |
| `LIN-CAND-016` | Candidate coverage reduced by unavailable search shard | explicit |
| `LIN-CAND-017` | Same final inputs different history | same candidate graph |
| `LIN-CAND-018` | 1/2/N pair workers | same pairs/proposals/order |
| `LIN-CAND-019` | Cancellation during block generation | no complete component publication |
| `LIN-CAND-020` | Repository/popularity changes block priority | mutation fails |

## Ambiguity, copies, splits, and merges

| ID | Case | Required result |
|---|---|---|
| `LIN-AMB-001` | One before/one after candidate | OneToOne component; not auto-proven |
| `LIN-AMB-002` | One before/many after candidates | OneToMany component |
| `LIN-AMB-003` | Many before/one after candidate | ManyToOne component |
| `LIN-AMB-004` | Many-to-many candidates | all alternatives retained |
| `LIN-AMB-005` | Before-only entity | unmatched, not Removed yet |
| `LIN-AMB-006` | After-only entity | unmatched, not Introduced yet |
| `LIN-AMB-007` | Unique high-rank candidate only | remains Candidate |
| `LIN-AMB-008` | Greedy maximum-weight assignment | forbidden as truth |
| `LIN-AMB-009` | Tie hidden by source order | mutation fails |
| `LIN-AMB-010` | Old entity remains and similar new copy appears | copy/extraction ambiguity, not move |
| `LIN-AMB-011` | One entity actually split with qualifying evidence | accepted split participants |
| `LIN-AMB-012` | One-to-many similarity without split evidence | ambiguous Candidate |
| `LIN-AMB-013` | Many entities actually merge with qualifying evidence | accepted merge participants |
| `LIN-AMB-014` | Many-to-one similarity without merge evidence | ambiguous Candidate |
| `LIN-AMB-015` | Vendored/generated identical bodies | no automatic lineage |
| `LIN-AMB-016` | Stable identity collision | explicit conflict/block |
| `LIN-AMB-017` | Component exceeds node/edge budget | explicit truncation/continuation |
| `LIN-AMB-018` | Component alternatives omitted from explanation | mutation fails |
| `LIN-AMB-019` | Ambiguity resolved by producer count | reject |
| `LIN-AMB-020` | Candidate/component input order shuffled | same components/IDs |

## Review, promotion, and conflicts

| ID | Case | Required result |
|---|---|---|
| `LIN-REV-001` | Valid exact review target/attestation | accepted for evaluation |
| `LIN-REV-002` | Review targets another generation/profile | reject |
| `LIN-REV-003` | Unknown reviewer authority class | reject |
| `LIN-REV-004` | Accept within effective proof ceiling | accepted assertion |
| `LIN-REV-005` | Review asks above relation ceiling | cap/reject |
| `LIN-REV-006` | Review promotes search-only Candidate to Proven | reject |
| `LIN-REV-007` | Review rejects proposal | proposal retained, no inverse proof |
| `LIN-REV-008` | Review defers component | unresolved preserved |
| `LIN-REV-009` | Conflicting review decisions | explicit conflict |
| `LIN-REV-010` | Later decision supersedes exact predecessor | chain retained |
| `LIN-REV-011` | Review note changes proof rules | mutation fails |
| `LIN-REV-012` | Review note contains executable/tool instruction | inert data/reject payload |
| `LIN-REV-013` | Reviewer count used as majority truth | reject unless exact profile requires fixed set |
| `LIN-REV-014` | Decision application order shuffled | same accepted/conflict set |
| `LIN-REV-015` | Rejected/deferred proposal deleted | mutation fails |
| `LIN-REV-016` | Stable identity and review disagree | conflict/block |
| `LIN-REV-017` | Review fills missing removal coverage | reject |
| `LIN-REV-018` | Accepted assertion explanation lacks proposal/review evidence | reject |
| `LIN-REV-019` | Last-write-wins conflict resolution | mutation fails |
| `LIN-REV-020` | Unaffected component under conflict elsewhere | remains evaluable |

## Publication and immutable snapshot

| ID | Case | Required result |
|---|---|---|
| `LIN-PUB-001` | First lineage snapshot publication | valid sealed snapshot |
| `LIN-PUB-002` | Replace one producer partition | own stale proposals removed only |
| `LIN-PUB-003` | Dependent assertions/change records recomputed | pass |
| `LIN-PUB-004` | Stale base generation | reject |
| `LIN-PUB-005` | Partial store commit | old snapshot only |
| `LIN-PUB-006` | Cancel before commit | no target publication |
| `LIN-PUB-007` | Commit inactive then validation fails | quarantine/inactive, not sealed current |
| `LIN-PUB-008` | Response lost after commit/validation | idempotent exact recovery |
| `LIN-PUB-009` | Same operation ID different request digest | reject |
| `LIN-PUB-010` | Published snapshot modified in place | reject |
| `LIN-PUB-011` | E2 GraphSnapshot modified with lineage edges | mutation fails |
| `LIN-PUB-012` | Complete partition membership missing | validation fails |
| `LIN-PUB-013` | Stale producer proposal remains | validation fails |
| `LIN-PUB-014` | Cross-generation/entity leakage | validation fails |
| `LIN-PUB-015` | Reverse/index mismatch | validation fails |
| `LIN-PUB-016` | Golden lineage/change/migration/impact query mismatch | validation fails |
| `LIN-PUB-017` | Random partition/proposal order | same logical snapshot |
| `LIN-PUB-018` | 1/2/N publication workers | same logical snapshot |
| `LIN-PUB-019` | Physical SQLite bytes differ under allowed class | same logical snapshot/results |
| `LIN-PUB-020` | Last-known-good relabeled as failed target | mutation fails |

## Change classification

| ID | Case | Required result |
|---|---|---|
| `LIN-CHG-001` | Accepted pair with equal requested facets | UnchangedIdentity scoped |
| `LIN-CHG-002` | Accepted pair name changed | Renamed |
| `LIN-CHG-003` | Fuzzy similar names without accepted pair | no Renamed |
| `LIN-CHG-004` | Accepted pair source/container changed with move evidence | Moved |
| `LIN-CHG-005` | Old and copied new entity both exist | not Moved automatically |
| `LIN-CHG-006` | Accepted split assertions | Split |
| `LIN-CHG-007` | Accepted merge assertions | Merged |
| `LIN-CHG-008` | Signature structured change | SignatureChanged |
| `LIN-CHG-009` | Optionality/nilability/multiple returns collapsed | mutation fails |
| `LIN-CHG-010` | Type structured change | TypeChanged |
| `LIN-CHG-011` | Unknown type treated as `any` | mutation fails |
| `LIN-CHG-012` | Exact Reference restriction transition | RestrictionChanged with Reference authority |
| `LIN-CHG-013` | Source implementation suggests restriction | no Reference authority |
| `LIN-CHG-014` | Owner relation changes | OwnershipChanged |
| `LIN-CHG-015` | TOC/load role changes | LoadRoleChanged, no runtime claim |
| `LIN-CHG-016` | Direct relation set changes | RelationSetChanged |
| `LIN-CHG-017` | Path difference rendered as direct relation change | mutation fails |
| `LIN-CHG-018` | Known A -> Known B | Changed |
| `LIN-CHG-019` | Missing -> Known with complete before field scope | Added |
| `LIN-CHG-020` | Missing -> Known with partial before field scope | NotEvaluated/Unknown |
| `LIN-CHG-021` | Known -> Missing with complete after field scope | Removed facet |
| `LIN-CHG-022` | Known -> Unknown/Conflict | Unknown/Conflict, not selected value |
| `LIN-CHG-023` | ExplicitNull -> Missing | distinct state transition |
| `LIN-CHG-024` | Compound rename+move+signature change | three typed records |
| `LIN-CHG-025` | Change computed before lineage pair acceptance | reject/candidate comparison only |
| `LIN-CHG-026` | Field origins/evidence missing | reject |
| `LIN-CHG-027` | Same change records reordered | same ChangeSet ID/bytes |

## Removal and introduction authority

| ID | Case | Required result |
|---|---|---|
| `LIN-ABS-001` | Complete before/after closed scope, old entity unmatched | RemovedWithAuthority |
| `LIN-ABS-002` | Partial after inventory | UnmatchedBefore/NotEvaluated |
| `LIN-ABS-003` | Candidate generation truncated | no Removed |
| `LIN-ABS-004` | Unresolved candidate component | no Removed |
| `LIN-ABS-005` | Conflicting continuity assertion | conflict, no Removed |
| `LIN-ABS-006` | Privacy-hidden after entity | no Removed unless independent metadata authority |
| `LIN-ABS-007` | Empty FTS result only | no Removed |
| `LIN-ABS-008` | Empty graph result only | no Removed |
| `LIN-ABS-009` | NoNewEvidence only | no Removed |
| `LIN-ABS-010` | Complete before/after closed scope, new entity unmatched | IntroducedWithAuthority |
| `LIN-ABS-011` | Partial before inventory | UnmatchedAfter/NotEvaluated |
| `LIN-ABS-012` | Profile/entity kind/root excluded | no introduction/removal |
| `LIN-ABS-013` | Source deleted but stale graph/search record remains | validation failure |
| `LIN-ABS-014` | Negative decision from another profile/build | reject |
| `LIN-ABS-015` | Review asserts absence without coverage | reject |
| `LIN-ABS-016` | Accepted counterpart exists plus removal record | exclusivity conflict |
| `LIN-ABS-017` | Scope manifests reordered | same absence decision |
| `LIN-ABS-018` | Failure/cancellation in decisive producer | no exact absence |

## Replacement and migration

| ID | Case | Required result |
|---|---|---|
| `LIN-MIG-001` | Explicit Reference replacement | accepted scoped `replaced_by` |
| `LIN-MIG-002` | Explicit deprecation without target | deprecated, no replacement |
| `LIN-MIG-003` | Same lineage only | no replacement |
| `LIN-MIG-004` | Similar name/signature only | migration Candidate only |
| `LIN-MIG-005` | Top E4-A candidate only | migration Candidate only |
| `LIN-MIG-006` | Missing old + one new entity | no automatic replacement |
| `LIN-MIG-007` | One old/multiple explicit replacement targets | preserve exact relation shape |
| `LIN-MIG-008` | Candidate promoted directly to recipe | reject |
| `LIN-MIG-009` | Valid recipe exact scopes/assertions/preconditions | structurally valid |
| `LIN-MIG-010` | Recipe missing applicability precondition | invalid |
| `LIN-MIG-011` | Unknown/unsupported signature field hidden | invalid/NotEvaluated |
| `LIN-MIG-012` | Raw executable code/shell/regex transform | reject |
| `LIN-MIG-013` | Typed parameter mapping | validated under exact signatures |
| `LIN-MIG-014` | Default/optional/nil semantics lost | invalid |
| `LIN-MIG-015` | Secret guard step from exact Reference restriction | allowed advisory step |
| `LIN-MIG-016` | `pcall`/conversion claimed declassification | reject |
| `LIN-MIG-017` | EventRegistry custom/native classes collapsed | reject |
| `LIN-MIG-018` | Hook structure claimed taint/combat safe | reject |
| `LIN-MIG-019` | Validation plan omits required client/runtime checks | recipe partial/invalid |
| `LIN-MIG-020` | Recipe claims edit applied/successful | reject |
| `LIN-MIG-021` | Same recipe inputs reordered | same ID/bytes |
| `LIN-MIG-022` | Source/target generation advances | old recipe does not silently follow current |
| `LIN-MIG-023` | Conflicted replacement target | candidate/conflict, no validated recipe |
| `LIN-MIG-024` | Recipe tier | plan_only or validated_recipe only |

## Static impact

| ID | Case | Required result |
|---|---|---|
| `LIN-IMP-001` | Exact changed API root -> direct project use | DirectReferenceImpact |
| `LIN-IMP-002` | Changed function -> direct caller | DirectCallImpact |
| `LIN-IMP-003` | Two-edge caller path | BoundedTransitiveImpact with path |
| `LIN-IMP-004` | Path flattened to direct edge | mutation fails |
| `LIN-IMP-005` | `Possible` relation in path | PossibleImpact cap |
| `LIN-IMP-006` | Candidate root permitted | Candidate-limited result |
| `LIN-IMP-007` | Conflicted root or edge | ConflictBlockedImpact |
| `LIN-IMP-008` | Partial relation coverage | NotEvaluated/partial impact |
| `LIN-IMP-009` | Exact cross-universe `uses_api` bridge | allowed with separate identities |
| `LIN-IMP-010` | Same-name implicit cross-universe bridge | reject |
| `LIN-IMP-011` | Native/custom/CVar signals collapsed | mutation fails |
| `LIN-IMP-012` | Hook path claimed taint/combat safe | reject |
| `LIN-IMP-013` | Load path claimed runtime readiness | reject |
| `LIN-IMP-014` | Structural dependency claimed runtime breakage/severity | reject |
| `LIN-IMP-015` | High-fanout graph | deterministic truncation |
| `LIN-IMP-016` | Cyclic graph | cycle-safe bounded traversal |
| `LIN-IMP-017` | Depth/node/edge/path budget reached | explicit stop/continuation |
| `LIN-IMP-018` | NoNewEvidence | not global no-impact proof |
| `LIN-IMP-019` | Exact closed relation scope no target | scoped no-impact result only |
| `LIN-IMP-020` | Forbidden relation/direction | reject |
| `LIN-IMP-021` | Root/change generation mismatch | reject |
| `LIN-IMP-022` | Target GraphSnapshot switches mid-query | mutation fails |
| `LIN-IMP-023` | Shuffled graph adjacency | same paths/order |
| `LIN-IMP-024` | 1/2/N traversal workers | same result bytes |
| `LIN-IMP-025` | Cancellation during traversal/serialization | no complete/background work |

## Queries and continuation

| ID | Case | Required result |
|---|---|---|
| `LIN-QUERY-001` | Exact entity generation comparison | full assertion/change closure |
| `LIN-QUERY-002` | Trace one accepted lineage edge | pass |
| `LIN-QUERY-003` | Trace multi-hop accepted path | path, not direct assertion |
| `LIN-QUERY-004` | Include Candidate/Possible explicitly | labeled |
| `LIN-QUERY-005` | Fuzzy/natural-language root | reject |
| `LIN-QUERY-006` | Whole-overlay unbounded export | reject |
| `LIN-QUERY-007` | Explain assertion | all producers/evidence/review/ceilings |
| `LIN-QUERY-008` | Query missing mandatory coverage/conflict data | noncomplete/fail |
| `LIN-QUERY-009` | Whole assertion/component/change/impact records per page | pass |
| `LIN-QUERY-010` | Split record across pages | reject |
| `LIN-QUERY-011` | Same cursor replay | identical page |
| `LIN-QUERY-012` | Cursor another snapshot/generation | reject |
| `LIN-QUERY-013` | Cursor another request/profile | reject |
| `LIN-QUERY-014` | Cursor tampered | reject |
| `LIN-QUERY-015` | Continuation resets cumulative budget | reject |
| `LIN-QUERY-016` | Input generation GCed | exact failure, no current fallback |
| `LIN-QUERY-017` | Current advances between pages | old exact snapshot retained |
| `LIN-QUERY-018` | Prior ambiguity/truncation hidden later | reject |
| `LIN-QUERY-019` | Cancellation | explicit cancelled/no background work |
| `LIN-QUERY-020` | Deterministic tie at page boundary | stable assignment |

## Store, retention, and recovery

| ID | Case | Required result |
|---|---|---|
| `LIN-STORE-001` | Registered lineage schema/operation catalogs | accepted |
| `LIN-STORE-002` | Raw SQL/connection/table/rowid/PRAGMA | architecture test fails |
| `LIN-STORE-003` | Separate overlay leaves E2 graph untouched | pass |
| `LIN-STORE-004` | Exact read snapshot during writer publication | old or new, never mixed |
| `LIN-STORE-005` | Active continuation retention | all exact inputs preserved |
| `LIN-STORE-006` | GC removes referenced owner generation | mutation fails |
| `LIN-STORE-007` | GC removes rejected proposal needed by audit | mutation fails |
| `LIN-STORE-008` | Unreferenced old overlay generation | eligible after closure proof |
| `LIN-STORE-009` | Backup/restore | original logical IDs/query results |
| `LIN-STORE-010` | Corrupt overlay | fail/quarantine, no in-place repair |
| `LIN-STORE-011` | Breaking schema/profile update | new epoch/generation |
| `LIN-STORE-012` | Physical layout/checkpoint changes logical ID | mutation fails |
| `LIN-STORE-013` | Response loss after publication | idempotent exact receipt/recovery |
| `LIN-STORE-014` | Windows sharing/busy state | finite operational state, not corruption |
| `LIN-STORE-015` | Empty SQL rows used as removal proof | mutation fails |

## Security and privacy

| ID | Case | Required result |
|---|---|---|
| `LIN-SEC-001` | Source/review text contains system/tool instructions | inert data |
| `LIN-SEC-002` | Source-controlled relation/proof profile | reject |
| `LIN-SEC-003` | Lua/XML/generated code execution | absent |
| `LIN-SEC-004` | Hook/workflow/test/package-manager execution | absent |
| `LIN-SEC-005` | Filesystem/network/process/editor/client access | absent |
| `LIN-SEC-006` | Raw SQL/extension/VFS/ATTACH | absent |
| `LIN-SEC-007` | Model/embedding/CBM accepts/promotes relation | reject |
| `LIN-SEC-008` | Private absolute path in public record | reject/redact to handle |
| `LIN-SEC-009` | Credential/token/private URL in input/output | deny/leak test fails |
| `LIN-SEC-010` | Local-private proposal reused externally | reject |
| `LIN-SEC-011` | Unknown privacy/license state | safest explicit policy |
| `LIN-SEC-012` | Huge producer/review payload | bounded rejection |
| `LIN-SEC-013` | All-pairs resource bomb | rejected/bounded |
| `LIN-SEC-014` | Huge component/fingerprint/evidence graph | bounded rejection/truncation |
| `LIN-SEC-015` | Huge migration recipe/steps | bounded rejection |
| `LIN-SEC-016` | Huge impact path graph | bounded truncation |
| `LIN-SEC-017` | Tampered proposal/review/cursor/store artifact | reject |
| `LIN-SEC-018` | Unicode/control/bidi source/review text | inert escaped data |
| `LIN-SEC-019` | Error/log leaks raw source/review/search text | mutation fails |
| `LIN-SEC-020` | Timeout treated as removal/impact proof | mutation fails |
| `LIN-SEC-021` | Cancellation at every loop/publication stage | bounded stop |
| `LIN-SEC-022` | Search rank/popularity/repository name changes proof | mutation fails |

## Evaluation and anti-overfitting

| ID | Case | Required result |
|---|---|---|
| `LIN-EVAL-001` | Stable identity continuity corpus | expected accepted assertions |
| `LIN-EVAL-002` | Rename corpus | correct rename/no replacement |
| `LIN-EVAL-003` | Move versus copy corpus | no false move |
| `LIN-EVAL-004` | Split/merge corpus | correct participant sets/ambiguity |
| `LIN-EVAL-005` | Same-name unrelated corpus | no accepted lineage |
| `LIN-EVAL-006` | Identical copied/vendor/generated body corpus | no accepted lineage by fingerprint |
| `LIN-EVAL-007` | High search-rank false pair | Candidate only |
| `LIN-EVAL-008` | Complete removal/introduction corpus | exact absence decisions |
| `LIN-EVAL-009` | Partial removal/introduction corpus | zero false absence |
| `LIN-EVAL-010` | Reference explicit replacement corpus | scoped replacement |
| `LIN-EVAL-011` | Deprecated without replacement corpus | no target invented |
| `LIN-EVAL-012` | Typed change corpus | expected facets |
| `LIN-EVAL-013` | Migration candidate/recipe corpus | boundary respected |
| `LIN-EVAL-014` | Direct/transitive/possible impact corpus | exact paths/caps |
| `LIN-EVAL-015` | Cross-universe collision corpus | no lineage collapse |
| `LIN-EVAL-016` | Missing implementation/corpus/ground truth | NotEvaluated, not pass |
| `LIN-EVAL-017` | Candidate recall threshold | frozen quantitative gate |
| `LIN-EVAL-018` | Accepted lineage precision threshold | frozen quantitative gate |
| `LIN-EVAL-019` | False Proven/Derived lineage count | zero |
| `LIN-EVAL-020` | False replacement/removal/introduction/recipe count | zero |
| `LIN-EVAL-021` | False runtime/severity/safety impact count | zero |
| `LIN-EVAL-022` | Better recall but new false authority | reject change |
| `LIN-EVAL-023` | Unknown/partial cases omitted from metrics | reject report |
| `LIN-EVAL-024` | Named corpus identifiers used in production | mutation fails |
| `LIN-EVAL-025` | Repository/path/owner/popularity mutation | semantics stable |
| `LIN-EVAL-026` | Decisive identity/transition/coverage mutation | expected output changes |
| `LIN-EVAL-027` | 1/2/N and shuffled-order evaluation | identical logical outputs |
| `LIN-EVAL-028` | Latency/memory/pair/store/impact threshold exceeded | gate fails |
| `LIN-EVAL-029` | Human/model preference overrides hard gate | forbidden |

## Determinism and compatibility

| ID | Case | Required result |
|---|---|---|
| `LIN-DET-001` | Random entity/producer order | same proposals/components |
| `LIN-DET-002` | Random review order | same assertions/conflicts |
| `LIN-DET-003` | Random store/graph adjacency order | same queries/impact paths |
| `LIN-DET-004` | 1/2/N workers | same canonical bytes |
| `LIN-DET-005` | Different temp roots/hosts | same semantic IDs |
| `LIN-DET-006` | Different SQLite pages/checkpoints | same logical IDs/results |
| `LIN-DET-007` | Clock/timezone/locale changes | no semantic change |
| `LIN-DET-008` | Equivalent request JSON key order | same request/result |
| `LIN-DET-009` | Canonical parse/serialize round trip | same bytes/ID |
| `LIN-DET-010` | Profile version changes | explicit new IDs/incompatibility |
| `LIN-DET-011` | Unknown required newer field ignored | reject |
| `LIN-DET-012` | Rebuild from exact inputs | byte-identical logical artifacts |
| `LIN-DET-013` | Cache cold/warm | same result |
| `LIN-DET-014` | Continuation repeated | same page chain |

## Freeze and activation

| ID | Case | Required result |
|---|---|---|
| `LIN-FREEZE-001` | Documentation state with null pins | allowed only while not-started |
| `LIN-FREEZE-002` | First Rust commit with null prerequisite pins | reject |
| `LIN-FREEZE-003` | E2 graph or E4-A search implementation missing | block E4-B |
| `LIN-FREEZE-004` | Missing owner producer/read catalogs | block |
| `LIN-FREEZE-005` | Missing relation/proof/review/change/migration/impact profiles | block |
| `LIN-FREEZE-006` | Missing paired corpora/ground truth | block |
| `LIN-FREEZE-007` | Missing quantitative thresholds | block |
| `LIN-FREEZE-008` | Missing store/runtime profile and vectors | block |
| `LIN-FREEZE-009` | Missing member/bundle SHA-256 | block |
| `LIN-FREEZE-010` | Tests rewrite golden fixtures | mutation fails |
| `LIN-FREEZE-011` | Rust/Cargo/CI added in documentation-only package | reject |
| `LIN-FREEZE-012` | Implementation state advanced without executable evidence | reject |

## Completion gate

E4-B is incomplete until all active tests execute and prove:

```text
zero generation-local entity rewrites or cross-universe lineage merges
zero score/name/path/fingerprint/unique-candidate promotions above allowed ceiling
zero false accepted lineage, move, rename, split, merge, replacement, removal, introduction or migration-recipe authority
zero hidden alternatives, rejected proposals, conflicts, partial coverage or truncation
zero reason-path flattening and zero runtime/severity/safety impact claims
bounded no-all-pairs candidate generation and bounded impact traversal
immutable deterministic publication/read/continuation under 1/2/N workers
accepted paired-corpus recall/precision/performance thresholds
complete prerequisite/profile/vector/member/checksum freeze
```

Missing implementations, corpora, client/runtime probes or benchmarks are blocked/`NotEvaluated`, never pass.
