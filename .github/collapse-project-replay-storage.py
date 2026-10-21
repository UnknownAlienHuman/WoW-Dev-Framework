from pathlib import Path
import re

CURRENT = "wow-project.live-replay.current.v1"
OLD = (
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
    "wow-project.live-replay.aceaddon-local-result.v1",
    "wow-project.live-replay.xml-schema-classification.v1",
    "wow-project.live-replay.physical-function-declarations.v1",
    "wow-project.live-replay.xml-schema-classification-and-physical-function-declarations.v1",
    "wow-project.live-replay.physical-function-declaration-syntax-and-body-shape.v1",
    "wow-project.live-replay.xml-schema-classification-and-physical-function-declaration-syntax-and-body-shape.v1",
    "wow-project.live-replay.selected-components.v1",
    "wow-project.live-replay.primary-callables.v1",
    "wow-project.live-replay.native-library-input.v1",
    "wow-project.live-replay.native-api-uses.v1",
)


def require_sub(text: str, pattern: str, replacement: str, label: str) -> str:
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.DOTALL)
    if count != 1:
        raise SystemExit(f"{label}: expected one match, found {count}")
    return updated


# One stored-record schema; replay configuration remains the semantic selector.
path = Path("crates/wow-project/src/replay/mod.rs")
text = path.read_text()
text, removed = re.subn(
    r'pub\(super\) const [A-Z0-9_]+_STORAGE_SCHEMA: &str =\n\s+"wow-project\.live-replay\.[^"]+";\n',
    "",
    text,
)
if removed != 10:
    raise SystemExit(f"replay storage constants: expected 10 removals, found {removed}")
needle = "const MAX_FILES: usize = 8192;\n"
if text.count(needle) != 1:
    raise SystemExit("MAX_FILES insertion point changed")
text = text.replace(
    needle,
    f'pub(super) const REPLAY_STORAGE_SCHEMA: &str = "{CURRENT}";\n{needle}',
    1,
)
text = require_sub(
    text,
    r"    fn storage_schema\(&self\) -> ProjectResult<&'static str> \{.*?\n    \}\n\}\n/// A finite recipe matrix",
    "    fn storage_schema(&self) -> ProjectResult<&'static str> {\n"
    "        if self.schema != self.expected_schema()? {\n"
    "            return Err(invalid());\n"
    "        }\n"
    "        Ok(REPLAY_STORAGE_SCHEMA)\n"
    "    }\n"
    "}\n"
    "/// A finite recipe matrix",
    "replay storage mapping",
)
path.write_text(text)


path = Path("crates/wow-project/src/replay/publication.rs")
text = path.read_text()
text = require_sub(
    text,
    r"pub const STORAGE_SCHEMAS: &\[&str\] = &\[.*?\n\];\n"
    r"pub const STORAGE_CHECK: &str = \"wow-project\.live-pair-native-replay\.v1\";",
    "pub const STORAGE_SCHEMAS: &[&str] = &[\n"
    "    super::REPLAY_STORAGE_SCHEMA,\n"
    "    \"wow-project.live-pair.v1\",\n"
    "];\n"
    "pub const STORAGE_CHECK: &str = \"wow-project.live-pair-native-replay.v1\";",
    "publication catalog",
)
text = require_sub(
    text,
    r"        let replay_schema = read\n"
    r"            \.manifest\(\)\n"
    r"            \.members\n"
    r"            \.iter\(\)\n"
    r"            \.find\(\|member\| member\.key == REPLAY_KEY\)\n"
    r"            \.map\(\|member\| member\.schema\.as_str\(\)\)\n"
    r"            \.ok_or_else\(invalid\)\?;\n"
    r"        if !matches!\(.*?\n"
    r"        \}\n"
    r"        let replay: ProjectReplay = load\(read, REPLAY_KEY, replay_schema, stop\)\?;",
    "        let replay_schema = read\n"
    "            .manifest()\n"
    "            .members\n"
    "            .iter()\n"
    "            .find(|member| member.key == REPLAY_KEY)\n"
    "            .map(|member| member.schema.as_str())\n"
    "            .ok_or_else(invalid)?;\n"
    "        if replay_schema != super::REPLAY_STORAGE_SCHEMA {\n"
    "            return Err(invalid().into());\n"
    "        }\n"
    "        let replay: ProjectReplay = load(read, REPLAY_KEY, replay_schema, stop)?;",
    "replay storage read admission",
)
path.write_text(text)


# Update tests and active documentation. The payload's own replay schema remains
# unchanged; only the physical ProjectStore record schema is collapsed.
for path in Path(".").rglob("*"):
    if not path.is_file() or path.parts[0] == ".github":
        continue
    if path.suffix not in {".rs", ".md", ".json"}:
        continue
    text = path.read_text(errors="ignore")
    updated = text
    for old in OLD:
        updated = updated.replace(old, CURRENT)
    if updated != text:
        path.write_text(updated)


# Fail closed: no former physical replay schema or named storage constant may survive.
for path in Path(".").rglob("*"):
    if not path.is_file() or path.parts[0] == ".github":
        continue
    if path.suffix not in {".rs", ".md", ".json"}:
        continue
    text = path.read_text(errors="ignore")
    for old in OLD:
        if old in text:
            raise SystemExit(f"obsolete replay storage schema remains in {path}: {old}")
    if re.search(r"[A-Z0-9_]+_STORAGE_SCHEMA", text) and "CANDIDATE_STORAGE_SCHEMA" not in text:
        # Other domains own storage constants; only removed Project replay names matter.
        forbidden = (
            "ACEADDON_LOCAL_RESULT_STORAGE_SCHEMA",
            "XML_CLASSIFICATION_STORAGE_SCHEMA",
            "PHYSICAL_FUNCTION_DECLARATIONS_STORAGE_SCHEMA",
            "XML_CLASSIFICATION_PHYSICAL_FUNCTION_DECLARATIONS_STORAGE_SCHEMA",
            "PHYSICAL_FUNCTION_BODY_SHAPE_STORAGE_SCHEMA",
            "XML_CLASSIFICATION_PHYSICAL_FUNCTION_BODY_SHAPE_STORAGE_SCHEMA",
            "SELECTED_COMPONENTS_STORAGE_SCHEMA",
            "PRIMARY_CALLABLES_STORAGE_SCHEMA",
            "NATIVE_LIBRARY_INPUT_STORAGE_SCHEMA",
            "NATIVE_API_USES_STORAGE_SCHEMA",
        )
        for name in forbidden:
            if name in text:
                raise SystemExit(f"obsolete Project replay storage constant remains in {path}: {name}")

publication = Path("crates/wow-project/src/replay/publication.rs").read_text()
if publication.count(CURRENT) != 0:
    raise SystemExit("publication must use the parent constant, not duplicate the schema literal")
replay = Path("crates/wow-project/src/replay/mod.rs").read_text()
if replay.count(CURRENT) != 1:
    raise SystemExit("current Project replay storage schema must have one owner")
