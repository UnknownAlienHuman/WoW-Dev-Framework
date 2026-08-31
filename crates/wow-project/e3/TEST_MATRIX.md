# E3-B Blizzard UI source test matrix

**Status:** normative executable acceptance and mutation matrix.

## Source profile and provider

| ID | Case | Expected |
|---|---|---|
| `UISRC-PROFILE-001` | Valid source/provider/root/profile set | pass |
| `UISRC-PROFILE-002` | Floating branch or unqualified `latest/current` | reject |
| `UISRC-PROFILE-003` | Provider owner/name changes with same sealed content/profile | semantic outputs stable |
| `UISRC-PROFILE-004` | Repository popularity affects trust/semantics | mutation fails |
| `UISRC-PROFILE-005` | Unknown profile field | preserve/reject per version policy; never ignore silently |
| `UISRC-PROFILE-006` | Breaking root/identity rule without new version | reject |
| `UISRC-PROFILE-007` | Unbounded file/parser/graph policy | reject |
| `UISRC-PROFILE-008` | Executable callback/regex/script in profile | reject |
| `UISRC-PROFILE-009` | Source profile selects incompatible graph registry | reject |
| `UISRC-PROFILE-010` | Source profile directly names user project current record | reject |
| `UISRC-PROFILE-011` | Equivalent profile field order | same profile ID/bytes |
| `UISRC-PROFILE-012` | Provider locator used as source entity key | mutation fails |

## Materialized snapshot and paths

| ID | Case | Expected |
|---|---|---|
| `UISRC-MAT-001` | Valid sealed immutable snapshot | pass |
| `UISRC-MAT-002` | Snapshot not sealed or changes during read | reject/quarantine |
| `UISRC-MAT-003` | Complete admitted file/content manifest | pass |
| `UISRC-MAT-004` | Missing file without omission record | incomplete/reject complete claim |
| `UISRC-MAT-005` | File digest/length mismatch | reject |
| `UISRC-MAT-006` | `..`, absolute, drive, UNC, device or NUL path | reject |
| `UISRC-MAT-007` | Unicode/case-normalization collision | conflict/reject |
| `UISRC-MAT-008` | Host checkout path changes | semantic identities stable |
| `UISRC-MAT-009` | File mtimes/executable bits change only | no semantic change |
| `UISRC-MAT-010` | File enumeration shuffled | identical manifests |
| `UISRC-MAT-011` | Symlink/junction/reparse traversal | disabled/reject |
| `UISRC-MAT-012` | Submodule traversal | disabled/reject |
| `UISRC-MAT-013` | Git LFS network resolution | disabled/reject unresolved pointer |
| `UISRC-MAT-014` | Nested archive expansion not declared | reject |
| `UISRC-MAT-015` | Provider hook/workflow/build script present | data only, never executed |
| `UISRC-MAT-016` | Materializer cancellation | no sealed complete snapshot |
| `UISRC-MAT-017` | Partial staging reused as complete | mutation fails |
| `UISRC-MAT-018` | Byte-normalizing transformation unreported | reject |
| `UISRC-MAT-019` | Reported transformation with original/target digests | distinct valid snapshot under profile |
| `UISRC-MAT-020` | Suspicious credential/private data admitted | quarantine/redact policy |

## Build binding and compatibility

| ID | Case | Expected |
|---|---|---|
| `UISRC-BIND-001` | Independent exact source/client/reference evidence agrees | `ExactBuildMatched` |
| `UISRC-BIND-002` | Provider label only | `ProviderDeclared`, not exact |
| `UISRC-BIND-003` | Content correlation without exact proof | `ContentCorrelated` |
| `UISRC-BIND-004` | Revision known, compatibility unknown | `Unverified` |
| `UISRC-BIND-005` | Provider/internal/reference build evidence conflicts | `Mismatch`/conflict |
| `UISRC-BIND-006` | Nearest build fallback | reject |
| `UISRC-BIND-007` | Cross-flavor source/reference join | incompatible/NotEvaluated |
| `UISRC-BIND-008` | Exact build state upgraded from provider popularity | mutation fails |
| `UISRC-BIND-009` | Build evidence changes, bytes same | new binding/publication eligibility; no relabel |
| `UISRC-BIND-010` | Current selector lacks profile/build key | reject ambiguous |
| `UISRC-BIND-011` | Update build B replaces current build A selector | mutation fails |
| `UISRC-BIND-012` | Exact source requested, only last-known-good other build exists | explicit mismatch fallback only |

## Root, package, TOC and global units

| ID | Case | Expected |
|---|---|---|
| `UISRC-ROOT-001` | Declared shared/global/package roots | pass |
| `UISRC-ROOT-002` | Root role inferred from directory name only | mutation fails |
| `UISRC-ROOT-003` | Root path renamed, role/contents/profile equivalent | semantics stable except source identities as defined |
| `UISRC-ROOT-004` | Overlapping incompatible roots | conflict/reject |
| `UISRC-ROOT-005` | Unknown root with explicit reviewed role/coverage | preserve |
| `UISRC-ROOT-006` | Excluded tooling root explicitly outside universe | excluded with coverage record |
| `UISRC-ROOT-007` | Silent excluded in-scope root | incomplete |
| `UISRC-PKG-001` | Valid TOC package under declared package root | pass |
| `UISRC-PKG-002` | Global unit without TOC from explicit manifest | pass |
| `UISRC-PKG-003` | Directory treated as package by path heuristic | mutation fails |
| `UISRC-PKG-004` | Multiple TOC variants | exactly one selected per profile |
| `UISRC-PKG-005` | Cross-variant merge fills missing facts | mutation fails |
| `UISRC-PKG-006` | Required vs optional dependencies | remain distinct |
| `UISRC-PKG-007` | TOC file order shuffled by filesystem | mutation fails |
| `UISRC-PKG-008` | LoadOnDemand/bootstrap | static metadata only |
| `UISRC-PKG-009` | SavedVariables declarations | names/scopes only |
| `UISRC-PKG-010` | SavedVariables contents read | architecture test fails |
| `UISRC-PKG-011` | Unknown directive/file kind | preserved with narrow coverage effect |
| `UISRC-PKG-012` | Missing TOC-referenced file | conflict/partial, no runtime inference |

## XML and load model

| ID | Case | Expected |
|---|---|---|
| `UISRC-XML-001` | Valid XML includes/templates/objects/scripts | pass |
| `UISRC-XML-002` | DTD/external entity/XInclude/network catalog | disabled/reject |
| `UISRC-XML-003` | XML include cycle | bounded conflict |
| `UISRC-XML-004` | Include fanout/depth/byte bomb | deterministic bounded failure |
| `UISRC-XML-005` | Inline Lua virtual unit | exact virtual/physical mapping |
| `UISRC-XML-006` | External Script file outside snapshot/root | reject |
| `UISRC-XML-007` | Parent and inheritance collapsed | mutation fails |
| `UISRC-XML-008` | Duplicate template/object identity | ambiguity/conflict |
| `UISRC-XML-009` | Unknown element/attribute | preserved, narrow coverage impact |
| `UISRC-LOAD-001` | Direct package/file/XML/script load edges | pass |
| `UISRC-LOAD-002` | Reachable/conditional/unreachable/unknown | remain distinct |
| `UISRC-LOAD-003` | Full transitive edges materialized | mutation fails |
| `UISRC-LOAD-004` | Static load rendered as client success/readiness | mutation fails |
| `UISRC-LOAD-005` | Same direct graph, different input order | same load model |
| `UISRC-LOAD-006` | Load conflict hidden by first path | mutation fails |

## Lua analyzer and source maps

| ID | Case | Expected |
|---|---|---|
| `UISRC-AN-001` | Exact physical/virtual source workspace | pass |
| `UISRC-AN-002` | Annotation library mixed into source universe | reject |
| `UISRC-AN-003` | User project file mixed into source workspace | reject |
| `UISRC-AN-004` | Analyzer snapshot generation/config mismatch | reject |
| `UISRC-AN-005` | Missing requested unit/fact/source map | partial/reject complete claim |
| `UISRC-AN-006` | Second Lua parser/raw-source fallback | architecture test fails |
| `UISRC-AN-007` | Malformed Lua unit | affected capability partial; no execution |
| `UISRC-AN-008` | Analyzer failure | no raw-source reconstruction |
| `UISRC-AN-009` | Virtual/physical source-map mismatch | reject |
| `UISRC-AN-010` | Symbol key derived only from line/name | mutation fails |
| `UISRC-AN-011` | 1/2/N analyzer worker scheduling | same normalized manifests where upstream contract allows |
| `UISRC-AN-012` | Source comment alters analyzer/project instructions | data only |

## Fact adapters and recognizers

| ID | Case | Expected |
|---|---|---|
| `UISRC-FACT-001` | Typed inventory/TOC/XML/load/analyzer fact bundles | pass |
| `UISRC-FACT-002` | Fact loses source generation/handle/evidence | reject |
| `UISRC-FACT-003` | Unsupported mapping omitted without loss record | reject |
| `UISRC-FACT-004` | Source and reference fact universes collapsed | reject |
| `UISRC-FACT-005` | Same facts shuffled | same bundle ID/bytes |
| `UISRC-FACT-006` | Dynamic/ambiguous fact promoted exact | mutation fails |
| `UISRC-REC-001` | Approved core pack over normalized facts | pass |
| `UISRC-REC-002` | Recognizer reparses raw source | architecture test fails |
| `UISRC-REC-003` | Rule conditions on provider/repository/path popularity | mutation fails |
| `UISRC-REC-004` | Named Blizzard heuristic activated without reviewed pack | reject |
| `UISRC-REC-005` | Competing matches | all retained |
| `UISRC-REC-006` | First/last match chosen | mutation fails |
| `UISRC-REC-007` | Partial input yields complete no-match | mutation fails |
| `UISRC-REC-008` | Rule disabled/version changed | only owning partition removed/rebuilt |
| `UISRC-REC-009` | Hook recognition claims runtime safety | mutation fails |
| `UISRC-REC-010` | Native/custom/EventRegistry/CVar signals merged | mutation fails |
| `UISRC-REC-011` | SavedVariables root without TOC declaration | no confirmed root |
| `UISRC-REC-012` | Renamed provider/root with same structural facts | universal matches stable |

## Source graph

| ID | Case | Expected |
|---|---|---|
| `UISRC-GRAPH-001` | Valid source entities and direct relations | pass |
| `UISRC-GRAPH-002` | Universe equals `blizzard_ui_source` exact generation | pass |
| `UISRC-GRAPH-003` | Source function merges with reference API by name | reject |
| `UISRC-GRAPH-004` | Source entity merges with user project by path/signature | reject |
| `UISRC-GRAPH-005` | Provider/checkout path enters semantic key | mutation fails |
| `UISRC-GRAPH-006` | Direct/source/recognizer/bridge partitions separate | pass |
| `UISRC-GRAPH-007` | Graph rejects proposal | retained in candidate report |
| `UISRC-GRAPH-008` | Graph schema weakened to accept proposal | architecture mutation fails |
| `UISRC-GRAPH-009` | Generic parent relation | reject |
| `UISRC-GRAPH-010` | Object parent/inheritance axes separate | pass |
| `UISRC-GRAPH-011` | Possible call emitted as proven call | reject |
| `UISRC-GRAPH-012` | Transitive path persisted as direct edge | mutation fails |
| `UISRC-GRAPH-013` | Source file removed | stale entities/relations/assertions gone |
| `UISRC-GRAPH-014` | Other producer partition unchanged | pass |
| `UISRC-GRAPH-015` | Complete store but partial source coverage | graph summary remains partial |
| `UISRC-GRAPH-016` | Same assertion set randomized | same generation/manifest |

## Reference/source bridges

| ID | Case | Expected |
|---|---|---|
| `UISRC-BRIDGE-001` | Exact analyzer-resolved source call to exact reference API | Derived `uses_api` |
| `UISRC-BRIDGE-002` | Source and reference profile incompatible | NotEvaluated/incompatible |
| `UISRC-BRIDGE-003` | Same string, unresolved lexical/member target | no exact bridge |
| `UISRC-BRIDGE-004` | Case-insensitive nearest/suffix match | reject |
| `UISRC-BRIDGE-005` | Alias resolution under exact reference alias record | pass with evidence |
| `UISRC-BRIDGE-006` | Multiple compatible reference endpoints | ambiguity, no arbitrary choice |
| `UISRC-BRIDGE-007` | Missing reference endpoint complete authority | typed bridge no-match only |
| `UISRC-BRIDGE-008` | Missing endpoint partial authority | nonauthoritative/NotEvaluated |
| `UISRC-BRIDGE-009` | UI source call rendered public API declaration | mutation fails |
| `UISRC-BRIDGE-010` | UI source absence rendered API absence | mutation fails |
| `UISRC-BRIDGE-011` | Native frame event bridge exact | distinct event relation |
| `UISRC-BRIDGE-012` | Custom EventRegistry string treated native event | mutation fails |
| `UISRC-BRIDGE-013` | CVar callback treated frame event | mutation fails |
| `UISRC-BRIDGE-014` | Bridge emits Proven confidence | reject |
| `UISRC-BRIDGE-015` | Possible endpoint promoted Derived | reject |
| `UISRC-BRIDGE-016` | Bridge rule/profile update | exact partition invalidated |
| `UISRC-BRIDGE-017` | Reference generation update, source unchanged | only bridge/dependent partitions invalidated |
| `UISRC-BRIDGE-018` | User-project hook bridge without ProjectSnapshot | reject/deferred |
| `UISRC-BRIDGE-019` | Project/UI same method name creates hook relation | mutation fails |
| `UISRC-BRIDGE-020` | Exact future project hook target resolution | structurally derived, no safety claims |

## Authority, coverage and conflicts

| ID | Case | Expected |
|---|---|---|
| `UISRC-AUTH-001` | Source bytes used as implementation-structure evidence | pass |
| `UISRC-AUTH-002` | Source used as APIDocumentation replacement | reject |
| `UISRC-AUTH-003` | Source call proves addon API support | reject |
| `UISRC-AUTH-004` | Source absence proves API absence | reject |
| `UISRC-AUTH-005` | Static source proves runtime frame exists now | reject |
| `UISRC-AUTH-006` | Static source proves event payload readable | reject |
| `UISRC-AUTH-007` | Static source proves Secret/taint/combat/protected safety | reject |
| `UISRC-AUTH-008` | Comment claims behavior | source comment evidence only |
| `UISRC-COV-001` | All intended roots/files resolved | complete source profile coverage |
| `UISRC-COV-002` | Unknown omitted in-scope root/file | incomplete |
| `UISRC-COV-003` | Parser partial but inventory complete | separate states |
| `UISRC-COV-004` | Recognizer failed but direct facts complete | direct facts remain; recognizer partial |
| `UISRC-COV-005` | Bridge unavailable but source graph complete | separate capability |
| `UISRC-COV-006` | Query truncated | explicit nonauthoritative bounded result |
| `UISRC-COV-007` | Empty source relation complete exact scope | scoped negative authority only |
| `UISRC-COV-008` | Empty relation partial scope | no absence claim |
| `UISRC-CONFLICT-001` | Provider revision/content disagreement | conflict |
| `UISRC-CONFLICT-002` | Duplicate normalized path/content | conflict/reject |
| `UISRC-CONFLICT-003` | Build evidence disagreement | mismatch/conflict |
| `UISRC-CONFLICT-004` | Source/reference endpoint disagreement | bridge conflict |
| `UISRC-CONFLICT-005` | Last-write/provider-majority resolution | mutation fails |

## License and redistribution

| ID | Case | Expected |
|---|---|---|
| `UISRC-LIC-001` | Exact license records for every admitted root/file | pass |
| `UISRC-LIC-002` | Missing/unknown license | local-only or ineligible, never redistributable |
| `UISRC-LIC-003` | Conflicting license declarations | conflict |
| `UISRC-LIC-004` | Public repository treated as license grant | mutation fails |
| `UISRC-LIC-005` | Raw source export without positive decision | reject |
| `UISRC-LIC-006` | Source excerpt local-only | enforce channel |
| `UISRC-LIC-007` | Source excerpt requires notice | notice closure required |
| `UISRC-LIC-008` | Derived facts assumed unrestricted automatically | mutation fails |
| `UISRC-LIC-009` | Graph/database reconstructs substantial source | separate review decision |
| `UISRC-LIC-010` | Required notice missing/wrong digest | release reject |
| `UISRC-LIC-011` | Decision for old revision reused | reject |
| `UISRC-LIC-012` | License changes, source semantic bytes same | semantic facts may reuse; decisions/artifacts invalidated |
| `UISRC-LIC-013` | Minimal synthetic fixture | unrestricted under repository fixture policy |
| `UISRC-LIC-014` | Real-source test copies whole tree into repository | architecture/license test fails |
| `UISRC-LIC-015` | Pack local publication confused with release eligibility | mutation fails |

## Invalidation and removal

| ID | Case | Expected |
|---|---|---|
| `UISRC-INV-001` | Lua file content change | exact analyzer/downstream invalidation |
| `UISRC-INV-002` | XML include/script change | exact transitive dependent invalidation |
| `UISRC-INV-003` | TOC order/variant/dependency change | load/workspace/downstream invalidation |
| `UISRC-INV-004` | Root role/boundary change | affected root or wider re-key/rebuild |
| `UISRC-INV-005` | Analyzer profile change | analyzer/adapters/recognizers/graph/bridges invalidated |
| `UISRC-INV-006` | Recognizer version change | owning output/graph partitions only |
| `UISRC-INV-007` | Graph registry identity change | affected graph/bridge/publication invalidated |
| `UISRC-INV-008` | Reference generation change | bridge/dependent partitions invalidated |
| `UISRC-INV-009` | Build-binding change only | eligibility/current selector/bridges invalidated; no relabel |
| `UISRC-INV-010` | License decision change only | export/notice/artifact decisions invalidated |
| `UISRC-INV-011` | Unrelated exact partition with reuse proof | reused |
| `UISRC-INV-012` | Same path/name/mtime only | insufficient reuse proof |
| `UISRC-INV-013` | Removed file | complete stale-derived closure |
| `UISRC-INV-014` | Removed reference alias | stale bridge removed |
| `UISRC-INV-015` | Unknown impact | conservative widening |
| `UISRC-INV-016` | Equivalent final target through different update orders | same target IDs/bytes |
| `UISRC-INV-017` | Exact no-change | no new semantic generation/current record |
| `UISRC-INV-018` | Cross-build lineage inferred inside E3-B | reject/deferred |

## Publication, readers and recovery

| ID | Case | Expected |
|---|---|---|
| `UISRC-PUB-001` | Valid dedicated source publication bundle | pass |
| `UISRC-PUB-002` | Bundle contains raw SQL/connection/table/PRAGMA | reject |
| `UISRC-PUB-003` | Build inactive generation | current unchanged |
| `UISRC-PUB-004` | Fresh read-back validates source/graph/bridges/license | `ValidatedInactive` |
| `UISRC-PUB-005` | Activation CAS exact base | current advances |
| `UISRC-PUB-006` | Stale current before CAS | activation fails, target inactive |
| `UISRC-PUB-007` | Current advanced in build transaction | mutation fails |
| `UISRC-PUB-008` | SQLite commit succeeds, domain validation fails | no activation/quarantine |
| `UISRC-PUB-009` | User project current record mutated | architecture test fails |
| `UISRC-PUB-010` | Old reader open while new activates | old exact snapshot retained |
| `UISRC-PUB-011` | New reader after activation | new coherent source publication |
| `UISRC-PUB-012` | Reader mixes source/graph/reference generations | reject |
| `UISRC-PUB-013` | Crash before inactive commit | current intact/recovery classified |
| `UISRC-PUB-014` | Crash after inactive commit before validation | recover inactive, not current |
| `UISRC-PUB-015` | Crash during/after CAS | atomic old-or-new current record |
| `UISRC-PUB-016` | Failed target relabels last-known-good | mutation fails |
| `UISRC-PUB-017` | Explicit rollback to retained validated source publication | new CAS record, identities preserved |
| `UISRC-PUB-018` | GC deletes current/reader/evidence/notice dependency | mutation fails |
| `UISRC-PUB-019` | Build B current replaces build A selector | mutation fails |
| `UISRC-PUB-020` | Partial/local publication presented production exact | reject |

## Security and budgets

| ID | Case | Expected |
|---|---|---|
| `UISRC-SEC-001` | Direct network/Git/GitHub access in library | architecture test fails |
| `UISRC-SEC-002` | Arbitrary filesystem discovery/open | architecture test fails |
| `UISRC-SEC-003` | Process/shell/editor/LSP/WoW-client access | architecture test fails |
| `UISRC-SEC-004` | Raw SQL/SQLite extension/ATTACH/store handle | architecture test fails |
| `UISRC-SEC-005` | Lua/XML/TOC/build/generated code execution | architecture test fails |
| `UISRC-SEC-006` | Source prompt/tool/SQL/shell text | bounded data only |
| `UISRC-SEC-007` | Huge paths/files/lines/comments | bounded failure/partial |
| `UISRC-SEC-008` | XML entity/include bomb | disabled/bounded |
| `UISRC-SEC-009` | Lua analyzer complexity/cancellation | bounded/typed state |
| `UISRC-SEC-010` | Massive duplicate declarations/evidence refs | bounded conflict/report |
| `UISRC-SEC-011` | Graph/bridge fanout/path explosion | bounded deterministic truncation |
| `UISRC-SEC-012` | Provider credential/private path in log/error | redacted; mutation fails if leaked |
| `UISRC-SEC-013` | Invalid/unlimited/overflow budgets | reject |
| `UISRC-SEC-014` | Cancellation in each major phase | bounded stop, no current advance |
| `UISRC-SEC-015` | Partial/cancelled output marked complete/current | mutation fails |

## Determinism and freeze

| ID | Case | Expected |
|---|---|---|
| `UISRC-DET-001` | 1/2/N workers | same semantic IDs/manifests/artifacts |
| `UISRC-DET-002` | Shuffled root/file/fact/assertion order | same output |
| `UISRC-DET-003` | Host/temp/checkout path/clock/locale/timezone | no semantic change |
| `UISRC-DET-004` | SQLite page/WAL/checkpoint differences | no semantic change |
| `UISRC-DET-005` | Repeated build from identical sealed snapshot | same logical outputs |
| `UISRC-DET-006` | Allowed exported member bytes | deterministic under exact profile |
| `UISRC-DET-007` | Golden bytes changed without version/checksum update | fail |
| `UISRC-FIX-001` | Null real-source/profile/vector pins before code | allowed |
| `UISRC-FIX-002` | First Rust commit with required nulls | fail |
| `UISRC-FIX-003` | Synthetic fixture frozen | pass |
| `UISRC-FIX-004` | Real source snapshot/build/license fixture frozen before production claims | pass |
| `UISRC-FIX-005` | Tests rewrite golden fixtures automatically | fail |

## Acceptance

E3-B remains incomplete until all nondeferred cases execute and prove sealed snapshot admission, exact build/profile/license state, parser/analyzer reuse without execution, source/reference/project universe isolation, conservative bridges and authority ceilings, complete removal/invalidation closure, dedicated inactive-validate-CAS publication, old/new reader coherence, secure local-only defaults, and deterministic canonical outputs under ordering and concurrency mutations.
