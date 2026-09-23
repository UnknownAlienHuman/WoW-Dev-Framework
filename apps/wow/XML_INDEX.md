# XML syntax index and inline character data

Selected-TOC input now exposes `owner_analysis.load_plan.xml_documents`, keyed by
logical XML path. Commands/configuration are unchanged. Each unique captured XML
file has one index; repeated include occurrences stay in the load records. The
index is created in the same quick-xml token stream used for file acquisition.

## Declarations and references

Every element retains a source-occurrence ID, qualified name, containing XML node,
full/start/end tag spans, and ordered attribute observations. Spans are zero-based
half-open UTF-8 byte offsets plus one-based line and byte-column coordinates.
Attribute values are unescaped by quick-xml; unknown values are accessible through
the owner API but JSON contains their digest and raw/value spans, not their body.

Unprefixed elements in the admitted Ui namespace project explicit `name`, `virtual`,
`intrinsic`, `parent`, `parentKey`, `parentArray`, ordered `inherits`, and `mixin`
declarations. These are syntactic observations, not target-XSD-validated object
kinds or accepted graph facts. XML containment and the `parent` name reference
are distinct. Anonymous elements remain in the structural index. Duplicate names
never merge; parent/template/mixin references are not resolved by name guessing.

Top-level `Script` and direct `Scripts` children retain owner, element/handler name,
`file`, `function`, `method`, `inherit`, and `intrinsicOrder` spellings. Their source
is classified as external file, reference-only, inline body, or unresolved. Mixed
body/reference forms, multiple reference forms, and nested markup inside a script
are unresolved rather than silently converted into executable source.

## Inline Lua extraction

`XmlScriptRecord.inline_lua` contains the extracted digest/length, source-scoped
unit ID, and source-map segments. `XmlInlineLua::text()` provides the retained
character data to owner consumers; source bodies are not serialized to CLI JSON.
Text and CDATA contribute literal content with XML 1.0 CR/CRLF-to-LF normalization;
predefined/numeric entities contribute decoded characters without re-normalizing
character references. Comments and CDATA delimiters are excluded. DTDs, custom
entities and processing instructions remain rejected. No Lua parser or executor
is introduced.

`map_range(start, end)` maps a nonempty, UTF-8-boundary extracted range to exact
XML source pieces. Literal pieces map byte-for-byte, normalized newlines and
entities map to their complete original spellings. Gaps remain gaps, not a widened
single range. Empty/caret ranges reject rather than selecting a guessed neighbor.

This is **extraction**, not analyzer integration. Bodies are not registered as
physical Main files, no callback wrapper/parameters are invented, and no inline
analyzer diagnostics are claimed. A versioned virtual-unit/source-map adapter is
still required. Existing incomplete-load blockers remain; syntax indexing does not
make object/lifecycle/inheritance semantics complete or certify runtime loading.

XML character-data normalization follows the W3C XML 1.0 specification,
sections 2.11 and 4.1 (`https://www.w3.org/TR/xml/`). Raw attribute values are
unescaped observations; full DTD/schema attribute normalization is not claimed.

## Identity, bounds and output

Load profile/domain advance to v4. Each XML index has its own versioned digest;
the full index joins the load-plan identity and hence the project generation.
Node IDs identify exact document/source occurrences, not semantic object lineage
across revisions. Host paths, clock values and include traversal ordinal are not
inputs to a node identity. Original TOC/XML bytes remain available through the
retained plan, without reopening files.

The captured closure is additionally limited to 32,768 indexed XML nodes, 65,536
attributes and 65,536 inline map segments. Existing 1 MiB/document, 16 MiB/closure,
1,024-file, depth/attribute/parse-work, cancellation and output limits remain.
Literal extraction is bounded by the source document; no unbounded DOM is built.

Text `check` output adds document/node/declaration/script/extracted-body counts
and each index digest. Status exposes `project.xml.syntax.indexed`; pending inline
analysis remains partial. Plain external-file XML without inline bodies does not
become degraded merely because the syntax index exists.

Implementation sources: `crates/wow-project/src/load/xml.rs` and `xml_index/`.
Normative route: `crates/wow-project/e2/XML_MODEL.md`. Syntax examples reviewed
2026-09-23 from Gethe `live` resolved to
`09b9db7948abc9b9648dedaab51eb0cf3ee67b31`: `Shared/Frame/EventFrame.xml` and
`Shared/FrameTemplate/RingedFrameTemplate.xml` under
`Interface/AddOns/Blizzard_SharedXML/`. No donor source or fixed client inventory
is embedded. The existing lockfile retains quick-xml 0.38.4; no dependency changes.
