# Selected TOC input

`wow status` and `wow check --config project.json` accept
`wow-service/local-project-toc/1`. Start with the metadata and Library declarations
from [FILES_INPUT.md](FILES_INPUT.md), change `schema`, and replace `main.files`:

```json
{
  "root": "addon",
  "toc": { "path": "MyAddon.toc" }
}
```

This fragment is the `main` object, not a complete configuration. `profile`,
`reference_view`, `analyzer`, `library`, and project identities remain required.
`main.toc` optionally accepts the same digest/length pair as other source files.
The old inline and explicit-file schemas are unchanged. No config may combine a
TOC with a manual Main inventory. Library remains explicit and separate.

## Implemented load projection

`wow-project::load` reads only the chosen TOC and its reachable files below the
explicit Main root. It captures exact bytes once per physical logical path;
repeated references reuse those bytes. Nothing is scanned or executed.

TOC processing preserves line spans, comments, unknown directives and encounter
order. UTF-8 BOM, LF/CRLF, slash/backslash source separators and `[Bootstrap]`
are supported. A base `Interface` directive must contain the selected profile's
Interface; mismatch rejects. Missing/duplicate directives remain partial.
Conditional file and metadata records can use an explicit `main.load_context`;
see [TOC_CONTEXT.md](TOC_CONTEXT.md). Unknown conditions/directives remain blockers.
Exactly the named variant is used; no highest-Interface or filename fallback.

Streaming XML processing expands unprefixed `Ui` → `Include file="...xml"` and
`Ui` → `Script file="...lua"` in document order, including nested includes.
The standard Blizzard default namespace and unnamespaced form are admitted.
References resolve relative to their declaring document. Source `.` and `..`
segments are reduced within the declared Main root; attempting to pop above it
rejects. Configuration paths themselves still require canonical relative spelling.
No absolute/drive/URI/device path or source placeholder becomes a host open.

Lua files are registered once with the existing analyzer; every load occurrence
and bootstrap context is retained. Duplicate loads, cycles, missing files,
unsupported namespaces/attributes/elements, and inline Lua produce explicit load
issues. Required dependencies remain unresolved blockers; optional dependencies
are recorded without forcing a required failure. Dependencies are never fetched.
There is no claim that repeated loads, XML callbacks or runtime initialization
have been analyzed by registering a physical Lua file once.

## Identity and output

The `ProjectLoadPlan` contains source digests/lengths, ordered source-mapped
records, blockers and a versioned `wow-project/toc-xml-files/2` digest. Original
TOC/XML text is retained privately for exact span lookup, not serialized.
Configuration and generation bind the plan digest, complete target profile and
explicit load context. TOC/XML/order/selection changes therefore
invalidate the project identity even if all Lua bytes remain unchanged. Project
inventory construction and update generation derivation reject a stale plan whose
Lua receipts no longer match; recapture a plan after editing its sources.

JSON `owner_analysis.load_plan` exposes the receipt without source bodies or host
roots. Text output prints its summary and issues. `status` exposes the same plan
identity and the narrow `project.load_files.resolved` capability without running
analysis. A blocking issue degrades that component and the overall check result;
partial acquisition cannot become `clean`. Existing `project.toc.complete`,
`project.xml.complete` and full load-graph capabilities remain deferred.

## Resource and security policy

The existing no-follow, regular-file, bounded, cancellable reader is reused.
The complete captured closure is limited to 1,024 files, 1 MiB per file, 16 MiB
aggregate; load records to 32,768; XML include depth to 32 and element depth to 64;
attributes to 64 per element/16 KiB per value; TOC lines to 16 KiB. Cumulative XML/
TOC parse work, including repeated includes, is capped at 64 MiB. Resource exhaustion
rejects rather than publishing silently truncated input. XML DTDs, custom entities,
processing instructions and non-UTF-8 declarations reject. Namespace/schema URLs
are inert data and are never fetched. No Lua/XML handler executes.

This is the E2-C **external-file acquisition slice**, not full E2-C indexing:
inline Lua source maps, object/template/inheritance modeling, remaining conditional
dialects and package-level load gates, dependency-package resolution, recognizer/graph integration and durable E2-D
publication remain separate. Existing fixture-only rule policy is unchanged.
Build verification does not certify real-addon behavior or client load success.

Source-format review: Gethe `live` resolved on 2026-09-23 to
`09b9db7948abc9b9648dedaab51eb0cf3ee67b31`; inspected
`Interface/AddOns/Blizzard_SharedXML/Blizzard_SharedXML.toc`, `VectorLib.xml` and
`UI.xsd`. This identifies reviewed evidence, not a permanent client/build target.
`quick-xml` 0.38 supplies the streaming tokenizer; Cargo.lock records the resolved
version. No parser dependency or patch version is asserted to be permanent truth.
