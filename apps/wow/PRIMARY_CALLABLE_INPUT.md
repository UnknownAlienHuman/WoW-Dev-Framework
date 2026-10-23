# Ordinary Primary callable files input

**Status:** partial executable #122 admission contract. The focused native
publication/reopen/context and update/removal lifecycle passes for the three
independent selections. This is an ordinary physical user-project route.

## Manifest and independent selections

`wow-service/local-primary-callables-files/1` uses the existing
[files manifest layout](FILES_INPUT.md), with two independent top-level recipe
fields. At least one must be present and non-null; omit the other when unselected.
Declaration-only, signature-only and combined selection share this one family.

| Field | Contract |
|---|---|
| `schema` | `wow-service/local-primary-callables-files/1` |
| `project_id`, `workspace_id`, `source_origin_id`, `logical_root` | Existing exact logical identities; logical_root is not a host directory |
| `profile` | Pinned complete ProfileIdentity artifact; ordinary Fixture or Repository profile |
| `reference_view` | Pinned exact validated ReferenceView artifact |
| `analyzer` | Existing declaration with pinned compatibility_report, accepted_pin_id, configuration_digest, contract_id, fixture_contract_id and library_contract_id |
| `main` | Explicit root and nonempty files inventory of physical Main Lua |
| `library` | Independent root for bounded Lua discovery; optional nonempty `files` selects a subset |
| `physical_function_declarations` | Optional `wow-emmy/physical-function-declaration-syntax/1` (DeclaredSyntaxV1) |
| `physical_function_signatures` | Optional `wow-emmy/physical-function-indexed-types/1` (IndexedTypesV1) |
| `hook_target_recipe` | Optional `hook_target_literals_v1`, with an explicit `function_call_budget` |
| `function_call_budget` | Existing bounded native FunctionCallBudget; admitted here only with the hook recipe |

Pinned metadata references retain `path`, `content_digest` and `byte_length`.
Source file entries retain the existing path and optional digest/length pair;
unpinned entries capture exact read bytes, while pinned entries refuse drift.
All paths resolve relative to the registered config directory under the existing
bounded no-follow file reader. Main and Library are genuine separate native
roles; Library type information does not turn Library declarations into Main.
Library discovery is confined to its configured root; no Lua execution or second
parser is introduced.

Unknown/duplicate fields, explicit null recipe fields, unknown recipe strings,
body-shape declaration selection and an empty selection refuse. This family
requires ordinary Fixture/Repository physical input without selected TOC/load
context, package plans, platform graph/XML classification or candidate selection.
It cannot relabel those richer inputs as ordinary Primary. Old inline/files
schemas retain their unselected contract; use the original schema when neither
recipe is wanted rather than an empty selected envelope.

## Native observations and context

The optional hook recipe resolves arbitrary literal global-name and exact Main
table/member targets in the existing Main/Library analyzer session. Select
DeclaredSyntaxV1 to retain exact global callback declarations in the graph.
Dynamic, shadowed, ambiguous and unsupported targets or callbacks remain
conservative. It does not establish runtime hook execution or safety. Omit both
new fields to preserve the existing query, configuration and replay identities;
explicit null or unknown selections refuse.

Selections bind ProjectConfiguration and its derived identity, then the genuine
function-call report built in the same native Main/Library session. DeclaredSyntaxV1
retains declaration form, colon flag and ordered declared name/vararg tokens with
exact source support. It does not select body shape or inferred types.
IndexedTypesV1 retains conditional analyzer-indexed parameter/return observations,
backend flags and uncertainty. Supported ordered typed fragments and resolved
empty returns survive; missing/unresolved/unsupported types remain explicit,
never synthesized Any or complete signatures.

A selected recipe does not promise an Observed/Resolved row for every callable.
Unavailable/Unsupported/Partial observations keep their original reason/support.
In native context facets, an omitted recipe remains NotSelected rather than an
empty positive observation. Context inspection/build remains Partial or Truncated
with absence_authoritative=false; no body/excerpt, runtime, load-completeness or
semantic type-equivalence claim follows.

## Existing publication and acquisition route

Use existing [project publish/read](LIVE_PROJECT.md) grammar with this config:

```text
wow project publish --config <primary-files.json> --project <ProjectId> --store-root <new-private-directory> --operation-id <id> --expected-current absent --initialize --allow-partial
wow project read --store-root <directory> --store-generation current
wow project read --store-root <directory> --store-generation <StoreGenerationId>
```

Ordinary create/publication/reopen selects the exact
[Primary callable replay/catalog](../../crates/wow-project/PRIMARY_CALLABLE_REPLAY.md)
for the retained configuration. No platform --selection document or new callable
CLI flag is introduced. [context inspect](CONTEXT_INSPECT.md) and
[context build](CONTEXT_BUILD.md) retain their existing explicit host, store,
publication, exact root, level and profile grammar over the acquired native pair.
Their host Reference/profile must match; neither command selects analyzer recipes.

With the hook recipe, publication selects the existing selected-components
replay/catalog instead of the unselected Primary callable catalog. The retained
recipe and native source bytes reconstruct the original Project/Graph pair after
the source files are unavailable. This is an explicit component selection, with
no migration or widening of an existing store catalog.

The config-loading publish/update operation binds its frontend graph request to
the admitted configuration, then preflights it, as the existing selected route
does. Direct input/store operations still require exact request recipe matching.

Existing source, analyzer, replay, store and output budgets/cancellation remain in
force. Library acquisition uses the shared ceilings in [FILES_INPUT.md](FILES_INPUT.md);
there is no automatic fallback after acquisition or analysis fails.
Full E3/Tier A/X and client/runtime qualification remain open under
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122).
