# ProjectStore benchmark and physical-profile freeze

**Status:** normative gate; no benchmark result is claimed in the documentation phase.

## Candidate models

```text
A. whole SQLite database per project generation
B. duplicated generation-keyed rows in one WAL DB
C. manifested immutable partition versions with complete membership maps
D. recursive base + delta chain
```

E2-D selects C. A, B, or D may replace it only through a reviewed contract revision with measured evidence.

## Corpus

Minimum:

```text
synthetic small project
synthetic medium project
synthetic high-fanout graph project
pinned UnknownAlienHuman/roth-ui structural fixture
mutation-expanded TOC/XML/Lua/recognizer/graph corpus
```

The real fixture remains read-only and repository-name independent.

## Workloads

```text
cold create/full publish
warm open/current snapshot
exact generation open
one Lua file update
TOC order/directive/dependency update
XML include/inline-script update
recognizer rule version update
graph registry update
no-change update
old reader during activation
high-fanout bounded graph query
checkpoint under readers
retention/GC
backup/restore/recovery
schema/profile epoch rebuild
```

## Metrics

```text
logical correctness
publish CPU/wall p50/p95/p99
writer lock wait
pages/bytes written
new/reused partition versions
membership rows/bytes
DB and WAL peak/steady bytes
current/exact read latency
graph query latency/rows/pages
reader memory/lifetime pressure
checkpoint duration/busy/remaining frames
startup/recovery time
retention/GC reclaimed bytes
backup/restore time/bytes
```

Hardware/platform/SQLite binding/profile are recorded; timing is not canonical identity.

## Qualitative gates

Selected profile must prove one-file work proportional to affected partitions plus membership, no recursive ancestry lookup, old-reader stability, stale-base rejection, inactive validation before activation, no cross-generation leakage, bounded WAL, deterministic logical output, safe recovery/GC, and acceptable full build.

## Quantitative freeze

Before first E2-D Rust merge, freeze corpus IDs/digests, hardware/platform class, SQLite/binding profile, methodology, latency/size/write-amplification ceilings, WAL/checkpoint ceilings, reader assumptions, GC/backup/recovery ceilings, and accepted baseline report.

Missing benchmark is blocked/skipped, never pass.

## Regression policy

Repeat on binding/SQLite/profile/schema/model changes, compare normalized corpora, retain last-known-good, report confidence/noise, use tail/peak metrics where meaningful, and never weaken correctness to meet performance targets.
