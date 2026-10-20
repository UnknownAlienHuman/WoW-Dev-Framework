from pathlib import Path
import re


def load(path: str) -> tuple[Path, str]:
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


# wow-emmy function-call collection: retain only current report recipes.
path, text = load("crates/wow-emmy/src/function_calls.rs")
text = replace_once(
    text,
    '/// Explicit global-access work/text selection retained by schema13 archives.\n'
    'pub const FUNCTION_CALL_GLOBAL_BUDGET_LEGACY_PROFILE: &str = "wow-emmy/function-call-facts/9";\n',
    "",
    "legacy function-call profile constant",
)
text = replace_once(
    text,
    '/// Exact historical report recipe used by retained global-access replay records.\n'
    '#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n'
    'pub enum FunctionCallReplayRecipe {\n'
    '    LegacyGlobalAccessV9,\n'
    '}\n',
    "",
    "legacy replay recipe enum",
)
text = replace_count(
    text,
    "    replay_recipe: Option<FunctionCallReplayRecipe>,\n",
    "",
    3,
    "legacy collector parameters",
)
text = replace_count(
    text,
    "        replay_recipe,\n",
    "",
    2,
    "legacy collector forwarding",
)
text = replace_once(
    text,
    "            (FUNCTION_CALL_GLOBAL_BUDGET_LEGACY_PROFILE, Some(limits))\n"
    "                if limits.global_access_budget().is_some() =>\n"
    "            {\n"
    "                limits\n"
    "            }\n",
    "",
    "legacy report validation arm",
)
text = replace_once(
    text,
    "    let legacy_global_access_profile =\n"
    "        replay_recipe == Some(FunctionCallReplayRecipe::LegacyGlobalAccessV9);\n"
    "    if (legacy_global_access_profile\n"
    "        && (local_call_result_profile.is_some() || hook_targets.is_some()))\n"
    "        || (legacy_global_access_profile\n"
    "            && !selected_budget.is_some_and(|budget| budget.global_access_budget().is_some()))\n"
    "    {\n"
    "        return Err(invalid());\n"
    "    }\n",
    "",
    "legacy collection validation",
)
text = replace_once(
    text,
    "    let include_local_constants = local_call_result_profile.is_some()\n"
    "        || (limits.global_access_budget().is_some() && !legacy_global_access_profile);\n",
    "    let include_local_constants =\n"
    "        local_call_result_profile.is_some() || limits.global_access_budget().is_some();\n",
    "current local constant selection",
)
text = replace_once(
    text,
    "        } else if legacy_global_access_profile {\n"
    "            FUNCTION_CALL_GLOBAL_BUDGET_LEGACY_PROFILE\n"
    "        } else if selected_budget.is_some_and(|budget| budget.global_access_budget().is_some()) {\n",
    "        } else if selected_budget.is_some_and(|budget| budget.global_access_budget().is_some()) {\n",
    "current report profile selection",
)
path.write_text(text)


# wow-emmy query/session seam: remove the replay-only selector and argument plumbing.
path, text = load("crates/wow-emmy/src/references.rs")
text = replace_once(
    text,
    "    function_call_replay_recipe: Option<crate::function_calls::FunctionCallReplayRecipe>,\n",
    "",
    "query profile legacy field",
)
text = replace_once(
    text,
    "            function_call_replay_recipe: None,\n",
    "",
    "query profile legacy initializer",
)
text = sub_once(
    text,
    r"\n    /// Rebuild one exact retained schema13 function-call report recipe\.\n"
    r"    /// The selected budget must carry global-access work/text limits\.\n"
    r"    #\[must_use\]\n"
    r"    pub const fn with_function_call_replay_recipe\(\n"
    r"        mut self,\n"
    r"        recipe: crate::function_calls::FunctionCallReplayRecipe,\n"
    r"    \) -> Self \{\n"
    r"        self.function_call_replay_recipe = Some\(recipe\);\n"
    r"        self\n"
    r"    \}\n",
    "\n",
    "query profile legacy method",
)
text = replace_once(
    text,
    "                || self.function_call_replay_recipe.is_some()\n",
    "",
    "query profile sidecar predicate",
)
text = sub_once(
    text,
    r"\n        if self\.function_call_replay_recipe\n"
    r"            == Some\(crate::function_calls::FunctionCallReplayRecipe::LegacyGlobalAccessV9\)\n"
    r"            && !self\n"
    r"                \.function_call_budget\n"
    r"                \.is_some_and\(\|budget\| budget\.global_access_budget\(\)\.is_some\(\)\)\n"
    r"        \{\n"
    r"            return Err\(EmmyMemberCallError::new\(\n"
    r"                EmmyMemberCallErrorCode::InvalidMainWorkspace,\n"
    r"                \"legacy global-access report replay requires its selected global-access budget\",\n"
    r"                None,\n"
    r"            \)\);\n"
    r"        \}\n",
    "\n",
    "query profile legacy validation",
)
text = replace_count(
    text,
    "                    profile.function_call_replay_recipe,\n",
    "",
    2,
    "component collector legacy arguments",
)
text = replace_count(
    text,
    "                        profile.function_call_replay_recipe,\n",
    "",
    2,
    "ordinary collector legacy arguments",
)
path.write_text(text)


# Project analyzer: one current mode and one current global-access recipe.
path, text = load("crates/wow-project/src/analyzer.rs")
text = replace_once(
    text,
    "/// Original non-global recipes and schema13/14/15 report9 used this former W6\n"
    "/// callable query as retained lookup data. It grants no recognizer admission.\n",
    "/// Non-global recipes retain this former W6 callable query as analyzer-session\n"
    "/// lookup data. It grants no recognizer admission.\n",
    "legacy callable lookup comment",
)
text = replace_once(
    text,
    "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n"
    "pub(crate) enum FunctionCallAnalysisMode {\n"
    "    Disabled,\n"
    "    Current,\n"
    "    LegacyGlobalAccessV9,\n"
    "}\n"
    "impl FunctionCallAnalysisMode {\n"
    "    pub(crate) const fn enabled(self) -> bool {\n"
    "        !matches!(self, Self::Disabled)\n"
    "    }\n\n"
    "    pub(crate) const fn uses_legacy_global_access_profile9(self) -> bool {\n"
    "        matches!(self, Self::LegacyGlobalAccessV9)\n"
    "    }\n"
    "}\n",
    "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n"
    "pub(crate) enum FunctionCallAnalysisMode {\n"
    "    Disabled,\n"
    "    Current,\n"
    "}\n"
    "impl FunctionCallAnalysisMode {\n"
    "    pub(crate) const fn enabled(self) -> bool {\n"
    "        !matches!(self, Self::Disabled)\n"
    "    }\n"
    "}\n",
    "legacy analyzer mode",
)
text = replace_once(
    text,
    "    let legacy_global_access_profile9 = function_call_mode.uses_legacy_global_access_profile9();\n",
    "",
    "legacy analyzer selection",
)
text = replace_once(
    text,
    "        // Only report10's explicitly versioned global-access recipe changed this\n"
    "        // query set. Earlier non-global namespaces retain their original session\n"
    "        // inputs, even when this lookup has no resolved target.\n"
    "        let report10_global_access = !legacy_global_access_profile9\n"
    "            && budget\n"
    "                .function_call_budget()\n"
    "                .is_some_and(|budget| budget.global_access_budget().is_some());\n",
    "        // The current global-access recipe owns its explicit query set.\n"
    "        // Non-global recipes retain the original lookup input above.\n"
    "        let report10_global_access = budget\n"
    "            .function_call_budget()\n"
    "            .is_some_and(|budget| budget.global_access_budget().is_some());\n",
    "current global-access query selection",
)
text = replace_once(
    text,
    "    let query_profile = if legacy_global_access_profile9 {\n"
    "        query_profile.with_function_call_replay_recipe(\n"
    "            wow_emmy::function_calls::FunctionCallReplayRecipe::LegacyGlobalAccessV9,\n"
    "        )\n"
    "    } else {\n"
    "        query_profile\n"
    "    };\n",
    "",
    "legacy query profile selection",
)
path.write_text(text)


# Project publisher: remove the private frozen report9 owner.
path, text = load("crates/wow-project/src/publication.rs")
text = replace_once(
    text,
    "    /// Rebuilds retained global-access replay data with its frozen report9 recipe.\n"
    "    pub(crate) fn legacy_global_access_profile9_replay() -> Self {\n"
    "        Self {\n"
    "            function_call_mode: FunctionCallAnalysisMode::LegacyGlobalAccessV9,\n"
    "            ..Self::new()\n"
    "        }\n"
    "    }\n\n"
    "    pub(crate) const fn uses_legacy_global_access_profile9(&self) -> bool {\n"
    "        self.function_call_mode.uses_legacy_global_access_profile9()\n"
    "    }\n\n",
    "",
    "legacy publisher owner",
)
path.write_text(text)


# Project replay: current global-access replays use only report10/schema16.
path, text = load("crates/wow-project/src/replay/mod.rs")
text = replace_once(
    text,
    'const GLOBAL_ACCESS_BUDGET_REPLAY_SCHEMA: &str = "wow-project/native-project-replay/13";\n',
    "",
    "legacy replay schema constant",
)
text = sub_once(
    text,
    r"        let platform_schema =\n"
    r"            if platform_owner\.is_some_and\(\|owner\| owner\.retention_limits\(\)\.is_some\(\)\) \{\n"
    r".*?\n"
    r"            \};\n"
    r"        let base_schema =",
    "        let platform_schema =\n"
    "            if platform_owner.is_some_and(|owner| owner.retention_limits().is_some()) {\n"
    "                let has_global_access_budget = configuration\n"
    "                    .function_call_budget()\n"
    "                    .is_some_and(|budget| budget.global_access_budget().is_some());\n"
    "                if configuration.function_call_evidence_profile()\n"
    "                    == Some(\n"
    "                        wow_emmy::function_calls::FunctionCallEvidenceProfile::AceAddonLocalResultV1,\n"
    "                    )\n"
    "                {\n"
    "                    ACEADDON_LOCAL_RESULT_REPLAY_SCHEMA\n"
    "                } else if has_global_access_budget {\n"
    "                    GLOBAL_ACCESS_REPORT10_REPLAY_SCHEMA\n"
    "                } else if let Some(budget) = configuration.source_graph_budget() {\n"
    "                    graph_budget_schema(budget)\n"
    "                } else if configuration.function_call_budget().is_some() {\n"
    "                    FUNCTION_BUDGET_REPLAY_SCHEMA\n"
    "                } else {\n"
    "                    RETAINED_PLATFORM_REPLAY_SCHEMA\n"
    "                }\n"
    "            } else {\n"
    "                platform_schema\n"
    "            };\n"
    "        let base_schema =",
    "current retained platform replay selection",
)
text = replace_once(
    text,
    "                || self.uses_legacy_global_access_profile9()\n",
    "",
    "hook recipe legacy exclusion",
)
text = replace_once(
    text,
    "                && !self.uses_legacy_global_access_profile9()\n",
    "",
    "AceAddon legacy exclusion",
)
text = replace_once(
    text,
    "            let selected = if self.uses_legacy_global_access_profile9() {\n"
    "                self.configuration\n"
    "                    .source_graph_budget()\n"
    "                    .map(graph_budget_schema)\n"
    "                    .unwrap_or(GLOBAL_ACCESS_BUDGET_REPLAY_SCHEMA)\n"
    "            } else if self.schema == GLOBAL_ACCESS_REPORT10_REPLAY_SCHEMA {\n"
    "                GLOBAL_ACCESS_REPORT10_REPLAY_SCHEMA\n"
    "            } else {\n"
    "                return Err(invalid());\n"
    "            };\n",
    "            if self.schema != GLOBAL_ACCESS_REPORT10_REPLAY_SCHEMA {\n"
    "                return Err(invalid());\n"
    "            }\n"
    "            let selected = GLOBAL_ACCESS_REPORT10_REPLAY_SCHEMA;\n",
    "current global-access expected schema",
)
text = replace_once(
    text,
    "        let legacy_global_access_profile9 = self.uses_legacy_global_access_profile9();\n"
    "        let mut publisher = if self.generation_schema_version.is_none() {\n"
    "            ProjectPublisher::legacy_replay(self.function_calls)\n"
    "        } else if self.function_calls {\n"
    "            if legacy_global_access_profile9 {\n"
    "                ProjectPublisher::legacy_global_access_profile9_replay()\n"
    "            } else {\n"
    "                ProjectPublisher::with_function_call_facts()\n"
    "            }\n"
    "        } else {\n"
    "            ProjectPublisher::new()\n"
    "        };\n",
    "        let mut publisher = if self.generation_schema_version.is_none() {\n"
    "            ProjectPublisher::legacy_replay(self.function_calls)\n"
    "        } else if self.function_calls {\n"
    "            ProjectPublisher::with_function_call_facts()\n"
    "        } else {\n"
    "            ProjectPublisher::new()\n"
    "        };\n",
    "current replay hydration owner",
)
text = replace_once(
    text,
    "    fn uses_legacy_global_access_profile9(&self) -> bool {\n"
    "        self.configuration\n"
    "            .function_call_budget()\n"
    "            .is_some_and(|budget| budget.global_access_budget().is_some())\n"
    "            && matches!(\n"
    "                self.schema.as_str(),\n"
    "                GLOBAL_ACCESS_BUDGET_REPLAY_SCHEMA\n"
    "                    | GRAPH_METADATA_BUDGET_REPLAY_SCHEMA\n"
    "                    | GRAPH_XML_COUNT_BUDGET_REPLAY_SCHEMA\n"
    "            )\n"
    "    }\n",
    "",
    "legacy replay classifier",
)
text = replace_once(
    text,
    '            GLOBAL_ACCESS_BUDGET_REPLAY_SCHEMA => Ok("wow-project.live-replay.v13"),\n',
    "",
    "legacy storage schema mapping",
)
text = replace_once(
    text,
    "                GLOBAL_ACCESS_BUDGET_REPLAY_SCHEMA\n"
    "                    | GRAPH_METADATA_BUDGET_REPLAY_SCHEMA\n"
    "                    | GRAPH_XML_COUNT_BUDGET_REPLAY_SCHEMA\n",
    "                GRAPH_METADATA_BUDGET_REPLAY_SCHEMA\n"
    "                    | GRAPH_XML_COUNT_BUDGET_REPLAY_SCHEMA\n",
    "selected component legacy schema guard",
)
path.write_text(text)


# Current fact documentation: describe only the report10 path and rebuild policy.
path, text = load("crates/wow-emmy/FACT_MODEL.md")
text = replace_once(
    text,
    "The former global-access report9 remains an explicit replay-only recipe for\n"
    "retained Project schema13/14/15 archives. It does not collect local constants and\n"
    "keeps report9's identity inputs; new global-access analysis uses report10. Project\n"
    "selects the legacy recipe only from those exact replay schemas, never by retry,\n"
    "hash, or a report-name fallback.\n",
    "Current retained Project replay rebuilds the same report10 recipe; incompatible\n"
    "derived replay records are regenerated from source and configuration.\n",
    "fact model legacy replay paragraph",
)
path.write_text(text)


fixture = Path(
    "crates/wow-service/src/local/platform_input/tests/data/"
    "legacy-global-access-replay13-profile9-805.json"
)
if not fixture.is_file():
    raise SystemExit("legacy report9 fixture is missing before removal")
fixture.unlink()


# This slice must leave no runtime, fixture or instruction source for report9.
for target in Path(".").rglob("*"):
    if not target.is_file() or target.suffix not in {".rs", ".md", ".json"}:
        continue
    content = target.read_text(errors="ignore")
    for forbidden in (
        "FunctionCallReplayRecipe",
        "LegacyGlobalAccessV9",
        "FUNCTION_CALL_GLOBAL_BUDGET_LEGACY_PROFILE",
        "legacy_global_access_profile9",
        "wow-emmy/function-call-facts/9",
        "wow-project/native-project-replay/13",
        "wow-project.live-replay.v13",
        "report9",
        "schema13",
    ):
        if forbidden in content:
            raise SystemExit(f"{forbidden!r} remains in {target}")

if fixture.exists():
    raise SystemExit("legacy report9 fixture survived removal")
