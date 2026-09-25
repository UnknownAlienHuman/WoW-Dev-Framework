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
asks `wow-graph` to validate the complete proposal batch and materialize the source partition plus independent
`wow-recognizers.lua-direct-calls` and `wow-recognizers.xml-script-bindings`
partitions. Rejected proposals fail the operation;
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
remain unsupported. XML runtime objects and runtime relationships are not generated.
Captured Lua functions and direct calls are projected as described below. Source XML declarations and explicit inheritance are projected below. Negative authority is false throughout this route.

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
Ownership axis and ordinary `inherits` subgraphs can inspect these nodes. The
Inheritance axis also includes the explicit Main mixin links described below.

## XML mixin source links

Explicit XML `mixin` entries now reuse the retained XML-to-Lua binding report.
A Derived `MixesIn` edge connects the XML declaration to one uniquely resolved
Main Lua declaration. Each target is a `lua_source_declaration`, identified by
exact document and byte span, not by its name. A Derived `Owns` edge connects
the source file to that declaration. Different spellings resolving to the same
location share a node; ordered XML entries retain separate proposals/receipts.

Only complete, full-path declaration lookups are admitted. Main snapshot, file
digest and UTF-8 span must match the captured project. The target Lua file must
have exactly one included load before the XML declaration's unique load record.
Ambiguous/missing/failed lookup, invalid XML, unlocated targets and uncertain
load order remain explicit outcomes rather than guessed edges. Library targets
stay outside the addon graph even if their relative path matches a Main file.

`provenance.xml_mixins` addresses each original binding by `binding_index` in
`provenance.xml_binding_report`. The complete unmodified report preserves lookup
candidates, source locations and analyzer identity, including for skipped links.
`provenance.lua_declarations` holds exact target handles/evidence; `lua_nodes`
maps those declarations to final materialized node IDs. Edge evidence retains
both the XML attribute and Lua target bytes. No source is reread or reanalyzed.

The existing Inheritance axis can now traverse both source template inheritance
and these declared mixin links. Forward walks from a base/mixin to consumers;
reverse walks from consumers to source ancestors. This does not assert that a
Lua assignment runs, returns a table, copies fields, or selects runtime method
precedence. Inherited mixins are reached through explicit `Inherits` paths, not
copied into fabricated transitive `MixesIn` edges. Coverage stays Partial with
no negative authority; zero mixin entries yield NotEvaluated, not proven absence.

The source projection/registry and graph-build request/result use version 5. Existing retained graphs and graph-read request formats are unchanged.
Source review: Gethe `live` resolved to
`09b9db7948abc9b9648dedaab51eb0cf3ee67b31` on 2026-09-24;
`Interface/AddOns/Blizzard_SharedXML/Shared/FrameTemplate/RingedFrameTemplate.xml`.
This observation is not a fixed runtime dependency or client verification.

## Lua functions and direct calls

The complete source-to-recognizer-to-graph route is now part of `wow graph build`.
It requests `wow-emmy/function-call-facts/2` from the **same semantic session** that
already collects member calls and XML lookup results. It does not reparse files,
compile another workspace or infer calls from token spelling. Plain named calls,
member/colon calls, aliases and immediate closure calls are eligible when Emmy
returns one concrete captured closure signature for the callee expression.

Each healthy Main Lua file contributes a chunk scope; each actual closure
contributes a `lua_source_function` node. Function identity binds the exact Main
snapshot, path, content digest, scope kind and byte range. Calls are assigned to
their nearest enclosing closure or the file's chunk. A target closure uses the
same identity as its own calls' source, allowing multi-hop call chains, including
cross-file chains. Physical declaration names and runtime object identities are
not substituted for callable identity. File-to-function `Owns` means source
ownership; enclosing-function IDs are retained as facts, not invented runtime
ownership or lexical-axis edges.

`wow-project` creates the callable source occurrences and exact call-site
SourceHandles/EvidenceRecords. `wow-recognizers` validates the source crosswalk,
context, checksums, ranges, original proposals and accepted input-generation
nodes, then feeds structured `DirectCall` observations to the existing closed
`wow.direct-call` rule (version `1`). It emits Derived `Calls` proposals in the
separate `wow-recognizers.lua-direct-calls` partition. Only `wow-graph` validates
and materializes the producer partitions and rebinds their final generation. There is no
source-owned Calls inference or parallel graph engine.

`provenance.function_call_report` retains per-file parse health, all captured
functions, calls and target outcomes. `provenance.functions` and `call_sites`
resolve those facts to source evidence. `call_recognition` retains the original
recognizer report and an outcome for every captured call. `function_nodes` and
`call_edges` map facts to **final materialized** node/edge IDs; recognizer report
IDs/endpoints remain scoped to their original input graph. The Call axis and the
existing `neighbors`, `path`, `subgraph`, `entity` and `explain` commands work with
these records without changes to graph-read request formats.

Library closures are explicitly foreign targets, not Main nodes, even with the
same relative path. Unresolved/unknown types, multiple or documented callable
types, uncaptured signatures and direct self-recursion remain separate outcomes.
The current graph schema rejects self-edges: recursive call facts are retained
with `self_recursion_unsupported`, never silently erased. Calls in syntactically
failed Main files are not mined from recovery syntax; those files retain their
parse-error count. XML inline function semantics, dynamic dispatch, `__call`,
dynamic callback registration and the other non-call recognizer families remain
unevaluated; the XML source-assignment slice is described below. No
claim is made that a discovered function runs or a particular call happens in
WoW. Every layer retains partial/unevaluated coverage and no negative authority.

Ordinary `wow check` and `wow status` do not request this extra fact collection;
their previous analyzer bytes/IDs remain unchanged. Native graph callers use
`ProjectPublisher::with_function_call_facts()`. With the option enabled, the
analyzer snapshot identity includes the sidecar's exact report ID and the source
graph seed binds that analyzer through the project snapshot. The publisher keeps
the option across updates; no stale callable report is copied to a new snapshot.
Default publishers may still export the source-only subset with no call report.

## XML handlers and source assignments

`wow graph build` also produces `SetsScript` associations through the existing
`wow.script-assignment` recognizer. No new command or input configuration is
required. The receiver is an exact `xml_source_declaration`, not a runtime frame.

For `function="Addon.Initialize"`, Emmy retains the concrete callable signature
from the **same** full-path lookup already used by the XML binding report. The
new graph-only `named_targets` sidecar resolves that signature to a captured Main
closure, not to a declaration's display name or a guessed source range. A direct
unique association is Derived. Non-callable, ambiguous, Library, failed and
uncaptured targets have explicit receipts and do not produce guessed edges.

For `method="OnLoad"`, each independently resolved callable from the existing
receiver/mixin query set remains a **Possible** association, even when only one
candidate is found. All located candidates remain separate; no override winner
is selected. Inherited handler sites reuse the previously retained source graph
and resolve methods for the consuming XML declaration, not just the template.
Every inherited association is Possible, including a fixed `function` or inline
source. Incomplete ancestry and unresolved consumer/load order block projection.

A valid inline handler gets an `xml_source_handler` node and a file-to-handler
source `Owns` edge. It references the already extracted virtual unit and its
existing syntax report. The graph stage does not copy the Lua body, generate a
wrapper, parse it again, or pretend it is a Main callable. Direct inline source
associations are Derived; parse-failed bodies retain a skipped-site receipt.
Source evidence covers the exact XML element; piecewise body locations remain
in the original load plan and `xml_lua_analysis`. Calls inside inline bodies
remain outside the semantic call graph.

The target Lua file must have one admitted load before the consumer's unique XML
load. A `function` target must additionally precede the original declaring
source's load. These conservative source-order guards do not prove statement
execution or runtime table construction. Original `inherit` and `intrinsicOrder`
values are preserved verbatim; append/prepend/override/intrinsic dispatch order,
parameter types and effective installed handlers are not computed.

`provenance.script_sources` retains every captured UI script source and support;
`script_sites` retains direct/inherited consumers, original binding indices,
per-query outcomes, admitted binding IDs and blockers. `script_bindings` supplies
only normalized source facts to the recognizer owner. The adapter checks exact
source partition endpoints, context, generation and each source/evidence record
before invoking the closed matcher; it is not a second XML/Lua analyzer.
`script_recognition` retains observation/assertion/proposal crosswalks and the
original recognition report in the independent script partition.

`handler_nodes` maps inline sources to final node IDs. `script_edges` maps each
admitted site association to final receiver/handler/edge IDs and confidence.
All file, declaration, function and call maps are rebound **after** both recognizer
partitions are published, so no preceding generation's IDs escape. Named handlers
share the existing `lua_source_function` nodes, allowing explicit
`sets_script` + `calls` path/subgraph queries into downstream calls. Methods and
inherited associations require `confidence: "include_possible"`; the default
Proven/Derived policy intentionally excludes them. Querying these stored edges
does not make a Possible path executable or a selected handler effective.
The full Registration axis is still unsupported until its other families exist.

Coverage is Partial for admitted script associations, NotEvaluated without them,
and never authoritative absence. Existing graph-read encodings and function/call
occurrence-key recipes are unchanged; the graph-only report identity advances
with its new data. No source writes, extra semantic sessions or background work
are introduced.

## SavedVariables declarations and source accesses

The selected TOC now retains `SavedVariables` (account) and
`SavedVariablesPerCharacter` (character) declarations with exact directive and
entry ordinals, source spans, selection and validation state. No saved data file,
account name, character identity or runtime value is opened or inferred. Only
included metadata from that exact selected variant seeds `state_root` nodes.
The supported name grammar is an ASCII Lua identifier other than reserved words,
`_G` and `_ENV`. Invalid entries remain receipts; duplicate entries and cross-scope
name conflicts mark their roots ambiguous and prevent access projection.

The optional existing Emmy session collects generic global-slot accesses in
Main files. Its declaration lookup distinguishes globals from local bindings,
parameters and implicit `self`; a local variable with the same spelling never
becomes a saved root. The project joins only names declared by the selected TOC.
A target must resolve to a Main global declaration in a parse-valid captured file.
Library, unresolved, parse-failed declaration sources, uncertain metadata selection,
unsupported assignments and dynamic keys retain explicit skipped-site outcomes.
Undeclared global names remain generic analyzer facts, not persistent state.

For example, with `## SavedVariables: AddonDB`, direct accesses to
`AddonDB`, `AddonDB.settings.enabled` and `AddonDB["settings"]["enabled"]` are
represented. Dot and string-literal spellings share the same ordered symbolic key
path; different paths never collapse by dotted display text. Plain assignment
and function-definition targets are writes; other occurrences are reads. Each
maximal index/parenthesis chain is one slot access associated with its containing
function/chunk. Assignment receiver evaluation does not synthesize extra reads
of all path prefixes, and a call through a field does not imply a state mutation.

State paths express *source references*, not proof that fields exist, hold a
particular type/value, were executed, or persisted successfully. `_G`/environment
indirection, computed keys, aliases returned by calls, and accesses inside inline
XML remain outside this profile. No metatable effects, initialization/lifecycle
order, migration or storage contents are evaluated.

### Typed literal keys and lexical aliases

Keys are tagged values: `{"kind":"string","value":"settings"}`,
`{"kind":"integer","value":"1"}`, or `{"kind":"boolean","value":true}`.
Integer values use canonical signed decimal strings because core semantic JSON
forbids negative number tokens; the `kind` tag preserves the numeric domain.
`AddonDB[1]`, `AddonDB["1"]` and `AddonDB[true]` are distinct paths. Decimal and
hexadecimal integer tokens normalize to the same integer, including a single
unary minus and zero. Only integers in `[-9007199254740991, 9007199254740991]`
are admitted; fractional/exponent tokens, overflow, suffixes, computed indices
and safe-navigation syntax remain explicit unsupported paths. The adapter does
not use the upstream number accessor's fallback-to-zero behavior. Parentheses
are transparent, so `(AddonDB.settings)[1]` and `AddonDB.settings[1]` join.

The graph-only semantic pass now indexes exact local declaration IDs and their
rooted-path initializers. For example:

```lua
local db = AddonDB
local settings = (db.settings)
function ChangeSetting()
    settings[1] = true
end
```

The write has a **Possible** association to `AddonDB.settings[1]`. Chains of
aliases and captured lexical aliases are supported without treating a local's
name as a global. Each hop retains its declaration, initializer and full
statement spans. Exact evidence for every initializer is required at the
project-to-recognizer boundary; the original access and global declaration
anchors are also retained. A local `settings = other` never writes the saved
slot. Any syntactic rebinding of an alias, including in a branch or nested
closure, blocks that alias's entire chain with `reassigned_alias` or
`local_alias_rebinding`; the initializer's own direct read is still recorded.

This is deliberately not flow-sensitive runtime alias analysis. A stable local
can retain an older table after a global or field is replaced. Consequently
all alias-based edges stay Possible, never Derived from a unique name alone.
`state_edges.confidence` exposes that ceiling. Include them explicitly with
`confidence: "include_possible"` in State-axis, neighbor, path or subgraph
queries. Direct accesses remain Derived; an exhausted or blocked alias path is
not a clean negative. Initializers using calls, `and`/`or`, vararg expansion or
other expressions do not fabricate a known alias origin.

Bounds are cumulative across the optional report: 2,000,000 alias/path work
steps and 8 MiB charged text, plus the existing source/fact/output limits. Each
file has at most 16,384 alias definitions/write entries, each access at most
16 alias hops and each path at most 64 syntactic levels/keys. A limit fails the
operation without publishing a partial, unsupported alias as an exact slot.

Source-owned roots and paths connect through namespace `Owns` edges. The existing
`state-read` / `state-write` recognizers produce Derived direct or Possible alias
`ReadsState` / `WritesState` relations in an independent `wow-recognizers.saved-variable-access` partition.
Before matching, the adapter validates the immutable analyzer report, exact
function/root/path proposals, binding digest, context and original source handles
and evidence. Each observation retains its own access span, global declaration,
containing function and TOC declaration support. Reused paths retain their first
canonical observation's node support without substituting it for later access
locations. The graph stage does not reread or parse Lua/TOC.

`state_nodes.roots` maps declared namespaces to final node IDs and preserves scope
and ambiguity. `state_nodes.paths` maps exact key vectors to final IDs.
`state_edges` maps each admitted access to its function, state node and final edge
ID; `state_recognition` keeps the original matcher report and proposal crosswalk.
`provenance.state_declarations`, `state_roots`, `state_paths`, `state_sites` and
`state_bindings` retain the source join. The complete generic access report remains
in `provenance.function_call_report.global_accesses`.

The State axis and ordinary `reads_state` / `writes_state` queries now work on
these exports. Combined `sets_script`, `calls` and state relation queries can
trace source-level reachability from XML handlers to state-accessing functions;
Possible handler and alias associations still require explicit confidence opt-in.
All prior file/XML/function/call/handler maps are rebound after the state partition,
not left pointing at a previous graph generation. Coverage remains Partial for
observed modes and NotEvaluated otherwise; neither is authoritative absence.
Ordinary `wow check` / `status` retain TOC declarations but do not enable the
optional semantic graph-access collection.

The source graph projection/registry and graph-build request/result are v7;
the optional Emmy report is v4 and global-access/state-recognizer profiles are v2.
Typed keys intentionally change state-path identities and the v7 build receipt
shape. The selected TOC profile, ordinary check/status path, function/call
occurrence recipes and graph-read request formats are unchanged by this update.
Previously retained graph snapshots keep their original identities and remain
readable; they are not relabeled or rewritten.

Source review: Gethe `live` resolved to
`09b9db7948abc9b9648dedaab51eb0cf3ee67b31` on 2026-09-24;
`Interface/AddOns/Blizzard_SavedSets/Blizzard_SavedSets.toc` and
`Interface/AddOns/Blizzard_RaidUI/Blizzard_RaidUI_Mainline.toc`. These declaration
examples are not a runtime acceptance probe or a fixed dependency.

## Reading a retained bundle

All graph reads accept this v7 JSON receipt directly through `--bundle`.
`wow graph explain --bundle build.json --request explain.json` resolves the
retained source/evidence tables and derivation inputs without rerunning the
build. The bare-snapshot route remains unchanged. See
[GRAPH_EVIDENCE.md](GRAPH_EVIDENCE.md) for exact admission and proof boundaries.

## Artifact and provenance formats

`json` (default) emits `wow-service/graph-build-result/7`: request, status,
`snapshot`, `file_nodes`, `xml_nodes`, `lua_nodes`, `function_nodes`, `call_edges`,
`handler_nodes`, `script_edges`, `provenance`, `call_recognition`,
`script_recognition`, `state_nodes`, `state_edges`, `state_recognition`, boundaries and canonical digests.
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
wow graph path --snapshot graph.json --request path.json --format json
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

At most 4,096 files, 8,192 admitted non-self load proposals, 4,096 XML declarations,
8,192 inspected inheritance references, 4,096 Main mixin targets/links, 8,192
callable scopes and 8,192 call facts; at most 4,096 script sources/inline nodes,
8,192 direct/inherited sites, 8,192 handler associations and 16,384 script query
visits. Source ownership contributes at most one edge per source occurrence.
State projection admits at most 1,024 declaration entries, 8,192 paths and 8,192
access sites; each observation carries at most 32 source/evidence references.
Combined owner ceilings are 33,792 nodes and 74,752 edges (including recognizer
calls/assignments/state accesses); charged projection text is capped
at 4 MiB. The Emmy sidecar separately caps 65,536 callable records/signatures,
65,536 calls, 65,536 generic global accesses (64 literal-key levels, 1 KiB per
name/key and 8 MiB of charged access text), 4,096 named callable targets, 2,000,000 AST visits, 256 ancestor steps per scope lookup and 32 MiB
of serialized report. Source projection applies its smaller bounds before
creating graph proposals. Over-limit input aborts rather than returning a
truncated snapshot. Existing source/input/analyzer/load limits
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

Owners: `crates/wow-project/src/graph.rs`, `crates/wow-project/src/graph/xml.rs`, `crates/wow-service/src/graph/build.rs`,
`apps/wow/src/graph_build.rs`. Contracts: project `e2/README.md`,
`e2/DATA_MODEL.md`, graph `e2/KIND_AND_RELATION_REGISTRY.md` and
`e2/CONFLICT_COVERAGE_AND_PROVENANCE.md`.
