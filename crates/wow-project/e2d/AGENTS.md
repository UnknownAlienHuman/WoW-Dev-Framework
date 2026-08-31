# AGENTS.md — `wow-project` E2-D

## Scope

Implement only integrated project/graph publication orchestration and coherent read-view acquisition.

Do not implement SQLite, transactions, graph algorithms, recognizer matching, Lua/TOC/XML parsing, diagnostics, search, runtime probes, or application transport.

## Required reading

1. repository and `crates/` instructions;
2. [`../e2/README.md`](../e2/README.md) and E2-C contracts;
3. [`../../wow-store/e2/README.md`](../../wow-store/e2/README.md);
4. [`../../wow-graph/e2/README.md`](../../wow-graph/e2/README.md);
5. [`../../wow-recognizers/e2/OUTPUT_AND_GRAPH_HANDOFF.md`](../../wow-recognizers/e2/OUTPUT_AND_GRAPH_HANDOFF.md);
6. all files in this package;
7. current external KB routes only for patch-sensitive claims—not for persistence mechanics.

## Before coding

- Freeze all prerequisite component implementation commits and fixture digests.
- Freeze ProjectStore profile/bundle IDs.
- Freeze candidate, base, proposal validation, graph plan, publication bundle, snapshot, head, failure, recovery, and view vectors.
- Verify no required value remains null.
- Do not code against draft type names without updating the owning contract and fixtures.

## Coherence discipline

One publication must bind:

```text
ProfileIdentity / ReferenceGeneration
ProjectGeneration / ProjectIndexCandidate
AnalyzerSnapshot and unit/file manifests
Recognizer packs/rules/result partitions
Graph registry/base/replacement plan/target snapshot
ProjectStore profile/bundles/target generation/artifact
ProjectSnapshot and GraphSnapshot
PublicationHead
```

Any mismatch is fatal. No “closest,” current, latest, fallback, or mixed generation.

## Store discipline

- Submit registered logical plans only.
- Never issue SQL, open connections, mutate store paths, or manage WAL.
- Treat store generation success as physical evidence, not domain completion.
- Run independent project and graph post-open validation.
- Authorize head CAS only after all mandatory gates pass.

## Graph discipline

- Preserve graph proposal rejection/conflict reports.
- Do not weaken registry constraints or rewrite recognizer proposals.
- Graph derives final semantic keys/assertion IDs and snapshot.
- Project may not claim a graph relation from source/load order alone outside graph contracts.

## Failure discipline

- Prior head remains until successful CAS.
- Last-known-good retains original identity.
- Sealed inactive generation is not current.
- CAS conflict does not trigger silent rebase/rebuild.
- Cancellation after CAS success cannot label the publication cancelled.
- Recovery adoption requires exact fresh validation and CAS.

## Completion report

```text
request/candidate/base/head identities
profile/reference/analyzer/recognizer/graph/store pins
project/graph logical plan and publication bundle IDs
store staging/seal/open reports
post-open ProjectView/GraphView validation
snapshot/head/CAS result
failure/inactive/LKG state
fault/cancel/read-lease tests
determinism and unresolved/deferred scope
```
