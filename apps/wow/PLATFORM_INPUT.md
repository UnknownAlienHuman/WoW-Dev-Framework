# Manifest-bound platform graph input

`wow graph build --config project.json --project <ProjectId> --format json`
accepts `wow-service/local-platform-source/1` through the existing local input
service. This explicit read-only source/package/Main/graph route does not acquire
source, discover packages or resolve a moving selector.

## Configuration

The JSON object rejects unknown and duplicate fields. Required fields are:

| Field | Meaning |
|---|---|
| `schema` | `wow-service/local-platform-source/1` |
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

## Scope

Shared acquisition also makes this input available to ordinary local
`status`/`check`. The selected CLI graph route is a bounded prerequisite for actual
owner measurement. It does not implement full-source capacity, partition
composition, candidate/freeze/Skeleton acceptance or a new project publication
CLI selector. Durable callers retain their existing exact graph-request contract.
Issue [#113](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/113)
owns remaining full-corpus qualification.
