# Explicit local source census

```text
wow source census --config <source-census.json> [--format json|text]
```

This read-only command calls `wow-service::source_census::census_local_source`
once. The configuration parent is the explicit source snapshot root. The project
disk owner reads only manifest-declared members through the retained directory
handle, checks lengths/digests and refuses symlinks or changed content. No source
download, directory scan, Lua execution or store write occurs. The ordinary
inventory/manifest census measures unique documents without include expansion.

The strict configuration has four fields:

| Field | Contract |
|---|---|
| `schema` | `wow-service/source-census-input/1` |
| `profile` | `BlizzardUiSourceProfileRequest` from [source admission](../../docs/PLATFORM_SOURCE_ADMISSION.md) |
| `inventory` | `wow-project/platform-source-inventory/1`, bound to that exact profile digest |
| `selection` | Explicit `package_root` and optional `load_context` using the existing TOC context contract |

Paths remain snapshot-relative. The selected reference profile supplies the
exact flavor/Interface identity; the operation does not infer them from TOC names
or a moving branch. Caller Git/version, materializer and root-completeness claims
remain assertions. A fixture cannot become vendor-source evidence.
An actual native source-only profile/generation can be obtained with
[`wow source reference`](SOURCE_REFERENCE.md), without constructing Main or
analyzer inputs. Its returned identities remain bound to its exact API selection.

The same command also accepts `wow-service/source-census-manifest-input/1` with
`profile`, `selection` and `source` instead of `inventory`. This uses the existing
strict xtask v1 manifest decoder and derives entries through
`ProjectInputDirectory::admit_platform_source_manifest`; callers do not construct
a second inventory or profile digest.

`source` has these fields:

| Field | Contract |
|---|---|
| `source_root` | Explicit directory relative to the configuration parent; `.` selects that parent |
| `manifest` | `ProjectDiskFile`: configuration-relative JSON path, SHA-256 content digest and byte length |
| `source_revision`, `source_version` | Exact expected manifest revision and version.txt value |
| `origin` | Existing platform origin; Git commit/object format must match the manifest, or a synthetic fixture digest must equal the manifest bytes |
| `materializer`, `license`, `compatibility_evidence` | Existing platform caller assertions, validated without attestation |

The platform census accepts the two existing xtask v1 acquisition labels:
`local_git_object_database` and `github_api_exact_blob_snapshot`. The latter
consumes an already materialized immutable [API snapshot](../../docs/SOURCE_API_MATERIALIZATION.md);
the census does not make network requests or verify the materializer's durable
operation records. Both labels remain manifest assertions. Unknown labels are
refused. The generated-API Lua closure's separate admission route still accepts
only `local_git_object_database`.
The project owner verifies the manifest wire, self-digest, every member and kind
count, and pinned version.txt bytes before census. Selected roots and TOCs remain
explicit. Included members inside those roots are verified through the existing
platform byte owner; reviewed exact exclusions remain Excluded. Fixture members
never acquire Git membership assertions. An archive origin is rejected.

The manifest result uses `wow-service/source-census-manifest-result/1` and adds a
`source_manifest` receipt alongside the ordinary census. It retains manifest and
version identities, declared file/byte/generated-API counts, the count of xtask's
unlisted extension exclusions, and exact declared identities of members outside
the selected roots. Those outside bytes are not verified. Every derived root is
`declared_partial`: the extension-selected manifest cannot attest full root
closure or enumerate the excluded extensions. Consequently this route exits `2`
even when every selected document parses. Git membership and root completeness
remain `not_attested`, with no negative authority.

The result retains source/profile/manifest/admission identities, target, origin,
root assertions, omissions and per-document outcomes. Counts include every
declared file by kind/disposition, immediate package child directories observed
in manifest entries, their selected/included/declared TOCs, and selected-TOC
lexical file records. Direct files beneath `package_root` and entries outside it
are counted separately. Empty directories absent from the manifest are unknown.

Every included XML and XSD document is parsed once using the existing XML owner.
Selected TOCs use the existing TOC parser; other TOCs receive `unselected_toc`.
XML and schema counts are separate. Elements, attributes, script source kinds,
inline units/bytes and mapping segments are exact observations of successfully
parsed unique documents, independent of how often a load path might reach them.
The XSD count is lexical XML measurement, not schema-component admission.
An initial UTF-8 BOM stays in the source identity; XML positions compensate for
the tokenizer's removed prefix so attribute, element and inline-Lua maps address
the original bytes. Syntax, entity, DTD and namespace checks remain in force.

Processing retains one syntax index at a time. The inherited parser bounds remain
in force, including 1 MiB per document and 32,768 lexical records. Invalid UTF-8,
oversize documents, parser syntax/budget refusal and missing selected TOCs remain
explicit; refused documents do not contribute invented syntax counts. Integer
counts use checked arithmetic. Configuration is bounded by the existing 32 MiB
disk port; result/index encoding is bounded by the 64 MiB native-artifact ceiling.

Selected TOCs excluded by their explicit target filters receive
`toc_target_excluded`; ambiguous filters or missing target context receive
`toc_target_unresolved`. These per-document refusals retain path/digest/length,
contribute no syntax counts and keep the census Partial while other documents
are measured. Cancellation and acquisition/identity errors still abort. Ordinary
package loading continues to reject these TOCs. The census `/1` contracts are
transient and unreleased; existing document outcome bytes remain unchanged.

`index_json_bytes` measures serialized XML indexes through a counting sink,
without allocating their JSON. Skipped decoded attribute values and inline text
have separate byte counts. `max_measured_document_*` records the largest successful
document observation. These numbers are not allocator or peak-RSS estimates.
The census digest binds its deterministic result with the `digest` field zeroed;
it is not a hash over the bytes after that field is filled.

JSON output is the exact service result plus LF. Text prefixes the same receipt
with a scope label. Exit codes: `0` measures all declared included members with
no declared omissions/refusals; `2` preserves partial inventory, omissions,
selection issues or refusals; `4` is acquisition/identity/budget/output failure;
`64` is invalid transport/configuration input; `130` is cancellation. Root
completeness is unverified even at exit `0`.

## Explicit package load measurements

The same command accepts `wow-service/source-package-census-input/1`, containing
`profile`, `source`, `selection` as above and a required `packages` array. Each
package declares `name`, logical `root`, `selected_root`, and `variants`; a variant
contains a `ProjectDiskFile` named `toc` and a `selected` boolean. Exactly one TOC
per package and at least one root package must be selected. The selected TOC set
must equal the exact source profile's set. Names/roots/variants use the existing
native package validation and TOC identity-pinning owner, with no discovery or
flavor inference.

This route expands selected closures through the existing native loader, projects
already-normalized dependency metadata, and uses the native dependency/SCC/order
owner. Iterative SCC traversal avoids package-depth recursion and checks
cancellation. It does not execute includes or Lua. Its named result contract is
`wow-service/source-package-census-result/2`; `package_loads` accompanies the
unchanged lexical `census` and manifest receipt. Both input/result contracts and
`wow-project/platform-package-census/2` are transient unreleased contracts. The
package input remains `/1`; ordinary lexical request/result bytes are unchanged.
The previous package `/1` receipts lack the two XML-reference populations below;
those populations are unknown in an old receipt, never measured zero. Census
receipts are serialized observations, with no graph/replay/store reopen decoder.

Empty or inline-only selected closures are measured through the same native
loader with a private census purpose. Their physical `main_lua_files` count is
zero; they acquire no Main/analyzer capability. Ordinary disk, retained replay
and admitted Main loading still require physical Lua. Missing or unresolved
targets retain the existing load coverage and issues; no dummy file is inserted.

Measurement capacity equals the validated selected-TOC count, whose existing
profile ceiling is 1024. Ordinary Main specialization, graph and replay retain
their 64-package limit. Census retains data-only observations and native record
projections; it never constructs an expanded retained Main plan. Aggregate source
limits remain 4096 package-qualified sources
and 64 MiB, including unselected TOC variants. Per-package parser/load bounds also
remain in force. Processed records/elements/attributes/map segments/dependencies
cannot exceed the prior maximum fanout of 64 ordinary packages. Each package is
loaded and measured once, then its captured text, physical Lua and XML indexes
are dropped before loading the next package. Ordinary retained loading and census
use one native dependency/SCC/order/unit/digest builder.

The 64-MiB encoding ceiling bounds each individual load plan, aggregate retained
compact projections and measurements, final native package records and the
census result. `native_plan_json_bytes` sums observed individual encodings using
checked arithmetic; dropped syntax plans do not consume the compact retention
budget. Encoded bytes are not an allocator or resident-memory ceiling.
These finite measurement limits are not a qualified full-corpus graph profile.

The receipt binds admitted source/profile/manifest identities, explicit physical
package roots, context, selected TOCs and native load/package-plan digests. It
reports per-package and aggregate retained source counts/bytes, unselected TOCs,
Main Lua files, expanded records and native load units, XML script/inline/map
counts, native XML-reference populations, dependencies by kind/resolution and
exact SCC/order groups. `xml_reference_declarations` counts every entry in the
held `XmlReferenceReport` declaration map; `xml_inheritance_references` counts
every `Inherits` record, including invalid, unresolved or cyclic observations.
These match the source-graph XML preflight populations; they do not count only
projected edges and are separate from lexical elements and script sites. Each
package is measured before its native plan is dropped, and the aggregate uses
checked addition without package-reachability filtering. Dependency
declarations, native package issues and per-package load issues preserve their
source identity/spans and coverage. Native units can refer to a missing target;
`units_without_retained_target` exposes that distinction. Counts include repeated
load records but count each retained document once within each package; they are
not a deduplicated physical-file census or runtime load evidence.

The command exits `2` when either lexical or package coverage is Partial. Invalid
package declarations, identity/acquisition refusals and budgets abort with the
existing service error/exit contract; no package is silently clipped and no
failed stage is reported as measured zero. Cancellation discards the whole
operation. Source handles/evidence, graph nodes/edges and metadata, allocator/peak
memory, full-corpus capacity and runtime remain NotEvaluated. The lexical census
still lists expanded loads/dependency closure as unevaluated in its own scope;
only the separate `package_loads` receipt evaluates those stages.

An aborted owner stage retains the native project's fixed failure explanation
in the CLI error. Existing service error classes and exits are unchanged;
source text, host paths and raw OS errors are never forwarded. A failed operation
has no measurement receipt, even when an earlier stage completed internally.

These operations are measurement prerequisites in Issue #113; they do not close
the measured-profile or W17 application/candidate/Skeleton acceptance gates.

For the separate native graph build, existing `wow graph build --config <project.json>
--project <ProjectId> --diagnostics` emits bounded nonsemantic JSON on stderr.
Before collector metadata charges, its held native report supplies three input
scalars: `function_total_metadata_estimate_bytes` sums every
`4 * path.len() + 512` function charge; `call_base_total_metadata_estimate_bytes`
sums every `2 * path.len() + 256` call base charge; and
`callable_base_total_metadata_estimate_bytes` is their checked sum. The explicit
counting method is `conservative_estimate`; exact sums of these formulas are
neither serialized artifact sizes nor allocation/peak-memory measurements.
Existing input populations and phase counters remain separate. Conditional Main
declarations from call arguments are unmeasured by these base sums; later stages
retain `not_evaluated` until executed. Cancellation never publishes a partial sum
as a total; absent reports or unfinished sums remain null. This adds no source
census result, graph/replay/configuration identity or corpus-capacity qualification.
