# E3-A test matrix

**Status:** normative acceptance and mutation matrix.

## Source profile and materialization

| ID | Case | Expected |
|---|---|---|
| `BUI-SRC-001` | Valid exact source profile/snapshot | pass |
| `BUI-SRC-002` | Floating branch/latest input | reject |
| `BUI-SRC-003` | Commit/tree label and content manifest disagree | reject |
| `BUI-SRC-004` | File enumeration shuffled | same snapshot ID |
| `BUI-SRC-005` | Clone/extraction root changes | same semantic IDs |
| `BUI-SRC-006` | Provider/repository renamed, bytes same | universal outputs same |
| `BUI-SRC-007` | Path traversal/absolute/UNC/device path | reject |
| `BUI-SRC-008` | Case-fold collision | conflict/reject by profile |
| `BUI-SRC-009` | Symlink/junction escapes root | reject/not materialized |
| `BUI-SRC-010` | Submodule/LFS pointer without bytes | explicit unsupported/partial |
| `BUI-SRC-011` | Unknown file extension | inventory retained |
| `BUI-SRC-012` | Silently omitted configured-root file mutation | fail |
| `BUI-SRC-013` | Unsupported/ambiguous text encoding | scoped NotEvaluated |
| `BUI-SRC-014` | Raw bytes change after digest | reject |
| `BUI-SRC-015` | Build/profile/source assertion mismatch | block publication |
| `BUI-SRC-016` | License unknown/conflict | index policy classified, redistribution denied |
| `BUI-SRC-017` | Provider unavailable after materialization | existing generation unaffected |
| `BUI-SRC-018` | Materializer report missing/security profile mismatch | reject |
| `BUI-SRC-019` | File/byte/count budget exceeded | bounded failure/partial |
| `BUI-SRC-020` | Cancellation during snapshot validation | no candidate/current change |

## Universe, packages, TOC, and load

| ID | Case | Expected |
|---|---|---|
| `BUI-PKG-001` | Platform source project identity | `blizzard_ui_source` scoped |
| `BUI-PKG-002` | Same package name in user project | distinct key |
| `BUI-PKG-003` | Same symbol/path in another build | distinct key |
| `BUI-PKG-004` | Package discovery via configured roots | complete accounting |
| `BUI-PKG-005` | Hard-coded `Blizzard_` prefix mutation | fails |
| `BUI-PKG-006` | Directory with no valid TOC | inventory-only/NotEvaluated package |
| `BUI-PKG-007` | One compatible TOC variant | selected exactly |
| `BUI-PKG-008` | Multiple compatible variants | conflict |
| `BUI-PKG-009` | Facts filled from another flavor variant | mutation fails |
| `BUI-PKG-010` | PTR/Retail/Classic profiles | never merged |
| `BUI-PKG-011` | Required dependency missing | unresolved/partial, no download |
| `BUI-PKG-012` | Optional dependency missing | conditional state retained |
| `BUI-PKG-013` | TOC file order differs from filesystem order | TOC order wins |
| `BUI-PKG-014` | Shared file listed by two packages | one source identity, both memberships |
| `BUI-PKG-015` | LoadOnDemand/bootstrap metadata | static role only |
| `BUI-PKG-016` | Runtime-ready claim from static load | reject |
| `BUI-PKG-017` | Dependency/load cycle | explicit conflict |
| `BUI-PKG-018` | Include/script direct order | exact source order |
| `BUI-PKG-019` | Transitive all-pairs edges materialized | mutation fails |
| `BUI-PKG-020` | Reachability under partial dependency coverage | Unknown/conditional, not proven |
| `BUI-PKG-021` | Package/path/display rename preserving facts | universal proposals invariant |
| `BUI-PKG-022` | Unknown TOC directive/file kind | raw record + narrow coverage impact |
| `BUI-PKG-023` | TOC cancellation/truncation | no complete package claim |
| `BUI-PKG-024` | Flavor/profile changes | new generation and invalidation |

## XML, virtual Lua, and analyzer

| ID | Case | Expected |
|---|---|---|
| `BUI-XML-001` | Valid bounded XML package | pass |
| `BUI-XML-002` | DTD/external entity/XInclude/network resolution | disabled/reject |
| `BUI-XML-003` | XML include cycle | bounded conflict |
| `BUI-XML-004` | Include depth/fanout/byte bomb | bounded failure/truncation |
| `BUI-XML-005` | Unknown element/attribute | retained, scoped coverage |
| `BUI-XML-006` | Parent vs inheritance | distinct relations |
| `BUI-XML-007` | Multiple template targets | ambiguity/conflict |
| `BUI-XML-008` | First-found template mutation | fails |
| `BUI-XML-009` | External script resolution outside snapshot | reject |
| `BUI-XML-010` | Inline script virtual source mapping | exact bytes/spans/order |
| `BUI-XML-011` | Malformed inline Lua | affected analyzer scope only |
| `BUI-XML-012` | XML/script execution attempt | absent/reject |
| `BUI-XML-013` | Mixin/prototype exact literal | Derived proposal with evidence |
| `BUI-XML-014` | Dynamic mixin target | Possible/NotEvaluated |
| `BUI-XML-015` | Name-based mixin inference | mutation fails |
| `BUI-ANA-001` | Exact Main/Library unit set | snapshot accepted |
| `BUI-ANA-002` | Missing/extra analyzer unit | reject |
| `BUI-ANA-003` | Analyzer/project generation mismatch | reject |
| `BUI-ANA-004` | Annotation artifact/reference mismatch | reject |
| `BUI-ANA-005` | Second Lua parser/raw-source fallback | architecture test fails |
| `BUI-ANA-006` | Silent `any` globals/diagnostic suppression | reject |
| `BUI-ANA-007` | Unsupported adapter fact | loss/coverage record |
| `BUI-ANA-008` | Dynamic call converted to exact call | mutation fails |
| `BUI-ANA-009` | Worker/update order shuffled | identical manifests |
| `BUI-ANA-010` | Removed virtual/physical unit | all downstream records absent |

## Recognizers, graph, authority, and coverage

| ID | Case | Expected |
|---|---|---|
| `BUI-GRAPH-001` | Source inventory producer partition | accepted exact facts |
| `BUI-GRAPH-002` | TOC/load producer partition | separate ownership |
| `BUI-GRAPH-003` | XML producer partition | separate ownership |
| `BUI-GRAPH-004` | Analyzer adapter partition | separate ownership |
| `BUI-GRAPH-005` | Core recognizer rule partition | versioned owner |
| `BUI-GRAPH-006` | Named source/package condition | reject/mutation fail |
| `BUI-GRAPH-007` | Recognizer tries `Proven` beyond contract | reject |
| `BUI-GRAPH-008` | Dynamic relation | Possible/NotEvaluated |
| `BUI-GRAPH-009` | Rejected proposal | retained in report |
| `BUI-GRAPH-010` | Project weakens registry to accept proposal | forbidden |
| `BUI-GRAPH-011` | Producer update | only its partition replaced |
| `BUI-GRAPH-012` | Rule disabled | stale assertions removed, coverage down |
| `BUI-GRAPH-013` | Cross-universe same-name merge | reject |
| `BUI-GRAPH-014` | Exact allowed source-to-reference edge | accepted with endpoints/evidence |
| `BUI-GRAPH-015` | Source shape used as API authority | reject |
| `BUI-GRAPH-016` | Source shape used as Secret/taint/runtime authority | reject |
| `BUI-GRAPH-017` | Complete inventory, partial analyzer | separate coverage states |
| `BUI-GRAPH-018` | Empty relation lookup under partial coverage | nonauthoritative absence |
| `BUI-GRAPH-019` | Exact structural absence under closed scope | scoped negative result |
| `BUI-GRAPH-020` | Conflict blocks dependent capability | explicit blocker |
| `BUI-GRAPH-021` | Confidence aggregation upgrade | mutation fails |
| `BUI-GRAPH-022` | Persisted transitive path as edge | mutation fails |
| `BUI-GRAPH-023` | Graph registry/profile changes | revalidate affected proposals |
| `BUI-GRAPH-024` | Graph publication/query golden checks | pass |

## Candidate, incremental update, and publication

| ID | Case | Expected |
|---|---|---|
| `BUI-UPD-001` | First full candidate | valid NotPublishedE3A |
| `BUI-UPD-002` | Lua-only change | exact dependent invalidation |
| `BUI-UPD-003` | TOC order/dependency change | load/analyzer/downstream invalidation |
| `BUI-UPD-004` | XML include/script change | virtual units and downstream invalidated |
| `BUI-UPD-005` | Analyzer pin change | analyzer/downstream invalidated |
| `BUI-UPD-006` | Recognizer rule update | rule partition/downstream only |
| `BUI-UPD-007` | Unknown impact | conservative widening |
| `BUI-UPD-008` | Reuse without exact profile/dependency proof | reject |
| `BUI-UPD-009` | Removed file leaves analyzer fact | fail stale closure |
| `BUI-UPD-010` | Removed entity leaves graph/source handle | fail stale closure |
| `BUI-UPD-011` | Same final state via different update order | same target IDs/manifests |
| `BUI-UPD-012` | Canonical no-change | no new generation/work |
| `BUI-PUB-001` | Valid E2-D bundle | inactive generation built |
| `BUI-PUB-002` | User project/store reused for platform source | reject |
| `BUI-PUB-003` | Cross-generation identity mix | reject |
| `BUI-PUB-004` | Store build failure/cancel | current unchanged |
| `BUI-PUB-005` | Fresh read-back validates all owners | target ValidatedInactive |
| `BUI-PUB-006` | Stale-row/removal golden mismatch | validation fails |
| `BUI-PUB-007` | Current CAS success | exact target active |
| `BUI-PUB-008` | Stale CAS | target stays inactive |
| `BUI-PUB-009` | Last-known-good relabeled as target | mutation fails |
| `BUI-PUB-010` | Reader during activation | old or new coherent set only |
| `BUI-PUB-011` | Provider outage after publish | read unaffected |
| `BUI-PUB-012` | Physical SQLite/WAL layout variation | logical IDs/results unchanged |

## Fingerprints and skeleton-input view

| ID | Case | Expected |
|---|---|---|
| `BUI-FP-001` | Stable exact-generation fingerprint | deterministic |
| `BUI-FP-002` | Same fingerprint across builds treated as identity | reject |
| `BUI-FP-003` | Fingerprint profile changes | invalidate records |
| `BUI-FP-004` | Low-information/collision case | classified, no lineage |
| `BUI-SKEL-001` | Exact package/file/entity root | bounded records |
| `BUI-SKEL-002` | Snapshot generation mismatch | reject |
| `BUI-SKEL-003` | Direct relations retain assertions/evidence | pass |
| `BUI-SKEL-004` | Partial/conflict state retained | pass |
| `BUI-SKEL-005` | Bounded exact source slice | valid range/license |
| `BUI-SKEL-006` | Source slice outside object/range | reject |
| `BUI-SKEL-007` | Redistribution/excerpt policy denies bytes | handles/metadata only |
| `BUI-SKEL-008` | Unbounded all-source request | reject |
| `BUI-SKEL-009` | Project Map/prose/token ranking generated here | architecture test fails |
| `BUI-SKEL-010` | Deterministic ordering/pagination | stable |
| `BUI-SKEL-011` | Cursor against another snapshot/request | reject |
| `BUI-SKEL-012` | No-new-evidence | distinct from runtime absence |
| `BUI-SKEL-013` | Budget truncation | explicit continuation/partial |
| `BUI-SKEL-014` | Source comment presented as framework instruction | mutation fails |

## Security, resource, determinism, and freeze

| ID | Case | Expected |
|---|---|---|
| `BUI-SEC-001` | Repository hook/workflow/generator/source execution | absent/reject |
| `BUI-SEC-002` | Filesystem outside snapshot/store roots | absent/reject |
| `BUI-SEC-003` | Network/process/editor/client access | absent |
| `BUI-SEC-004` | Prompt injection in Lua/XML/TOC/comment/string | inert source data |
| `BUI-SEC-005` | Private absolute path/token in report | redacted |
| `BUI-SEC-006` | SavedVariables/log/runtime payload ingestion | absent |
| `BUI-SEC-007` | Huge package/XML/analyzer/graph fanout | bounded failure/truncation |
| `BUI-SEC-008` | Cancellation at every long phase | no current advance/background work |
| `BUI-SEC-009` | Unknown license redistributed | reject release/export |
| `BUI-SEC-010` | Metadata-only/local-source/excerpt classes | correctly separated |
| `BUI-DET-001` | 1/2/N workers | same logical outputs |
| `BUI-DET-002` | Hash-map/DB/insertion/completion order | no semantic change |
| `BUI-DET-003` | Host/time/temp/WAL/checkpoint differences | excluded from IDs |
| `BUI-DET-004` | Provider/path/package rename mutation | universal facts invariant |
| `BUI-DET-005` | Byte/profile decisive change | expected IDs change |
| `BUI-FIX-001` | Null pins while implementation not started | allowed |
| `BUI-FIX-002` | First Rust commit with required null pins | fail |
| `BUI-FIX-003` | Missing prerequisite implementation/fixture digest | block |
| `BUI-FIX-004` | Missing real mirror source/license manifest | block |
| `BUI-FIX-005` | Tests rewrite frozen fixtures | fail |
| `BUI-FIX-006` | All profiles/vectors/member SHA-256 frozen | pass entry gate |

## Acceptance

E3-A is incomplete until all nondeferred cases execute against the exact frozen implementation and fixtures, including a pinned real mirror snapshot, and demonstrate source/profile exactness, universe separation, complete inventory accounting, no duplicate parsers, producer-owned graph partitions, authority preservation, coherent E2-D publication, exact stale-removal closure, bounded skeleton inputs, security, cancellation, and deterministic logical output.
