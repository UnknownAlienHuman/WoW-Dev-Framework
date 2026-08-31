# E2-D integrated publication test matrix

**Status:** normative executable acceptance and mutation matrix.

## Request, base, and candidate

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2D-REQ-001` | Valid first publication request | accepted |
| `PROJECT-E2D-REQ-002` | Valid update from exact head | accepted |
| `PROJECT-E2D-REQ-003` | Missing/floating candidate/profile/reference/store/graph profile | rejected |
| `PROJECT-E2D-REQ-004` | Candidate is mutable/not E2-C/incorrect digest | rejected |
| `PROJECT-E2D-REQ-005` | Candidate partial but policy requires complete | rejected |
| `PROJECT-E2D-REQ-006` | Candidate partial with exact permitted blockers | accepted partial target |
| `PROJECT-E2D-REQ-007` | Expected head stale | reject before graph/store |
| `PROJECT-E2D-REQ-008` | Base graph/store/head disagree | rejected |
| `PROJECT-E2D-REQ-009` | Request exceeds budget/cancelled | bounded no-target |
| `PROJECT-E2D-REQ-010` | Source re-read/floating branch requested | forbidden |

## Coherence

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2D-COH-001` | All profile/reference/project/analyzer/recognizer/graph/store IDs agree | pass |
| `PROJECT-E2D-COH-002` | Analyzer snapshot wrong project generation | reject |
| `PROJECT-E2D-COH-003` | Recognizer partition wrong pack/rule/generation | reject |
| `PROJECT-E2D-COH-004` | Graph registry/base wrong | reject |
| `PROJECT-E2D-COH-005` | Store profile/bundle incompatible | reject |
| `PROJECT-E2D-COH-006` | One source handle/content digest stale | reject |
| `PROJECT-E2D-COH-007` | Reference/profile differs from candidate | reject |
| `PROJECT-E2D-COH-008` | Same display name but different universe/generation | remains distinct |
| `PROJECT-E2D-COH-009` | Timestamp/temp/worker changes | no logical identity change |
| `PROJECT-E2D-COH-010` | Omitted coherence field mutation | collision test fails |

## Project plan

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2D-PLAN-001` | Candidate records map to complete registered project plan | pass |
| `PROJECT-E2D-PLAN-002` | Source/TOC/XML/load/Lua/analyzer/recognizer manifest missing | reject |
| `PROJECT-E2D-PLAN-003` | Removed source record retained | reject |
| `PROJECT-E2D-PLAN-004` | Project plan rewrites analyzer facts | prohibited |
| `PROJECT-E2D-PLAN-005` | Project plan rewrites recognizer outcomes | prohibited |
| `PROJECT-E2D-PLAN-006` | Exact duplicate records/evidence | canonical dedupe with evidence retained |
| `PROJECT-E2D-PLAN-007` | Shuffled candidate serialization | same plan |
| `PROJECT-E2D-PLAN-008` | Project operation payload contains SQL/path/callback | rejected |

## Graph plan

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2D-GRAPH-001` | Valid project+recognizer proposal partitions | exact graph plan |
| `PROJECT-E2D-GRAPH-002` | Graph rejects invalid endpoint/kind | rejection retained; policy applies |
| `PROJECT-E2D-GRAPH-003` | Graph conflict affects mandatory capability | publication blocked |
| `PROJECT-E2D-GRAPH-004` | Allowed conflict/partial policy | explicit partial snapshot/head |
| `PROJECT-E2D-GRAPH-005` | Project changes rejected proposal to pass | mutation fails |
| `PROJECT-E2D-GRAPH-006` | Stale producer partitions removed | pass |
| `PROJECT-E2D-GRAPH-007` | Other producer assertions remain | pass |
| `PROJECT-E2D-GRAPH-008` | Graph plan base generation mismatch | reject |
| `PROJECT-E2D-GRAPH-009` | Candidate/path relation promoted to proven | reject |
| `PROJECT-E2D-GRAPH-010` | Graph plan order shuffled | same canonical plan |

## Publication bundle

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2D-BUNDLE-001` | Valid merged project+graph+store plan | pass |
| `PROJECT-E2D-BUNDLE-002` | Registered invocation ID collision | reject |
| `PROJECT-E2D-BUNDLE-003` | Missing/cyclic cross-domain prerequisite | reject |
| `PROJECT-E2D-BUNDLE-004` | Expected counts/digests inconsistent | reject |
| `PROJECT-E2D-BUNDLE-005` | Object refs missing/foreign | reject |
| `PROJECT-E2D-BUNDLE-006` | Capability/conflict/coverage record omitted | reject |
| `PROJECT-E2D-BUNDLE-007` | Raw SQL/store path/connection included | reject |
| `PROJECT-E2D-BUNDLE-008` | 1/2/N/shuffled inputs | identical bundle ID/bytes |

## Store integration

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2D-STORE-001` | Store transaction/seal/open succeeds | inactive validated result |
| `PROJECT-E2D-STORE-002` | Store rejects stale head | no target/head change |
| `PROJECT-E2D-STORE-003` | Project operation fails | graph writes absent |
| `PROJECT-E2D-STORE-004` | Graph operation fails | project writes absent |
| `PROJECT-E2D-STORE-005` | Checkpoint/seal/open/checksum fails | no head |
| `PROJECT-E2D-STORE-006` | Store returns wrong generation/artifact/profile | reject |
| `PROJECT-E2D-STORE-007` | Store says success but read report missing | reject |
| `PROJECT-E2D-STORE-008` | Physically valid generation under another bundle | reject |
| `PROJECT-E2D-STORE-009` | Cancellation during store phase | exact inactive/staging classification |
| `PROJECT-E2D-STORE-010` | Prior head/readers during failed target | unchanged/stable |

## Post-open validation

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2D-VALID-001` | Project records/manifests equal candidate | pass |
| `PROJECT-E2D-VALID-002` | Stale/removed project record exists | fail |
| `PROJECT-E2D-VALID-003` | Analyzer/recognizer binding mismatch | fail |
| `PROJECT-E2D-VALID-004` | Graph registry/assertion/partition manifests equal plan | pass |
| `PROJECT-E2D-VALID-005` | Dangling/stale graph assertion | fail |
| `PROJECT-E2D-VALID-006` | Project golden read fails | no head |
| `PROJECT-E2D-VALID-007` | Graph golden query differs | no head |
| `PROJECT-E2D-VALID-008` | Coverage/conflict summary stronger than records | fail |
| `PROJECT-E2D-VALID-009` | Candidate partial hidden as complete | fail |
| `PROJECT-E2D-VALID-010` | Writer report trusted without independent reads | mutation fails |

## Snapshot and coherence manifests

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2D-SNAPSHOT-001` | Valid shared coherence manifest | pass |
| `PROJECT-E2D-SNAPSHOT-002` | Valid ProjectSnapshot | pass |
| `PROJECT-E2D-SNAPSHOT-003` | Valid GraphSnapshot | pass |
| `PROJECT-E2D-SNAPSHOT-004` | Project/graph snapshot cross-reference mismatch | reject |
| `PROJECT-E2D-SNAPSHOT-005` | Snapshot identity cycle | reject |
| `PROJECT-E2D-SNAPSHOT-006` | Store generation/artifact mismatch | reject |
| `PROJECT-E2D-SNAPSHOT-007` | Snapshot finalized before sealed validation | mutation fails |
| `PROJECT-E2D-SNAPSHOT-008` | Post-seal DB mutation to add snapshot | prohibited |
| `PROJECT-E2D-SNAPSHOT-009` | Equivalent logical publication | same snapshot/coherence IDs |
| `PROJECT-E2D-SNAPSHOT-010` | Physical profile changes only | logical equality explicitly classified |

## Head publication

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2D-HEAD-001` | First publication | one coherent head |
| `PROJECT-E2D-HEAD-002` | Exact update H1 to H2 | CAS success |
| `PROJECT-E2D-HEAD-003` | Another publisher advances first | CAS conflict |
| `PROJECT-E2D-HEAD-004` | Separate project/graph heads | prohibited |
| `PROJECT-E2D-HEAD-005` | Head mixes store/project/graph generations | reject |
| `PROJECT-E2D-HEAD-006` | Crash before CAS | old head |
| `PROJECT-E2D-HEAD-007` | Crash during CAS | resolve exact old/new |
| `PROJECT-E2D-HEAD-008` | Crash after CAS | new headed publication |
| `PROJECT-E2D-HEAD-009` | Replay exact current target | AlreadyPublished |
| `PROJECT-E2D-HEAD-010` | CAS conflict silently rebase/retry | mutation fails |

## Published views and readers

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2D-VIEW-001` | Open current head once | coherent project+graph view |
| `PROJECT-E2D-VIEW-002` | Publish H2 while H1 leased | H1 view stable |
| `PROJECT-E2D-VIEW-003` | Resolve project and graph independently | prohibited |
| `PROJECT-E2D-VIEW-004` | Exact historical retained head | coherent historical view |
| `PROJECT-E2D-VIEW-005` | Required capability partial/conflicted | typed unavailable/NotEvaluated |
| `PROJECT-E2D-VIEW-006` | Missing exact head falls back current | prohibited |
| `PROJECT-E2D-VIEW-007` | Close releases lease | pass |
| `PROJECT-E2D-VIEW-008` | Read view exposes mutable store/graph/analyzer | API test fails |

## Failure, cancellation, and recovery

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2D-RECOVERY-001` | Pre-store failure | no target/current change |
| `PROJECT-E2D-RECOVERY-002` | Store rollback | staging cleanup state |
| `PROJECT-E2D-RECOVERY-003` | Sealed invalid | quarantine/inactive |
| `PROJECT-E2D-RECOVERY-004` | Valid sealed inactive, base still current | adoptable after full revalidation |
| `PROJECT-E2D-RECOVERY-005` | Valid sealed inactive, base stale | not adoptable |
| `PROJECT-E2D-RECOVERY-006` | Domain validation fails transiently then fixed validator | explicit exact revalidation policy |
| `PROJECT-E2D-RECOVERY-007` | Ambiguous CAS result | registry resolution |
| `PROJECT-E2D-RECOVERY-008` | Current generation corrupt | critical; no silent LKG switch |
| `PROJECT-E2D-RECOVERY-009` | Recovery mutates sealed store | prohibited |
| `PROJECT-E2D-RECOVERY-010` | Cancel before CAS | old head; classified target |
| `PROJECT-E2D-RECOVERY-011` | Cancel after CAS success | published result |
| `PROJECT-E2D-RECOVERY-012` | Late background work after cancel | prohibited |

## Last-known-good

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2D-LKG-001` | Failed target with prior head | prior head reported unchanged |
| `PROJECT-E2D-LKG-002` | LKG equals current | explicit |
| `PROJECT-E2D-LKG-003` | LKG relabeled target generation | prohibited |
| `PROJECT-E2D-LKG-004` | LKG merged with target graph/project data | prohibited |
| `PROJECT-E2D-LKG-005` | No LKG | typed unavailable |
| `PROJECT-E2D-LKG-006` | Explicit historical LKG read | original identities |

## Security and boundaries

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2D-SEC-001` | Source/repository instruction influences publication | inert data |
| `PROJECT-E2D-SEC-002` | SQL/path/connection/store callback from project | rejected |
| `PROJECT-E2D-SEC-003` | Filesystem/network/process/editor/client access | absent |
| `PROJECT-E2D-SEC-004` | Runtime/taint/combat safety claim from static publication | prohibited |
| `PROJECT-E2D-SEC-005` | Private path/token/source payload in report | fail/redact |
| `PROJECT-E2D-SEC-006` | Oversized/cyclic bundle/manifests | bounded failure |
| `PROJECT-E2D-SEC-007` | CI/workflow added | policy test fails |
| `PROJECT-E2D-SEC-008` | Dependency outside ceiling | architecture test fails |

## Determinism and freeze

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2D-DET-001` | 1/2/N workers | same logical publication |
| `PROJECT-E2D-DET-002` | Candidate/plan/input order shuffled | same IDs/bytes |
| `PROJECT-E2D-DET-003` | Temp root/time/process/SQLite row order | excluded |
| `PROJECT-E2D-DET-004` | Different valid update sequence, same final candidate | same publication |
| `PROJECT-E2D-FIX-001` | Null pins before implementation | allowed |
| `PROJECT-E2D-FIX-002` | First Rust commit with required nulls | fail |
| `PROJECT-E2D-FIX-003` | Fixture bytes change without checksum | fail |
| `PROJECT-E2D-FIX-004` | All vectors/checksums frozen | pass |

## Acceptance

E2-D is incomplete until every nondeferred case executes and the combined store/project/graph fault matrix proves one coherent old-or-new head under failure and concurrency.
