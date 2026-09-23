# XML to Lua declaration lookup

`wow check` now resolves XML `mixin` and `function` spellings and queries direct
mixin candidates for `method` references. Commands and input schemas are unchanged.
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

For `method`, only the owner's explicitly declared mixins form queries such as
`ExampleMixin.OnLoad`. A single observed target is `declared_mixin_candidates`,
not an accepted runtime binding. Different targets remain ambiguous; no mixin,
unknown receiver or unsupported expression is unresolved. XML inheritance is not
flattened, mixin precedence is not invented, and `CreateFromMixins` is never run.
Function and mixin spellings use a closed ASCII dotted-identifier dialect; calls,
index expressions and `$parent` substitution are not evaluated.

Unresolved/candidate records enter ordinary informational findings with fixed
`xml.lua.*` codes and exact XML anchors. Successful declaration lookups remain in
the detailed report. Text output includes reference/unresolved counts. Status does
not run Emmy: it reports pending lookup only when XML actually contains these
references. The `project.xml.lua_bindings.queried` capability denotes that queries
ran; receiver capability remains partial for method references. File-only XML
introduces no new pending binding component.

## Identity and limits

The adapter profile enters TOC configuration identity before project-generation
derivation. The final report binds the generation, load plan, Main/Library analyzer
identities, source health and lookup results; the analyzer snapshot binds that
report. Inline-input/explicit-file E0 identity fields are unchanged when no TOC
plan is present. Existing APIs remain available; the cancellable member-call
entry point also accepts explicit symbol queries.

At most 4,096 binding records/unique queries, 16,384 binding-to-query references,
16 components/4,096 bytes per query, 256 candidates per lookup stage and 65,536
retained target references are admitted. Query/source spellings in each owner
report have a 16 MiB aggregate bound. Budget exhaustion rejects, not truncates.
Cancellation is polled between files, syntax visits and query stages; one upstream
semantic operation is not forcibly interruptible. No dependency/lockfile/fixture
changes accompany this checkpoint.

Implementation: `crates/wow-emmy/src/bindings.rs`,
`crates/wow-project/src/xml_bindings.rs`, and
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
