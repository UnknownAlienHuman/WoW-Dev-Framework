# Platform source packages

**Status:** implemented. Three focused native cases and workspace policy, fmt, check,
strict Clippy, tests (922 passed, 1 ignored, 108 targets), strict rustdoc and build
passed on 2026-10-10. Implemented scope is not full W17 acceptance.

Byte admission is documented in
[PLATFORM_SOURCE_ADMISSION](PLATFORM_SOURCE_ADMISSION.md). E3-A remains normative for
universe and package separation in
[UNIVERSE_AND_PACKAGE_MODEL](../crates/wow-project/e3/UNIVERSE_AND_PACKAGE_MODEL.md).

The owner lives in
[platform_source/packages.rs](../crates/wow-project/src/platform_source/packages.rs) and
[platform_source/package_binding.rs](../crates/wow-project/src/platform_source/package_binding.rs),
reusing the existing load owners in
[load/mod.rs](../crates/wow-project/src/load/mod.rs) and
[load/package_closure.rs](../crates/wow-project/src/load/package_closure.rs).

## What the owner produces

`AdmittedPlatformSource::specialize_packages` binds one held source admission to a
native package-load plan and Main plan, returning `PlatformPackageSpecialization`. It
validates the package declarations, requires an explicit selected root, checks the
joined selected-TOC set against the profile's own selection, reads every demanded member
through the retained bytes, then runs the existing admitted package path, the Main
namespacing and their actual validation.

Parsing, static dependency closure and Main namespacing remain the existing load owners;
this owner adds no parser, scanner or generic trait.

It is not a project configuration, an analyzer binding, a graph, a store publication or a
skeleton-input view, and it grants no such capability.

## Retained bytes are the only member source

`load_member` is the crate-private demand port. It checks cancellation, validates the
root and selected path, confirms configured-root membership, forms the joined snapshot
path, and answers from the held inventory by binary search. Only an `Included`
disposition yields bytes, bounded by the native consumed-byte limit and re-verified
against the caller's pinned identity. There is no disk fallback and no source lookup.

Each non-included disposition maps to its own typed outcome with the exact logical path:
a missing undeclared entry is `MissingDeclaredFile`, an exclusion is
`PackageTargetExcluded`, an unsupported special entry is `InvalidFileLanguage`, and
external, conflict or failed evidence is `PackageTargetUnresolved`. Requested omissions
refuse specialization, while untouched omissions stay recorded on the admitted receipt.

## Package and TOC selection is exact

Roots, variants and the selected TOC set come from the caller, never from inference.
Each variant is looked up by its exact joined path, must be an admitted TOC member, and
is read through `load_member` at the native disk-source byte cap before its pinned
identity is rebuilt from the observed bytes. A selected variant that is not in the
profile's selection, a duplicate selection, or a final set that differs from the profile
selection each reject.

Unknown inventory bytes are never decoded unless actually demanded, so a retained binary
member stays a byte record rather than becoming an analyzed input.

## The binding is sealed and derive-checked

`PlatformPackageBinding` is serialize-only with no `Deserialize` and no input
constructor. It records the source profile, content-manifest and admission digests, the
source snapshot ID, the universe ID, the target, the load and Main plan digests, and a
binding digest over all of them, on schema
`wow-project/platform-package-binding/1`.

The universe ID is `blizzard_ui_source:<profile-digest>:<source-snapshot-id>`, so two
different source sets cannot share one universe label. Raw bytes, host roots and provider
display labels stay out of it, while the admission evidence digest is retained and remains
part of the binding. `validate` reconstructs the binding from the exact source and plans
and compares every field, so decoded evidence cannot author or alter a binding.

## What the universe is not

The existing Emmy `LuaWorkspaceUniverse::BlizzardUi` is Library-only. In the current
analyzer owner a Blizzard UI snapshot is registered as a library workspace, and the
local-flow owner rejects it outright as a Main project workspace. It is therefore not a
platform Main universe and must never be used as a fallback.

Platform implementation units remain technical Main. Mapping them onto a typed universe
without reusing `BlizzardUi` as a library fallback is a separate decision for the
configuration and analyzer owners, and this document does not make it.

## Preserved authority

Specialization changes nothing admission already asserted. The source inventory stays
Partial, every unevaluated capability stays unevaluated, and omitted and unknown records
stay on the held source. Raw admission limits do not widen existing consumed load,
package or parser limits, and the result adds no project configuration, publisher, disk
route or activation capability.

## Not claimed

No project publication, analyzer snapshot, graph, store publication, skeleton-input view
or full W17 acceptance is claimed. The passing checks verify this bounded owner.
