# Native package XML bindings

Verified 2026-10-10. Genuine platform inputs can explicitly select named XML
binding analysis through the existing Main/Library analyzer session. The
default package route retains its historical analysis and identities.

## Selection and native output

An already configured platform builder selects the recipe with:

```text
builder
    .platform_packages(admitted_packages)?
    .with_package_xml_bindings(PackageXmlBindingProfile::SameSessionV1)
    .build()?
```

`admitted_packages` is an actual `Arc<PlatformPackageSpecialization>` from
[native package specialization](PLATFORM_SOURCE_PACKAGES.md). Configuration
requires its exact Reference profile/generation and load/Main owners; ordinary
configuration or a naked platform kind cannot select this route. The selector
enters configuration identity. No source acquisition or discovery is added.

`ProjectAnalyzerBinding::package_xml_bindings()` returns the sealed
`ProjectPackageXmlLuaBindings` under `wow_project::xml_bindings`. Its profile is
`wow-project/package-xml-lua-bindings/1`. The serialize-only owner retains
generation, package load/Main plan digests, actual Main/Library snapshot IDs,
ordered package groups and one optional original union symbol lookup report.

Each group retains its selected TOC, native local plan digest, reachability,
phase, local XML documents and owner-resolved qualified paths/digests/lengths.
Rows, receiver IDs, inherited script sources and binding indices remain local.
Equal local XML bytes and occurrence IDs in different packages are legitimate.
Use `address(package, binding_index)` and `resolve_binding(&address)` to resolve
a row against the aggregate analysis ID, exact package and local plan digest.
The address grants source-row access, not graph membership or runtime authority.

## One session and preserved uncertainty

Preparation forms one sorted distinct XML query union. The existing native
session resolves it against shared Main and exact Libraries; classification
borrows the same original report for every group. Empty groups remain retained
beside a nonempty aggregate lookup. Groups do not contain sliced lookup reports.
Package-qualified paths do not isolate Lua globals or choose an ambiguous name.

`ProjectPublisher::with_function_call_facts()` separately requests callable
facts in that session. When requested, the callable report must include the
original XML lookup analysis ID as an exact constituent. Package selection
alone does not enable callable facts.

Malformed or unsupported spellings, parse health, native ambiguity, receiver
blockers and inherited-source completeness remain explicit. Method declaration
candidates do not prove a constructed receiver. Reachability, unique source
declarations and source enumeration do not prove execution, cross-package load
order or effective callback dispatch. Receiver semantics remain `not_evaluated`.

Preparation and classification share the existing separate category bounds
across all groups. Scope/document retention is also bounded. Cancellable
streaming counting precedes canonical allocation and seals the final report's
byte length. Analyzer admission adds its retained records and report bytes to
the configured fact/output budgets. The fixed 64 MiB owner ceiling does not
replace the stricter configured output or complete 32 MiB replay-record ceiling.

## Exact replay and frozen catalogs

Selected genuine platform inputs use `wow-project/native-project-replay/6`
and storage `wow-project.live-replay.v6`. Unselected platform inputs keep /5;
ordinary v1-v4 routes retain their recipes. Schema, typed selector, generation
recipe and genuine platform presence must agree. Hydration re-enters native
source/package/configuration/analyzer owners without disk and compares exact
snapshot identities plus full recapture. Failure has no legacy fallback.

The original V5 catalog remains frozen. New stores support v6; live, migration
and quarantine reopen paths admit each historical catalog exactly. An old V5
epoch refuses v6 publication before retaining an operation, preserving Current
and its old native pair. See [platform replay](PLATFORM_SOURCE_REPLAY.md).

## Executed evidence and remaining work

Two native project cases cover selector refusal, frozen /3 identities, default
/5 replay, repeated local XML in two packages, shared Lua ambiguity, an empty
group, unsupported spelling, inherited blockers, scoped borrowed row access,
same-session callable evidence and exact /6 replay after source removal. Schema
downgrade, missing selector and unknown selector refuse. One native service case
covers namespace publication/reopen and frozen V5 refusal with unchanged
Current/epoch and no retained failed operation.

Policy, fmt, all-target/all-feature check, strict Clippy, workspace tests
(936 passed, 1 ignored, 111 targets), strict rustdoc and build pass. Independent
static owner/replay/analyzer audits found no concrete P1/P2 defect. Native
package budget-exhaustion/cancellation fault cases remain NotEvaluated.

The direct source producer remains monolithic `wow-project/source-load-proposals/19`.
Package XML/state graph projection, complete raw-member graph/read bindings,
independent platform producers, structural fingerprints, SkeletonInputView and
source service/CLI transport remain open. Full W17/E3/package and real
Gethe/Ketho corpus, performance and WoW runtime acceptance remain open.
