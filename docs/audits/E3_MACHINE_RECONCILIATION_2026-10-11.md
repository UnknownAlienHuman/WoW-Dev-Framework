# E3-A machine reconciliation

The source-profile Rust module was added at
`fd7d5d1f2d627031faaa94a6942e9ea1b5129708` on 2026-10-09. At that exact commit,
E3-A `CONTRACT.json` still declared no Rust code, while `examples/CHECKSUMS.json`
declared not-started and contained null prerequisite pins. At inspected base
`a174c55fba81eed6095457a585da15e09c96a2cb`, substantial native source, package,
analyzer, graph and replay code coexisted with those declarations.

The documented before-first-implementation freeze was missed/NotEvaluated.
Current values cannot retroactively pass it. Current implementation is partial
functional; source/corpus, candidate, fingerprints, SkeletonInputView and package
acceptance remain open in Issues #106 and #113–#118. The selected 273-package
request is not proof of complete configured roots or dependency closure.

The reconciled machine schema 2 is consumed by `xtask::repository::e3`.
`CONTRACT.json` owns operation routing; `examples/CHECKSUMS.json` owns explicitly
pending acceptance inputs. Empty hashes and null future pins cannot become an
accepted bundle. The checker validates bounded, strictly decoded declarations,
route files and status consistency. It does not prove Rust symbol existence,
runtime reachability, source freshness or semantic acceptance.

The canonical E3-A ID is
`wow-project/e3-a/blizzard-ui-source-universe-index`. Downstream availability
records partial functional code separately from the accepted/frozen prerequisite.
Their acceptance requirement remains in force.

Native version consumers remain the product owners, independent of this machine
schema. Replay `/1`–`/15` is selected and validated in `src/replay/mod.rs`, mapped
to storage `wow-project.live-replay.v1`–`v15`, and hydrated through native source,
package and analyzer owners. `src/replay/publication.rs::STORAGE_SCHEMA_CATALOGS`
retains the actual admitted catalogs; there is no invented frozen V11 catalog.
The pair header remains `/1`. Service live publication, store reopen, recovery
and migration consume these native catalogs; machine reconciliation changes none
of their bytes or identity recipes. Released/external use is NotEvaluated.

Source projections `/19`–`/22` remain legacy explicit selections. Independent
capabilities use `source-load-capabilities/1` and `platform-capabilities/1`, with
registry identity derived from the selected capability set. Direct stages feed
the existing graph owner; the service and CLI preserve their selected identities.
These executable consumers do not establish full W17 acceptance or permit another
profile ladder. Version compaction and broader machine reconciliation remain
Issues #112 and #120.
