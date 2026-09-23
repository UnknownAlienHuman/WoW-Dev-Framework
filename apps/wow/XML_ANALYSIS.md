# XML inline Lua syntax in local check

`wow check` now sends retained inline bodies to the pinned EmmyLua parser through
`wow-emmy::virtual_syntax`. The adapter uses Lua 5.1 chunk syntax with EmmyLua
annotation parsing. This is a syntax pass, not Main/Library type resolution or a
model of implicit callback parameters. No Lua is executed and no wrapper is added.

Commands and TOC configuration are unchanged. XML files may also be selected:

```text
wow check --config project.json --project my-addon --file project-file:UI/Frames.xml
```

Only XML documents captured in this exact load plan are selectable. A physical
Lua-only file scope does not receive unrelated XML diagnostics. A whole-project
check includes both; XML-only selection leaves WoW-rule semantics NotEvaluated.

## Identities and output

Each virtual unit binds the project generation, load-plan digest, exact XML
content, script occurrence, extraction identity and versioned parser adapter.
Its `wow-xml-lua:///...lua` URI is a logical address; no file is created or opened.
Repeated XML includes reuse one source-body parse, while their separate load
occurrences and unresolved runtime semantics remain in the load plan.

`owner_analysis.xml_lua_report` retains whole-project unit identities, parser
identity, mapped diagnostics and unresolved script occurrences. It contains no
source bodies or upstream free-text errors. The analyzer and project snapshot
identities include this report. The adapter profile also enters TOC configuration
identity before project-generation derivation; non-TOC E0 identities are unchanged.

Inline diagnostics enter ordinary `raw_findings` and the presentation graph.
Each has the XML file digest and a `source_mapping`: virtual-unit ID, virtual byte
range, mapping kind and all XML locations. `location` is the first **display anchor**,
not an enclosing approximation of a discontinuous range. `exact_pieces` preserves
comment/CDATA gaps; `caret_boundaries` preserves every possible source boundary
around a removed delimiter. Entities and normalized newlines use the retained
extraction map. No range is widened to bridge a gap.

Text output includes unit/diagnostic/unresolved counts and the normal finding
records. Status still does not run analysis. After parsing, the narrow capability
`project.xml.inline_lua.syntax` is available when all recognized inline sources
were representable. `project.xml.inline_lua.analyzed` remains partial: successful
syntax does not establish types, callback binding, inheritance, load order,
secret-value safety, or runtime execution. The load plan is an immutable
**acquisition** receipt; its pending-analysis issues are not rewritten after parsing.

## Bounds and cancellation

At most 4,096 virtual units, 1 MiB per unit, 16 MiB extracted source, 4,096 parser
diagnostics per unit and 65,536 total are admitted by the parser adapter. Project
file/source/diagnostic budgets apply to physical plus virtual units. Mapped XML
pieces are additionally capped at 65,536. Invalid parser ranges or missing source
mappings reject publication, rather than losing diagnostics or reporting clean.

The old `ProjectPublisher::publish_initial` API remains; the local backend uses
`publish_initial_cancellable`. Cancellation is checked between physical analyzer
stages and virtual units and before publishing the snapshot. An individual upstream
parser/analyzer call cannot be forcibly interrupted. No new dependencies are used.

Implementation: `crates/wow-emmy/src/virtual_syntax.rs`,
`crates/wow-project/src/xml_lua.rs`, and `crates/wow-service/src/local/xml_lua.rs`.
The normative virtual-unit route is `crates/wow-project/e2/XML_MODEL.md`.
The exact parser API was reviewed at the existing lockfile revision
`aaaca68425d9362876228649b0b8d92f07654daa` (LuaParser, ParserConfig, LuaSyntaxTree,
LuaParseError). This is dependency evidence, not a permanent target-client version.
Full virtual-unit semantic analysis, XML inheritance/reference resolution,
real-addon execution and E2/R0 acceptance remain open.
