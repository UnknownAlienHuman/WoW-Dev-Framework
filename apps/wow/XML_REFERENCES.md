# XML local declaration linking

`wow check` now includes `owner_analysis.load_plan.xml_references` for selected-TOC
input; `wow status` exposes the linking capability and plan digest without analysis.
Commands and configuration are unchanged. `--file` XML
selection includes link diagnostics anchored in that document; lookup still uses
the entire captured XML closure, so selecting one file does not invent missing
references to its included siblings.

The project owner links `parent` and each ordered `inherits` entry to exact,
case-sensitive declaration names in the retained XML indexes. Anonymous and
same-named declarations retain distinct source-occurrence IDs. Candidate groups
are stored once in `names`; ambiguity never picks a first/last/local-file winner.
Malformed declarations remain visible and cannot turn another candidate unique.
A unique link is `unique_local_declaration`, **not** an XSD-validated template,
accepted object type, Lua binding, or runtime object. Declared virtual/intrinsic
states are retained without inferring kind from names or suffixes.

Missing local names yield `not_in_captured_scope`, not an assertion that the
client/dependency lacks them. `$parent` and bracket expressions stay dynamic;
other spellings outside the bounded ASCII identifier dialect stay unsupported.
No substitution, dependency discovery, source reread or Lua execution occurs.

Actual load-record ordinals distinguish prior/forward/self references and repeated
loads. Forward links retain their target but are explicitly reported; this does
not guess the client's initialization behavior. Repeated includes retain all
occurrences rather than certifying one execution. XML containment, explicit parent
references, and inheritance are separate relations.

The linker detects strongly connected components independently in the explicit
parent and inheritance graphs, using only unique valid local links. Self-loops
are included. Ambiguous candidate combinations are not expanded into speculative
cycles. A cycle receipt contains all member declaration and internal reference
IDs; it describes the source graph, not observed client behavior. No transitive
field/method flattening or inheritance precedence is invented.

## Output, identity and bounds

JSON retains declarations, name groups, ordered references, resolutions, load
order, cycles and issues. Every diagnostic links the exact document digest and
source occurrence. Reference spans address the complete original attribute value;
`ordinal` disambiguates comma-separated entries. They are not approximate decoded
substring ranges. Normal findings expose fixed codes and source anchors: absent,
dynamic, forward and repeated-load cases are informational; declaration conflicts
and source cycles are warnings. Full details remain in the owner receipt.

Text output includes declaration/reference/unique-link/cycle/issue counts and the
report digest. `project.xml.references.indexed` denotes completed bounded linking;
`project.xml.references.local` is partial when that report has issues. Neither
capability promotes full XML semantics, dependencies or runtime to complete.
Existing load/semantic blockers remain in place.

The versioned reference digest binds the exact input index digests and all derived
records. Load profile/domain advance to v5 and include that digest, so the report
belongs to the same selected target/configuration/project generation. Other input
modes, dependencies, lockfile, fixtures and tests are unchanged.

Limits: 32,768 declaration sites, 65,536 references, 131,072 issues and 16 MiB of
retained source spellings/paths. Candidate tables avoid quadratic copies; cycle
traversal is iterative and linear in graph size, apart from deterministic map/sort
costs. Existing closure/input/output limits still apply. Budget exhaustion and
cancellation abort publication, never silently truncate a complete report.

Implementation: `crates/wow-project/src/load/xml_references/` and
`crates/wow-service/src/local/xml_references.rs`.
Contract: `crates/wow-project/e2/XML_MODEL.md` (Templates and objects, Inheritance,
Coverage, resolve_xml_references). Source example reviewed on 2026-09-23: Gethe
`live`, `09b9db7948abc9b9648dedaab51eb0cf3ee67b31`,
`Interface/AddOns/Blizzard_SharedXML/Shared/FrameTemplate/RingedFrameTemplate.xml`.
That revision is evidence, not a fixed client target. Lua binding/type resolution,
XSD object validation, inheritance materialization and real-addon/runtime
acceptance remain separate.
