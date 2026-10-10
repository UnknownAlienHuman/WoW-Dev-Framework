# Platform source configuration

**Status:** implemented and verified at the bounded scope below, 2026-10-10.
Scope: configuration, native analyzer Main, registry and source-graph identity.
Full W17 acceptance remains open.

Byte admission is documented in
[PLATFORM_SOURCE_ADMISSION](PLATFORM_SOURCE_ADMISSION.md) and package binding in
[PLATFORM_SOURCE_PACKAGES](PLATFORM_SOURCE_PACKAGES.md). E3-A remains normative for
universe and package separation in
[UNIVERSE_AND_PACKAGE_MODEL](../crates/wow-project/e3/UNIVERSE_AND_PACKAGE_MODEL.md).

The owner lives in [configuration.rs](../crates/wow-project/src/configuration.rs) and
[configuration/platform.rs](../crates/wow-project/src/configuration/platform.rs), with
registry identity in [registry.rs](../crates/wow-project/src/registry.rs) and graph
identity in [graph.rs](../crates/wow-project/src/graph.rs).

## Owner-bound project kind

`ProjectKind` gains exactly one value, `BlizzardUiPlatformSource`. It is owner-bound, not
a label: the configuration digest includes the retained platform package binding digest,
and the kind's presence must agree with whether packages are retained at all. A
configuration that claims the kind without a bound specialization, or that carries a
binding without the kind, is rejected.

`ProjectConfigurationBuilder::platform_packages` binds one genuine
`PlatformPackageSpecialization` and requires the platform kind. Binding on an ordinary
Fixture or Repository kind refuses, and a naked platform kind with no binding refuses.
Competing TOC or package-plan setters cannot substitute a different plan under a platform
binding, and validation reconstructs the binding and its plan association exactly.

The retained wrapper holds the specialization without pointer identity or raw bytes in
configuration serialization. Equality is content-based over the sealed binding and the
native load plan, Main plan and files.

The platform kind preserves the source profile class: synthetic sources require Fixture;
vendor mirrors require Release. The configuration checks the specialization's complete
Reference profile and exact Reference generation.

## Distinct technical Main universe

Genuine platform configurations use `LuaWorkspaceUniverse::BlizzardUiMain` for both
physical Main and XML virtual Main. The native analyzer rejects mixed platform/ordinary
Main pairs and rejects `BlizzardUiMain` as Library. Existing `BlizzardUi` keeps its
Library role; ordinary project Main keeps `Project`. The same analyzer/session owns
all classes, with no second parser.

## Registry and source-origin identity

Registry identity is project-owned and distinct: `ProjectSourceOriginKind` gains
`BlizzardUiPlatformSource` alongside `FixtureProject` and `RepositoryProject`. Every
registry, view and graph source handle agrees on the class for one configuration.

The core handle class is chosen from the admitted source class, not from the project kind
alone. A `SyntheticFixture` source yields `SourceOriginKind::Fixture`, and a
`VendorUiSourceMirror` source yields `SourceOriginKind::GeneratedArtifact`.

The indexed project handle uses `GeneratedArtifact` so it can retain both generation
bindings under the existing core contract. `SourceOriginKind::Repository` forbids these
bindings. The original source class, revision and provenance remain on the admitted
source; the core matrix is unchanged.

The revision recorded on the handle is the sealed binding's `source_snapshot_id`, so the
exact admitted source set is what a handle identifies. The reference generation and the
platform project generation are both preserved on each handle. The registry origin also
retains the project generation separately.

## Graph identity

The source-graph universe for a platform configuration is the sealed binding's
`universe_id`, which is derived from the profile digest and the source snapshot ID. The
ordinary Fixture and Repository recipe is unchanged. Two different source sets therefore
cannot share one graph universe label, and a configuration cannot relabel its graph
identity independently of its admitted bytes.

The native source-graph registry selects one exact class for that ProjectKind,
including TOC/XML definitions. Old ordinary registry bytes stay unchanged. This
admits actual source graph materialization; independently owned platform producer
partitions and structural fingerprints remain separate work.

## Durable replay is deferred

Native replay capture refuses a platform configuration with `DeferredCapability` before
any archive or store effect. No platform replay schema, transport or round trip is
claimed, and the refusal is explicit rather than a silent empty result.

## Preserved schemas and recipes

Old schema versions, domains, field ordering and identity recipes are unchanged. The
configuration gains an optional binding digest serialized with `None` omission, so existing
configurations keep their exact bytes and their existing configuration digests.

## Open gaps

Root completeness, Git membership, materializer security, client compatibility, license
permission, decoding, package load, analyzer, graph and runtime remain unverified from
admission, and the platform Main universe does not by itself establish analyzer or graph
coverage. Genuine raw/source/package replay, stable logical store namespace/publication,
platform partitions/fingerprints, SkeletonInputView, and full W17 or E2 acceptance
remain open.

## Executed validation

Four native platform cases and workspace policy, fmt, all-target/all-feature check,
strict Clippy, tests (923 passed, 1 ignored, 108 targets), strict rustdoc and build pass.
The new lifecycle removes its input directory before specialization, then publishes
native physical/XML Main and materializes the source graph through actual owners.
It verifies exact source handles, target/plan/naked-kind refusals, Main-as-Library
and mixed physical/virtual class refusal, plus DeferredCapability at replay capture.

Changing only a retained binary inventory member preserves native load/Main files
and Library, but changes the bound configuration. The real guarded updater publishes
a new generation rather than NoChange; Main workspace identity stays unchanged and
the old snapshot still validates. Frozen ordinary replay fixtures also pass.

These are synthetic in-memory owner checks. Full mirror, live source/network,
durable platform replay/store, performance and WoW runtime acceptance are NotEvaluated
for this checkpoint.
