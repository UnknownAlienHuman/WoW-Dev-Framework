# Agent workstreams and integration order

**Status:** operational routing through documentation frontier E3-C.

Documentation-ready remains `implementation_state=not-started` until executable code, probes, fixtures, checksums, and required evaluations exist.

## Global order

```text
E0-A wow-core
-> E0-B wow-reference fixture + E0-C wow-emmy
-> E0-D wow-project fixture
-> E0-E wow-rules
-> E0-F wow-service + apps/wow diagnostics

-> E1-A wow-store foundation
-> E1-B persistent wow-reference
-> E1-C wow-annotations
-> E1-D Reference Pack build/validate

-> E2-A wow-graph
-> E2-B wow-recognizers
-> E2-C full wow-project candidate
-> E2-D ProjectStore coherent publication

-> E3-A wow-project Blizzard UI source universe
-> E3-B wow-context Project Map/L0/L1/context pack
-> E3-C wow-service/apps context acquisition/use cases

-> E4-A wow-search core retrieval
-> E4-B explicit lineage/migration/impact
-> E5 calibration packs
-> E6 Codebase Memory candidate bridge
-> E7 LSP/MCP/release/publishing
```

## Global rules

- One agent owns one primary work package/crate.
- Shared seams are proposed before dependent implementation.
- A separate review pass validates every wave.
- Missing tools, implementations, probes, benchmarks, tokenizers, evaluations, runtime tests, or source producers are blocking/`NotEvaluated`, never pass.
- Patch-sensitive WoW claims route through the current external KB and exact pinned source/runtime evidence.
- Repository/addon/path/provider/popularity names never become hidden production semantics.
- No CI/workflow without explicit owner instruction.

## E0 — diagnostic vertical slice

- `wow-core`: identities, evidence, coverage, conflicts, result primitives.
- fixture `wow-reference`: one exact profile and restricted lookup.
- `wow-emmy`: pinned analyzer adapter and normalized facts/diagnostics.
- minimal `wow-project`: one coherent workspace/generation.
- `wow-rules`: `wow.api.exists` plus one Secret-local rule.
- `wow-service` + `apps/wow`: `status`, `check`, exact context, canonical result/exit mapping.

## E1 — Reference Pack

- generic `wow-store` foundation and immutable ReferenceStore;
- persistent `wow-reference` ingestion/corrections/coverage/negative authority;
- deterministic `wow-annotations` with source maps/loss/parity/consumer probes;
- service/builder app build, nonrepairing validation, rebuild comparison.

## E2 — project graph and persistence

- assertion-based typed `wow-graph` with partitions/snapshots/bounded queries;
- declarative universal `wow-recognizers` over normalized facts;
- exact TOC/XML/load/analyzer project candidate and incremental invalidation;
- one SQLite WAL ProjectStore epoch with immutable partition versions, complete membership, inactive validation and current CAS.

## E3-A — Blizzard UI source index (`wow-project`)

Exact separately published `blizzard_ui_source` universe, package/TOC/XML/Lua/analyzer/recognizer/graph pipeline, source/license/coverage/fingerprint records, incremental removal closure, and bounded `SkeletonInputView`.

No floating source, source execution, flavor merge, runtime/API authority upgrade, context rendering, or redistribution assumption.

## E3-B — Project Map and context (`wow-context`)

Exact immutable user/platform/reference universe binding; separate/combined maps; L0/L1; published-fact control/effect projection; deterministic expansion/selection/pruning; source/privacy/license boundaries; semantic packs; JSON/Markdown; continuation/cache identities; metrics/evaluation.

Exact roots only. No search/model/parser/store internals, authority upgrade, hidden omission, mandatory evidence pruning, or physical cache.

## E3-C — service and CLI context operations

Primary contracts:

- [`wow-service/e3/`](wow-service/e3/README.md)
- [`../apps/wow/e3/`](../apps/wow/e3/README.md)

Public service/CLI operations:

```text
context_status
context_map
context_inspect
context_build
context_continue
context_validate
context_render
```

E3-C service path:

```text
symbolic selector
-> resolve current once or require exact publication
-> acquire primary/platform/reference retained views in fixed order
-> validate exact compatibility and capabilities
-> bind E3-B ContextUniverseSet
-> invoke exactly one context use-case plan
-> validate semantic/render artifacts
-> admit continuation retention where required
-> close resources in reverse order
-> publish canonical service result
-> thin CLI JSON/text/artifact output and exit code
```

### E3-C ownership

`wow-service` owns request/config validation, profile-alias resolution, exact selector resolution, owner-port acquisition, compatibility validation, operation sequencing, status/envelopes, continuation retention orchestration, cancellation, and resource closure.

`apps/wow` owns strict CLI/config/artifact transport input, one service call, outputs, signals, and exit codes.

### E3-C hard stops

```text
no distributed atomic-current claim across independent stores
no second current read, hidden retry, rebase, LKG or generation fallback
no floating ReferenceView
no raw SQLite/analyzer/parser/mutable owner handle
no search/fuzzy/name/path/natural-language root
no context/map/skeleton/renderer algorithm inside service or app
no source/privacy/authority broadening
no success before all mandatory resource closes
no continuation current-resolution or total-budget reset
no physical context cache/model/CBM/runtime/edit/tool authorization
apps import wow-service only
no implicit config/source/repository/editor/client discovery
no background work/double output
```

### E3-C implementation gate

```text
implemented/frozen E0-E3-B prerequisites
exact owner acquisition/read/retention/context ports
all selector/guard/compatibility/profile/status/envelope/security/app profiles
synthetic current-race/failure/close/artifact/continuation/CLI corpora
pinned roth-ui and Blizzard UI combined integration corpus
exact canonical service JSON, artifact stdout, text fields and exit vectors
E0/E1 regression reports
1/2/N and shuffled owner scheduling deterministic
all checksums populated
```

## Next — E4-A `wow-search`

E4-A should define exact, alias, FTS, structural-shape, and bounded graph-assisted retrieval lanes plus deterministic ranking/explanations.

Boundary:

```text
wow-search returns ranked candidates with exact evidence and lane scores
wow-service exposes the operation and any explicit candidate-selection policy
wow-context continues to accept exact selected roots only
```

Similarity/ranking never authorizes lineage, replacement, remediation, or platform truth. Those require E4-B explicit lineage/impact contracts.

## E4-B and later

- E4-B: explicit cross-build lineage assertions, migration candidates, patch-impact plans and proof ceilings.
- E5: audited named calibration packs emitting universal roles only, with rename/path mutations.
- E6: optional Codebase Memory MCP bridge; external results remain Candidate.
- E7: thin LSP/MCP transports, releases, signing, publishing, rollback, operational gates.

## Seam request format

```text
requesting work package/crate
owning crate
required operation/data contract
rejected workaround
why existing read view/artifact/orchestration is insufficient
smallest proposed seam
cycle/security/evidence/privacy/license impact
fixture/mutation proving it
implementation/freeze impact
```

Do not implement a missing seam in the wrong crate.
