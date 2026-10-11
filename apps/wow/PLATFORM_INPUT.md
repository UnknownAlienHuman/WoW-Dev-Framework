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
`library` has the existing disk inventory shape
`{"root":"logical-root","files":[<ProjectDiskFile>,...]}`. Absence means an
actually empty Library snapshot. It inserts no dummy Lua or generated annotation
and does not establish annotation or external Library completeness.

For all three currently available capabilities, `graph_profile` is:

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
Retained-profile archives use `wow-project/native-project-replay/11`; ordinary
replay1-10 retain their bytes and selection. Hydration re-admits the exact source
and package owners under the captured limits. Changed limits or schema downgrade
refuse. A successful retained Main is not full graph or persistence acceptance:
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
`wow-project/native-project-replay/12`. The configuration digest and report
identity both retain the selected values. Hydration rebuilds them without source
files and refuses removal, downgrade or changed values against the original
identities. Ordinary replay1-11 recipes remain unchanged.

This component does not widen alias work/text, scope, argument/literal,
callable-input, flow, graph/map or replay/store budgets. Choose finite values from
the actual selected operation; neither this selector nor a synthetic boundary
case qualifies the full corpus or its peak memory.

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
