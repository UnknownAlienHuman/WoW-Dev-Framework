# E2-D integrated publication decisions

**Status:** normative.

## PROJECT-E2D-001 — E2-C candidate is mandatory input

E2-D never reparses source or rebuilds analyzer/recognizer facts implicitly. It consumes one exact validated E2-C candidate.

## PROJECT-E2D-002 — One publication head defines current state

Current project and graph state is one `ProjectPublicationHead`, not separately mutable pointers.

## PROJECT-E2D-003 — Head includes all coherence identities

Store, project, graph, analyzer, recognizer, profile, reference, candidate, and publication bundle identities are explicit.

## PROJECT-E2D-004 — Project owns orchestration, not physical transaction

Project builds semantic plans and validates results. Store owns SQLite, transaction, seal, CAS storage, leases, and GC.

## PROJECT-E2D-005 — Graph owns final graph truth

Project supplies proposals and context; graph validates and derives final keys/assertions/conflicts/views/snapshot.

## PROJECT-E2D-006 — One store generation contains both project and graph partitions

A publication cannot head a project snapshot stored separately from its graph snapshot.

## PROJECT-E2D-007 — Candidate generation and store generation are distinct

Project generation describes semantic project input state. Store generation describes the immutable persisted logical bundle/profile.

## PROJECT-E2D-008 — Snapshot manifests are finalized after sealed reopen

Precomputed expectations are not final snapshot evidence until independent read validation succeeds.

## PROJECT-E2D-009 — CAS is the only current-state transition

No direct overwrite, merge, rebase, pointer pair, or application-owned current update.

## PROJECT-E2D-010 — Stale base/head fails

A request based on another current head cannot silently rerun against the new head.

## PROJECT-E2D-011 — Sealed inactive is not published

Physical validity and even read validation do not establish current project state without the coherent head.

## PROJECT-E2D-012 — Adoption is explicit recovery

Adoption revalidates exact candidate/bundle/snapshots/current base and executes a fresh CAS.

## PROJECT-E2D-013 — Last-known-good is not target completion

It remains independently addressable and clearly stale relative to a failed target.

## PROJECT-E2D-014 — Readers acquire one head and lease

Service/read clients receive one immutable coherent view; they never resolve project and graph separately.

## PROJECT-E2D-015 — Partial candidate publication is policy-bound

A partial E2-C candidate may publish only when exact requested capability policy permits every blocker and head/snapshot manifests expose the partial state.

## PROJECT-E2D-016 — Graph proposal rejection cannot disappear

Rejected proposals, conflicts, and coverage effects are publication artifacts and capability blockers according to explicit policy.

## PROJECT-E2D-017 — No final graph generation before store plan closure

Target graph snapshot identity derives from exact validated assertion/partition/manifests under the target store publication bundle.

## PROJECT-E2D-018 — No source/analyzer/recognizer late reads

The publication request freezes all inputs. Re-reading current component state mid-publication is forbidden.

## PROJECT-E2D-019 — No automatic retry after ambiguous CAS result

Resolve the head registry first; then classify AlreadyPublished, conflict, or recovery-required.

## PROJECT-E2D-020 — Determinism is logical

Equivalent final candidate and plans yield identical project/graph logical generations, bundle, manifests, and head. Physical artifact equality is separately reported by store.

## PROJECT-E2D-021 — E2-D does not activate service/search/context

It exposes project-owned APIs for later service integration. No transport or discovery operation is added.

## PROJECT-E2D-022 — No runtime claim

Published static index coherence does not prove client load, taint, combat, event payload access, or performance.

## PROJECT-E2D-023 — Current KB remains external

Patch-sensitive rules remain routed to the engineering KB; persistence contracts contain only stable identity/evidence boundaries.

## PROJECT-E2D-024 — No CI

Manual/local validation is recorded. Workflows are not added by convention.
