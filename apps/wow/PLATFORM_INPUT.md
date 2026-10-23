# Manifest-bound platform graph input

`wow graph build --config project.json --project <ProjectId> --format json`
accepts `wow-service/local-platform-source/1` and `/2` through the existing local input
service. This explicit read-only source/package/Main/graph route does not acquire
source, discover packages or resolve a moving selector.

## Configuration

The JSON object rejects unknown and duplicate fields. Required fields are:

| Field | Meaning |
|---|---|
| `schema` | `wow-service/local-platform-source/1`, or `/2` with explicit retained project budgets below |
| `project_id`, `workspace_id`, `source_origin_id`, `logical_root` | Existing validated project identities and logical root |
| `source_profile` | Existing `BlizzardUiSourceProfileRequest`: exact target Reference profile/generation, admitted roots, selected TOCs, exclusions and finite source limits |
| `source` | Existing `ManifestedPlatformInput`: confined source root, pinned xtask manifest, exact revision/version, origin/materializer/license assertions and compatibility evidence |
| `analyzer` | Existing disk declaration: compatibility report artifact, accepted pin, configuration digest, contract, fixture contract and Library contract |
| `packages` | Native `ProjectPackageInput` declarations with physical root, selected root and selected/unselected variants |
| `graph_profile` | Existing `PlatformGraphProfile`, including the canonical capability set below |

Select exactly one Reference input; neither, both and explicit `null` refuse:

- `reference_view`: `ProjectDiskFile` naming a native `ReferenceView` JSON artifact,
  read through the ordinary 8-MiB artifact port.
- `native_reference`: the existing native Generated API profile and manifested
  source request, composed in memory through the native callable owner. Its
  required `profile` object has the same shape as the source-reference profile;
  its `native_source` object requires `root`, `manifest` and `toc`.

See [source-reference](SOURCE_REFERENCE.md) for the profile and source shapes.
Native composition requires the same source root and exact manifest descriptor,
revision and client version/build as `source`. Pinned-file lists refuse. The
derived whole Reference profile and final generation must equal `source_profile`'s
target. Derive these identities with the source-reference operation first; do not
invent IDs or substitute a partial selection with another generation.

This choice avoids serializing and rereading an intermediate Reference artifact;
it does not widen any acquisition or semantic bound. Native Generated API
acquisition currently admits local Git manifests, so checkout-free API snapshots
remain unsupported by this choice. The callable owner preserves Partial coverage,
projection losses and conflicts; successful composition grants no runtime or
full-platform negative authority.

`load_context` is optional and uses the existing validated TOC context. Optional
`library` selects `{"root":"library-directory"}` with bounded Lua discovery;
an optional nonempty `files` list selects only those relative paths. See
[Library input](FILES_INPUT.md#main-and-library). Absence means an
actually empty Library snapshot. It inserts no dummy Lua or generated annotation
and does not establish annotation or external Library completeness.

For the inventory, structural-role and XML-map capabilities, `graph_profile` is:

```json
{
  "direct_capabilities_v1": [
    "inventory_spans_v1",
    "structural_roles_v1",
    "xml_source_maps_v1"
  ]
}
```

The [capability contract](../../docs/PLATFORM_GRAPH_CAPABILITIES.md) defines
independent versions, canonical ordering and dependencies. Invalid sets refuse
without expansion. Fixture profile, Interface, revision and analyzer identities
are test inputs, never permanent product defaults.

To add reviewed static XML classification, include
`xml_schema_classification_v1` in that exact capability set and provide
`xml_schema_classification: {"schema_selection": ..., "profile_selection": ...}`.
Both fields are strict native-owner selections. The schema selection names exact
manifested XSD members; the classification selection binds their admitted catalog
and source identities plus reviewed component anchors. Derive those anchors
through the schema owner described in
[the classification contract](../../crates/wow-project/e3/XML_SCHEMA_CLASSIFICATION.md).
Names, decoded receipts and an arbitrary type table cannot supply authority.
Capability without the reviewed owner, owner without the capability, explicit
null and mismatched source/catalog/profile identities refuse.

The same `wow graph build` route emits the selected classification result schema.
Native namespaced persistence also records XML classification in the explicit
`PlatformStoreSelection`. The physical store uses the one current Project catalog;
publication/readback validate the genuine source-bound configuration and replay.

To collect physical function declarations, explicitly provide
`"physical_function_declarations":"wow-emmy/physical-function-declaration-syntax/1"`
with a `direct_capabilities_v1` graph profile. Unknown recipes and explicit null
refuse. The existing Main session emits form, ordered parameter/vararg tokens and
explicit colon syntax; it supplies no inferred types/returns or runtime claim.
The selection binds configuration, report and replay identity. Namespaced
persistence records the same declaration/XML choices in `PlatformStoreSelection`;
readback checks them against reconstructed configuration. `--fingerprints` then
includes the selected declared-header component;
see [the component policy](../../crates/wow-project/e3/FUNCTION_FINGERPRINTS.md).

### Declaration/body shape selection

`physical_function_declarations` also selects
`wow-emmy/physical-function-declaration-syntax-and-body-shape/1`, corresponding
to `DeclaredSyntaxAndBodyShapeV1`, on the genuine native
`direct_capabilities_v1` route. The same Main session supplies a separate
optional `SourceFunctionFact.shape` only for this combined producer.
The existing `wow-emmy/physical-function-declaration-syntax/1` remains the exact
header-only selection; the CLI does not parse Lua or promote it implicitly.

The complete body preserves ordered Enter/Leave/Token events and nested closures,
finite owned structural/operator/punctuation tags, anonymous names and only
Number/String/Boolean/Nil literal categories. Built-in special calls normalize
to Call. Trivia, source identities, identifier names and literal payloads are
excluded from shape comparison; dialect/unknown/attributes are unsupported.

The normalization recipe, native manifest schema/profile and typed project
shape outcomes are defined in
[the component contract](../../crates/wow-project/e3/FUNCTION_FINGERPRINTS.md).
Configuration/report and cold/retained/replay bind this exact producer selection.
Namespaced persistence selects
`PlatformStoreSelection::with_physical_function_declarations_recipe_catalog`
with `PhysicalFunctionDeclarationRecipe::DeclaredSyntaxAndBodyShapeV1`, adding
`with_xml_classification_catalog()` for selected XML classification. The existing
no-argument `with_physical_function_declarations_catalog()` selects header-only.
Configuration and explicit selection must match; incompatible derived stores rebuild.

With signatures absent, replay retains the body-shape payload schema, with a
separate XML/body-shape payload tag when classification is selected. Both are
members of the one current Project catalog; configuration owns the exact recipe.

### Indexed-type signature selection

Optional local field `physical_function_signatures` accepts
`"wow-emmy/physical-function-indexed-types/1"`, native
`PhysicalFunctionSignatureRecipe::IndexedTypesV1`. It requires genuine retained
platform packages, `direct_capabilities_v1` and same-session XML binding. The
native configuration builder exposes `with_physical_function_signatures(recipe)`.
Absence omits the field and retains old identities; null/unknown recipes refuse.
The held native signature index supplies typed observations, not a second parser.
Existing `--fingerprints` exposes the selected component; see
[the finite comparison recipe](../../crates/wow-project/e3/FUNCTION_FINGERPRINTS.md).

Selected signatures use `ProjectReplay`.
Exact declaration/XML/signature choices remain in configuration and the explicit
selection; the one current Project catalog does not enumerate their combinations.

## Native authority and bounds

The configuration directory is registered once. The project disk owner validates
manifest digest/version and every consumed member; the existing strict package
owner creates retained physical package plans and namespaced Main. Requested
package TOCs agree exactly with the source profile selection. The native source
target agrees with the admitted project Reference profile and generation.
Decoded receipts cannot construct these owners.

A package member may contain only metadata or XML, including inline Lua. It
retains its actual TOC/XML/load facts without inventing a physical Lua file.
The aggregate namespaced Main still requires at least one physical Lua file
across its reachable packages. Standalone TOC Main selection retains the same
nonempty physical-Lua requirement. Disk acquisition, native admission and retained
package replay use the same member semantics and exact load-plan identities.

Shared local assembly validates the real analyzer backend and Reference, retains
the source/package owner in configuration, and selects existing same-session XML
bindings. Graph execution uses those retained owners without reopening source or
interpreting census counts as graph facts.

The CLI constructs the selected request using
`GraphBuildRequest::with_input_profile`. Direct service callers can use that helper
or explicitly select the exact recipe. Execution still rejects mismatched requests
and changed capability sets; it never repairs their selection. Existing selected
result/snapshot, graph, replay and catalog identities are preserved.

Ordinary admission still enforces 64 packages, retained syntax/source budgets,
physical Main and existing analyzer/graph/map/output limits. The larger streaming
package census grants no Main/graph capability. Missing or incomplete native
coverage remains Partial or a refusal. No full-corpus or runtime authority follows
from this entry point. Partial graphs exit `2`; existing CLI configuration,
operation and output failure behavior is preserved.

An explicit `retained_package_limits` selects the native retention profile:

```json
{
  "profile": "wow-project/platform-package-retention/1",
  "max_packages": 320,
  "max_total_plan_encoding_bytes": 268435456
}
```

Omission keeps ordinary package64; explicit `null` refuses. The retained profile
admits only the original exact source/package requests through the same owners.
Its package ceiling is the source owner's existing 1024 selected-TOC ceiling.
It charges every complete load-plan encoding before retaining that member, with a
256-MiB hard ceiling and the existing cumulative 64-package syntax-work fanout.
Individual plans retain their 64-MiB encoding bound; source4096/64MiB, physical
Main, analyzer, graph/map/output and storage bounds apply independently.

These example limits are an explicit operation budget, not source discovery or a
default. The inspected selected corpus measured273 packages and234,131,914 summed
plan-encoding bytes; 320 leaves47 package slots and256MiB leaves34,303,542 bytes
(14.65%) of encoding headroom. Exact source evidence belongs to
[#113](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/113).
Encoded bytes are an accounting unit, not an allocator or resident-memory ceiling.
Different populations must still pass every independent owner budget.

The selected limits enter the sealed package binding and original replay request.
Retained-profile archives use `ProjectReplay`; other current
platform selections keep their exact selected recipe. Hydration re-admits the exact
source and package owners under the captured limits. Changed limits or schema
substitution refuse. A successful retained Main is not full graph or persistence
acceptance:
the existing raw replay and record32MiB limits can still refuse a larger corpus.

### Explicit project budgets

`wow-service/local-platform-source/2` requires both `retained_package_limits`
and `project_budgets`. The latter is the existing `ProjectBudgetPolicy` object:

| Required field | Accounting unit |
|---|---|
| `max_files` | Physical Main files |
| `max_total_source_bytes` | Retained physical Main UTF-8 source bytes |
| `max_single_file_bytes` | Bytes of one physical Main file |
| `max_update_operations` | Operations in one project update |
| `max_analyzer_facts` | Existing aggregate member/flow/XML/package-binding facts |
| `max_generic_findings` | Existing aggregate syntax/XML diagnostics |
| `max_output_bytes` | Existing serialized syntax/member/flow/XML and package-binding reports |

Every field is a positive `u64`; the single-file limit must not exceed the total.
Missing, unknown, duplicate, invalid and explicit `null` values refuse before
source acquisition. Schema `/1` rejects `project_budgets` and keeps its fixed
policy, including the16-MiB analyzer-output ceiling. Schema `/2` without native
retention refuses. The service passes the selected policy to native configuration
and the existing inventory/publication gates; it does not reinterpret it as facts.

All seven values already enter the configuration digest and are captured/rebuilt
by native replay. Equal values in `/1` and `/2` preserve the same owner bundle;
changed values change configuration identity. Without a sidecar selection,
retained replay remains `/11`,
and hydration rejects changed budgets against the captured configuration digest.
Transport selection does not change an analyzer report recipe.

These budgets do not widen source admission, disk acquisition, Library, sidecar,
graph/map or replay/store bounds. In particular the function-call sidecar has
separate work/count/report limits and is outside this aggregate output accounting.
Select a finite policy from actual owner measurements; this route alone does not
qualify a complete current corpus or its allocator/peak memory.

### Explicit function-call budget

Schema `/2` also accepts an optional `project_budgets.function_calls` object.
All six fields are required positive `u64` values:

| Field | Independent accounting unit |
|---|---|
| `max_ast_visits` | Cumulative first Main/Library and second Main AST visits |
| `max_functions` | Main functions and, separately, temporary captured signatures |
| `max_calls` | Physical Main call facts |
| `max_global_accesses` | Physical Main global-access facts |
| `max_named_targets` | Retained concrete callable lookup targets |
| `max_report_bytes` | Encoded function-call report, including the original ID allowance |

Missing fields, unknown/duplicate keys, zero, invalid types and explicit `null`
refuse. Omission preserves the previous policy, report `/7` and replay `/11`.
Selection requires genuine native retained packages, reaches the same semantic
session, and produces budget-bound `wow-emmy/function-call-facts/8` and
`ProjectReplay`. The configuration digest and report
identity both retain the selected values. Hydration rebuilds them without source
files and refuses removal, substitution or changed values against the original
identities. Other current replay recipes remain unchanged.

The graph callable projector reuses explicitly selected `max_functions` and
`max_calls` for the same physical Main report arrays after verifying the report
budget equals the configuration budget. Without selection, its ordinary limits
remain 8192/8192, independently of the native sidecar's defaults. This count
admission preserves previously successful fact bytes and identity recipes;
numeric selection already binds the configuration and native report/replay.

The six fields alone do not widen alias work/text, scope, argument/literal,
callable-input, flow, aggregate graph nodes/edges, metadata, other graph/map or
replay/store budgets. Choose finite values from
the actual selected operation; neither this selector nor a synthetic boundary
case qualifies the full corpus or its peak memory.

#### Literal secure-hook target selection

For the selected retained source input, optional
`project_budgets.hook_target_recipe: "hook_target_literals_v1"` enables exact
literal target lookup for genuine Library `hooksecurefunc` calls. It reuses the
same native semantic session and the continuing query, target, AST and output
bounds. Select the existing physical function declaration recipe when named
callbacks need their original declaration custody.

The admitted forms are a direct global-name string and an exact Main global
table with a direct member-name string. Dynamic, ambiguous, reassigned and
shadowed values retain conservative results; local target strings and local
table propagation are not part of this recipe. These facts describe source
structure, not runtime hook execution or safety.

Omission preserves the original callable queries and replay identities. The
selected recipe is retained in configuration and the existing selected-component
replay envelope; capture and source-absent acquisition must preserve its exact
selection. This does not grant another catalog or activate unselected owners.

#### Independent global-access work and text

An optional `project_budgets.function_calls.global_access` object selects two
required positive `u64` fields: `max_work` and `max_text_bytes`. Work is cumulative
alias-index traversal and path expansion across all healthy Main files; text
accounts the existing path/alias/access projection charges. This is separate from
function AST visits, access population and encoded sidecar bytes. Other owner
limits remain independent.

Omission preserves the original six-field bytes and the 2,000,000-work/8-MiB-text
defaults. Explicit selection uses report `/9`, native replay `/13` and stored
`live-replay.v13`; configuration/report identity retains both values. Missing,
unknown or duplicate fields, zero, invalid types and explicit `null` refuse.
Replay rejects component removal, downgrade or changed values. Report `/7` and
`/8` and replay1-12 retain their identity recipes. The actual previous v12 store
catalog is frozen: it can reopen its retained records but refuses v13 before an
operation or `Current` mutation. Choose finite values from observed owner guards;
this selector does not establish complete-corpus fit or memory qualification.

### Explicit source graph budgets

Schema `/2` accepts `project_budgets.source_graph` only with genuine native
retained packages and an explicit platform graph selection. Its required positive
`max_metadata_bytes` selects the existing metadata allowance. Omission preserves
the ordinary 4-MiB limit; metadata-only selection retains native replay `/14`.

The optional `xml_counts` child selects the complete retained XML projection
preflight populations, including invalid declarations and unresolved or cyclic
`Inherits` records. Both positive `u64` fields are required:

```json
"source_graph": {
  "max_metadata_bytes": 67108864,
  "xml_counts": {
    "max_declarations": 32768,
    "max_inheritance_references": 8192
  }
}
```

These are explicit example selections, not a full-corpus capacity guarantee.
Without `xml_counts`, the previous serialized bytes and declaration/inheritance
limits of 4096/8192 remain. Explicit null, missing required counts, zero, invalid
types, unknown fields and duplicate keys refuse. The selected child binds the
configuration and native replay `/15`, stored as `live-replay.v15`; numeric
changes stay within that shape and must match the captured configuration digest
on hydration. The actual previous v14 catalog remains frozen and can reopen its
populated records while refusing v15 before operation or `Current` mutation.

This component does not widen XML-fact/containment, global node/edge/assertion,
metadata, analyzer, map, output or store limits. Every independent guard must
still admit the operation. Native preflight success alone is not complete graph,
persistence or runtime acceptance.

## Scope

`graph build` writes an available native owner failure explanation to stderr.
The session retains the typed member cause, and Project/backend preserve its
message through the calling frontend. This runtime explanation is excluded from
the canonical graph result, digest and stored records; normalized failure codes
remain unchanged. Cancellation or result-sealing failure replaces the earlier
explanation. A caught upstream panic remains opaque. A generic failed result
without an available cause does not identify a particular budget or owner.

Shared acquisition also makes this input available to ordinary local
`status`/`check`. The selected CLI graph route is a bounded prerequisite for actual
owner measurement. It does not implement full-source capacity, partition
composition, candidate/freeze/Skeleton acceptance or a new project publication
CLI selector. Durable callers retain their existing exact graph-request contract.
Issue [#113](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/113)
owns remaining full-corpus qualification.
