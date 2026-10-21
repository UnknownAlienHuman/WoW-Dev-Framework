# Selected Ketho type catalogs

The supported generator accepts repeated `--alias-catalog` inputs from an
explicit donor checkout/revision. Read the [Ketho map](../../docs/KETHO_RUST_PORT.md)
for the native owners and supported declaration forms; it is not a promise that
every donor resource or arbitrary mixin can be projected.

## Current workflow selection

At `1a87ce15`, `.github/workflows/current-source-bundle.yml` explicitly selects
six `Annotations/Core/Type` resources:

```text
BlizzardType.lua
EmoteToken.lua
FunctionContainer.lua
GlobalColors.lua
StringEnum.lua
Structure.lua
```

It does not discover every file in that donor directory. The previous nine-file
list also named `Mixin.lua`, `Namespace.lua` and `UnitToken.lua`; those are not
selected by this workflow. Their omission does not prove their parser support is
absent. Describe the actual selection and source revision, not a "complete"
catalog inferred from a directory or an old report.

The alias reader checks exact tree paths and regular-file modes and reads Git
objects rather than dirty worktree contents. TOC/API reads have a separate mode
check gap; do not generalize the catalog guarantee to every source input.
Duplicates, unresolved dependencies and unsupported forms remain explicit rather
than being widened to `any` or silently overriding Blizzard declarations.

The workflow's post-generation verifier still expects the old report shape;
see the [documented handoff gap](../../docs/BLIZZARD_API_REFERENCE.md#current-handoff-gap).
The [semantic probes](../../docs/ANNOTATION_CONSUMER_PROBES.md) also use the old
driver. #201 must connect the supported producer and its consumers before this
workflow is cited as integrated evidence. A partial exit alone proves neither
complete type closure nor that a nonempty usable Library was generated.
