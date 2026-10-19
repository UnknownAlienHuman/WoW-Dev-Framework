# Selected native platform graph capabilities

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
