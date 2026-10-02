# XML inline Lua analysis in local check

`wow check` analyzes admitted inline Lua bodies in the same generation-bound EmmyLua session as the
exact physical Main workspace and ordered Library snapshots. The pinned analyzer produces syntax,
semantic diagnostics and direct member/call facts. Source remains the original unwrapped Lua body:
no Lua is executed, no second parser or semantic session is opened, and no synthetic callback
wrapper, guessed `self` or runtime dispatch model is introduced.

Commands and TOC configuration are unchanged. XML files may also be selected:

```text
wow check --config project.json --project my-addon --file project-file:UI/Frames.xml
```

Only XML documents captured in this exact load plan are selectable. A physical Lua-only scope does
not receive unrelated XML diagnostics. A whole-project check includes both physical and XML units.
An XML-only selection now runs `wow.api.exists@1` for exact static namespace member/call facts when
the selected unit has `exact_xml_script_site` authority and complete unit fact coverage. Receiver-
dependent rules and units without admitted semantic facts remain explicitly `NotEvaluated`.

## Identities and output

Each virtual unit binds the project generation, load-plan digest, exact XML content, script
occurrence, extraction identity, versioned semantic-context policy and analyzer profile. Its
`wow-xml-lua:///...lua` URI is a logical address; no file is created or opened. Repeated XML includes
reuse one source-body analysis while retaining separate load occurrences and unresolved runtime
semantics.

`owner_analysis.xml_lua_report` retains whole-project unit identities, parser/analyzer identities,
mapped syntax and semantic diagnostics, direct member references/calls and unresolved script
occurrences. It contains no source bodies or upstream free-text errors. The analyzer and project
snapshot identities include this report. The profile also enters TOC configuration identity before
project-generation derivation; non-TOC E0 identities are unchanged.

Every unit carries the versioned authority tuple:

```text
script_site = exact_xml_script_site
implicit_receiver = not_evaluated_unwrapped_source
runtime_dispatch = not_evaluated_static_load_evidence_only
```

The first value permits consumers to use only direct static source associations. The latter two
forbid treating a successful semantic pass as proof of callback receiver type, inherited template
state, lifecycle/event timing, dispatch order, frame readiness or actual execution.

Mapped diagnostics enter ordinary `raw_findings` and the presentation graph. Each has the XML file
digest and a `source_mapping`: virtual-unit ID, virtual byte range, mapping kind and all XML
locations. `location` is the first **display anchor**, not an enclosing approximation of a
discontinuous range. `exact_pieces` preserves comment/CDATA gaps; `caret_boundaries` preserves every
possible source boundary around a removed delimiter. Entities and normalized newlines use the
retained extraction map. No range is widened to bridge a gap.

For `wow.api.exists@1`, the first exact member-name piece is the primary source. Additional exact
pieces, the full member reference and the optional call mapping are related evidence. The service
resolves these locations through the immutable captured XML artifact rather than pretending the
virtual unit is a physical project file.

Text output includes unit/diagnostic/unresolved counts and the normal finding records. Status still
does not run analysis. `project.xml.inline_lua.syntax` is available when all recognized inline
sources were representable. The broad `project.xml.inline_lua.analyzed` capability remains Partial:
unit-specific complete semantic/fact coverage does not establish types for implicit receivers,
inheritance, lifecycle, secret-value safety or runtime execution.

## Bounds and cancellation

At most 4,096 virtual units, 1 MiB per unit, 16 MiB extracted source, 4,096 parser diagnostics per
unit and 65,536 total are admitted by the adapter. Project file/source/diagnostic budgets apply to
physical plus virtual units. Mapped XML pieces are additionally capped at 65,536. Invalid parser
ranges, stale document digests, missing source artifacts, non-exact member/call mappings or a
non-closed fact graph reject publication/evaluation rather than losing evidence or reporting clean.

The old `ProjectPublisher::publish_initial` API remains; the local backend uses
`publish_initial_cancellable`. Cancellation is checked between physical analyzer stages and virtual
units and before publishing the snapshot. An individual upstream parser/analyzer call cannot be
forcibly interrupted. No new dependencies are used.

Implementation: `crates/wow-emmy/src/virtual_semantics.rs`,
`crates/wow-project/src/xml_lua.rs`, `crates/wow-project/src/snapshot.rs`,
`crates/wow-rules/src/engine.rs`, and `crates/wow-service/src/local/projection.rs`.
The normative virtual-unit route is `crates/wow-project/e2/XML_MODEL.md` and the exact API rule
contract is `crates/wow-rules/API_EXISTS_RULE.md`. The pinned EmmyLua revision remains
`aaaca68425d9362876228649b0b8d92f07654daa`.

Effective receiver/inheritance/dispatch semantics, rich expression/type/value-flow facts, XSD
conformance, WoW runtime observation and full E2/R0 acceptance remain open.
