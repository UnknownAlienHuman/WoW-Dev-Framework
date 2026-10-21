from pathlib import Path
import re


def read(path: str) -> tuple[Path, str]:
    target = Path(path)
    return target, target.read_text()


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one match, found {count}")
    return text.replace(old, new, 1)


def replace_count(text: str, old: str, new: str, expected: int, label: str) -> str:
    count = text.count(old)
    if count != expected:
        raise SystemExit(f"{label}: expected {expected} matches, found {count}")
    return text.replace(old, new)


def sub_once(text: str, pattern: str, replacement: str, label: str) -> str:
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.DOTALL)
    if count != 1:
        raise SystemExit(f"{label}: expected one regex match, found {count}")
    return updated


# One current ordinary Project catalog. Candidate receipt custody remains a
# separate physical catalog assembled by Service.
path, text = read("crates/wow-project/src/replay/publication.rs")
text = sub_once(
    text,
    r"pub const STORAGE_SCHEMAS:.*?\nconst HEADER_KEY:",
    '''/// Every current Project replay payload is admitted by one rebuildable catalog.
/// Exact feature selection remains in ProjectConfiguration and replay validation.
pub const STORAGE_SCHEMAS: &[&str] = &[
    "wow-project.live-replay.v4",
    "wow-project.live-replay.v5",
    "wow-project.live-replay.v6",
    "wow-project.live-replay.v7",
    "wow-project.live-replay.v8",
    "wow-project.live-replay.v9",
    "wow-project.live-replay.v10",
    "wow-project.live-replay.v11",
    "wow-project.live-replay.v12",
    "wow-project.live-replay.v14",
    "wow-project.live-replay.v15",
    "wow-project.live-replay.v16",
    super::ACEADDON_LOCAL_RESULT_STORAGE_SCHEMA,
    super::XML_CLASSIFICATION_STORAGE_SCHEMA,
    super::PHYSICAL_FUNCTION_DECLARATIONS_STORAGE_SCHEMA,
    super::XML_CLASSIFICATION_PHYSICAL_FUNCTION_DECLARATIONS_STORAGE_SCHEMA,
    super::PHYSICAL_FUNCTION_BODY_SHAPE_STORAGE_SCHEMA,
    super::XML_CLASSIFICATION_PHYSICAL_FUNCTION_BODY_SHAPE_STORAGE_SCHEMA,
    super::SELECTED_COMPONENTS_STORAGE_SCHEMA,
    super::PRIMARY_CALLABLES_STORAGE_SCHEMA,
    super::NATIVE_LIBRARY_INPUT_STORAGE_SCHEMA,
    super::NATIVE_API_USES_STORAGE_SCHEMA,
    "wow-project.live-pair.v1",
];
pub const STORAGE_CHECK: &str = "wow-project.live-pair-native-replay.v1";
const HEADER_KEY:''',
    "Project catalog block",
)
text = replace_once(
    text,
    "    replay_storage_schema: &'static str,\n",
    "",
    "bundle catalog-dispatch field",
)
text = sub_once(
    text,
    r"\n    /// Exact schema already selected and validated for the retained replay record\.\n"
    r"    /// Catalog dispatch borrows this native handoff without decoding record bytes\.\n"
    r"    #\[must_use\]\n"
    r"    pub const fn replay_storage_schema\(&self\) -> &'static str \{\n"
    r"        self\.replay_storage_schema\n"
    r"    \}\n",
    "\n",
    "bundle catalog-dispatch accessor",
)
text = replace_once(
    text,
    "    let replay_storage_schema = replay.storage_schema()?;\n",
    "    let storage_schema = replay.storage_schema()?;\n",
    "publication storage schema local",
)
text = replace_once(
    text,
    "        PartitionRecord::new(REPLAY_KEY, replay_storage_schema, replay)\n",
    "        PartitionRecord::new(REPLAY_KEY, storage_schema, replay)\n",
    "publication replay record",
)
text = replace_once(
    text,
    "        replay_storage_schema,\n",
    "",
    "publication bundle construction",
)
path.write_text(text)


# Candidate catalog = unified Project catalog + one candidate receipt/check.
path, text = read("crates/wow-service/src/live_project/candidate.rs")
text = replace_once(
    text,
    "pub(super) fn catalog(project_schemas: &[&'static str]) -> ServiceResult<RecordCatalog> {\n"
    "    let mut schemas = GraphPartitionSnapshot::STORAGE_SCHEMAS.to_vec();\n"
    "    schemas.extend_from_slice(project_schemas);\n"
    "    schemas.push(CANDIDATE_STORAGE_SCHEMA);\n",
    "pub(super) fn catalog() -> ServiceResult<RecordCatalog> {\n"
    "    let mut schemas = GraphPartitionSnapshot::STORAGE_SCHEMAS.to_vec();\n"
    "    schemas.extend_from_slice(publication::STORAGE_SCHEMAS);\n"
    "    schemas.push(CANDIDATE_STORAGE_SCHEMA);\n",
    "candidate catalog input",
)
text = replace_once(
    text,
    "/// Epoch selection is exact. No optional receipt fallback and no old widening.\n"
    "pub(crate) fn is_selected(actual: &RecordCatalog) -> ServiceResult<bool> {\n"
    "    for &schemas in publication::STORAGE_SCHEMA_CATALOGS {\n"
    "        if actual == &catalog(schemas)? {\n"
    "            return Ok(true);\n"
    "        }\n"
    "        if actual == &catalog_for(schemas)? {\n"
    "            return Ok(false);\n"
    "        }\n"
    "    }\n"
    "    Err(fail(ServiceErrorCode::IdentityMismatch))\n"
    "}\n",
    "/// Epoch selection distinguishes only the optional candidate receipt.\n"
    "pub(crate) fn is_selected(actual: &RecordCatalog) -> ServiceResult<bool> {\n"
    "    if actual == &catalog()? {\n"
    "        Ok(true)\n"
    "    } else if actual == &catalog_for(publication::STORAGE_SCHEMAS)? {\n"
    "        Ok(false)\n"
    "    } else {\n"
    "        Err(fail(ServiceErrorCode::IdentityMismatch))\n"
    "    }\n"
    "}\n",
    "candidate catalog selection",
)
path.write_text(text)


# Service opens one ordinary catalog or the one candidate-receipt variant.
path, text = read("crates/wow-service/src/live_project.rs")
text = replace_once(
    text,
    "use wow_emmy::function_calls::{\n"
    "    PhysicalFunctionDeclarationRecipe, PhysicalFunctionSignatureRecipe,\n"
    "};\n",
    "",
    "obsolete catalog recipe import",
)
text = sub_once(
    text,
    r"    // This ordinary recipe family never selects a platform or candidate namespace\..*?\n"
    r"    pub fn current\(&self\)",
    '''    pub fn open(root: &Path) -> ServiceResult<Self> {
        Self::open_selected(root, None)
    }
    /// Retained Service consumers share the same current catalog admission.
    pub(crate) fn open_retained_owner_store(root: &Path) -> ServiceResult<ProjectStore> {
        Self::open(root).map(|owner| owner.store)
    }
    fn open_selected(root: &Path, candidate_catalog: Option<bool>) -> ServiceResult<Self> {
        match candidate_catalog {
            Some(true) => ProjectStore::open(root, &candidate::catalog()?)
                .map(|store| Self { store })
                .map_err(store_error),
            Some(false) => ProjectStore::open(root, &catalog()?)
                .map(|store| Self { store })
                .map_err(store_error),
            None => match ProjectStore::open(root, &catalog()?) {
                Ok(store) => Ok(Self { store }),
                Err(error) if error.code() == StoreErrorCode::IntegrityViolation => {
                    ProjectStore::open(root, &candidate::catalog()?)
                        .map(|store| Self { store })
                        .map_err(store_error)
                }
                Err(error) => Err(store_error(error)),
            },
        }
    }
    pub fn current(&self)''',
    "live Project open routes",
)
text = sub_once(
    text,
    r"// The selected-component family cannot attest XML/declaration choices itself\..*?\nfn catalog_for",
    "fn catalog_for",
    "catalog feature-combination helpers",
)
path.write_text(text)


# Namespace selection still validates exact reconstructed configuration, but no
# longer chooses a physical catalog for every feature combination.
path, text = read("crates/wow-service/src/live_project/namespace.rs")
text = replace_once(
    text,
    "    LiveProjectUpdateRequest, ReadFailureClosure, catalog_accepts_components, catalog_for, fail,\n"
    "    store_error,\n",
    "    LiveProjectUpdateRequest, ReadFailureClosure, catalog_for, fail, store_error,\n",
    "namespace imports",
)
text = sub_once(
    text,
    r"    fn selected_catalog\(&self, schemas: &\[&'static str\]\) -> ServiceResult<RecordCatalog> \{.*?\n"
    r"    fn require_catalog",
    '''    fn selected_catalog(&self) -> ServiceResult<RecordCatalog> {
        if self.composed_candidate_catalog {
            super::candidate::catalog()
        } else {
            catalog_for(publication::STORAGE_SCHEMAS)
        }
    }

    fn catalog(&self) -> ServiceResult<RecordCatalog> {
        self.selected_catalog()
    }

    fn require_catalog''',
    "namespace catalog selection",
)
text = sub_once(
    text,
    r"    fn require_catalog\(&self, actual: &RecordCatalog\) -> ServiceResult<\(\)> \{.*?\n"
    r"    \}\n\n    /// Immutable configuration getters",
    '''    fn require_catalog(&self, actual: &RecordCatalog) -> ServiceResult<()> {
        if &self.selected_catalog()? == actual {
            Ok(())
        } else {
            Err(fail(ServiceErrorCode::IdentityMismatch))
        }
    }

    /// Immutable configuration getters''',
    "namespace catalog validation",
)
text = sub_once(
    text,
    r"        let store = Self::open_selected\(\n"
    r"            root,\n"
    r"            Some\(\(.*?\)\),\n"
    r"            Some\(selection\.composed_candidate_catalog\),\n"
    r"        \)\n",
    "        let store = Self::open_selected(root, Some(selection.composed_candidate_catalog))\n",
    "namespace open route",
)
path.write_text(text)


# Ordinary publication creation no longer dispatches on the replay record schema.
path, text = read("crates/wow-service/src/live_project/operations.rs")
text = replace_once(
    text,
    "use wow_project::replay::publication;\n",
    "",
    "operation catalog import",
)
text = sub_once(
    text,
    r"    let mut store = if request\.initialize \{.*?\n"
    r"    \} else \{\n"
    r"        LiveProjectStore::open\(root\)\?\n"
    r"    \};\n"
    r"    if store\.store",
    '''    let mut store = if request.initialize {
        LiveProjectStore::create(root, &owner)?
    } else {
        LiveProjectStore::open(root)?
    };
    if store.store''',
    "ordinary publication catalog dispatch",
)
path.write_text(text)


# Tests retain semantic replay/configuration checks, not catalog-family checks.
path, text = read("crates/wow-service/src/local/platform_input/tests/event_registry_cross_registry.rs")
text = replace_once(
    text,
    "    let project_schemas = publication::STORAGE_SCHEMA_CATALOGS\n"
    "        .iter()\n"
    "        .find(|schemas| schemas.contains(&bundle.replay_storage_schema()))\n"
    "        .ok_or(\"registered captured schema catalog\")?;\n"
    "    let mut schemas = GraphPartitionSnapshot::STORAGE_SCHEMAS.to_vec();\n"
    "    schemas.extend_from_slice(project_schemas);\n",
    "    let mut schemas = GraphPartitionSnapshot::STORAGE_SCHEMAS.to_vec();\n"
    "    schemas.extend_from_slice(publication::STORAGE_SCHEMAS);\n",
    "event registry unified catalog",
)
path.write_text(text)

path, text = read("crates/wow-service/src/local/platform_input/tests/w5_hook_targets.rs")
text = replace_once(
    text,
    "    // The explicitly registered selected-components catalog is the real\n"
    "    // AcquiredProjectPair consumer; no expected-hash reader search is involved.\n"
    "    let bundle = ProjectPublicationBundle::build(&publisher, &snapshot, &stop)?;\n"
    "    assert_eq!(\n"
    "        bundle.replay_storage_schema(),\n"
    "        publication::STORAGE_SCHEMAS_SELECTED_COMPONENTS_V1[0]\n"
    "    );\n"
    "    let mut schemas = GraphPartitionSnapshot::STORAGE_SCHEMAS.to_vec();\n"
    "    schemas.extend_from_slice(publication::STORAGE_SCHEMAS_SELECTED_COMPONENTS_V1);\n",
    "    // The current catalog admits the selected replay record; reconstructed\n"
    "    // configuration still proves the exact hook recipe on readback.\n"
    "    let bundle = ProjectPublicationBundle::build(&publisher, &snapshot, &stop)?;\n"
    "    let mut schemas = GraphPartitionSnapshot::STORAGE_SCHEMAS.to_vec();\n"
    "    schemas.extend_from_slice(publication::STORAGE_SCHEMAS);\n",
    "hook target unified catalog",
)
path.write_text(text)

path, text = read("crates/wow-project/tests/primary_callable_replay.rs")
text = replace_once(
    text,
    "    OperationId, StoreErrorCode,\n",
    "    OperationId,\n",
    "primary replay StoreErrorCode import",
)
text = sub_once(
    text,
    r"    assert!\(!publication::STORAGE_SCHEMAS\.contains\(&PRIMARY_STORAGE\)\);.*?\n"
    r"    selected\.close\(\)\?;",
    '''    assert!(publication::STORAGE_SCHEMAS.contains(&PRIMARY_STORAGE));
    let root = StoreRoot::new()?;
    let store = ProjectStore::create(
        root.0.join("store"),
        "fixture.primary-callables",
        catalog(publication::STORAGE_SCHEMAS)?,
    )?;
    PublicationRequest::new(
        store.epoch(),
        OperationId::new("primary-selected")?,
        None,
        bindings,
        records,
    )?;
    store.close()?;''',
    "primary replay catalog compatibility block",
)
path.write_text(text)

path, text = read("crates/wow-service/tests/primary_callable_lifecycle.rs")
text = replace_once(
    text,
    "    let schemas = if selected {\n"
    "        publication::STORAGE_SCHEMAS_PRIMARY_CALLABLES_V1\n"
    "    } else {\n"
    "        publication::STORAGE_SCHEMAS\n"
    "    };\n"
    "    let store = ProjectStore::open(root, &catalog(schemas)?)?;\n",
    "    let store = ProjectStore::open(root, &catalog(publication::STORAGE_SCHEMAS)?)?;\n",
    "primary lifecycle unified open",
)
text = sub_once(
    text,
    r"            let rejected = publish_local_project\(.*?\n"
    r"            assert_eq!\(\n"
    r"                publish_local_project\(",
    "            assert_eq!(\n                publish_local_project(",
    "primary lifecycle obsolete catalog refusal",
)
path.write_text(text)

path, text = read("crates/wow-service/src/local/platform_input/tests/xml_classification.rs")
text = replace_once(
    text,
    "    for wrong in [&ordinary, &xml_only, &declaration_only] {\n"
    "        assert_eq!(\n"
    "            LiveProjectStore::open_in_namespace(&store_root, wrong)\n"
    "                .err()\n"
    "                .ok_or(\"different component catalog reopened combined epoch\")?\n"
    "                .code(),\n"
    "            crate::ServiceErrorCode::IdentityMismatch\n"
    "        );\n"
    "    }\n",
    "    for wrong in [&ordinary, &xml_only, &declaration_only] {\n"
    "        let wrong_store = LiveProjectStore::open_in_namespace(&store_root, wrong)?;\n"
    "        let refusal = wrong_store\n"
    "            .read_in_namespace(wrong, &ReadSelector::Current, &stop)\n"
    "            .err()\n"
    "            .ok_or(\"different component selection read combined epoch\")?;\n"
    "        assert_eq!(refusal.code(), crate::ServiceErrorCode::IdentityMismatch);\n"
    "    }\n",
    "combined XML/declaration selection validation",
)
text = replace_once(
    text,
    "    let wrong_catalog = LiveProjectStore::open_in_namespace(&store_root, &default_selection)\n"
    "        .err()\n"
    "        .ok_or(\"ordinary selection reopened classification epoch\")?;\n"
    "    assert_eq!(\n"
    "        wrong_catalog.code(),\n"
    "        crate::ServiceErrorCode::IdentityMismatch\n"
    "    );\n",
    "    let wrong_store = LiveProjectStore::open_in_namespace(&store_root, &default_selection)?;\n"
    "    let wrong_selection = wrong_store\n"
    "        .read_in_namespace(&default_selection, &ReadSelector::Current, &stop)\n"
    "        .err()\n"
    "        .ok_or(\"ordinary selection read classified configuration\")?;\n"
    "    assert_eq!(\n"
    "        wrong_selection.code(),\n"
    "        crate::ServiceErrorCode::IdentityMismatch\n"
    "    );\n",
    "XML classification selection validation",
)
path.write_text(text)

path, text = read("crates/wow-service/src/local/platform_input/tests/function_fingerprints.rs")
text = replace_once(
    text,
    "    assert_eq!(\n"
    "        LiveProjectStore::open_in_namespace(&store_root, &default_selection)\n"
    "            .err()\n"
    "            .ok_or(\"ordinary catalog reopened declaration epoch\")?\n"
    "            .code(),\n"
    "        ServiceErrorCode::IdentityMismatch\n"
    "    );\n",
    "    let wrong_store = LiveProjectStore::open_in_namespace(&store_root, &default_selection)?;\n"
    "    assert_eq!(\n"
    "        wrong_store\n"
    "            .read_in_namespace(&default_selection, &ReadSelector::Current, &stop)\n"
    "            .err()\n"
    "            .ok_or(\"ordinary selection read declaration configuration\")?\n"
    "            .code(),\n"
    "        ServiceErrorCode::IdentityMismatch\n"
    "    );\n",
    "function declaration selection validation",
)
path.write_text(text)


# Current docs describe semantic selection and one physical catalog.
Path("crates/wow-project/PRIMARY_CALLABLE_REPLAY.md").write_text('''# Ordinary Primary callable replay

This route retains physical Main, exact Library files and the selected declaration/
signature recipe, then rebuilds the real Emmy analysis when read. Its replay payload
is `wow-project/native-project-replay/primary-callables/1`; hook/witness selections
use `wow-project/native-project-replay/selected-components/1`.

All current Project replay payloads share `publication::STORAGE_SCHEMAS`. The store
catalog no longer encodes feature combinations. Exact declaration, signature, hook,
XML and platform choices remain in `ProjectConfiguration`; hydration and readback
must reproduce those choices and analyzer/graph identities. A different selection
may open the same physical store but its read is rejected after configuration is
reconstructed.

Current publication performs inactive owner readback before Current activation.
Incompatible derived stores are rebuilt from source and configuration; no legacy
catalog reader, migration or feature-specific epoch is provided. These observations
remain source structure, not runtime guarantees. Broader context work is tracked by
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122).
''')

path, text = read("apps/wow/PLATFORM_INPUT.md")
text = replace_once(
    text,
    "Native namespaced persistence additionally requires\n"
    "`PlatformStoreSelection::with_xml_classification_catalog()` on the exact selection;\n"
    "this chooses the registered classification catalog for create/open/publication.\n"
    "An ordinary epoch is not upgraded implicitly. The selector is catalog intent;\n"
    "publication still validates the genuine source-bound configuration and replay.\n",
    "Native namespaced persistence also records XML classification in the explicit\n"
    "`PlatformStoreSelection`. The physical store uses the one current Project catalog;\n"
    "publication/readback validate the genuine source-bound configuration and replay.\n",
    "platform XML catalog prose",
)
text = replace_once(
    text,
    "The selection binds configuration, report and replay identity. Namespaced\n"
    "persistence requires `PlatformStoreSelection::with_physical_function_declarations_catalog()`,\n"
    "combined with `with_xml_classification_catalog()` when classification is selected.\n"
    "Create, publication and reopen require the exact combination; old epochs are not\n"
    "widened. `--fingerprints` then includes the selected declared-header component;\n",
    "The selection binds configuration, report and replay identity. Namespaced\n"
    "persistence records the same declaration/XML choices in `PlatformStoreSelection`;\n"
    "readback checks them against reconstructed configuration. `--fingerprints` then\n"
    "includes the selected declared-header component;\n",
    "platform declaration catalog prose",
)
text = replace_once(
    text,
    "Configuration and catalog intent must match; retained epochs are not widened.\n",
    "Configuration and explicit selection must match; incompatible derived stores rebuild.\n",
    "platform body-shape selection prose",
)
text = replace_once(
    text,
    "With signatures absent, the replay schemas are\n"
    "`wow-project/native-project-replay/physical-function-declaration-syntax-and-body-shape/1`\n"
    "and, with XML classification,\n"
    "`wow-project/native-project-replay/xml-schema-classification-and-physical-function-declaration-syntax-and-body-shape/1`.\n"
    "Their exact catalogs are `STORAGE_SCHEMAS_PHYSICAL_FUNCTION_BODY_SHAPE_V1` and\n"
    "`STORAGE_SCHEMAS_XML_CLASSIFICATION_PHYSICAL_FUNCTION_BODY_SHAPE_V1`: respectively\n"
    "`wow-project.live-replay.physical-function-declaration-syntax-and-body-shape.v1`\n"
    "and `wow-project.live-replay.xml-schema-classification-and-physical-function-declaration-syntax-and-body-shape.v1`,\n"
    "each paired only with `wow-project.live-pair.v1`. Full corpus/reopen acceptance\n"
    "remains open; these registered catalogs are not acceptance evidence.\n",
    "With signatures absent, replay retains the body-shape payload schema, with a\n"
    "separate XML/body-shape payload tag when classification is selected. Both are\n"
    "members of the one current Project catalog; configuration owns the exact recipe.\n",
    "platform body-shape catalog paragraph",
)
text = replace_once(
    text,
    "Selected signatures use one replay family\n"
    "`wow-project/native-project-replay/selected-components/1` and exact catalog\n"
    "`STORAGE_SCHEMAS_SELECTED_COMPONENTS_V1`: `wow-project.live-replay.selected-components.v1`\n"
    "plus `wow-project.live-pair.v1`. Exact declaration/XML/signature selections remain\n"
    "in configuration; this family does not enumerate their combinations. Namespaced\n"
    "store intent uses `PlatformStoreSelection::with_physical_function_signatures_catalog`\n"
    "with `PhysicalFunctionSignatureRecipe::IndexedTypesV1`, alongside exact selected\n"
    "declaration/XML intent. Configuration/catalog admission must agree. All prior\n"
    "signature-None schemas, catalogs and omitted fields retain their old routes;\n"
    "retained epochs are not widened. Full replay/corpus acceptance remains pending.\n",
    "Selected signatures use `wow-project/native-project-replay/selected-components/1`.\n"
    "Exact declaration/XML/signature choices remain in configuration and the explicit\n"
    "selection; the one current Project catalog does not enumerate their combinations.\n",
    "platform signature catalog paragraph",
)
path.write_text(text)

path, text = read("apps/wow/LIVE_PROJECT.md")
text = replace_once(
    text,
    "Older or incompatible derived stores are not opened or migrated; rebuild\n"
    "them from current source and configuration. Selected native recipes still require\n"
    "their explicitly matched current catalogs.\n",
    "Older or incompatible derived stores are not opened or migrated; rebuild\n"
    "them from current source and configuration. One current Project catalog admits\n"
    "the payload schemas; reconstructed configuration proves exact selections.\n",
    "live project catalog summary",
)
path.write_text(text)

path, text = read("docs/IMPLEMENTATION.md")
text = replace_once(
    text,
    "Use one current database layout. Historical exact Project catalog sets V1–V16\n"
    "are no longer registered; an incompatible derived database must be rebuilt from\n"
    "source and configuration. Current replay recipe variants remain temporary until\n"
    "#112 consolidates their real callers. Switch only after successful rebuild and\n"
    "preserve user files. If inputs are absent, report the requirement. #111 removes\n"
    "remaining retention/GC administration after normal reads replace it. No\n"
    "compatibility reader, converter or new lifecycle subsystem.\n",
    "Use one current database layout and one ordinary Project replay catalog; the only\n"
    "second physical variant adds the candidate receipt/check. Exact feature choices\n"
    "are validated from reconstructed configuration, not catalog identity. Payload\n"
    "schema tags remain to be collapsed only after tracing their real readers. An\n"
    "incompatible derived database must be rebuilt from source/configuration; preserve\n"
    "user files and report missing inputs. #111 removes remaining retention/GC\n"
    "administration. No compatibility reader, converter or new lifecycle subsystem.\n",
    "implementation catalog boundary",
)
path.write_text(text)

path, text = read("docs/REFACTOR_QUEUE.md")
text = sub_once(
    text,
    r"\| 6 \| \[#112 — current formats\].*?\|\n",
    "| 6 | [#112 — current formats](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/112) | Historical and feature-combination catalogs are removed; collapse remaining replay payload tags only from real readers. |\n",
    "queue current formats row",
)
path.write_text(text)


# Fail closed: no source or current instructions may recreate physical catalog
# families or dispatch on the replay record schema.
for target in Path(".").rglob("*"):
    if not target.is_file() or target.suffix not in {".rs", ".md", ".json"}:
        continue
    body = target.read_text(errors="ignore")
    for forbidden in (
        "STORAGE_SCHEMA_CATALOGS",
        "STORAGE_SCHEMAS_ACEADDON_LOCAL_RESULT_V1",
        "STORAGE_SCHEMAS_XML_CLASSIFICATION_V1",
        "STORAGE_SCHEMAS_PHYSICAL_FUNCTION_DECLARATIONS_V1",
        "STORAGE_SCHEMAS_XML_CLASSIFICATION_PHYSICAL_FUNCTION_DECLARATIONS_V1",
        "STORAGE_SCHEMAS_PHYSICAL_FUNCTION_BODY_SHAPE_V1",
        "STORAGE_SCHEMAS_XML_CLASSIFICATION_PHYSICAL_FUNCTION_BODY_SHAPE_V1",
        "STORAGE_SCHEMAS_SELECTED_COMPONENTS_V1",
        "STORAGE_SCHEMAS_PRIMARY_CALLABLES_V1",
        "STORAGE_SCHEMAS_NATIVE_LIBRARY_INPUT_V1",
        "STORAGE_SCHEMAS_NATIVE_API_USES_V1",
        "replay_storage_schema",
        "catalog_accepts_components",
        "catalog_components",
        "create_primary_callables",
        "create_native_library_input",
    ):
        if forbidden in body:
            raise SystemExit(f"{forbidden!r} remains in {target}")
