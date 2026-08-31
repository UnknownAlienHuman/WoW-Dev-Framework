# Producer partitions and graph publication

**Status:** normative E2-A write/publication contract.

## Producer partition

A partition is the smallest replaceable assertion ownership unit:

```text
producer ID/version
universe/scope
source/reference/project generation
capability partition
partition kind/key
```

Examples:

```text
project.toc:Mainline
project.xml:Core.xml
project.file:Core.lua:semantic-facts
recognizer:event-registration:project-generation-X
reference.api-system:C_UnitAuras
```

## Replacement protocol

```text
validate registry and batch
-> resolve endpoint dependencies against exact base snapshot/batch
-> derive assertion/partition manifests
-> compute affected semantic keys/conflicts/views
-> build GraphStoreReplacementPlan
-> execute through wow-store one-writer transaction/publication
-> run graph/store validation catalog
-> publish exact target GraphGenerationId
-> reopen immutable GraphView and run golden checks
```

## Atomicity

- Readers see old or new snapshot, never a partial partition.
- Failure/cancellation does not advance current graph generation.
- Prior graph snapshot remains under its original identity.
- All stale assertions from the replaced partition are removed.
- Assertions from other partitions/producers remain unchanged.
- Derived view/conflict/coverage indexes are updated in the same publication boundary.

## Base and target identity

A replacement request names exact base graph generation and target input generations. Stale base is rejected; no optimistic merge or silent rebase.

Target `GraphGenerationId` derives from:

- graph ID and registry bundle;
- exact reference/project/external generation inputs;
- ordered partition manifest IDs/digests;
- conflict/coverage manifests;
- logical graph schema/operation bundle versions;
- graph contract version.

It excludes transaction order, SQLite row IDs, clock, host, worker count, and WAL state.

## Multiple partitions

A publication may replace an ordered set of independent partitions in one target generation. The plan lists all base/target partition changes. Partial success is forbidden.

## Validation gates

- no dangling semantic endpoints;
- no invalid relation endpoints/axis membership;
- no cross-scope generation mismatch;
- required source/evidence/coverage records resolve;
- forbidden cycles/multiplicity violations classified;
- conflict and capability impacts complete;
- expected counts/digests/index closure match;
- exact read queries after publication match planned views.

## Removal and producer disable

Disabling a recognizer/producer is a partition replacement to empty assertions plus explicit coverage downgrade. Core semantic meanings and other producer assertions remain unchanged.

## Retention

Graph/store generations are retained by explicit current/last-known-good/reader/evidence/debug policy. No age-only deletion. GC validates assertion/view/conflict/object/reference closure.

## Recovery

If store generation publishes but graph post-open validation fails, retain it inactive/quarantined and require exact recovery/revalidation. Never mutate/relabel it as complete.
