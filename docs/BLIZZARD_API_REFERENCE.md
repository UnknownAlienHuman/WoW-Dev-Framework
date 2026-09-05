# Generated Blizzard API reference producer

The generated API producer converts Blizzard's declarative API documentation into a deterministic, normalized reference draft. It is update-oriented: no client build, Interface value, source commit, or moving branch head is compiled into the tool.

## Inputs

The producer requires a local Git checkout, a verified source manifest created for one exact revision, and the generated documentation records listed by that manifest. Local source is preferred. A moving selector is resolved before manifest creation; its exact revision is retained as evidence for that operation only. A later update creates another manifest and another draft.

## Non-executing parser

`scripts/wow_api_reference.py` implements a bounded parser for the declarative Lua-table subset used by `Blizzard_APIDocumentationGenerated`. It supports keyed and array tables, quoted and long-bracket strings, numbers, booleans, `nil`, symbol references, opaque expression preservation, comments, escapes, and source spans.

It rejects trailing statements, mismatched documentation-table registration, malformed or unbounded input, unsafe paths, oversized files, digest mismatch, and Git-object mismatch. It never starts Lua or executes repository hooks, scripts, submodules, package managers, or generated code.

## Coverage and authority

The coverage scope is:

```text
Interface/AddOns/Blizzard_APIDocumentationGenerated/*Documentation.lua
```

Candidate, parsed, and failed files are explicit. Negative authority is enabled only when every candidate file matched the source manifest and parsed successfully.

`--allow-partial` is investigative. A partial draft always has `negative_authority = false`. Generated docs do not prove runtime behavior; implementation, XML, TOC, data/hotfix state, and exact client probes remain separate evidence lanes.

## Build and verify

```bash
python scripts/build-blizzard-api-reference.py \
  --source "$WOW_UI_SOURCE_DIR" \
  --manifest .wow-dev/source-manifest.json \
  --output .wow-dev/api-reference.json \
  --json

python scripts/verify-blizzard-api-reference.py \
  .wow-dev/api-reference.json \
  --source "$WOW_UI_SOURCE_DIR" \
  --manifest .wow-dev/source-manifest.json \
  --current-ref origin/live \
  --require-complete \
  --json
```

Exit codes:

- `0`: valid and, when requested, current;
- `2`: invalid, inconsistent, unreproducible, or incomplete when completeness is required;
- `3`: internally valid but the checked moving ref advanced.

Staleness preserves historical evidence and signals creation of a new generation. It does not permit mixing files from the old and new revisions.

The output retains producer/parser versions, source-manifest digest, selector, reported version, exact revision, per-file Git object and SHA-256, source line spans, normalized systems and members, declared restrictions, coverage, conflicts, limitations, and a canonical self-digest.

The executable `wow-reference::generated_api` import validates this producer-owned draft and exposes source-bound facts, coverage and conflicts. `wow-reference-source` binds API and topology products from the same source. Further typed owner integration and the actual EmmyLua analyzer adapter remain subsequent work; see [implementation status](IMPLEMENTATION_STATUS.md).

The producer accepts named systems and the unnamed table-only groups handled by Blizzard `APIDocumentationMixin:AddDocumentationTable`. For an unnamed group, `name` is only the registered local declaration binding: `attributes.name_origin = "declaration_binding"`. It is not a runtime system or namespace. Individual member names and source spans remain unchanged.
