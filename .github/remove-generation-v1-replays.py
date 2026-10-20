from pathlib import Path
import re


def read(path: str) -> tuple[Path, str]:
    target = Path(path)
    return target, target.read_text()


def once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected one match, found {count}")
    return text.replace(old, new, 1)


def many(text: str, old: str, new: str, expected: int, label: str) -> str:
    count = text.count(old)
    if count != expected:
        raise SystemExit(f"{label}: expected {expected} matches, found {count}")
    return text.replace(old, new)


def sub_once(text: str, pattern: str, replacement: str, label: str) -> str:
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.DOTALL)
    if count != 1:
        raise SystemExit(f"{label}: expected one regex match, found {count}")
    return updated


# One current generation recipe.
path, text = read("crates/wow-project/src/configuration.rs")
text = once(
    text,
    "/// Frozen legacy receipt recipe. New publication selects generation v2 explicitly.\n"
    "pub const PROJECT_GENERATION_SCHEMA_VERSION: u64 = 1;\n",
    "/// Current project generation identity recipe.\n"
    "pub const PROJECT_GENERATION_SCHEMA_VERSION: u64 = 2;\n",
    "generation version constant",
)
path.write_text(text)

Path("crates/wow-project/src/generation.rs").write_text(
    '''use serde::Serialize;
use wow_core::{CanonicalResult, ContentDigest, ProjectGenerationId, ReferenceGenerationId};
use wow_emmy::LuaWorkspaceSnapshot;

use crate::configuration::PROJECT_GENERATION_SCHEMA_VERSION;
use crate::identity::canonical_id;
use crate::{
    ProjectConfiguration, ProjectError, ProjectErrorCode, ProjectFileManifestEntry,
    ProjectInputInventory, ProjectPhase, ProjectResult,
};

/// Immutable derivation receipt for one intended final project state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectGenerationCandidate {
    candidate_id: Box<str>,
    project_configuration_digest: ContentDigest<CanonicalResult>,
    selected_profile_id: Box<str>,
    reference_generation: ReferenceGenerationId,
    analyzer_pin_id: Box<str>,
    analyzer_probe_report_id: Box<str>,
    analyzer_configuration_digest: ContentDigest<CanonicalResult>,
    final_file_manifest_digest: ContentDigest<CanonicalResult>,
    project_generation_schema_version: u64,
    library_snapshot_ids: Vec<Box<str>>,
    function_call_facts: bool,
    derived_project_generation: ProjectGenerationId,
}

impl ProjectGenerationCandidate {
    /// Derive the current identity from the exact Library corpus and analyzer fact profile.
    pub fn derive_with_analysis(
        configuration: &ProjectConfiguration,
        inventory: &ProjectInputInventory,
        libraries: &[LuaWorkspaceSnapshot],
        function_call_facts: bool,
    ) -> ProjectResult<Self> {
        let mut ids = libraries
            .iter()
            .map(|library| Box::<str>::from(library.snapshot_id()))
            .collect::<Vec<_>>();
        ids.sort();
        Self::derive_from_ids(configuration, inventory, ids, function_call_facts)
    }

    fn derive_from_ids(
        configuration: &ProjectConfiguration,
        inventory: &ProjectInputInventory,
        library_snapshot_ids: Vec<Box<str>>,
        function_call_facts: bool,
    ) -> ProjectResult<Self> {
        configuration.validate()?;
        if let Some(plan) = configuration.load_plan() {
            plan.validate_main_files(inventory.files())?;
        }
        if let Some(plan) = configuration.package_main_plan() {
            plan.validate_main_files(inventory.files())?;
        }
        let manifest = inventory.manifest_entries();
        #[derive(Serialize)]
        struct DerivationInput<'a> {
            project_generation_schema_version: u64,
            project_configuration_digest: ContentDigest<CanonicalResult>,
            project_id: &'a crate::ProjectId,
            project_kind: crate::ProjectKind,
            workspace_id: &'a crate::ProjectWorkspaceId,
            source_origin_id: &'a crate::ProjectSourceOriginId,
            logical_root: &'a wow_core::NormalizedSourcePath,
            selected_profile: &'a wow_core::ProfileIdentity,
            reference_generation: ReferenceGenerationId,
            analyzer_pin_id: &'a str,
            analyzer_probe_report_id: &'a str,
            analyzer_configuration_digest: ContentDigest<CanonicalResult>,
            capability_policy: &'a crate::ProjectCapabilityPolicy,
            budget_policy: crate::ProjectBudgetPolicy,
            final_file_manifest_digest: ContentDigest<CanonicalResult>,
            files: &'a [ProjectFileManifestEntry],
            library_snapshot_ids: &'a [Box<str>],
            function_call_facts: bool,
        }
        let input = DerivationInput {
            project_generation_schema_version: PROJECT_GENERATION_SCHEMA_VERSION,
            project_configuration_digest: configuration.configuration_digest(),
            project_id: configuration.project_id(),
            project_kind: configuration.project_kind(),
            workspace_id: configuration.workspace_id(),
            source_origin_id: configuration.source_origin_id(),
            logical_root: configuration.logical_root(),
            selected_profile: configuration.selected_profile(),
            reference_generation: configuration.reference_generation(),
            analyzer_pin_id: configuration.analyzer_binding().accepted_pin_id(),
            analyzer_probe_report_id: configuration
                .analyzer_binding()
                .compatibility_probe_report_id(),
            analyzer_configuration_digest: configuration
                .analyzer_binding()
                .analyzer_configuration_digest(),
            capability_policy: configuration.capability_policy(),
            budget_policy: configuration.budget_policy(),
            final_file_manifest_digest: inventory.manifest_digest(),
            files: &manifest,
            library_snapshot_ids: &library_snapshot_ids,
            function_call_facts,
        };
        let derived_project_generation = ProjectGenerationId::derive(&input).map_err(|source| {
            ProjectError::new(
                ProjectErrorCode::GenerationDerivationFailed,
                ProjectPhase::Generation,
                format!("project generation cannot be derived: {source}"),
            )
        })?;
        #[derive(Serialize)]
        struct CandidateIdentity<'a> {
            project_generation: ProjectGenerationId,
            derivation_input: &'a DerivationInput<'a>,
        }
        let candidate_id = canonical_id(
            "project-candidate:sha256:",
            "wow-project/generation-candidate/e0-d/2",
            &CandidateIdentity {
                project_generation: derived_project_generation,
                derivation_input: &input,
            },
            ProjectPhase::Generation,
        )?;
        Ok(Self {
            candidate_id,
            project_configuration_digest: configuration.configuration_digest(),
            selected_profile_id: configuration
                .selected_profile()
                .profile_id()
                .as_str()
                .into(),
            reference_generation: configuration.reference_generation(),
            analyzer_pin_id: configuration.analyzer_binding().accepted_pin_id().into(),
            analyzer_probe_report_id: configuration
                .analyzer_binding()
                .compatibility_probe_report_id()
                .into(),
            analyzer_configuration_digest: configuration
                .analyzer_binding()
                .analyzer_configuration_digest(),
            final_file_manifest_digest: inventory.manifest_digest(),
            project_generation_schema_version: PROJECT_GENERATION_SCHEMA_VERSION,
            library_snapshot_ids,
            function_call_facts,
            derived_project_generation,
        })
    }

    #[must_use]
    pub fn candidate_id(&self) -> &str {
        &self.candidate_id
    }

    #[must_use]
    pub const fn project_configuration_digest(&self) -> ContentDigest<CanonicalResult> {
        self.project_configuration_digest
    }

    #[must_use]
    pub fn selected_profile_id(&self) -> &str {
        &self.selected_profile_id
    }

    #[must_use]
    pub const fn reference_generation(&self) -> ReferenceGenerationId {
        self.reference_generation
    }

    #[must_use]
    pub fn analyzer_pin_id(&self) -> &str {
        &self.analyzer_pin_id
    }

    #[must_use]
    pub const fn analyzer_configuration_digest(&self) -> ContentDigest<CanonicalResult> {
        self.analyzer_configuration_digest
    }

    #[must_use]
    pub const fn final_file_manifest_digest(&self) -> ContentDigest<CanonicalResult> {
        self.final_file_manifest_digest
    }

    #[must_use]
    pub const fn project_generation(&self) -> ProjectGenerationId {
        self.derived_project_generation
    }

    #[must_use]
    pub const fn project_generation_schema_version(&self) -> u64 {
        self.project_generation_schema_version
    }

    #[must_use]
    pub fn library_snapshot_ids(&self) -> &[Box<str>] {
        &self.library_snapshot_ids
    }

    #[must_use]
    pub const fn function_call_facts(&self) -> bool {
        self.function_call_facts
    }

    pub(crate) fn validate_function_call_facts(&self, enabled: bool) -> ProjectResult<()> {
        if self.function_call_facts != enabled {
            return Err(ProjectError::new(
                ProjectErrorCode::AnalyzerSnapshotMismatch,
                ProjectPhase::Generation,
                "project generation fact profile differs from analyzer inputs",
            ));
        }
        Ok(())
    }

    pub(crate) fn validate_library_ids<'a>(
        &self,
        ids: impl IntoIterator<Item = &'a str>,
    ) -> ProjectResult<()> {
        let mut actual = ids.into_iter().collect::<Vec<_>>();
        actual.sort_unstable();
        if !self
            .library_snapshot_ids
            .iter()
            .map(AsRef::as_ref)
            .eq(actual)
        {
            return Err(ProjectError::new(
                ProjectErrorCode::AnalyzerSnapshotMismatch,
                ProjectPhase::Generation,
                "project generation Library identities differ from analyzer inputs",
            ));
        }
        Ok(())
    }

    pub fn validate(
        &self,
        configuration: &ProjectConfiguration,
        inventory: &ProjectInputInventory,
    ) -> ProjectResult<()> {
        let expected = Self::derive_from_ids(
            configuration,
            inventory,
            self.library_snapshot_ids.clone(),
            self.function_call_facts,
        )?;
        if &expected == self {
            Ok(())
        } else {
            Err(ProjectError::new(
                ProjectErrorCode::GenerationCollision,
                ProjectPhase::Generation,
                "project generation candidate does not match its canonical derivation inputs",
            ))
        }
    }
}
'''
)


# Publisher: remove the private legacy-generation switch.
path, text = read("crates/wow-project/src/publication.rs")
text = once(text, "    legacy_generation: bool,\n", "", "publisher legacy field")
text = once(text, "            legacy_generation: false,\n", "", "publisher legacy init")
text = sub_once(
    text,
    r"\n    /// Only old native archives may reproduce the frozen generation recipe\.\n"
    r"    pub\(crate\) fn legacy_replay\(function_calls: bool\) -> Self \{\n"
    r".*?\n"
    r"    \}\n",
    "\n",
    "legacy replay constructor",
)
text = many(
    text,
    "            self.legacy_generation,\n",
    "",
    2,
    "publisher legacy arguments",
)
text = once(
    text,
    "    legacy_generation: bool,\n",
    "",
    "build snapshot legacy parameter",
)
text = sub_once(
    text,
    r"    let function_calls = function_call_mode\.enabled\(\);\n"
    r"    let generation = if legacy_generation \{\n"
    r"        ProjectGenerationCandidate::derive\(&configuration, &inventory\)\?\n"
    r"    \} else \{\n"
    r"        ProjectGenerationCandidate::derive_with_analysis\(\n"
    r"            &configuration,\n"
    r"            &inventory,\n"
    r"            &libraries,\n"
    r"            function_calls,\n"
    r"        \)\?\n"
    r"    \};\n",
    "    let function_calls = function_call_mode.enabled();\n"
    "    let generation = ProjectGenerationCandidate::derive_with_analysis(\n"
    "        &configuration,\n"
    "        &inventory,\n"
    "        &libraries,\n"
    "        function_calls,\n"
    "    )?;\n",
    "current generation derivation",
)
path.write_text(text)


# Replay: schemas 1-3 and missing generation version are no longer executable formats.
path, text = read("crates/wow-project/src/replay/mod.rs")
for old, label in [
    ('const REPLAY_SCHEMA: &str = "wow-project/native-project-replay/1";\n', "replay1 constant"),
    ('const LOAD_REPLAY_SCHEMA: &str = "wow-project/native-project-replay/2";\n', "replay2 constant"),
    ('const PACKAGE_REPLAY_SCHEMA: &str = "wow-project/native-project-replay/3";\n', "replay3 constant"),
]:
    text = once(text, old, "", label)
text = once(
    text,
    "/// Data-only archive, distinct from the executable owner view. New publications\n"
    "/// use v4 with Library-bound generation, v5 for a genuine platform corpus, or v6\n"
    "/// for explicitly selected same-session package XML bindings. Package graph\n"
    "/// projection uses v7; projection with raw inventory uses v8. Earlier archives\n"
    "/// retain their original recipe.\n",
    "/// Data-only archive, distinct from the executable owner view. Current publication\n"
    "/// requires generation recipe v2; configuration selects the ordinary, platform or\n"
    "/// named component replay shape without a legacy fallback.\n",
    "replay model comment",
)
text = once(
    text,
    "    #[serde(default, skip_serializing_if = \"Option::is_none\")]\n"
    "    generation_schema_version: Option<u64>,\n",
    "    generation_schema_version: u64,\n",
    "required generation version field",
)
text = once(
    text,
    "        let generation_schema_version = match snapshot\n"
    "            .generation_candidate()\n"
    "            .project_generation_schema_version()\n"
    "        {\n"
    "            1 => None,\n"
    "            2 => Some(2),\n"
    "            _ => return Err(invalid()),\n"
    "        };\n"
    "        if (configuration.native_library_input().is_some()\n"
    "            || configuration.native_api_uses().is_some()\n"
    "            || configuration.state_projection().is_some()\n"
    "            || configuration.hook_target_recipe().is_some())\n"
    "            && generation_schema_version != Some(2)\n"
    "        {\n"
    "            return Err(invalid());\n"
    "        }\n",
    "        let generation_schema_version = snapshot\n"
    "            .generation_candidate()\n"
    "            .project_generation_schema_version();\n"
    "        if generation_schema_version != crate::PROJECT_GENERATION_SCHEMA_VERSION {\n"
    "            return Err(invalid());\n"
    "        }\n",
    "capture generation version",
)
text = once(
    text,
    "        let base_schema = if platform_owner.is_some() {\n"
    "            platform_schema\n"
    "        } else if generation_schema_version.is_some() {\n"
    "            LIBRARY_BOUND_REPLAY_SCHEMA\n"
    "        } else if packages.is_some() {\n"
    "            PACKAGE_REPLAY_SCHEMA\n"
    "        } else if load.is_some() {\n"
    "            LOAD_REPLAY_SCHEMA\n"
    "        } else {\n"
    "            REPLAY_SCHEMA\n"
    "        };\n",
    "        let base_schema = if platform_owner.is_some() {\n"
    "            platform_schema\n"
    "        } else {\n"
    "            LIBRARY_BOUND_REPLAY_SCHEMA\n"
    "        };\n",
    "current base replay schema",
)
text = once(
    text,
    "            generation_schema_version == Some(2)\n"
    "                && load.is_none()\n"
    "                && packages.is_none()\n"
    "                && platform_owner.is_none(),\n",
    "            load.is_none() && packages.is_none() && platform_owner.is_none(),\n",
    "current physical primary selector",
)
text = once(
    text,
    "        let platform = if let Some(owner) = platform_owner {\n"
    "            if generation_schema_version != Some(2) {\n"
    "                return Err(invalid());\n"
    "            }\n"
    "            let envelope_size = encoded_size(\n",
    "        let platform = if let Some(owner) = platform_owner {\n"
    "            let envelope_size = encoded_size(\n",
    "platform generation guard",
)
text = once(
    text,
    "        if (self.configuration.native_library_input().is_some()\n"
    "            || self.configuration.native_api_uses().is_some()\n"
    "            || self.configuration.state_projection().is_some()\n"
    "            || self.configuration.hook_target_recipe().is_some())\n"
    "            && self.generation_schema_version != Some(2)\n"
    "        {\n"
    "            return Err(invalid());\n"
    "        }\n",
    "        if self.generation_schema_version != crate::PROJECT_GENERATION_SCHEMA_VERSION {\n"
    "            return Err(invalid());\n"
    "        }\n",
    "expected generation version",
)
text = many(
    text,
    "self.generation_schema_version == Some(2)\n                && ",
    "",
    2,
    "expected base current generation guards",
)
text = once(
    text,
    "        match (\n"
    "            self.generation_schema_version,\n"
    "            self.configuration.package_xml_binding_profile(),\n"
    "        ) {\n"
    "            (Some(2), Some(PackageXmlBindingProfile::SameSessionV1))\n"
    "                if self.configuration.is_platform() && self.platform.is_some() =>\n"
    "            {\n"
    "                Ok(PACKAGE_XML_REPLAY_SCHEMA)\n"
    "            }\n"
    "            (_, Some(_)) => Err(invalid()),\n"
    "            (Some(2), None) if self.platform.is_some() => Ok(PLATFORM_REPLAY_SCHEMA),\n"
    "            (Some(2), None) => Ok(LIBRARY_BOUND_REPLAY_SCHEMA),\n"
    "            (None, None) if self.packages.is_some() => Ok(PACKAGE_REPLAY_SCHEMA),\n"
    "            (None, None) if self.load.is_some() => Ok(LOAD_REPLAY_SCHEMA),\n"
    "            (None, None) => Ok(REPLAY_SCHEMA),\n"
    "            (Some(_), None) => Err(invalid()),\n"
    "        }\n",
    "        match self.configuration.package_xml_binding_profile() {\n"
    "            Some(PackageXmlBindingProfile::SameSessionV1)\n"
    "                if self.configuration.is_platform() && self.platform.is_some() =>\n"
    "            {\n"
    "                Ok(PACKAGE_XML_REPLAY_SCHEMA)\n"
    "            }\n"
    "            Some(_) => Err(invalid()),\n"
    "            None if self.platform.is_some() => Ok(PLATFORM_REPLAY_SCHEMA),\n"
    "            None => Ok(LIBRARY_BOUND_REPLAY_SCHEMA),\n"
    "        }\n",
    "current expected base schema",
)
text = once(
    text,
    "                    || self.generation_schema_version != Some(2)",
    "",
    "platform validation generation guard",
)
text = once(
    text,
    "        let mut publisher = if self.generation_schema_version.is_none() {\n"
    "            ProjectPublisher::legacy_replay(self.function_calls)\n"
    "        } else if self.function_calls {\n"
    "            ProjectPublisher::with_function_call_facts()\n"
    "        } else {\n"
    "            ProjectPublisher::new()\n"
    "        };\n",
    "        let mut publisher = if self.function_calls {\n"
    "            ProjectPublisher::with_function_call_facts()\n"
    "        } else {\n"
    "            ProjectPublisher::new()\n"
    "        };\n",
    "current replay publisher",
)
text = once(
    text,
    "    /// Whether this archive carries the modern Library-bound physical profile\n"
    "    /// whose retained owner may accept a durable update. Legacy v1/v2/v3 and\n"
    "    /// standalone or package corpora remain read-only compatibility records.\n"
    "    pub(crate) fn supports_physical_update(&self) -> bool {\n"
    "        self.generation_schema_version == Some(2)\n"
    "            && self.load.is_none()\n"
    "            && self.packages.is_none()\n"
    "            && self.platform.is_none()\n"
    "    }\n",
    "    /// Whether this current archive is the ordinary physical profile whose retained\n"
    "    /// owner may accept a durable update. Standalone, package and platform inputs do not.\n"
    "    pub(crate) fn supports_physical_update(&self) -> bool {\n"
    "        self.load.is_none() && self.packages.is_none() && self.platform.is_none()\n"
    "    }\n",
    "physical update predicate",
)
for old, label in [
    ('            REPLAY_SCHEMA => Ok("wow-project.live-replay.v1"),\n', "replay1 storage mapping"),
    ('            LOAD_REPLAY_SCHEMA => Ok("wow-project.live-replay.v2"),\n', "replay2 storage mapping"),
    ('            PACKAGE_REPLAY_SCHEMA => Ok("wow-project.live-replay.v3"),\n', "replay3 storage mapping"),
]:
    text = once(text, old, "", label)
path.write_text(text)


# Store catalogs and read admission: drop v1-v3 everywhere.
path, text = read("crates/wow-project/src/replay/publication.rs")
for version in (1, 2, 3):
    text = many(
        text,
        f'    "wow-project.live-replay.v{version}",\n',
        "",
        5,
        f"v{version} catalog memberships",
    )
    text = once(
        text,
        f'                | "wow-project.live-replay.v{version}"\n',
        "",
        f"v{version} read admission",
    )
text = once(
    text,
    "    /// Consume the validated owner; legacy and loader profiles cannot be updated\n"
    "    /// through the physical-input protocol.\n",
    "    /// Consume the validated owner; standalone and package profiles cannot be updated\n"
    "    /// through the physical-input protocol.\n",
    "update owner comment",
)
text = once(
    text,
    "    /// Ordinary physical/legacy admission remains in into_update_publisher.\n",
    "    /// Ordinary physical admission remains in into_update_publisher.\n",
    "platform update comment",
)
path.write_text(text)


# Compatibility-only tests and fixtures disappear; one obsolete-input refusal remains.
path, text = read("crates/wow-project/tests/project_e0.rs")
text = sub_once(
    text,
    r"\n#\[test\]\n"
    r"fn frozen_native_replays_preserve_original_generation_recipe\(\) -> TestResult \{\n"
    r".*?\n"
    r"\}\n\n"
    r"#\[test\]\n"
    r"fn library_order_is_canonical_and_fact_profile_changes_generation",
    "\n#[test]\nfn library_order_is_canonical_and_fact_profile_changes_generation",
    "frozen replay test",
)
text = once(
    text,
    "    assert_eq!(\n"
    "        left.generation_candidate().function_call_facts(),\n"
    "        Some(false)\n"
    "    );\n",
    "    assert!(!left.generation_candidate().function_call_facts());\n",
    "plain fact profile assertion",
)
text = once(
    text,
    "    assert_eq!(\n"
    "        extended.generation_candidate().function_call_facts(),\n"
    "        Some(true)\n"
    "    );\n",
    "    assert!(extended.generation_candidate().function_call_facts());\n",
    "callable fact profile assertion",
)
text = sub_once(
    text,
    r"    for mutation in 0\.\.3 \{\n"
    r".*?\n"
    r"    \}\n"
    r"    Ok\(\(\)\)\n"
    r"\}\n\n"
    r"#\[test\]\n"
    r"fn cancelled_update_keeps_current_and_records_the_last_good_identity",
    "    let mut mismatched = archive.clone();\n"
    "    mismatched[\"function_calls\"] = false.into();\n"
    "    assert!(\n"
    "        serde_json::from_value::<ProjectReplay>(mismatched)?\n"
    "            .hydrate(&stop)\n"
    "            .is_err()\n"
    "    );\n"
    "    let mut obsolete = archive;\n"
    "    obsolete\n"
    "        .as_object_mut()\n"
    "        .ok_or(\"archive object missing\")?\n"
    "        .remove(\"generation_schema_version\");\n"
    "    obsolete[\"schema\"] = \"wow-project/native-project-replay/1\".into();\n"
    "    assert!(serde_json::from_value::<ProjectReplay>(obsolete).is_err());\n"
    "    Ok(())\n"
    "}\n\n"
    "#[test]\n"
    "fn cancelled_update_keeps_current_and_records_the_last_good_identity",
    "current replay mutation checks",
)
path.write_text(text)

path, text = read("crates/wow-project/tests/package_xml_bindings.rs")
text = once(
    text,
    "fn package_binding_selection_requires_native_owner_and_preserves_legacy_replay() -> TestResult {",
    "fn package_binding_selection_requires_native_owner_and_replays_current_input() -> TestResult {",
    "package replay test name",
)
text = sub_once(
    text,
    r"\n    let frozen: serde_json::Value =\n"
    r"        serde_json::from_str\(include_str!\(\"data/native-replay-legacy/packages\.json\"\)\)\?;\n"
    r".*?\n"
    r"    Ok\(\(\)\)\n"
    r"\}\n\n"
    r"#\[test\]\n"
    r"fn package_xml_bindings_keep_local_scopes_and_one_shared_native_lookup_in_replay",
    "\n    Ok(())\n"
    "}\n\n"
    "#[test]\n"
    "fn package_xml_bindings_keep_local_scopes_and_one_shared_native_lookup_in_replay",
    "frozen package replay block",
)
path.write_text(text)


# The current lifecycle fixture owns its metadata instead of borrowing a retired archive.
path, text = read("crates/wow-service/tests/primary_callable_lifecycle.rs")
text = once(
    text,
    "use wow_core::{ContentDigest, ProfileIdentity, SourceContent};\n",
    "use wow_core::{\n"
    "    ContentDigest, ProfileIdentity, ProfileIdentityBuilder, ProfileKind, SchemaVersionEntry,\n"
    "    SourceContent, SourceKind, SourceLogicalSnapshot,\n"
    "};\n",
    "typed fixture imports",
)
text = once(
    text,
    "const STORAGE_SCHEMA: &str = \"wow-project.live-replay.primary-callables.v1\";\n",
    "const STORAGE_SCHEMA: &str = \"wow-project.live-replay.primary-callables.v1\";\n"
    "const WORKSPACE_ID: &str = \"workspace:main:project-pair-regression\";\n"
    "const SOURCE_ORIGIN_ID: &str = \"project-origin:project-pair-regression\";\n"
    "const LOGICAL_ROOT: &str = \"fixtures/project-pair/main\";\n"
    "const REFERENCE_GENERATION: &str =\n"
    "    \"generation:reference:sha256:97675ba211ab64855e16865ac469cfe32478719078c34f7c34d31a1a07d99918\";\n"
    "const ANALYZER_CONFIGURATION_DIGEST: &str =\n"
    "    \"sha256:0303030303030303030303030303030303030303030303030303030303030303\";\n"
    "const ANALYZER_CONTRACT_ID: &str = \"wow-emmy/e0-c/1\";\n"
    "const ANALYZER_LIBRARY_CONTRACT_ID: &str = \"wow-emmy-e0-c-library-v1\";\n",
    "fixture metadata constants",
)
text = once(
    text,
    "struct Fixture {\n"
    "    profile: ProfileIdentity,\n"
    "    reference: ReferenceView,\n"
    "    metadata: Value,\n"
    "}\n",
    "struct Fixture {\n"
    "    profile: ProfileIdentity,\n"
    "    reference: ReferenceView,\n"
    "}\n",
    "fixture metadata field",
)
text = sub_once(
    text,
    r"    fn new\(\) -> TestResult<Self> \{\n"
    r"        // Borrow the already retained synthetic native-pair input metadata,\n"
    r"        // not its decoded configuration/replay as authority\.\n"
    r"        let legacy: Value = serde_json::from_str\(include_str!\(\n"
    r"            \"\.\./\.\./wow-project/tests/data/native-replay-legacy/physical\.json\"\n"
    r"        \)\)\?;\n"
    r"        let metadata = legacy\[\"configuration\"\]\.clone\(\);\n"
    r"        let profile: ProfileIdentity = serde_json::from_value\(metadata\[\"profile\"\]\.clone\(\)\)\?;\n"
    r"        profile\.validate\(\)\?;\n"
    r"        let reference = ReferenceView::new\(\n"
    r"            metadata\[\"reference_generation\"\]\n"
    r"                \.as_str\(\)\n"
    r"                \.ok_or\(\"fixture reference generation\"\)\?,\n"
    r"            Vec::new\(\),\n"
    r"            Vec::new\(\),\n"
    r"        \)\?;\n"
    r"        reference\.validate\(\)\?;\n"
    r"        Ok\(Self \{\n"
    r"            profile,\n"
    r"            reference,\n"
    r"            metadata,\n"
    r"        \}\)\n"
    r"    \}\n",
    "    fn new() -> TestResult<Self> {\n"
    "        let profile = ProfileIdentityBuilder::new(\n"
    "            \"profile:fixture:project-pair-regression-v1\".parse()?,\n"
    "            ProfileKind::Fixture,\n"
    "            \"retail\",\n"
    "            120_100,\n"
    "            SourceKind::SyntheticFixture,\n"
    "            \"fixture:live-pair-v1\",\n"
    "            ContentDigest::<SourceLogicalSnapshot>::from_bytes([7; 32]),\n"
    "        )\n"
    "        .schema_versions(vec![SchemaVersionEntry::new(\n"
    "            \"schema:wow:fixture-e0\".parse()?,\n"
    "            \"1.0.0\".parse()?,\n"
    "        )])\n"
    "        .fixture_scope(\"native live pair regression\")\n"
    "        .build()?;\n"
    "        let reference = ReferenceView::new(REFERENCE_GENERATION, Vec::new(), Vec::new())?;\n"
    "        reference.validate()?;\n"
    "        Ok(Self { profile, reference })\n"
    "    }\n",
    "current lifecycle fixture metadata",
)
text = once(
    text,
    "        let analyzer = &self.metadata[\"analyzer\"];\n",
    "",
    "borrowed analyzer metadata",
)
text = once(
    text,
    "            \"project_id\":PROJECT,\"workspace_id\":self.metadata[\"workspace_id\"],\n"
    "            \"source_origin_id\":self.metadata[\"source_origin_id\"],\n"
    "            \"logical_root\":self.metadata[\"logical_root\"],\n",
    "            \"project_id\":PROJECT,\"workspace_id\":WORKSPACE_ID,\n"
    "            \"source_origin_id\":SOURCE_ORIGIN_ID,\n"
    "            \"logical_root\":LOGICAL_ROOT,\n",
    "fixture owner identifiers",
)
text = once(
    text,
    "                \"configuration_digest\":analyzer[\"analyzer_configuration_digest\"],\n"
    "                \"contract_id\":analyzer[\"analyzer_contract_id\"],\n"
    "                \"fixture_contract_id\":analyzer[\"analyzer_fixture_contract_id\"],\n"
    "                \"library_contract_id\":analyzer[\"expected_library_contract_id\"]\n",
    "                \"configuration_digest\":ANALYZER_CONFIGURATION_DIGEST,\n"
    "                \"contract_id\":ANALYZER_CONTRACT_ID,\n"
    "                \"fixture_contract_id\":ANALYZER_CONTRACT_ID,\n"
    "                \"library_contract_id\":ANALYZER_LIBRARY_CONTRACT_ID\n",
    "fixture analyzer identifiers",
)
path.write_text(text)


# Active documentation describes the one current generation/replay route.
path, text = read("apps/wow/LIVE_PROJECT.md")
text = once(
    text,
    "The original ordinary publication profile uses native/storage replay v4 and\n"
    "generation recipe v2, binding exact Library snapshot identities and the\n"
    "function-call-facts profile. V1/V2/V3 stores retain their exact catalogs and reject\n"
    "v4 writes. Selected native recipes require their explicitly matched catalogs;\n"
    "there is no automatic migration.\n",
    "The ordinary publication profile uses native/storage replay v4 and generation\n"
    "recipe v2, binding exact Library snapshot identities and the function-call-facts\n"
    "profile. Older or incompatible derived stores are not opened or migrated; rebuild\n"
    "them from current source and configuration. Selected native recipes still require\n"
    "their explicitly matched current catalogs.\n",
    "live project format policy",
)
text = once(
    text,
    "current CAS. Standalone TOC/XML, package and legacy archives remain unavailable\n"
    "for this update route. Their read/publication routes remain supported.\n",
    "current CAS. Standalone TOC/XML and package publications remain unavailable for\n"
    "this update route. Their current read/publication routes remain supported.\n",
    "live update profile wording",
)
text = sub_once(
    text,
    r"\nThe standalone TOC/XML checkpoint uses native replay v2.*\Z",
    "\nStandalone TOC/XML and declared package inputs use the same current replay v4 and\n"
    "generation-v2 envelope as other ordinary publications. The replay retains their\n"
    "selected TOC/context/documents or package sources/variants and rebuilds those owners\n"
    "without reopening source directories. No replay-v1/v2/v3 catalog, generation-v1\n"
    "reader or store migration remains in the product; incompatible derived stores must\n"
    "be recreated from source and configuration.\n",
    "live project legacy tail",
)
path.write_text(text)

path, text = read("apps/wow/PLATFORM_INPUT.md")
text = once(
    text,
    "The selected limits enter the sealed package binding and original replay request.\n"
    "Retained-profile archives use `wow-project/native-project-replay/11`; ordinary\n"
    "replay1-10 retain their bytes and selection. Hydration re-admits the exact source\n"
    "and package owners under the captured limits. Changed limits or schema downgrade\n"
    "refuse. A successful retained Main is not full graph or persistence acceptance:\n",
    "The selected limits enter the sealed package binding and original replay request.\n"
    "Retained-profile archives use `wow-project/native-project-replay/11`; other current\n"
    "platform selections keep their exact selected recipe. Hydration re-admits the exact\n"
    "source and package owners under the captured limits. Changed limits or schema\n"
    "substitution refuse. A successful retained Main is not full graph or persistence acceptance:\n",
    "platform replay policy",
)
text = once(
    text,
    "Selection requires genuine native retained packages, reaches the same semantic\n"
    "session, and produces budget-bound `wow-emmy/function-call-facts/8` and\n"
    "`wow-project/native-project-replay/12`. The configuration digest and report\n"
    "identity both retain the selected values. Hydration rebuilds them without source\n"
    "files and refuses removal, downgrade or changed values against the original\n"
    "identities. Ordinary replay1-11 recipes remain unchanged.\n",
    "Selection requires genuine native retained packages, reaches the same semantic\n"
    "session, and produces budget-bound `wow-emmy/function-call-facts/8` and\n"
    "`wow-project/native-project-replay/12`. The configuration digest and report\n"
    "identity both retain the selected values. Hydration rebuilds them without source\n"
    "files and refuses removal, substitution or changed values against the original\n"
    "identities. Other current replay recipes remain unchanged.\n",
    "function budget replay wording",
)
path.write_text(text)

path, text = read("docs/PLATFORM_SOURCE_REPLAY.md")
text = sub_once(
    text,
    r"## Additive schema, no legacy rewrite\n\n"
    r".*?\n"
    r"Replay capture refuses an archive whose schema, platform presence and generation schema\n"
    r"version disagree, so a v5 claim on a non-platform archive or a platform archive claiming\n"
    r"v4 is rejected\.\n",
    "## Current schema selection\n\n"
    "All current archives use generation schema version 2. Ordinary non-platform inputs,\n"
    "including retained standalone TOC/XML and declared package inputs, use\n"
    "`wow-project/native-project-replay/4` and `wow-project.live-replay.v4`. Platform\n"
    "input selects `/5`; same-session package XML selects `/6`; package graph and raw\n"
    "inventory select `/7` and `/8`. Later direct, retained-budget and named-component\n"
    "routes keep their explicitly selected current schemas.\n\n"
    "The store registers only current catalog members. Replay v1-v3 and the generation-v1\n"
    "identity recipe are not read or migrated; an incompatible derived store must be\n"
    "rebuilt from source and configuration. Capture and hydration require generation\n"
    "version 2 and exact configuration/schema agreement, with no fallback.\n",
    "platform source schema policy",
)
text = once(
    text,
    "The selected report is budget-bound function-call `/8`. Omission keeps default\n"
    "report `/7`, earlier replay recipes and configuration bytes. A `/12` archive with\n",
    "The selected report is budget-bound function-call `/8`. Omission keeps the default\n"
    "report `/7`, current unselected replay and configuration bytes. A `/12` archive with\n",
    "retained report wording",
)
text = once(
    text,
    "Genuine disk-absent service publication,\n"
    "store reopen/read and replay verify the same project/analyzer identities. New\n"
    "epochs admit v11/v12; the original v10 catalog is frozen separately and reopens\n"
    "unchanged. It refuses the selected record before retaining an operation or\n"
    "changing Current. This does not raise the\n",
    "Genuine disk-absent service publication, store reopen/read and replay verify the\n"
    "same project/analyzer identities. Exact catalog selection refuses an unregistered\n"
    "record before retaining an operation or changing Current. This does not raise the\n",
    "retained catalog wording",
)
path.write_text(text)


for fixture in [
    "crates/wow-project/tests/data/native-replay-legacy/physical.json",
    "crates/wow-project/tests/data/native-replay-legacy/standalone.json",
    "crates/wow-project/tests/data/native-replay-legacy/packages.json",
]:
    target = Path(fixture)
    if not target.is_file():
        raise SystemExit(f"missing legacy fixture before deletion: {fixture}")
    target.unlink()


# Fail closed: no executable generation-v1/replay-v1-v3 route may survive.
source_root = Path("crates/wow-project/src")
for target in source_root.rglob("*.rs"):
    content = target.read_text()
    for forbidden in (
        "legacy_generation",
        "legacy_replay",
        'const REPLAY_SCHEMA: &str = "wow-project/native-project-replay/1"',
        "LOAD_REPLAY_SCHEMA",
        "PACKAGE_REPLAY_SCHEMA",
        '"wow-project/native-project-replay/1"',
        '"wow-project/native-project-replay/2"',
        '"wow-project/native-project-replay/3"',
        '"wow-project.live-replay.v1"',
        '"wow-project.live-replay.v2"',
        '"wow-project.live-replay.v3"',
        "generation_schema_version: Option",
        "generation_schema_version.is_none",
        "generation_schema_version.is_some",
        "Some(2)",
        "ProjectGenerationCandidate::derive(",
    ):
        if forbidden in content:
            raise SystemExit(f"{forbidden!r} remains in {target}")

for target in Path(".").rglob("*.rs"):
    if "native-replay-legacy" in target.read_text(errors="ignore"):
        raise SystemExit(f"retired fixture reference remains in {target}")

for fixture in Path("crates/wow-project/tests/data/native-replay-legacy").glob("*"):
    if fixture.is_file():
        raise SystemExit(f"legacy replay fixture survived: {fixture}")

if "PROJECT_GENERATION_SCHEMA_VERSION: u64 = 1" in Path(
    "crates/wow-project/src/configuration.rs"
).read_text():
    raise SystemExit("generation version 1 constant survived")
