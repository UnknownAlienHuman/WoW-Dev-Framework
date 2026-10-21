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


def sub_once(text: str, pattern: str, new: str, label: str) -> str:
    updated, count = re.subn(pattern, new, text, count=1, flags=re.DOTALL)
    if count != 1:
        raise SystemExit(f"{label}: expected one regex match, found {count}")
    return updated


def remove_macro_statements(text: str, macro: str, predicate) -> str:
    needle = macro + "!("
    cursor = 0
    removals: list[tuple[int, int]] = []
    while True:
        start = text.find(needle, cursor)
        if start < 0:
            break
        open_paren = start + len(macro) + 1
        depth = 0
        i = open_paren
        state = "code"
        escaped = False
        while i < len(text):
            ch = text[i]
            nxt = text[i + 1] if i + 1 < len(text) else ""
            if state == "string":
                if escaped:
                    escaped = False
                elif ch == "\\":
                    escaped = True
                elif ch == '"':
                    state = "code"
            elif state == "char":
                if escaped:
                    escaped = False
                elif ch == "\\":
                    escaped = True
                elif ch == "'":
                    state = "code"
            elif state == "line_comment":
                if ch == "\n":
                    state = "code"
            elif state == "block_comment":
                if ch == "*" and nxt == "/":
                    state = "code"
                    i += 1
            else:
                if ch == '"':
                    state = "string"
                elif ch == "'":
                    state = "char"
                elif ch == "/" and nxt == "/":
                    state = "line_comment"
                    i += 1
                elif ch == "/" and nxt == "*":
                    state = "block_comment"
                    i += 1
                elif ch == "(":
                    depth += 1
                elif ch == ")":
                    depth -= 1
                    if depth == 0:
                        end = i + 1
                        while end < len(text) and text[end] in " \t":
                            end += 1
                        if end < len(text) and text[end] == ";":
                            end += 1
                        segment = text[start:end]
                        if predicate(segment):
                            line_start = text.rfind("\n", 0, start) + 1
                            line_end = end
                            if line_end < len(text) and text[line_end] == "\n":
                                line_end += 1
                            removals.append((line_start, line_end))
                        cursor = end
                        break
            i += 1
        else:
            raise SystemExit(f"unterminated {macro}! macro")
    for start, end in reversed(removals):
        text = text[:start] + text[end:]
    return text


# One data shape and one physical replay record. Configuration/owner rebuild owns
# semantic admission; the DTO is not a second feature/version registry.
path, text = read("crates/wow-project/src/replay/mod.rs")
text = once(
    text,
    "    PackageXmlBindingProfile, PlatformGraphProfile, ProjectError, ProjectErrorCode,\n",
    "    ProjectError, ProjectErrorCode,\n",
    "replay imports",
)
text = sub_once(
    text,
    r"const LIBRARY_BOUND_REPLAY_SCHEMA:.*?const MAX_FILES:",
    'pub(super) const STORAGE_SCHEMA: &str = "wow-project.live-replay";\nconst MAX_FILES:',
    "replay/storage schema ladder",
)
text = once(text, "    schema: String,\n", "", "replay DTO schema field")
text = sub_once(
    text,
    r"        // The package graph projection is selected only together with the.*?\n"
    r"        let platform = if let Some\(owner\) = platform_owner \{",
    "        let platform = if let Some(owner) = platform_owner {",
    "capture schema selection",
)
text = once(text, "                    schema,\n", "", "platform envelope schema value")
text = once(text, "            schema: schema.into(),\n", "", "replay schema construction")
text = sub_once(
    text,
    r"    fn expected_schema\(&self\) -> ProjectResult<&'static str> \{.*?\n"
    r"    fn validate_budget\(&self, stop: &AtomicBool\) -> ProjectResult<\(\)> \{",
    "    fn validate_budget(&self, stop: &AtomicBool) -> ProjectResult<()> {\n        self.configuration.has_xml_classification()?;",
    "expected schema ladder",
)
text = once(
    text,
    "        let expected_schema = self.expected_schema()?;\n",
    "",
    "expected schema local",
) if "        let expected_schema = self.expected_schema()?;\n" in text else text
text = once(
    text,
    "            || self.schema != expected_schema\n",
    "",
    "schema equality gate",
) if "            || self.schema != expected_schema\n" in text else text
text = sub_once(
    text,
    r"    fn storage_schema\(&self\) -> ProjectResult<&'static str> \{.*?\n"
    r"    \}\n"
    r"\}\n"
    r"/// A finite recipe matrix.*?\n"
    r"/// Same envelope shape",
    "    fn storage_schema(&self) -> &'static str {\n        STORAGE_SCHEMA\n    }\n}\n/// Same envelope shape",
    "storage/component schema dispatch",
)
text = once(text, "    schema: &'a str,\n", "", "envelope schema field")
path.write_text(text)

# The one catalog contains the one replay record plus the coherent pair record.
path, text = read("crates/wow-project/src/replay/publication.rs")
text = sub_once(
    text,
    r"pub const STORAGE_SCHEMAS: &\[&str\] = &\[.*?\n\];",
    'pub const STORAGE_SCHEMAS: &[&str] = &[super::STORAGE_SCHEMA, "wow-project.live-pair.v1"];',
    "unified replay catalog contents",
)
text = once(
    text,
    "    let storage_schema = replay.storage_schema()?;\n",
    "    let storage_schema = replay.storage_schema();\n",
    "infallible replay storage schema",
)
path.write_text(text)

# Private replay-configuration getters used only by the removed tag-selection
# matrix are not another API surface.
path, text = read("crates/wow-project/src/replay/configuration.rs")
for pattern, label in [
    (r"    pub\(super\) const fn signal_witnesses.*?\n    \}\n", "signal witness getter"),
    (r"    pub\(super\) const fn library_witnesses.*?\n    \}\n", "library witness getter"),
    (r"    pub\(super\) const fn state_projection.*?\n    \}\n", "state projection getter"),
    (r"    pub\(super\) const fn native_api_uses.*?\n    \}\n", "native API getter"),
    (r"    pub\(super\) const fn physical_function_signatures.*?\n    \}\n", "signature getter"),
    (r"    pub\(super\) const fn physical_function_declarations.*?\n    \}\n", "declaration getter"),
    (r"    pub\(super\) const fn source_graph_budget.*?\n    \}\n", "source graph getter"),
    (r"    pub\(super\) const fn function_call_budget.*?\n    \}\n", "function budget getter"),
    (r"    pub\(super\) const fn function_call_evidence_profile.*?\n    \}\n", "evidence getter"),
    (r"    pub\(super\) const fn hook_target_recipe.*?\n    \}\n", "hook getter"),
    (r"    pub\(super\) fn package_xml_binding_profile.*?\n    \}\n", "package XML getter"),
    (r"    pub\(super\) fn platform_graph_profile.*?\n    \}\n", "platform graph getter"),
]:
    text, count = re.subn(pattern, "", text, count=1, flags=re.DOTALL)
    if count != 1:
        raise SystemExit(f"{label}: expected one match, found {count}")
path.write_text(text)

# Remove assertions that certified only the deleted replay tag. Negative mutation
# cases keep an unknown field so strict decode still proves old records are refused.
for target in Path(".").rglob("*.rs"):
    body = target.read_text()
    body = remove_macro_statements(
        body,
        "assert_eq",
        lambda segment: '["schema"]' in segment
        and (
            "wow-project/native-project-replay/" in segment
            or "PRIMARY_REPLAY" in segment
            or "REPLAY_SCHEMA" in segment
        ),
    )
    body = remove_macro_statements(
        body,
        "assert_ne",
        lambda segment: '["schema"]' in segment
        and "wow-project/native-project-replay/" in segment,
    )
    lines = []
    for line in body.splitlines(keepends=True):
        if "wow-project/native-project-replay/" in line and '["schema"]' in line:
            line = line.replace('["schema"]', '["obsolete_schema"]')
        lines.append(line)
    body = "".join(lines)
    body = re.sub(
        r'"wow-project/native-project-replay/[^"\\]*"',
        '"obsolete-project-replay-schema"',
        body,
    )
    body = re.sub(
        r'"wow-project\.live-replay(?:\.[^"\\]*)?"',
        '"wow-project.live-replay"',
        body,
    )
    body = re.sub(
        r'^const\s+[A-Z0-9_]*REPLAY[A-Z0-9_]*:\s*&str\s*=\s*"obsolete-project-replay-schema";\n',
        "",
        body,
        flags=re.MULTILINE,
    )
    target.write_text(body)

# One obsolete-schema regression remains explicit and does not depend on a
# historical value. It must fail strict ProjectReplay decoding.
path, text = read("crates/wow-project/tests/project_e0.rs")
text = text.replace(
    'obsolete["obsolete_schema"] = "obsolete-project-replay-schema".into();',
    'obsolete["obsolete_schema"] = true.into();',
)
text = text.replace(
    "    assert!(\n        serde_json::from_value::<ProjectReplay>(obsolete)?\n            .hydrate(&stop)\n            .is_err()\n    );\n",
    "    assert!(serde_json::from_value::<ProjectReplay>(obsolete).is_err());\n",
)
path.write_text(text)

# Current documentation names one DTO/record. Historical matrices remain in Git.
Path("crates/wow-project/PRIMARY_CALLABLE_REPLAY.md").write_text('''# Ordinary Primary callable replay

Project replay retains exact Main and Library files plus the selected declaration,
signature, hook, witness and native-input configuration. Every current replay uses
the same `ProjectReplay` DTO and `wow-project.live-replay` store record.

Feature choices are not encoded in a schema name. Hydration rebuilds
`ProjectConfiguration`, Emmy analysis and graph owners, verifies the captured
configuration and semantic identities, then compares the complete recaptured DTO.
A different selection is rejected by those owners.

Old derived records are rebuilt from source and configuration. There is no tag
ladder, compatibility reader or migration path. These source observations do not
claim runtime behavior. Broader context work is tracked by
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122).
''')

Path("docs/PLATFORM_SOURCE_REPLAY.md").write_text('''# Platform source replay

`ProjectReplay` retains the exact source/package inputs needed to rebuild a current
Project without reopening the source checkout. It stores raw member bytes, selected
TOC/package plans, Main and Library files, configuration and expected semantic IDs.
It never executes source code or treats serialized receipts as authority.

## Read path

1. Bound and decode one strict DTO.
2. Rebuild source/package/configuration owners from retained bytes.
3. Run the real Emmy/Project producer path.
4. Compare configuration, Project, analyzer and replay identities.
5. Admit the stored Project/graph pair only after those checks succeed.

All current feature combinations use one DTO and one
`wow-project.live-replay` record. XML classification, declarations, signatures,
function-call budgets, hooks and other selections remain typed configuration;
they do not create storage or replay versions. The composed candidate receipt is
a separate record because it is a separate owner.

Replay is bounded by the current file, source-byte and store-record ceilings.
Cancellation is checked during capture, decoding and rebuild. Incompatible derived
stores are recreated from source and user configuration; no migration or archive
reader is maintained.
''')

# Remove active documentation references to the deleted tag ladder. Keep wording
# compact; the owner/configuration, not a renamed tag, explains each capability.
for target in Path(".").rglob("*.md"):
    body = target.read_text()
    body = re.sub(r"`wow-project/native-project-replay/[^`]+`", "`ProjectReplay`", body)
    body = re.sub(r"`wow-project\.live-replay[^`]*`", "`wow-project.live-replay`", body)
    target.write_text(body)

path, text = read("docs/IMPLEMENTATION.md")
text = text.replace(
    "Payload schema tags remain to be collapsed only after tracing their real readers.",
    "All current payloads now use one strict ProjectReplay DTO and one store record.",
)
path.write_text(text)

path, text = read("docs/REFACTOR_QUEUE.md")
text = text.replace(
    "Historical and feature-combination catalogs are removed; collapse remaining replay payload tags only from real readers.",
    "Historical catalogs, feature catalogs and replay tag ladders are removed; keep one rebuildable ProjectReplay.",
)
path.write_text(text)

# Fail closed on source-level replay/version ladders. Other domains may still own
# unrelated schemas and versions.
for target in Path(".").rglob("*"):
    if not target.is_file() or target.suffix not in {".rs", ".md", ".json"}:
        continue
    body = target.read_text(errors="ignore")
    for forbidden in (
        "wow-project/native-project-replay/",
        "wow-project.live-replay.v",
        "wow-project.live-replay.aceaddon",
        "wow-project.live-replay.xml",
        "wow-project.live-replay.physical",
        "wow-project.live-replay.selected",
        "wow-project.live-replay.primary",
        "wow-project.live-replay.native",
        "expected_schema(&self)",
        "expected_base_schema(&self)",
        "selected_component_schema(",
        "selected_component_base_schema(",
        "graph_budget_schema(",
    ):
        if forbidden in body:
            raise SystemExit(f"{forbidden!r} remains in {target}")
