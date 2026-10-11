# Selected native platform graph capabilities

W17 uses the capability-composition model in [#115](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/115).
`PlatformGraphProfile::DirectCapabilitiesV1(PlatformGraphCapabilities)` selects an
exact finite set over genuine retained platform packages and `SameSessionV1` XML
binding. The publication `ProjectCapabilityPolicy` is a separate policy and does
not select these producers.

| Capability identity | Required selected dependencies | Native responsibility |
| --- | --- | --- |
| `inventory_spans_v1` | none | Known captured source spans and file containment |
| `structural_roles_v1` | `inventory_spans_v1` | Project/raw membership, TOC/load roles, lexical XML/site/reference roles |
| `xml_source_maps_v1` | `inventory_spans_v1`, `structural_roles_v1` | Retained virtual Lua units, XML mapping pieces and exact span associations |
| `xml_schema_classification_v1` | `inventory_spans_v1`, `structural_roles_v1` | Reviewed exact XSD class outcomes and occurrence/class evidence |

Construction and bounded deserialization reject duplicates, unknown versions and
missing dependencies. They do not expand a selection. Serialization emits the
canonical dependency order. The empty set explicitly selects the base direct
owners in this dialect; it is distinct from the admitted legacy `/1` selector.

The four ordered owners remain Inventory, TocLoad, AnalyzerStructure and
XmlStructure, preceded by genuine raw inventory. Capabilities enrich their
responsibilities rather than replacing predecessor admission. Mapping support
preparation precedes span collection; every stage uses the existing shared
budgets, cancellation and native finish validation.

## Exact consumers and identity

The selected service route can observe exact physical-function source-span
components without adding a graph capability or changing native project/replay
identity. Its explicit query and typed result are defined by
[the fingerprint recipe](../crates/wow-project/e3/FUNCTION_FINGERPRINTS.md).
Comparison bindings preserve the direct predecessor scope; they do not relabel
the service's final composed graph or supply a validated candidate.

`build_platform_graph_proposal_plan(project, stop)` reads the held configuration's
exact set. `build_platform_graph_proposal_plan_for_capabilities` supports explicit
native inspection; when the project selects this dialect, a different requested
set refuses. Plan and finished provenance expose `capabilities()`.

The complete canonical list participates in the existing configuration digest
and project generation. Registry `wow-project.source-load` uses the exact
`capabilities_v1:<selected IDs>` version; its digest determines the input graph
generation and proposal scope. Composed producer versions name each native owner
and only the capabilities affecting that owner's facts. For example, adding XML
maps changes Inventory and XmlStructure versions without changing TocLoad or
AnalyzerStructure versions. All partitions still bind the full registry and
generation, so equal individual producer versions cannot authorize mixed sets.

Service `GraphBuildRequest::with_platform_graph_profile` emits request `/13`,
projection `wow-project/source-load-capabilities/1`, and the canonical list. It
compares the set against the acquired configuration before composition. Result
`/20` retains that request and the actual native graph. The source-prefix route
and durable direct validation both use the same configuration-selected plan.
The augmented XML-map source report is `wow-project/platform-capabilities/1`;
the exact set is retained by the configuration, registry and stage metadata.

Replay `wow-project/native-project-replay/10` retains the selected list and
configuration digest. Storage `wow-project.live-replay.v10` is admitted only by
the new exact catalog. Hydration and pair validation reconstruct the same native
set, registry, foundation and direct partition closure before returning an owner.
Changing or omitting the list, downgrading the replay marker, or substituting
partitions cannot silently narrow the graph.

## Compatibility inventory

Physical declaration syntax is an explicit configuration producer selection,
`wow-emmy/physical-function-declaration-syntax/1`, rather than another graph
capability or aggregate profile. It shares the existing Main session, binds the
configuration/report identities and composes with classification through exact
named replay/storage catalogs. The selected service request/result recipes are
`physical-function-declarations/1` and, for the optional query,
`physical-function-declarations-and-fingerprints/1`. See
[component semantics](../crates/wow-project/e3/FUNCTION_FINGERPRINTS.md) and
[actual replay consumers](../crates/wow-project/REPLAY_PUBLICATION.md).

Classification additionally retains one sealed reviewed profile in configuration
before generation derivation. Its selection binds the same admitted source and
catalog; native replay re-admits both owners and validates their exact digests.
The selected recipes are `wow-project/native-project-replay/xml-schema-classification/1`
and `wow-project.live-replay.xml-schema-classification.v1`. The explicit
`STORAGE_SCHEMAS_XML_CLASSIFICATION_V1` catalog has a real service create/open/
publication consumer; prior catalogs and unselected bytes remain unchanged.
Selecting classification does not add another cumulative direct profile.
Service request/result use `wow-service/graph-build-{request,result}/xml-schema-classification/1`.
The result includes the typed classification report and actual source-predecessor
membership; the future #116 candidate must consume this handoff. Policy and
conservative unknown cases are defined in
[the owner contract](../crates/wow-project/e3/XML_SCHEMA_CLASSIFICATION.md).

| Former direct recipe | Classification and consumer | Current treatment |
| --- | --- | --- |
| `/1` | Reachable service request12/result19, source `/22`, persisted native replay `/9` and storage v9; existing fixture and exact epoch consumer | Preserved with its original config selector, registry16 and stage version1 |
| `/2` spans | Transient unreleased native development variant and handwritten lifecycle; no config/service/replay/catalog/checksum consumer | Consolidated into `inventory_spans_v1` |
| `/3` roles | Transient unreleased native development variant and handwritten lifecycle; no config/service/replay/catalog/checksum consumer | Consolidated into `structural_roles_v1` with explicit dependency |
| `/4` maps | Transient unreleased native development variant and handwritten lifecycle; no config/service/replay/catalog/checksum consumer | Consolidated into `xml_source_maps_v1` with explicit dependencies |

Repository inspection establishes these consumers; an external release of the
transient variants is not asserted. Their public constants, separate constructors
and registry17/18/19 identities are retired. The existing native lifecycle now
tests explicit sets. Historical checkpoint evidence remains historical; it is not
a compatibility obligation or a fixture auto-bless.

Frozen `STORAGE_SCHEMAS_V9` preserves the old exact catalog. Open, recovery and
migration use one explicit list of registered exact catalogs; no retained epoch
is widened. Old `/9` owners reopen exactly. A v9 store refuses v10 publication
and preserves Current, epoch, membership and operation state. Existing physical
migration retains its catalog; a catalog-upgrade operation is not supplied by
this slice. A selected v10 graph therefore needs a v10 store until such an
explicit migration exists.

## Remaining scope

Explicit function and XML comparison queries consume the acquired capabilities
without extending their set or changing native replay/catalog identity. The XML
query requires selected classification and retains accepted support and unavailable
outcomes. See the [query contract](../apps/wow/GRAPH_BUILD.md) and
[component recipes](../crates/wow-project/e3/FUNCTION_FINGERPRINTS.md).

The same exact-definition axis recipe is reconstructed for the selected registry;
span-map associations remain excluded from Ownership. Full corpus capacity,
classification acceptance, fingerprints, candidate freeze, SkeletonInputView and public CLI
source handoff remain tracked in #113 and #116–#120. This composition is not full
W17, package, runtime or launch acceptance. New capabilities extend the typed
identities and dependency rules; they do not add another cumulative profile.
