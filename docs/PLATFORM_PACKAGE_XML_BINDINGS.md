# Native package XML bindings

Platform package inputs can select XML-to-Lua binding analysis in the shared
Main/Library Emmy session. Configuration selects the behavior; a separate replay
version or storage catalog does not.

## Selection and owners

```text
builder
    .platform_packages(admitted_packages)?
    .with_package_xml_bindings(PackageXmlBindingProfile::SameSessionV1)
    .build()?
```

`admitted_packages` is the actual `Arc<PlatformPackageSpecialization>` from
[native package specialization](PLATFORM_SOURCE_PACKAGES.md), with its selected
Reference profile and load/Main plans. A platform label alone is insufficient.

`ProjectAnalyzerBinding::package_xml_bindings()` returns
`ProjectPackageXmlLuaBindings` from `wow_project::xml_bindings`. It holds package
plans, Main/Library identities, ordered groups and the original union lookup.
Each group retains its selected TOC, reachability, local XML and qualified source
locations. Equal local bytes/occurrence IDs in two packages are legitimate.
Use `address(package, binding_index)` and `resolve_binding(&address)` for rows;
package qualification does not isolate Lua globals.

## One session and uncertainty

Preparation gathers a sorted distinct query union. The shared Main/Library
session resolves it once; groups borrow the same original lookup rather than
creating sliced reports. Empty groups remain present. Explicit
`ProjectPublisher::with_function_call_facts()` adds callable facts to that same
session; selecting package XML alone does not enable them.

Keep malformed/unsupported names, parse failures, ambiguity, receiver blockers
and incomplete inherited sources visible. A declaration candidate does not prove
a constructed receiver, execution, package ordering or runtime callback dispatch.
Receiver semantics remain `not_evaluated`.

## Bounds and code to reuse

The owners are `crates/wow-project/src/xml_bindings/packages.rs`,
`xml_bindings.rs` and `xml_bindings/{scripts,receivers}.rs`; session query admission
is in `crates/wow-emmy/src/references.rs`. Change their real callers together,
not a parallel validator or acceptance registry.

Ordinary paths retain separate 4096-row/owner/source/query ceilings, a 16384 query
reference ceiling and 65536 raw member-pair floor. Explicit retained platform
input derives the relevant limits from Project fact/output budgets. Physical and
virtual collection share the selected limits; combined fact/output admission
still applies. A per-category allowance is not the total fact or memory budget.

Keep cancellation, per-name/text limits and cumulative accounting before copying
large inputs. The package owner has a 64-MiB serialized ceiling; configuration
may impose less, and the complete stored replay has its own 32-MiB ceiling.
Read exact formulas from the owners above instead of copying them into a new
wire profile. Physical flow uses the shared physical/virtual environment.

## Current replay and verification boundary

All inputs use one strict `ProjectReplay` DTO and one `wow-project.live-replay`
record. XML selection remains typed configuration and is rebuilt with actual
source/package/analyzer owners. Current reads still compare semantic identities
and recapture replay; #131 removes unnecessary reconstruction on ordinary reads.
No frozen V5/V6 catalog, old epoch reader, migration or compatibility fallback is
required. Rebuild incompatible derived stores from source and configuration.
See [platform replay](PLATFORM_SOURCE_REPLAY.md).

Existing Project package-XML and Service namespace cases contain useful checks
for shared names, scoped paths, source-absent reopen and refusal of inconsistent
inputs. Their existence does not mean they ran for the current revision.
Historical test totals and corpus milestones belong in Git/PR history, not this
instruction. Real addon/source qualification remains in #125/#201; use one
relevant existing check when changing this path rather than restoring a frozen
matrix.
