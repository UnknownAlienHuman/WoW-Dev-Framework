# Manifested retained graph storage

```text
wow graph publish --bundle build.json --store-root .wow-retained --operation-id publish-001 --expected-current absent --allow-partial --initialize --format json
wow graph reconcile --store-root .wow-retained --operation-id publish-001 --format json
wow graph neighbors --store-root .wow-retained --store-generation current --request neighbors.json --format json
wow graph explain --store-root .wow-retained --store-generation <project-store-generation:sha256:...> --request explain.json --source-root <Main-root> --format json
```

This is the first executable `project-store-wal-manifested-partitions-v1` slice,
with scope **`retained-graph-bundle-v7`**. It persists the already validated v7
[graph-build bundle](GRAPH_BUILD.md). It is not the complete E2-D
`ProjectPublicationSet`: retained project/analyzer IDs and sidecars do not
rehydrate a live ProjectView or prove all semantic owner checks. `--allow-partial`
is mandatory. A successful activation returns exit **2**, preserving Partial.
This is a local storage write, not a source edit or source analysis.

## Publication and replay

`--store-root` is a private storage directory, **not the addon source directory**.
`--initialize` explicitly permits creation of a new directory. A repeat can reopen
an existing matching registered store, but cannot adopt an arbitrary SQLite file,
partially created root, different owner, schema or SQLite runtime. Its parent
must exist. Without this flag, a missing store is an error. Root paths are local
configuration; they never enter semantic IDs or receipts.

The first activation requires `--expected-current absent`. Later publications
require the exact **current record ID**, not its generation or graph snapshot ID,
from `current.record_id` in a prior publication/reconcile result. A stale base is
rejected, never replaced by the latest value. The request digest binds complete
membership, owner bindings, epoch and expected current. Reusing an operation ID
with another request is a conflict, not an update.

The service admits the full bundle and plans records before creating a root.
The store then seals/reuses immutable partitions, commits full membership as
`published_inactive`, and leaves `current` untouched. A fresh read transaction
verifies every stored record and restores the complete bundle. Existing graph,
project source/evidence and service integrity validators check the read-back;
only their compiled adapters can supply the unserializable validation capability.
`validated_inactive` is recorded separately. Activation atomically commits the
current-record CAS, publication history and operation's activation receipt.

Phases are `prepared`, `published_inactive`, `validated_inactive`, `activated`.
`reconcile` observes the durable phase, original request digest, target generation
and activation receipt without resuming, repairing or rolling back. It returns
exit 0 for an observed operation and 3 for an operation not retained. Neither
result asserts complete graph coverage. An absent journal entry is not proof
that a failed root initialization created no filesystem objects.

Retry publication with the **same** bundle, operation ID and expected-current
argument. Inert sealed records can be reused. An activated request returns its
historical receipt; it does not reinstall that generation if a later activation
has advanced current. Ctrl-C before commit may leave sealed inactive records.
Cancellation during/after the noninterruptible commit cannot erase an observed
activation. After output failure or `store_outcome_unknown`, reconcile the same
operation ID before deciding whether to repeat. There is no automatic retry,
last-known-good selection or destructive repair.

## Physical records and pinned readers

The private directory contains `writer.lock`, `project-store-registry.json`, and
`epochs/<epoch-hash>/project.sqlite` with an exact `epoch-manifest.json`. Normal
updates use the same SQLite database, not a new database per generation. Each
generation stores its complete membership; reading never follows recursive deltas.

Graph registry, foundation, materialized graph and each producer partition are
separate immutable records. Each top-level project provenance field is also a
separate record; the service receipt retains the remaining envelope without
copying graph or provenance into it. Equivalent canonical records are hash-checked
and reused. This does not claim optimal source-file granularity: large materialized
graph and fact fields can still change as a whole. No new graph-build schema or
graph identity recipe is introduced.

All six graph reads accept exactly one of `--snapshot`, `--bundle`, or the pair
`--store-root` + `--store-generation`. The latter requires explicit `current` or
an exact stored generation ID. The JSON query still supplies its exact **graph
snapshot** and entity IDs. Store selection never rewrites a stale query. Path
continuation still binds the original exact graph/query; use the retained exact
store generation rather than moving `current` between pages.

Acquisition starts a real SQLite read transaction, resolves current once and
pins complete membership. The same in-process owner may publish another generation
without changing an acquired snapshot. Each read carries an owner-local generation
lease, closed with the transaction. The OS-held writer lock survives while any
read snapshot exists. Separate CLI processes are serialized: a competing owner
receives `operation_busy`, rather than bypassing the lease or waiting indefinitely.
Cross-process reader service/daemon scheduling is not implemented by this slice.

`wow-service/graph-stored-read-result/1` includes `store_context`: exact epoch,
stored generation, identity bindings and current at acquisition. Existing payloads,
evidence limits and optional source verification use the same owner operations.
Without `--source-root` no addon source is opened. An exact inactive generation
can be inspected but is not silently activated. Metadata corruption fails closed.
Resources close before output; a broken pipe never repeats a write or read.

## Bounded, registered storage

An existing database is inspected read-only before writable owner acquisition.
The fixed compiled SQL schema, application/user IDs, page size, encoding, vacuum
mode, epoch manifest, SQLite source/compile-option digest and registered owner
schema/check catalog must match. No project-defined SQL, migration callback, raw
connection, URI database, arbitrary attach or extension loading is exposed.
Triggers are disabled and trusted-schema mode is off. The owner enables WAL,
FULL synchronous mode, foreign keys, a 500 ms busy ceiling, 256-page automatic
checkpointing, 8 MiB journal retention and a 1 GiB database page ceiling.

Hard bounds: 256 partitions per generation, 32 MiB per record, 64 MiB per complete
generation, 1,024 retained generations, 8,192 immutable versions, 4,096 operation
records and 16 active in-process readers. The existing bundle limit remains
32 MiB. Before each write, a conservative WAL growth reservation must fit under
128 MiB; a pinned reader can cause explicit budget refusal. Compiled callers can
request a single PASSIVE checkpoint with busy/frame/reader counts, not an unbounded
retry loop. Nothing silently deletes old generations to make space.

Use an exclusively controlled local directory. Unix roots are created with mode
0700; Windows uses inherited access controls. Root/fixed descendant symlinks and
Windows reparse points are rejected at admission, but this is not protection
against a hostile process concurrently replacing the filesystem or opening the
database outside the owner. Read commands acquire the owner lock and may perform
SQLite operational WAL/checkpoint housekeeping; they do not publish semantic data.

## Remaining work

Full E2 domain schema composition/publication acceptance, live ProjectView
rehydration, incremental invalidation, multi-process daemon readers, retention/GC,
backup/restore, repair/quarantine and incompatible epoch replacement remain open.
No in-place migration or unknown-schema fallback is performed. All retained
versions remain until an explicit future retention implementation is available.
Process/crash/power-loss, Windows filesystem and real-addon acceptance have not
been exercised in this checkpoint. Clippy compilation is not that acceptance.
Source-byte verification and provenance authentication remain separate from
storage integrity; stored evidence stays Partial/Possible as originally recorded.

Owners: `wow-store::project`, `GraphPartitionSnapshot::storage_records/read_stored`,
`wow-project::graph::persistence`, `wow-service::graph` and the thin CLI.
