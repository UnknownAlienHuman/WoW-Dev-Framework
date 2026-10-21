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
across all groups. For a genuine platform universe with explicitly selected
native package retention, the separate binding-row, receiver-owner, inherited
script-source and distinct query-union ceilings use the configured
`ProjectBudgetPolicy::max_analyzer_facts`; preparation carries the same binding
ceiling to classification. Each counter bounds one constituent of the final fact
count; combined fact and output admission still applies. Ordinary and standalone routes
keep 4096 rows/owners/inherited sources/queries. Each inherited-source row is
directly charged by final combined fact admission. Its declaration-visit and
text-byte guards remain separate. The selected query ceiling reaches both native
symbol and callable input lanes through `MemberCallSessionQueryProfile`, including
the virtual-unit route. Native session inputs retain the original 4096 floor when
the configured fact ceiling is smaller: standard callable-only names are not
charged by that aggregate fact budget. Final combined admission remains separate.
Input counts include repeated names; each lane retains a
cancellable checked 16-MiB input-text bound and 4096-byte individual name bound.
Repeated query references are a separate unit from distinct lookup rows and are
not charged as analyzer facts. Only the explicitly retained native route selects
their ceiling from the configured output bytes, clamped to the original 16384
floor and the unchanged 16-MiB binding-text ceiling. Supported retained strings
remain verbatim in `binding.queries`, so each reference occupies at least one
serialized byte. Expanded candidates can later be discarded as unsupported or
invalid; their guard bounds temporary work and does not prove final output size.
The original floor preserves historical admission. One private ceiling reaches
expansion, retention and classification; ordinary routes retain 16384 references.
The same explicit retained native route selects a raw direct-member pair ceiling
from `max_analyzer_facts / 2`, converting after division and retaining the original
65536 floor. Physical, virtual and combined collectors share it, with checked
combined arithmetic. It bounds collection work rather than proving the final fact
count: virtual projection can filter raw records. The native cold flow consumer
uses the same registered semantic session and physical member prerequisite, so a
second default collector cannot reimpose65536. Ordinary/standalone routes remain
on their existing path. Aggregate fact/output admission and local-flow guards
remain independent; successful wire/profile/identity recipes are unchanged.
Physical flow shares the mixed native physical/virtual semantic environment;
no universal old-cohort report equality is claimed. Its full fixture freeze
remains open in [#119](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/119).
Method visits, per-method distinct names, receiver record/text/expansion,
inherited declaration visits/text, native lookup targets/text, callable named
targets and serialization retain their own independent limits.
The existing configuration/replay records already bind
the selected retention and budget; no additional wire selector is introduced.

Scope/document retention is also bounded. Cancellable
streaming counting precedes canonical allocation and seals the final report's
byte length. Analyzer admission adds its retained records and report bytes to
the configured fact/output budgets. The fixed 64 MiB owner ceiling does not
replace the stricter configured output or complete 32 MiB replay-record ceiling.

## Exact replay and frozen catalogs

Selected genuine platform inputs use `ProjectReplay`
and storage `wow-project.live-replay`. Unselected platform inputs keep /5;
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
package budget-exhaustion/cancellation fault cases remain NotEvaluated at that
historical checkpoint. The later focused native-retention lifecycle verifies
ordinary 4096-row admission/refusal, native 4098-row and receiver-owner admission, selected limit+1,
strict combined-fact refusal, cancellation and exact disk-absent replay11.
Its distinct-query case retains 8194 package-local rows and one original lookup
with 4097 distinct names, including a real fixture Library declaration, and
restores exact identities and bindings without source files. A focused
native-session lifecycle separately checks both 4097-name lanes, ordinary and
selected refusals, repeated inputs, input-text overflow, original lookup/sidecar
IDs, small-report equality, retained update, virtual routing and cancellation.
A small native configuration with 16 admitted facts and the 18 standard callable
queries also publishes and replays exactly, preserving default input admission.
The later real
273-package operation passes the previous query-union frontier. A further native
fixture retains 4098 inherited inline-handler source rows, refuses ordinary
capacity and combined-fact overflow without publication, honors cancellation,
and restores exact identities and full bindings without disk. The later real
operation passed the inherited-source frontier and next refused expanded query
references at 16385 against 16384. A further focused native fixture retains
16392 references with four distinct names, preserves two-method ambiguity and
original lookup/callable IDs, and restores exact reports and snapshot identities
without disk. Its exact reference ceiling reaches final serialized-output
refusal; limit+1 refuses during expansion, and cancellation publishes nothing.
The later freshly resolved 273-package operation completes preparation with
13445 bindings, 16995 references, 8768 distinct queries, 11951 method visits and
2091042 retained text bytes. Native analyzer budget admission then refuses with
its existing generic error; the exact internal resource/count is still unknown.
Complete native reports, classification and graph execution remain unqualified,
as do the completed-operation memory peak and downstream persistence.
Full real-corpus XML binding capacity remains unqualified under #113.

The direct source producer remains monolithic `wow-project/source-load-proposals/19`.
Package XML/state graph projection, complete raw-member graph/read bindings,
independent platform producers, structural fingerprints, SkeletonInputView and
source service/CLI transport remain open. Full W17/E3/package and real
Gethe/Ketho corpus, performance and WoW runtime acceptance remain open.
