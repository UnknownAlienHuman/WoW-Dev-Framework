# `wow-context`

`wow-context` turns selected Project, Blizzard UI, Graph, analyzer, and Reference
views into bounded maps, inspections, packs, and readable output. It consumes
owner-produced facts; it does not parse source, own storage/search, infer facts
with a model, or mutate a project.

## Current route

The implemented CLI supports selected context map/inspect/build/render paths.
Start with [`CONTEXT_MAP.md`](../../apps/wow/CONTEXT_MAP.md),
[`CONTEXT_INSPECT.md`](../../apps/wow/CONTEXT_INSPECT.md), and
[`CONTEXT_BUILD.md`](../../apps/wow/CONTEXT_BUILD.md), then follow the matching
public operation in `src/` and its Service caller. Root selection is explicit;
source excerpts retain the owner's physical range and remain untrusted source
text. `wow context --help` lists current forms.

Keep context output small, located, and honest about missing or partial inputs.
Preserve the selected root, source origin, and budget. Rendering is presentation,
not semantic authority. Do not add another parser, search engine, model/provider
lane, evidence framework, or context persistence layer.

Some existing context facets and protocol handoffs remain excess pending queue
reduction. They are not a release, compatibility, corpus, or acceptance campaign;
keep only facets a current developer workflow uses.
