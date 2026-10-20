from pathlib import Path


def replace(path: str, old: str, new: str, expected: int = 1) -> None:
    target = Path(path)
    text = target.read_text()
    count = text.count(old)
    if count != expected:
        raise SystemExit(f"{path}: expected {expected} matches, found {count}: {old!r}")
    target.write_text(text.replace(old, new))


replace(
    "crates/wow-project/src/configuration.rs",
    "/// Current project generation identity recipe.\npub const PROJECT_GENERATION_SCHEMA_VERSION: u64 = 2;\n",
    "",
)
replace(
    "crates/wow-project/src/lib.rs",
    "    PROJECT_CONFIGURATION_SCHEMA_VERSION, PROJECT_CONTRACT_ID, PROJECT_GENERATION_SCHEMA_VERSION,\n",
    "    PROJECT_CONFIGURATION_SCHEMA_VERSION, PROJECT_CONTRACT_ID,\n",
)

replace(
    "crates/wow-project/src/generation.rs",
    "use crate::configuration::PROJECT_GENERATION_SCHEMA_VERSION;\n",
    "",
)
replace(
    "crates/wow-project/src/generation.rs",
    "    project_generation_schema_version: u64,\n",
    "",
    expected=2,
)
replace(
    "crates/wow-project/src/generation.rs",
    "            project_generation_schema_version: PROJECT_GENERATION_SCHEMA_VERSION,\n",
    "",
    expected=2,
)
replace(
    "crates/wow-project/src/generation.rs",
    "\n    #[must_use]\n    pub const fn project_generation_schema_version(&self) -> u64 {\n        self.project_generation_schema_version\n    }\n",
    "",
)

replace(
    "crates/wow-project/src/replay/mod.rs",
    "/// Data-only archive, distinct from the executable owner view. Current publication\n/// requires generation recipe v2; configuration selects the ordinary, platform or\n/// named component replay shape without a legacy fallback.\n",
    "/// Data-only archive, distinct from the executable owner view. Current publication\n/// has one generation recipe; configuration selects the ordinary, platform or named\n/// component replay shape without a legacy fallback.\n",
)
replace(
    "crates/wow-project/src/replay/mod.rs",
    "    generation_schema_version: u64,\n",
    "",
    expected=2,
)
replace(
    "crates/wow-project/src/replay/mod.rs",
    "        let generation_schema_version = snapshot\n            .generation_candidate()\n            .project_generation_schema_version();\n        if generation_schema_version != crate::PROJECT_GENERATION_SCHEMA_VERSION {\n            return Err(invalid());\n        }\n",
    "",
)
replace(
    "crates/wow-project/src/replay/mod.rs",
    "                    generation_schema_version: 2,\n",
    "",
)
replace(
    "crates/wow-project/src/replay/mod.rs",
    "            generation_schema_version,\n",
    "",
)
replace(
    "crates/wow-project/src/replay/mod.rs",
    "        if self.generation_schema_version != crate::PROJECT_GENERATION_SCHEMA_VERSION {\n            return Err(invalid());\n        }\n",
    "",
)

replace(
    "crates/wow-project/src/replay/publication.rs",
    "            || self\n                .project\n                .snapshot()\n                .generation_candidate()\n                .project_generation_schema_version()\n                != 2\n",
    "",
)

replace(
    "crates/wow-project/tests/project_e0.rs",
    "    assert_eq!(archive[\"generation_schema_version\"], 2);\n",
    "",
)
replace(
    "crates/wow-project/tests/project_e0.rs",
    "    let mut obsolete = archive;\n    obsolete\n        .as_object_mut()\n        .ok_or(\"archive object missing\")?\n        .remove(\"generation_schema_version\");\n    obsolete[\"schema\"] = \"wow-project/native-project-replay/1\".into();\n    assert!(serde_json::from_value::<ProjectReplay>(obsolete).is_err());\n",
    "    let mut obsolete = archive;\n    obsolete[\"schema\"] = \"wow-project/native-project-replay/1\".into();\n    assert!(\n        serde_json::from_value::<ProjectReplay>(obsolete)?\n            .hydrate(&stop)\n            .is_err()\n    );\n",
)
replace(
    "crates/wow-service/src/live_project/tests.rs",
    "    assert_eq!(archive[\"generation_schema_version\"], 2);\n",
    "",
    expected=2,
)
replace(
    "crates/wow-project/tests/platform_source.rs",
    "    assert_eq!(archive[\"generation_schema_version\"], 2);\n",
    "",
)
replace(
    "crates/wow-project/tests/primary_callable_replay.rs",
    "        assert_eq!(wire[\"generation_schema_version\"], 2);\n",
    "",
)

replace(
    "apps/wow/LIVE_PROJECT.md",
    "The ordinary publication profile uses native/storage replay v4 and generation\nrecipe v2, binding exact Library snapshot identities and the function-call-facts\nprofile.",
    "The ordinary publication profile uses native/storage replay v4 and binds exact\nLibrary snapshot identities plus the function-call-facts profile.",
)
replace(
    "apps/wow/LIVE_PROJECT.md",
    "Standalone TOC/XML and declared package inputs use the same current replay v4 and\ngeneration-v2 envelope as other ordinary publications.",
    "Standalone TOC/XML and declared package inputs use the same current replay v4\nenvelope as other ordinary publications.",
)

for path in Path(".").rglob("*"):
    if not path.is_file() or path.suffix not in {".rs", ".md", ".json"}:
        continue
    text = path.read_text(errors="ignore")
    for forbidden in (
        "PROJECT_GENERATION_SCHEMA_VERSION",
        "project_generation_schema_version",
        "generation_schema_version",
    ):
        if forbidden in text:
            raise SystemExit(f"{forbidden!r} remains in {path}")
