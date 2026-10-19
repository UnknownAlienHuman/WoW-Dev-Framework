from pathlib import Path

Path("crates/wow-project/REPLAY_PUBLICATION.md").write_text("""# Current Project publication

Project publication persists a rebuildable local Project/graph snapshot for the
current tool. It is not a compatibility archive or a database migration product.

## Owners and flow

1. `ProjectPublisher` owns validated Main, Library, TOC/XML and selected Blizzard
   source inputs.
2. `ProjectReplay::capture` retains only the inputs needed by the current native
   reconstruction path.
3. `ProjectPublicationBundle` combines the replay, graph records and bindings.
4. `wow-store::project::ProjectStore` commits the generated records in one SQLite
   publication transaction.
5. `LiveProjectStore` selects the current catalog for the requested ordinary or
   named component route and exposes Current/Exact reads.

`ReadSnapshot` validates the selected manifest and membership when it opens, then
validates each requested record on read. `AcquiredProjectPair::read` still rebuilds
Project/Emmy state from the retained replay before returning a `ProjectView`; #131
replaces that reconstruction with reader-ready indexed facts.

## Required behavior

- Main and Library remain separate inputs to one Emmy analysis.
- TOC/XML selection, source flavor and source locations survive publication.
- Project and graph records refer to one selected generation.
- Writes are transactional; a failed publication does not replace Current.
- Reads remain bounded and cancellable.
- Source and user configuration are never deleted during cache rebuild.

## Format policy

Historical exact store catalogs `STORAGE_SCHEMAS_V1` through
`STORAGE_SCHEMAS_V16` are not registered or reopened. An old or incompatible
derived database must report a rebuild requirement and be recreated from current
source/configuration.

Current named component catalogs and the ordinary current catalog remain while
their real callers exist. The replay record still contains several current
recipe shapes selected by configuration; #112 removes those variants as their
callers are consolidated. Do not introduce `v17`, a converter, an archive reader,
a disabled compatibility mode or another catalog registry.

## Where to change code

- Capture/hydration: `src/replay/mod.rs` and its small owner modules.
- Store records/catalogs: `src/replay/publication.rs`.
- Service publication/read: `wow-service/src/live_project.rs` and operations.
- Direct reader replacement: #131.
- Legacy format deletion: #112.

The architecture and implementation documents define product scope. Git history
is the record of removed formats; runtime code is not.
""")

Path("docs/PLATFORM_GRAPH_CAPABILITIES.md").write_text("""# Selected native platform graph capabilities

Platform graph capabilities select useful facts from the currently configured
Blizzard UI source. They do not create a separate platform product.

| Capability | Depends on | Useful result |
| --- | --- | --- |
| `inventory_spans_v1` | none | Source files, spans and containment |
| `structural_roles_v1` | inventory spans | Project, TOC/load and XML structural roles |
| `xml_source_maps_v1` | spans, roles | XML-backed virtual Lua units and source maps |
| `xml_schema_classification_v1` | spans, roles | Reviewed XML class outcomes |

`PlatformGraphProfile::DirectCapabilitiesV1` carries one finite selected set.
Construction rejects duplicates, unknown versions and missing dependencies. The
selection is part of Project configuration and generation identity; Service may
request it but may not silently widen or replace it.

## Owners

- `wow-project` admits source/package/configuration inputs and builds the native
  producer plan.
- `wow-emmy` owns Lua syntax and semantic facts.
- `wow-graph` owns the resulting partitions and relationships.
- `wow-service` composes the selected owners and returns the result.

Producer order, cancellation and resource bounds use the existing native route.
Do not copy producer algorithms into Service or add another capability registry.

## Persistence

The selected configuration is retained through the current Project replay and
current named catalog. Historical exact catalog sets are not reopened. If an old
cache cannot be read by the current catalog, rebuild it from configured source.
No catalog migration, frozen-epoch fallback or replacement version ladder is
part of the product.

## Compatibility inventory

`DirectPlatformProducersWithRawInventoryV1` remains a current explicit route while
it has a real source/search consumer. The former cumulative development variants
for spans, roles and maps are represented by the capabilities above; do not
restore separate `/2`, `/3`, `/4` profiles or their tests.

Physical function declarations and XML classification are separate producer
selections, not graph capabilities. Their current consumers and source-map rules
are documented in `docs/IMPLEMENTATION.md` and
`crates/wow-project/e3/FUNCTION_FINGERPRINTS.md`.

## Completion rule

Keep only facts used by diagnostics, search, navigation or readable context. A
new capability needs a named user result and an existing owner that cannot
provide it. Historical receipts, proof records and compatibility claims are not
requirements.
""")

implementation = Path("docs/IMPLEMENTATION.md")
text = implementation.read_text()
old = """Use one current database layout. Rebuild incompatible derived data from source
and configuration; switch only after success and preserve user files. If inputs
are absent, report the rebuild requirement. #111 removes historical epochs,
backup/quarantine/migration/retention administration once normal reads and
rebuilds replace it. No compatibility reader or new lifecycle subsystem.
"""
new = """Use one current database layout. Historical exact Project catalog sets V1–V16
are no longer registered; an incompatible derived database must be rebuilt from
source and configuration. Current replay recipe variants remain temporary until
#112 consolidates their real callers. Switch only after successful rebuild and
preserve user files. If inputs are absent, report the requirement. #111 removes
remaining retention/GC administration after normal reads replace it. No
compatibility reader, converter or new lifecycle subsystem.
"""
if text.count(old) != 1:
    raise SystemExit("implementation storage paragraph not found")
implementation.write_text(text.replace(old, new))

queue = Path("docs/REFACTOR_QUEUE.md")
text = queue.read_text()
old = "| With touched owner | [#112 — legacy formats/docs](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/112) | One current format and regeneration; remove old readers and duplicated requirements. |"
new = "| With touched owner | [#112 — legacy formats/docs](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/112) | Historical exact Project catalogs are removed; collapse remaining replay recipe variants to one current rebuildable representation. |"
if text.count(old) != 1:
    raise SystemExit("queue #112 row not found")
queue.write_text(text.replace(old, new))

for path in Path(".").rglob("*"):
    if path.is_file() and path.suffix in {".rs", ".md"}:
        if "STORAGE_SCHEMAS_V" in path.read_text(errors="ignore"):
            raise SystemExit(f"historical catalog constant remains in {path}")
