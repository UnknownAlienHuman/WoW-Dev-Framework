# E4-B lineage decisions

**Status:** normative.

## LIN-001 — Generation-local identity is immutable

An entity in generation A and a similar entity in generation B remain distinct generation-scoped entities. Lineage is an explicit relation between them.

## LIN-002 — Lineage uses a separate overlay

E4-B publishes an immutable `LineageGraphSnapshot`. It does not add cross-generation edges to or mutate E2 generation-local GraphSnapshots.

## LIN-003 — Comparison scope is exact

Every lineage operation binds exact compatible before/after generations, universe class, profiles, owner views, graph snapshots, and coverage.

## LIN-004 — Lineage and replacement are different

Identity continuity, deprecation, replacement, migration compatibility, and static impact are separate records with separate proof requirements.

## LIN-005 — Proposals are primary inputs

Every producer emits immutable evidence-bearing `LineageProposal` records. Accepted assertions preserve all proposal/evidence/review lineage.

## LIN-006 — Producers remain independent

Project stable identity, source fingerprints, structural changes, ReferenceView transitions, search candidates, and review decisions own separate replaceable partitions.

## LIN-007 — Search evidence is Candidate-only

E4-A exact and approximate retrieval signals can discover candidate pairs, but search rank or signal count alone cannot exceed `Candidate` lineage confidence.

## LIN-008 — Same name/path/body/fingerprint is not proof

Each can contribute bounded Candidate evidence. None alone establishes same lineage, move, rename, split, merge, replacement, removal, or introduction.

## LIN-009 — A unique candidate is not automatically proven

Candidate-set cardinality does not replace evidence and coverage requirements.

## LIN-010 — No unrestricted all-pairs matching

Candidate generation uses reviewed bounded blocking keys and exact per-component limits. An overbroad or incomplete block is reported explicitly.

## LIN-011 — Ambiguity components are first-class

Connected candidate components preserve all competing before/after entities and proposals. No greedy assignment or hidden winner.

## LIN-012 — One-to-many and many-to-one are valid shapes

Split, merge, copy, extraction, consolidation and unresolved ambiguity cannot be forced into one-to-one continuity.

## LIN-013 — Split and merge require evidence

Cardinality alone is not proof. The relation schema and profile define qualifying evidence and proof ceilings.

## LIN-014 — Copy and move remain distinct

If the old entity remains and a similar new entity appears, the result may be copy/extraction/candidate, not move.

## LIN-015 — Removal requires closed negative authority

`RemovedAfter` requires complete relevant before and after coverage, exact comparison scope, no unresolved candidate component, and no conflict/truncation blocker.

## LIN-016 — Introduction requires closed negative authority

`IntroducedIn` requires the symmetric complete before/after authority. An unmatched target under partial coverage remains unmatched/NotEvaluated.

## LIN-017 — Change records do not imply lineage

A signature/type/restriction/ownership/load/relation difference can be recorded only after the compared entity pair is explicitly identified under the accepted relation/profile.

## LIN-018 — Field absence and unknown are distinct

Missing, explicit null, unknown, unsupported, conflicted and omitted values produce different change outcomes.

## LIN-019 — Reference transitions preserve Reference authority

Explicit deprecation/replacement/transition/correction records from one exact ReferenceView comparison retain their platform-contract authority and scope. Search/source similarity cannot manufacture them.

## LIN-020 — Manual review is bounded evidence

A review decision is auditable provenance and can accept/reject/defer a proposal only within the maximum proof ceiling allowed by the configured review policy and input evidence.

## LIN-021 — Review never erases rejected evidence

Rejected, superseded, conflicted and deferred proposals remain available under retention policy.

## LIN-022 — Majority vote and popularity are forbidden

Producer count, search frequency, stars, downloads, repository identity, source order and reviewer count do not automatically establish truth.

## LIN-023 — Conflicts block dependent conclusions

Incompatible accepted/proposed relations, proof scopes, field changes, coverage or review decisions produce explicit conflicts and downgrade/block dependent change, removal, migration and impact claims.

## LIN-024 — Migration candidates are not recipes

Similarity/shape/reference proximity can produce a migration candidate. A recipe requires exact applicability, transformation, constraints, postconditions and validation contracts.

## LIN-025 — Recipes are advisory artifacts

E4-B does not edit code, execute migration steps, run a client, or claim success. Validation requirements remain explicit.

## LIN-026 — Static impact is path-based

Impact is a bounded typed relation path from an exact change root to exact dependent entities. A path is not flattened into a direct edge.

## LIN-027 — Static impact is not runtime failure

A structurally affected entity is not automatically broken, severe, unsafe, slow, tainted, combat-invalid or user-visible.

## LIN-028 — Possible paths remain possible

Any `Possible`/Candidate/conflicted relation in an impact path caps the path/result accordingly.

## LIN-029 — Impact scope is explicit

Relation kinds, directions, axes, depth, fanout, path count, universes and confidence classes are profile inputs.

## LIN-030 — No current resolution in graph

`wow-graph` consumes exact generations and snapshots. E4-C service later resolves symbolic current and acquires retained inputs.

## LIN-031 — Logical store schema is graph-owned

`wow-graph` owns lineage record semantics, registries, indexes, operations and validation. `wow-store` owns physical SQLite/transactions/snapshots/recovery/retention/GC.

## LIN-032 — Lineage publication is immutable

Build inactive, validate exact record/evidence/coverage/index/golden-query closure, then seal. No in-place modification of a published lineage generation.

## LIN-033 — Continuation is comparison-snapshot-bound

Candidate, trace and impact continuations bind the exact lineage universe, input generations, profiles, result manifests and cumulative budgets.

## LIN-034 — No model inference in the authority path

Models, embeddings and external memory may later submit Candidate evidence only under separate provenance. They cannot validate/promote lineage.

## LIN-035 — Determinism includes ambiguity

Equivalent exact inputs and profiles produce the same proposals, components, conflicts, assertions, change records, migration candidates, impact paths, ordering and canonical bytes under 1/2/N workers.

## LIN-036 — Corpus identity cannot become semantics

Named addon/provider/repository/path/popularity values are fixture provenance only and cannot enable production rules.

## LIN-037 — Implementation-source and platform-contract changes stay separate

A Blizzard UI implementation-source change and a ReferenceView API/restriction transition are different evidence classes even when they concern similarly named entities.

## LIN-038 — No cross-universe lineage by convenience

User project, Blizzard UI source and Reference entities do not form lineage with one another. Existing bridge/use/reference relations remain separate.

## LIN-039 — Same-lineage symmetry is explicit

Only relation kinds declared symmetric may be queried symmetrically. Directional successor/moved/renamed/replaced relations retain direction and named inverse semantics.

## LIN-040 — Negative conclusions are scoped

No absence/removal/introduction conclusion extends beyond the exact entity kind, package/root/profile/generation/producer partitions whose coverage was proven complete.
