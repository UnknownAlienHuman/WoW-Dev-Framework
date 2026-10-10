# Selected native platform package graph

`PlatformGraphProfile::PackageProjectionV1` explicitly selects package XML,
script-binding and SavedVariables graph inputs from the existing retained load,
Main and analyzer owners. It requires a genuine
`ProjectKind::BlizzardUiPlatformSource` configuration and
`PackageXmlBindingProfile::SameSessionV1`:

```rust
builder
    .platform_packages(admitted_packages)?
    .with_package_xml_bindings(PackageXmlBindingProfile::SameSessionV1)
    .with_platform_graph_profile(PlatformGraphProfile::PackageProjectionV1)
    .build()?
```

Callable/script and state-access facts additionally require
`ProjectPublisher::with_function_call_facts()`. Structural XML and TOC observations
do not require that option. Neither selection establishes runtime receivers,
dispatch, successful loading or persisted state. See the preceding
[package XML binding owner](PLATFORM_PACKAGE_XML_BINDINGS.md).

## Native scope and identity

Each graph scope borrows an original selected-TOC plan and its native package
node. Qualified document paths come from the package source resolver. Lua load
order additionally requires exact native Main receipts, matching local source,
qualified path, digest and length. Captured TOC/XML membership and analyzer Main
membership remain distinct. Reachability follows the native explicit selected-root
and dependency closure receipts. A reachable missing Lua target is omitted from
order
evidence only when no local source bytes were captured and a native `MissingFile`
issue matches that load record's document and span. Unreachable packages retain
captured structural facts and explicit blockers without synthesized Main units.
An unreachable top-level chunk retains both `package_unreachable` and
`owner_not_captured`; a receiver-bound unreachable handler retains its reachability
blocker without an invented inline Main unit.

Native XML occurrence/reference IDs remain local. Package proposal addresses bind
the owning package, plan digest, qualified document and original local ID. Equal
XML bytes and local IDs in different packages therefore remain distinct. Named
bindings retain native aggregate addresses and the original shared symbol lookup;
the adapter does not manufacture per-package lookup reports or run another parser,
analyzer or temporary ProjectView.

Main declarations retain their actual paths and spans. Shared Lua globals can
remain ambiguous. A unique callable in another package cannot use local load
ordinals to prove order; it retains `LoadOrderUnresolved`. Method, inherited and
inline associations retain source-only confidence and unresolved runtime context.

The XML matcher joins by native selected-TOC/flavor/package/occurrence scope and
uses actual accepted receiver endpoints. SavedVariables roots bind qualified TOC,
spelling and scope. All selected package declarations precede one global-access
pass; repeated global spelling across TOCs remains ambiguous. These facts do not
establish runtime values or persistence.

Limits and cooperative cancellation apply across scopes. The original package
binding aggregate is charged once against the shared 4 MiB graph text allowance
before retention; handler, binding, site and query counters are also aggregate.
The analyzer's separate report ceilings and the replay record ceiling do not
replace this graph allowance.

## Explicit graph and replay channels

The selector participates in configuration and project-generation identity.
Selected graph output uses
`wow-project/source-load-proposals/20`; absence retains the published `/19`
recipe, including binding-only native replay `/6`. Selected native replay uses
`wow-project/native-project-replay/7` and `wow-project.live-replay.v7`.

`ProjectReplay::hydrate` re-admits retained source/package/configuration/analyzer
owners without disk and compares project/analyzer identities plus full canonical
recapture. Native pair acquisition additionally rebuilds the exact source batch,
registry, coverage and foundation, and validates graph/publication membership
under the held read. A serialized report or profile label supplies no native owner
authority. Existing v1-v6 recipes and catalogs stay frozen. An old V6 epoch
refuses v7 publication without retaining an operation or changing Current/epoch.

The native service request selects the same profile through
`GraphBuildRequest::with_platform_graph_profile`. Its request/result channels are
`wow-service/graph-build-request/10` and `wow-service/graph-build-result/17`.
The existing request `/9` and result `/16` transport continue to require the
original `/19` projection; they do not accept `/20`. Default evidence/persistence
entry points also require `/19`. Separate typed platform entry points admit `/20`
against the actual graph; they do not reconstruct a project/analyzer owner.

## Verification and remaining scope

Two native synthetic cases passed on 2026-10-10: the project case removes source
disk, preserves repeated local IDs, shared-global ambiguity, same-package bindings
and cross-package order refusal, and reproduces `/7` graph evidence after hydration.
The service case composes the source and recognizer chain, reopens exact Current
in the selected namespace after disk removal, and refuses v7 writes to a frozen
V6 epoch without effects. See
[project regression](../crates/wow-project/tests/package_xml_bindings.rs) and
[service lifecycle](../crates/wow-service/src/live_project/tests/namespaces.rs).
The final project regression also passes with captured XML in a package outside
the selected root closure: top-level chunks retain both blockers, and unreachable
inline callbacks retain their reachability omission without `inline_parse_failed`.

Final workspace gates completed at 2026-10-10 07:18:40 UTC: `cargo xtask check`,
`cargo fmt --all --check`, workspace all-target/all-feature check, strict Clippy,
full tests (**938 passed, 0 failed, 1 ignored, 111 targets**), strict rustdoc with
`RUSTDOCFLAGS=-D warnings`, and workspace all-target/all-feature build all passed.
These results include the final unreachable-source omission correction. This
records local verification; remote checkpoint publication is not asserted here.

The package source producer remains monolithic; the existing recognizer partitions
remain separate. A later explicit route adds the separate Included-member graph
and bounded native byte binding; see
[PLATFORM_RAW_INVENTORY_GRAPH.md](PLATFORM_RAW_INVENTORY_GRAPH.md). Independent
platform direct producers and fingerprints, bounded SkeletonInputView and source
service/CLI transport remain open. Full W17/E3/package acceptance, real Gethe/Ketho corpus,
performance, native fault/exhaustion and WoW runtime checks remain NotEvaluated.
Original source inventory remains Partial; consumed-file checks establish neither
whole-source completeness nor license permission.
Gethe materialization remains deferred.
