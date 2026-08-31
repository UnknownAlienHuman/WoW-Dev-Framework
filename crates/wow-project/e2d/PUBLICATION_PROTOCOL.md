# Integrated publication protocol

**Status:** normative.

## Phase 0 — Request preflight

- validate exact request/candidate/profile/reference/store/graph profiles;
- resolve one base head or explicit first publish;
- validate expected head and base store/graph snapshots;
- validate budgets/cancellation;
- reject stale/mixed/floating inputs before graph/store work.

## Phase 1 — Candidate validation

Revalidate E2-C candidate:

- immutable `NotPublishedE2C`;
- complete source/TOC/XML/load/Lua/analyzer/recognizer manifests;
- exact generation and source handles;
- no removed/stale target records;
- capability/conflict/truncation policy;
- real/synthetic fixture provenance where applicable;
- candidate digest.

No source is reopened.

## Phase 2 — Project logical plan

Build exact registered project records for:

```text
source inventory and universes
packages/TOC/XML/load model
Lua units/analyzer binding
recognizer packs/results/output partitions
proposal validation reports
candidate/capability/conflict/coverage manifests
project snapshot support
```

## Phase 3 — Graph plan

Submit validated project/recognizer proposals to `wow-graph` with exact registry/base/target context.

Graph returns:

- accepted proposal mappings;
- final entity/relation assertion records;
- stale producer partitions to remove;
- conflicts and coverage;
- target graph manifest candidate;
- registered operation plan;
- golden query catalog.

Project cannot edit this plan.

## Phase 4 — Publication bundle

Merge project and graph registered invocations in the frozen cross-domain phase order. Validate no operation ID collision, missing prerequisite, conflicting target partition, SQL/path/callback payload, or manifest mismatch.

Derive target ProjectStore generation logical identity and complete expected manifests.

## Phase 5 — Store generation

Invoke `wow-store` E2-D:

```text
stage
-> one transaction
-> validate
-> commit
-> checkpoint/close
-> seal/materialize
-> reopen read-only
```

No head update yet.

## Phase 6 — Post-seal domain validation

Using the exact store read handle/validation lease:

### Project validation

- all candidate records and manifests present;
- no stale/removed records;
- exact analyzer/recognizer bindings;
- load/source/capability summaries match;
- project registered reads/golden cases pass.

### Graph validation

- registry and producer partitions exact;
- assertion/conflict/coverage manifests match plan;
- no dangling endpoints/stale producer facts;
- exact entity/neighbor/axis/path/explain golden results;
- query coverage/truncation policies match.

Any failure leaves the generation inactive/quarantined and prior head unchanged.

## Phase 7 — Snapshot/coherence manifests

Build shared coherence manifest, ProjectSnapshot, GraphSnapshot, and publication report from exact sealed read results. Validate noncyclic IDs and all references.

The snapshot support records must already be present in the sealed store through a precommitted provisional/finalizable scheme defined by the registered bundle. E2-D must choose one exact nonmutable strategy:

```text
A. all final snapshot IDs determinable before store transaction from logical plan, then validated after reopen;
or
B. snapshot manifests stored as external immutable members atomically included in artifact manifest before final open.
```

E2-D selected implementation must freeze one strategy before Rust code. No post-seal database mutation.

## Phase 8 — Head CAS

Build exact new head. Authorize store CAS with expected prior head.

Result:

```text
success
    publication complete

conflict
    target remains inactive; no rebase

ambiguous transport/process result
    read registry and exact target before retry/classification
```

## Phase 9 — Published view smoke

Acquire a normal reader lease through the new head and run minimal project+graph coherent open. This verifies the consumer route, not runtime WoW behavior.

## No implicit retry

A failed or conflicted request returns exact state. Caller creates a fresh request against the new head if desired. It cannot mutate the original bundle's base identity.
