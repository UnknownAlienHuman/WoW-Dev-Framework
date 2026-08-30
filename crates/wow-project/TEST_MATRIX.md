# `wow-project` E0-D test matrix

**Status:** normative executable acceptance matrix for the future implementation.

Tests compare structured configuration/generation/source/analyzer/publication state and exact canonical bytes. Message-only assertions are insufficient.

## 1. Project configuration

| ID | Case | Expected |
|---|---|---|
| `PROJECT-CONFIG-001` | Valid closed E0 project configuration | accepted |
| `PROJECT-CONFIG-002` | Fixture marked release repository | `project_fixture_release_masquerade` |
| `PROJECT-CONFIG-003` | Floating profile/reference/current selector | rejected |
| `PROJECT-CONFIG-004` | Profile and reference generation mismatch | rejected |
| `PROJECT-CONFIG-005` | Missing/unaccepted analyzer pin/probe/config identity after implementation starts | rejected |
| `PROJECT-CONFIG-006` | Invalid capability policy overlap | rejected |
| `PROJECT-CONFIG-007` | Invalid/negative budget | rejected |
| `PROJECT-CONFIG-008` | Temp/local root included in public config | rejected or removed from canonical identity |
| `PROJECT-CONFIG-009` | Attempt to activate store/graph/recognizers in E0 | rejected |
| `PROJECT-CONFIG-010` | Equivalent map/order configuration permutations | identical canonical config bytes/digest |

## 2. Project input inventory

| ID | Case | Expected |
|---|---|---|
| `PROJECT-INPUT-001` | Exact four Main Lua files | accepted complete inventory |
| `PROJECT-INPUT-002` | Discovery order shuffled | identical canonical manifest |
| `PROJECT-INPUT-003` | Missing declared file | exact file/partition failure; no publish if mandatory |
| `PROJECT-INPUT-004` | Undeclared file | rejected |
| `PROJECT-INPUT-005` | Duplicate file ID/path | rejected |
| `PROJECT-INPUT-006` | Digest mismatch | rejected |
| `PROJECT-INPUT-007` | Byte length mismatch | rejected |
| `PROJECT-INPUT-008` | Invalid UTF-8 | rejected |
| `PROJECT-INPUT-009` | Absolute/traversal/UNC/device/tokenized path | rejected |
| `PROJECT-INPUT-010` | Case-fold collision | rejected deterministically |
| `PROJECT-INPUT-011` | Library role file inserted | rejected |
| `PROJECT-INPUT-012` | TOC/XML/non-Lua file inserted in E0 | typed unsupported/rejected |
| `PROJECT-INPUT-013` | File count/source byte budget exceeded | rejected |
| `PROJECT-INPUT-014` | Source fixture reference points to unknown E0-C member | rejected |

## 3. Source registry

| ID | Case | Expected |
|---|---|---|
| `PROJECT-SOURCE-001` | Register valid project origin and files | registry complete |
| `PROJECT-SOURCE-002` | File lookup by exact ID | exact record |
| `PROJECT-SOURCE-003` | File lookup by canonical path | exact record |
| `PROJECT-SOURCE-004` | Noncanonical/fuzzy/case-correcting path lookup | rejected/no guessing |
| `PROJECT-SOURCE-005` | Base source handle validates for exact generation/digest | pass |
| `PROJECT-SOURCE-006` | Source handle from another generation | rejected |
| `PROJECT-SOURCE-007` | Stale digest after update | rejected |
| `PROJECT-SOURCE-008` | Library handle claims project origin | rejected |
| `PROJECT-SOURCE-009` | Reference/external origin claims project role | rejected |
| `PROJECT-SOURCE-010` | Analyzer Main file maps one-to-one | pass |
| `PROJECT-SOURCE-011` | Extra/missing analyzer Main file | publication fails |
| `PROJECT-SOURCE-012` | Temp root differs across runs | public registry/handles identical |
| `PROJECT-SOURCE-013` | Absolute/private path leaks into error/output | test fails |
| `PROJECT-SOURCE-014` | Removed file resolved in new generation | rejected |
| `PROJECT-SOURCE-015` | Removed file resolved through retained old snapshot | valid only under old generation |

## 4. Generation derivation

| ID | Case | Expected |
|---|---|---|
| `PROJECT-GEN-001` | Baseline derivation inputs | deterministic target ID |
| `PROJECT-GEN-002` | File discovery order shuffled | same ID |
| `PROJECT-GEN-003` | Same final files through different valid update order | same ID |
| `PROJECT-GEN-004` | One file content digest changes | different ID |
| `PROJECT-GEN-005` | File add/remove/path/role changes | different ID |
| `PROJECT-GEN-006` | Profile/reference generation changes | different ID |
| `PROJECT-GEN-007` | Analyzer pin/probe/config changes | different ID |
| `PROJECT-GEN-008` | Capability/output-affecting budget policy changes | different ID |
| `PROJECT-GEN-009` | Timestamp/temp root/thread ID changes | same ID; excluded |
| `PROJECT-GEN-010` | Rendered diagnostic wording changes only | same project generation inputs |
| `PROJECT-GEN-011` | Omit semantic input from derivation mutation | test detects same-ID collision |
| `PROJECT-GEN-012` | Include volatile input mutation | test detects nondeterminism |
| `PROJECT-GEN-013` | Candidate ID treated as published generation | prohibited |
| `PROJECT-GEN-014` | Different semantic inputs forced same ID | validation/hash collision contract failure |

## 5. Initial analyzer binding

| ID | Case | Expected |
|---|---|---|
| `PROJECT-ANALYZER-001` | Build exact generation-bound update batch | matches candidate manifest/context |
| `PROJECT-ANALYZER-002` | Analyzer returns same project/profile/reference generation | pass |
| `PROJECT-ANALYZER-003` | Analyzer pin/config identity match | pass |
| `PROJECT-ANALYZER-004` | Main file manifest exact match | pass |
| `PROJECT-ANALYZER-005` | Analyzer project generation mismatch | reject publication |
| `PROJECT-ANALYZER-006` | Profile/reference mismatch | reject |
| `PROJECT-ANALYZER-007` | Pin/config mismatch | reject |
| `PROJECT-ANALYZER-008` | Extra/missing/wrong-digest Main file | reject |
| `PROJECT-ANALYZER-009` | Library file appears in project Main manifest | reject |
| `PROJECT-ANALYZER-010` | Project source handles cannot map to registry | reject |
| `PROJECT-ANALYZER-011` | Missing required analyzer coverage records | reject/degrade per exact policy |
| `PROJECT-ANALYZER-012` | Raw upstream analyzer handle exposed | API boundary test fails |
| `PROJECT-ANALYZER-013` | Project layer rewrites analyzer facts/findings | prohibited |

## 6. Snapshot assembly and publication

| ID | Case | Expected |
|---|---|---|
| `PROJECT-PUB-001` | Baseline valid candidate/analyzer/coverage | one immutable snapshot published |
| `PROJECT-PUB-002` | Current pointer changes only after validation | pass |
| `PROJECT-PUB-003` | Candidate/file manifest exposed before analyzer success | mutation test fails |
| `PROJECT-PUB-004` | Analyzer success but project snapshot digest invalid | reject |
| `PROJECT-PUB-005` | Mandatory capability unavailable | reject |
| `PROJECT-PUB-006` | Degradable per-file fact capability failed and policy permits | coherent degraded snapshot with exact coverage |
| `PROJECT-PUB-007` | Degraded capability hidden as clean empty output | mutation test fails |
| `PROJECT-PUB-008` | Deferred E2 capabilities explicit NotEvaluated | pass |
| `PROJECT-PUB-009` | Deferred capability returned Complete empty data | fail |
| `PROJECT-PUB-010` | Published snapshot mutated | rejected |
| `PROJECT-PUB-011` | Mixed old/new file/analyzer state | reject |
| `PROJECT-PUB-012` | Randomized insertion order | identical canonical snapshot bytes/digest |
| `PROJECT-PUB-013` | Published snapshot references unresolved ID | reject |
| `PROJECT-PUB-014` | Candidate publication cancelled | no target snapshot/current-pointer change |

## 7. Last-known-good

| ID | Case | Expected |
|---|---|---|
| `PROJECT-LKG-001` | Target analyzer update fails with prior snapshot | prior retained under old ID |
| `PROJECT-LKG-002` | Status reports failed candidate and retained old generation separately | pass |
| `PROJECT-LKG-003` | Prior snapshot relabeled target generation | prohibited |
| `PROJECT-LKG-004` | Request requires failed target but service substitutes old snapshot | prohibited |
| `PROJECT-LKG-005` | Old source/facts merged into candidate | prohibited |
| `PROJECT-LKG-006` | No prior snapshot | typed last-known-good unavailable |
| `PROJECT-LKG-007` | Retry uses stale expected generation after another publish | rejected |

## 8. Update preconditions and conflicts

| ID | Case | Expected |
|---|---|---|
| `PROJECT-UPDATE-001` | Update exact file with matching old digest/generation | candidate accepted |
| `PROJECT-UPDATE-002` | Stale expected project generation | reject before analyzer mutation |
| `PROJECT-UPDATE-003` | Stale expected old digest | reject |
| `PROJECT-UPDATE-004` | Add existing file | reject |
| `PROJECT-UPDATE-005` | Update/remove missing file | reject |
| `PROJECT-UPDATE-006` | Add+add same file | conflicting operations reject |
| `PROJECT-UPDATE-007` | Update+update incompatible base | reject |
| `PROJECT-UPDATE-008` | Remove+update same file | reject |
| `PROJECT-UPDATE-009` | Path/role/type/root violation | reject |
| `PROJECT-UPDATE-010` | Operation/final-state budget exceeded | reject |
| `PROJECT-UPDATE-011` | Same-content no-op | explicit NoChange; no analyzer mutation/new generation |
| `PROJECT-UPDATE-012` | No-op creates arbitrary new generation | mutation test fails |
| `PROJECT-UPDATE-013` | Profile/pin/config changes hidden in file update | reject |
| `PROJECT-UPDATE-014` | Explicit configuration change | full new generation/analyzer validation required |

## 9. Successful update effects

| ID | Case | Expected |
|---|---|---|
| `PROJECT-EFFECT-001` | Change generic-error file to clean | new generation; old generic finding absent |
| `PROJECT-EFFECT-002` | Change missing-api source to KnownApi | new generation; unresolved facts replaced by resolved facts |
| `PROJECT-EFFECT-003` | Update file retains logical file ID | pass |
| `PROJECT-EFFECT-004` | Update changes digest/length/source handles/spans | pass |
| `PROJECT-EFFECT-005` | Add optional synthetic Main Lua file | new file ID/manifest/analyzer state after publication |
| `PROJECT-EFFECT-006` | Remove optional file | absent current registry/analyzer facts/findings |
| `PROJECT-EFFECT-007` | Old snapshot still resolves old file/content | pass under old generation |
| `PROJECT-EFFECT-008` | Same final state via different operation order | same generation/snapshot bytes |

## 10. Analyzer/update failure isolation

| ID | Case | Expected |
|---|---|---|
| `PROJECT-FAIL-001` | Analyzer batch rejected | no target project publication |
| `PROJECT-FAIL-002` | Analyzer index refresh fails | no target publication; prior retained |
| `PROJECT-FAIL-003` | Analyzer session corrupts | no target publication; exact root/failure propagated |
| `PROJECT-FAIL-004` | Analyzer returns partial half-state | project validation rejects |
| `PROJECT-FAIL-005` | Analyzer cancellation | no target publish; exact state reported |
| `PROJECT-FAIL-006` | Project current pointer changes despite failure | mutation test fails |
| `PROJECT-FAIL-007` | Project derives candidate but treats failure as current | prohibited |
| `PROJECT-FAIL-008` | Per-file parse failure with coherent degraded snapshot and policy | publish only with explicit failed capability/no fabricated facts |

## 11. View/read surface

| ID | Case | Expected |
|---|---|---|
| `PROJECT-VIEW-001` | Open exact published snapshot/view | pass |
| `PROJECT-VIEW-002` | View requested for another generation | reject |
| `PROJECT-VIEW-003` | File by exact ID/path | exact record |
| `PROJECT-VIEW-004` | Facts requested with Complete capability | analyzer-bound facts returned |
| `PROJECT-VIEW-005` | Facts requested under failed capability | typed unavailable/NotEvaluated, not empty clean |
| `PROJECT-VIEW-006` | Generic findings requested under Complete capability | exact analyzer findings |
| `PROJECT-VIEW-007` | Project view changes severity/rule/root cause | prohibited |
| `PROJECT-VIEW-008` | Project view asserts API absence/Secret status | prohibited |
| `PROJECT-VIEW-009` | Raw mutable analyzer/project builder exposed | prohibited |
| `PROJECT-VIEW-010` | Canonical ordering stable | pass |

## 12. Deferred E2 capabilities

| ID | Capability | Expected E0-D |
|---|---|---|
| `PROJECT-DEFER-001` | TOC parse/model | `operation_not_implemented_for_milestone` / NotEvaluated |
| `PROJECT-DEFER-002` | XML parse/model | unavailable |
| `PROJECT-DEFER-003` | load/dependency/reachability graph | unavailable |
| `PROJECT-DEFER-004` | event/hook/state index | unavailable |
| `PROJECT-DEFER-005` | graph persistence/query | unavailable |
| `PROJECT-DEFER-006` | SQLite/project store | unavailable |
| `PROJECT-DEFER-007` | installed addons/SavedVariables/logs | unavailable |
| `PROJECT-DEFER-008` | repository scan/watch | unavailable |

Every case rejects empty/default success.

## 13. Security and policy

| ID | Case | Expected |
|---|---|---|
| `PROJECT-SEC-001` | Lua contains shell/IO/source instructions | treated as data; never executed |
| `PROJECT-SEC-002` | Repo hook/build/test/generator present | never executed |
| `PROJECT-SEC-003` | Comment asks agent to ignore policies | no policy/tool effect |
| `PROJECT-SEC-004` | Path traversal/symlink-like escape | rejected |
| `PROJECT-SEC-005` | Installed addon/client/SavedVariables path requested | rejected E0 |
| `PROJECT-SEC-006` | Excess files/source/output | bounded failure |
| `PROJECT-SEC-007` | Local path/token/private source leaked in errors | fail |
| `PROJECT-SEC-008` | Editor configuration mutation | fail |
| `PROJECT-SEC-009` | Arbitrary process/shell adapter added | API/security test fails |

## 14. Cross-crate seams

### `PROJECT-SEAM-001` — analyzer generation

Project derives target generation; analyzer batch/snapshot carries exact same ID; analyzer does not invent another project ID.

### `PROJECT-SEAM-002` — source registry

Project registers first-party files; analyzer exact spans resolve against registry; Library source remains separate.

### `PROJECT-SEAM-003` — API rule input

Project view exposes unresolved `RemovedApi` analyzer project facts/source handle only. Reference absence arrives independently later.

### `PROJECT-SEAM-004` — Secret rule input

Project view exposes local producer/use/guard facts only. Reference Secret facet arrives independently later.

### `PROJECT-SEAM-005` — generic findings

Project view exposes analyzer generic findings unchanged and generation-bound.

### `PROJECT-SEAM-006` — service snapshot acquisition

Service acquires one immutable `ProjectSnapshot`/view and cannot combine facts/findings from another generation.

## 15. Determinism matrix

Repeat baseline/final-state-equivalent updates with varied:

```text
input discovery order
independent operation order
worker/test scheduling
temporary root
hash-map insertion
analyzer fact/finding return order
```

Require byte-identical:

```text
configuration/input/source-registry manifests
ProjectGenerationId
ProjectSnapshot canonical bytes/digest
project-owned coverage/deferred records
analyzer binding identity references
```

## 16. Fixture/checksum freeze

| ID | Case | Expected |
|---|---|---|
| `PROJECT-FIXTURE-001` | Documentation-only null pin/digests/IDs | allowed only while implementation_state not-started |
| `PROJECT-FIXTURE-002` | Implementation starts with null required value | validation fails |
| `PROJECT-FIXTURE-003` | One fixture byte changes without checksum update | fail |
| `PROJECT-FIXTURE-004` | Project and analyzer fixture file digests diverge | fail |
| `PROJECT-FIXTURE-005` | Bundle/member SHA-256 and generation vectors valid after freeze | pass |

## 17. Acceptance gate

E0-D implementation is not complete until:

```text
all applicable non-deferred test IDs execute
one exact baseline ProjectSnapshot publishes
one update publishes a distinct coherent generation
stale/mismatched/failing updates never publish
last-known-good never changes identity
project/analyzer manifests and source handles match exactly
partial analyzer capabilities remain explicit
TOC/XML/load/graph capabilities never fake success
all security/no-execution/no-editor-mutation cases pass
same final state yields byte-identical project generation/snapshot
all fixture/pin/digest IDs are frozen and verified
```
