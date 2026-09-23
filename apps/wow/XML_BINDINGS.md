# XML to Lua declaration lookup

`wow check` resolves XML `mixin` and `function` spellings and queries direct and
inherited source mixin candidates for `method` references. Commands and input
schemas are unchanged.
`--file project-file:UI/Frames.xml` selects findings anchored in that XML file;
lookup still uses the complete captured Main and explicit Library universes.

The project owner emits bounded dotted-name queries. `wow-emmy` evaluates them
against the same populated semantic session that emits member-call facts, without
another compilation, generated query Lua, callback wrappers, physical files or
source execution. Roots come from Emmy's global-declaration index; member steps
use its inferred owner type and member lookup. Local variables are not promoted
to global handler names. Parser errors and unknown types remain explicit.

## Results

JSON `owner_analysis.xml_binding_report` contains ordered XML binding records and
one shared `symbol_lookup.lookups` table. Each target retains the exact workspace,
Main/Library role, logical path, content digest and UTF-8 byte span. XML records
retain their own document digest, element identity and original attribute span.
Targets from Library remain Library evidence, never first-party Main declarations.

A unique result is `unique_analyzer_declaration`: an observation of a declaration
in this exact analyzer snapshot, **not** proof of callability, execution, load-time
availability or a fully constructed receiver. Distinct definitions remain
ambiguous. Dotted lookup stops at an ambiguous/unknown prefix and records the
number of components processed; prefix candidates are not final-member matches.
`not_observed` means absent from this query result, not absent from the WoW client.

For `method`, the owner's direct mixins and explicitly declared mixins in linked
XML templates form queries such as `ExampleMixin.OnLoad`. Lookup reuses the same
Emmy session and shared query table. A single observed target is still
`declared_mixin_candidates`, not an accepted runtime binding; different targets
remain ambiguous. An unknown receiver or incomplete inheritance stays unresolved, rather
than implying that the method does not exist. Mixin precedence is not invented,
and `CreateFromMixins` is never run.

### Inherited receiver sources

`owner_analysis.xml_binding_report.receiver_sources` stores one source graph per
exact method-owner XML occurrence. Method records reference it by
`receiver_source_id`; other binding kinds omit that field. The graph retains
visited declaration IDs, ordered inheritance reference IDs, mixin spelling/list
ordinal/attribute span, and explicit blockers. IDs address the existing load-plan
XML element/declaration/reference tables, which retain document digests and locations.
All handlers on one owner share that graph, and identical queries are deduplicated
without discarding distinct mixin source occurrences.

Only the existing `inherits` links are traversed, never XML containment or
`parent`. A target must be a unique, valid local declaration explicitly marked
`virtual="true"` or `intrinsic="true"`. Unsupported/dynamic names, missing and
ambiguous declarations are not expanded into guessed candidates. A unique name
alone does not attest a template kind; other targets retain a blocker.

Forward, repeated-load, duplicate-inheritance and cyclic links preserve any
independently inspectable source candidates, but the graph stays incomplete.
`complete` means only that this bounded source enumeration had no blockers;
it does not certify client load order or effective method precedence. An
incomplete receiver graph cannot become an accepted candidate result merely
because a Lua query found one declaration: the method stays
`receiver_not_resolved` and its useful lookup results remain in the owner report.

Traversal is iterative. Each declaration is visited once per owner; all inspected
edges are retained, including shared and rejected edges. A diamond does not copy
the common ancestor for every route, and a cycle does not loop. Inheritance order
is retained per declaration, not advertised as a flattened runtime method order.
This does not instantiate inherited Script elements or materialize inherited
fields, callbacks, parameters, metatables or Lua runtime receivers.

Function and mixin spellings use a closed ASCII dotted-identifier dialect; calls,
index expressions and `$parent` substitution are not evaluated.

Unresolved/candidate records enter ordinary informational findings with fixed
`xml.lua.*` codes and exact XML anchors. Successful declaration lookups remain in
the detailed report. Text output includes reference/unresolved counts and shared
receiver/partial-receiver counts. Status does
not run Emmy: it reports pending lookup only when XML actually contains these
references. The `project.xml.lua_bindings.queried` capability denotes that queries
ran; receiver capability remains partial for method references. File-only XML
introduces no new pending binding component.

## Identity and limits

The adapter profile enters TOC configuration identity before project-generation
derivation. The final report binds the generation, load plan, Main/Library analyzer
identities, source health, receiver-source graphs and lookup results; the analyzer
snapshot binds that report. The XML binding profile advances to
`wow-project/xml-lua-bindings/2`, including in TOC configuration identity before
deriving the project generation. Inline-input/explicit-file E0 identity fields are unchanged when no TOC
plan is present. Existing APIs remain available; the cancellable member-call
entry point also accepts explicit symbol queries.

At most 4,096 binding records/unique queries, 16,384 binding-to-query references,
16 components/4,096 bytes per query, 256 candidates per lookup stage and 65,536
retained target references are admitted. Query/source spellings in each owner
report have a 16 MiB aggregate bound. Budget exhaustion rejects, not truncates.
Cancellation is polled between files, syntax visits and query stages; one upstream
semantic operation is not forcibly interruptible. No dependency/lockfile/fixture
changes accompany this extension.

Receiver enumeration additionally caps shared owners at 4,096, aggregate retained
receiver records at 65,536 and their variable strings at 16 MiB. At most 262,144
graph expansion steps and 262,144 method-query expansion visits are allowed per
operation, shared across all owners/handlers. Existing global query/output limits
still apply, including before retaining inherited per-handler query copies.
Exceeded budgets and cancellation abort the operation, never silently shorten a
receiver graph. No additional Lua compilation, parser or dependency is introduced.

Implementation: `crates/wow-emmy/src/bindings.rs`,
`crates/wow-project/src/xml_bindings.rs` (with `xml_bindings/receivers.rs`), and
`crates/wow-service/src/local/xml_bindings.rs`.
Contract: `crates/wow-project/e2/XML_MODEL.md` (Scripts, Inheritance, Coverage).
The existing pinned Emmy revision `aaaca68425d9362876228649b0b8d92f07654daa`
supplies the global/declaration/member/type and SemanticModel APIs. Source example
reviewed on 2026-09-23: Gethe `live` resolved to
`09b9db7948abc9b9648dedaab51eb0cf3ee67b31`,
`Interface/AddOns/Blizzard_SharedXML/Shared/Frame/EventFrame.lua` and its XML
bindings. These are evidence identities, not permanent client targets. Full
callback type analysis, inheritance materialization and client behavior are not
claimed by local declaration lookup.
