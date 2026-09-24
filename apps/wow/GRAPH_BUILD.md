# Source graph construction and export

```text
wow graph build --config project.json --project my-addon --format json
wow graph build --config project.json --project my-addon --format snapshot > graph.json
```

Both successful commands exit **2 (Partial)**. The second emits a valid bare
`GraphPartitionSnapshot`, not an error document. Do not interpret the nonzero
exit as permission to discard the artifact or silently retry. Shell redirection
is optional and is performed by the shell; the command writes only to stdout.
Use a fresh output path: redirection can truncate an existing file even on failure.

Configuration is the same inline, explicit-files or selected-TOC input accepted
by `wow check`; existing profile/reference/Library requirements still apply.
`--generation current` (default) derives the generation from this explicit input,
not a disk current pointer. An exact generation must match that derivation.

The shared project materialization runs once: file acquisition, project/analyzer
snapshot and retained TOC/XML reports. `wow-project` then constructs source
proposals without rereading files or running another Emmy session. `wow-service`
asks `wow-graph` to validate the complete proposal batch and materialize one
`wow-project.source-load` producer partition. Rejected proposals fail the operation;
they do not publish a partly accepted graph. Diagnostic rule execution is not
required by this export route.

## Implemented graph

Each captured first-party Lua, TOC and XML file becomes a `source_file` node.
Every admitted direct TOC or XML file reference produces a `Loads` proposal from
its containing document to its target file. Source handles and EvidenceRecords
bind the actual project/reference generation, content checksum and exact reference
span. Whole-file evidence supports file nodes. No source text is executed or
reopened; excluded or uncaptured files are not invented.

The proposal IDs retain load occurrence ordinals. Repeated references may share a
materialized edge but remain separate producer proposals in the artifact. The
original load receipt preserves the sequence, conditions, exclusions, cycles and
other blockers. Missing targets and self-loads are counted in provenance; self
edges are not representable in the current stored graph schema. This graph is a
static file-reference topology, not proof of client execution or global load order.

For explicit-file/inline input without a TOC, captured Lua nodes are still exported
but load coverage is NotEvaluated. With a TOC, Loads coverage is Partial. Package
DependsOn coverage is always NotEvaluated; no dependency edges are fabricated.
The registry supports the existing Load axis while unsupported axis families
remain unsupported. Lua calls, recognizers, XML runtime objects and runtime relationships
are not generated. Source XML declarations and explicit inheritance are projected below. Negative authority is false throughout this route.

## XML source topology

Each indexed XML declaration becomes a distinct `xml_source_declaration` node,
identified by document and exact occurrence ID, never by display name. This is a
source record, not a claim that the client constructs a frame of that type. The
containing file has a Proven `Owns` edge to the declaration; this is document
source ownership, not XML containment, `parent`, or runtime frame parentage.

A direct `Inherits` edge (Derived) is emitted only for a unique valid local target
explicitly marked virtual/intrinsic, loaded before its source, with no retained
inheritance cycle. It retains evidence for the original attribute and the target
declaration. Duplicate source entries remain separate proposals. Unresolved names,
invalid declarations, non-template targets, forward/repeated/unrecorded load order,
cycles and self-references have explicit outcomes in `provenance.xml_inheritance`;
the original reference ID resolves all details in the retained load plan. No
candidate is selected from an ambiguous name group. XML `parent` never becomes
`Owns` or `Inherits`.

`xml_nodes` maps occurrence IDs and source paths to materialized node IDs. Full
source locations and declaration properties stay in the existing XML index; bodies
are not copied, parsed or executed again. Ownership and inheritance coverage stay
Partial (NotEvaluated without captured XML), with no negative authority. The
Ownership axis and ordinary `inherits` subgraphs can inspect these nodes; the full
Inheritance axis remains unsupported until the registry also defines MixesIn.

## Artifact and provenance formats

`json` (default) emits `wow-service/graph-build-result/2`: request, status,
`snapshot`, `file_nodes`, `xml_nodes`, `provenance`, boundaries and canonical digests.
`file_nodes` maps logical source paths to final materialized node IDs, rather than
producer-input IDs. `provenance` retains the exact project/analyzer snapshot IDs,
GenerationContext, file manifest, real SourceHandles/EvidenceRecords and optional
load receipt. Source bodies and host paths are not included. Preserve this receipt
when external evidence dereferencing is needed; the bare graph retains handles.

`snapshot` emits only the canonical `snapshot` object, **without a newline**. Its
exact bytes hash to the receipt's `snapshot_input_digest`. This artifact can be
passed directly to the existing commands:

```text
wow graph subgraph --snapshot graph.json --request subgraph.json --format json
wow graph axis --snapshot graph.json --request axis.json --format json
wow graph explain --snapshot graph.json --request explain.json --format json
```

Prepare requests using snapshot/node IDs from the receipt and the existing
[graph request format](GRAPH_INPUT.md). A graph-build JSON receipt is not itself
a bare snapshot. `text` prints a status header and the full escaped canonical
receipt. JSON adds a transport newline; semantic hashes exclude that newline.
The result digest excludes its own field and binds the entire remaining receipt.

The source universe uses the explicit project/workspace/origin/root/profile
identity. Input graph generation additionally binds the exact project snapshot,
projection profile and registry; only the graph owner derives the final graph
generation. Source/project/E0 check identities are not relabeled or changed.

## Bounds and failures

At most 4,096 files, 8,192 admitted non-self load proposals, 4,096 XML declarations
and 8,192 inspected inheritance references. Document ownership adds at most one
edge per declaration. The combined ceiling is 8,192 nodes and 20,480 edges;
charged projection text stays below 4 MiB. Existing source/input/analyzer/load limits
remain in force. The bare graph must fit the existing graph reader's 16 MiB,
1,000,000 token/key, 64-level and 16 KiB decoded-string limits. Admission uses the
same bounded decoder before export. The complete provenance receipt is capped
at 32 MiB; serialization is size-counted before canonical allocation. A budget
failure returns no graph rather than stripping evidence or truncating a snapshot.

Cancellation is checked during materialization, source/proposal collection,
graph-owner work and around serialization. Failure/cancellation clears snapshot,
provenance and node maps; snapshot format emits no artifact for these outcomes.
Individual filesystem, owner-validation and serializer calls are not forcibly
interruptible. Broken output is not retried. No source mutation, ProjectStore
write, current-pointer change, lease or crash-recovery guarantee is introduced.

Exits: 2 for a produced Partial artifact; 3 for a structured construction/identity/
budget failure; 4 for encoding or output loss; 64 for CLI/config acquisition
errors; 130 for cancellation. Tests, real-addon/client validation and full E2/R0
acceptance remain separate from this functional implementation.

The source projection and graph-build request/result advance to v2; the local
registry advances to version 2. Existing retained v1 graph artifacts remain readable.
Project/E0 identities and graph-read schemas are unchanged; the new graph profile
and registry enter only newly built graph identities.

Owners: `crates/wow-project/src/graph.rs`, `crates/wow-project/src/graph/xml.rs`, `crates/wow-service/src/graph/build.rs`,
`apps/wow/src/graph_build.rs`. Contracts: project `e2/README.md`,
`e2/DATA_MODEL.md`, graph `e2/KIND_AND_RELATION_REGISTRY.md` and
`e2/CONFLICT_COVERAGE_AND_PROVENANCE.md`.
