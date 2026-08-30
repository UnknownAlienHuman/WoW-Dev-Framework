# `wow-annotations` implementation contract

**Status:** deferred to E1; contract scaffold only.

## Mission

`wow-annotations` projects canonical `wow-reference` facts into deterministic Ketho-compatible LuaCATS/EmmyLua annotation artifacts and WoW dialect configuration. It is a projection layer, never the canonical store for raw Blizzard metadata or restrictions.

## Owned responsibilities

- deterministic annotation directory/file layout;
- type/signature/namespace/widget/event/enum/CVar/template projections;
- WoW dialect globals and standard-library adjustments;
- nominal Secret analysis types where the selected profile supports the projection;
- source-map links from generated declarations back to canonical facts;
- semantic parity normalization/comparison with Ketho-compatible output;
- annotation syntax and analyzer-compatibility validation;
- projection warnings and loss report;
- artifact digests and stable ordering.

## Explicit non-responsibilities

`wow-annotations` does not:

- parse raw APIDocumentation as authority;
- apply hidden corrections;
- mutate VS Code, LuaLS, Emmy, or workspace settings;
- inject the full Blizzard UI implementation into an editor library;
- decide runtime Secret access;
- replace raw restriction facets with nominal types;
- execute Ketho, Lua, PHP, or editor extensions during normal consumption;
- own Reference Pack manifests or storage.

## Inputs and outputs

Input:

```text
one exact ReferenceView/generation
normalized API/UI facts
raw facet references and projection policy
WoW dialect profile facts
projection schema/version
```

Output:

```text
Annotations/Core/...
WoW dialect configuration projection
annotation source map
projection loss report
semantic parity report
artifact checksums
```

## Required operations

| Operation | Required behavior |
|---|---|
| `plan_annotation_projection` | Determine files/symbol groups and report unsupported fields before writing. |
| `project_api_symbols` | Emit canonical namespaces, functions, methods, params, returns, docs, and source links. |
| `project_data_catalogs` | Emit enums, events, CVars, types, script objects, widgets, and selected FrameXML declarations. |
| `project_wow_dialect` | Emit profile-scoped globals/removed globals/require-like behavior without editor mutation. |
| `project_restriction_types` | Emit only approved nominal/union analysis types while retaining canonical facet references. |
| `render_annotation_tree` | Produce deterministic paths/content/order/line endings. |
| `validate_annotation_syntax` | Parse generated artifacts and report exact generated/source handles. |
| `probe_emmy_compatibility` | Verify expected inference/diagnostic behavior against the pinned analyzer. |
| `compare_ketho_semantics` | Compare canonical symbol/type/signature meaning, not byte identity. |
| `build_projection_loss_report` | List raw fields/facets that could not be represented and affected capabilities. |
| `digest_annotation_artifacts` | Produce deterministic checksums excluding volatile metadata. |

## Projection rules

1. The source `ReferenceView` generation is mandatory and embedded in artifact metadata.
2. Raw unknown fields are never discarded by pretending the annotation format represents them.
3. Every lossy conversion appears in the loss report.
4. Equivalent logical inputs produce byte-identical text after canonicalization.
5. Annotation file partitioning is stable and does not depend on hash iteration or worker count.
6. Generated names cannot collide silently; conflicts are qualified or diagnosed.
7. Documentation text is untrusted content and cannot alter generator behavior.
8. Nominal Secret types are static analysis projections, not runtime wrapper claims.
9. No generated artifact writes outside the configured output root.
10. The generator never edits user-owned configuration files.

## Type-lowering decisions that must be explicit

The implementation brief for each lowering rule must state:

- source raw/normalized type;
- output annotation syntax;
- nilability/union behavior;
- generic/table/tuple handling;
- receiver/self semantics;
- overload behavior;
- unknown type fallback;
- restriction-facet projection;
- parity expectation;
- loss/coverage effect.

Do not scatter type-lowering decisions across string templates.

## E1 implementation sequence

1. Minimal deterministic renderer for the E0 fixture model.
2. Core scalar/table/function lowering.
3. systems/namespaces and events/enums/CVars.
4. widgets/script objects/types.
5. dialect profile.
6. restriction projections and loss report.
7. source maps and checksums.
8. Emmy compatibility probe.
9. Ketho semantic parity corpus.

## Required tests

- stable file partition/order;
- scalar, optional, union, tuple, table, callback, method receiver, overload, enum, event payload;
- unknown field/type loss reporting;
- collision handling;
- Secret nominal/union projection without raw-facet loss;
- source-map resolution;
- analyzer parse and inference probe;
- Ketho semantic parity fixtures;
- no editor settings changed;
- output-root traversal rejection;
- repeated build byte equality.

## Documentation sources

- [`../../docs/REFERENCE_PACK.md`](../../docs/REFERENCE_PACK.md)
- [`../../docs/EMMYLUA_AND_DIAGNOSTICS.md`](../../docs/EMMYLUA_AND_DIAGNOSTICS.md)
- [`../../docs/SECRET_VALUES_AND_RESTRICTIONS.md`](../../docs/SECRET_VALUES_AND_RESTRICTIONS.md)
- [`../../docs/RESEARCH_BASELINE.md`](../../docs/RESEARCH_BASELINE.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)
- [Ketho compatibility source route](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/external/External_Repositories.md)

## Definition of done

E1 projection is complete when the pinned Emmy analyzer consumes the generated tree, semantic parity is measured, every unsupported raw field is visible in a loss/coverage report, and repeated generation is byte-identical without mutating editor state.
