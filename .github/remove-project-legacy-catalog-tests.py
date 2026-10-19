from pathlib import Path


def load(path: str) -> str:
    return Path(path).read_text()


def save(path: str, text: str) -> None:
    Path(path).write_text(text)


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if text.count(old) != 1:
        raise SystemExit(f"{label}: expected one match, found {text.count(old)}")
    return text.replace(old, new)


def cut_once(text: str, start: str, end: str, replacement: str, label: str) -> str:
    start_at = text.find(start)
    if start_at < 0:
        raise SystemExit(f"{label}: start not found")
    end_at = text.find(end, start_at)
    if end_at < 0:
        raise SystemExit(f"{label}: end not found")
    return text[:start_at] + replacement + text[end_at + len(end):]


def remove_test(text: str, name: str) -> str:
    marker = f"#[test]\nfn {name}("
    start = text.find(marker)
    if start < 0:
        raise SystemExit(f"test {name}: start not found")
    next_test = text.find("\n#[test]\n", start + len(marker))
    if next_test < 0:
        return text[:start].rstrip() + "\n"
    return text[:start] + text[next_test + 1:]


# Project test: keep current selected/current catalog behavior, remove only old V4 assertion.
path = "crates/wow-project/tests/primary_callable_replay.rs"
text = load(path)
text = replace_once(
    text,
    "    assert!(!publication::STORAGE_SCHEMAS_V4.contains(&PRIMARY_STORAGE));\n",
    "",
    "primary callable V4 assertion",
)
save(path, text)

# Service live-project tests: remove old exact-catalog admission/reopen coverage.
path = "crates/wow-service/src/live_project/tests.rs"
text = load(path)
text = cut_once(
    text,
    '    let legacy_root = root.join("legacy-v2");\n',
    "    drop(legacy);\n",
    "",
    "package V2 refusal",
)
text = cut_once(
    text,
    "    // The public service rejection uses the exact platform owner in both epochs.\n",
    "    drop(legacy);\n    std::fs::remove_dir_all(path)?;",
    "    std::fs::remove_dir_all(path)?;",
    "platform V4 compatibility blocks",
)
text = remove_test(text, "original_v4_and_v15_catalogs_reopen_exactly_without_relabeling")
text = remove_test(text, "original_epochs_reopen_exactly_and_refuse_new_generation_recipe")
save(path, text)

# Namespace tests: preserve current publication/reopen behavior and drop frozen-epoch comparisons.
path = "crates/wow-service/src/live_project/tests/namespaces.rs"
text = load(path)
text = replace_once(
    text,
    "fn raw_inventory_reopens_with_exact_byte_binding_and_frozen_v7_refuses() -> TestResult {",
    "fn raw_inventory_reopens_with_exact_byte_binding() -> TestResult {",
    "raw inventory test name",
)
text = cut_once(
    text,
    "    let legacy_bundle = platform_bundle_with_profiles(\n        &path.join(\"legacy-input\"),\n",
    "    drop(legacy);\n    std::fs::remove_dir_all(path)?;",
    "    std::fs::remove_dir_all(path)?;",
    "raw inventory frozen V7 block",
)
text = replace_once(
    text,
    "fn selected_package_bindings_reopen_exactly_and_frozen_v5_refuses_without_effects() -> TestResult {",
    "fn selected_package_bindings_reopen_exactly() -> TestResult {",
    "package binding test name",
)
text = replace_once(
    text,
    '    let bundle = platform_bundle(&path.join("legacy-input"), 1, &stop)?;\n',
    "",
    "package binding legacy bundle",
)
text = cut_once(
    text,
    "    let (legacy_owner, legacy_graph) = owners_from_bundle(bundle)?;\n",
    "    drop(legacy);\n    std::fs::remove_dir_all(path)?;",
    "    std::fs::remove_dir_all(path)?;",
    "package binding frozen V5 block",
)
text = replace_once(
    text,
    "fn selected_package_graph_reopens_and_frozen_v6_refuses_without_effects() -> TestResult {",
    "fn selected_package_graph_reopens_exactly() -> TestResult {",
    "package graph test name",
)
text = replace_once(
    text,
    '    let legacy_bundle = platform_bundle_with_bindings(&path.join("legacy-input"), 1, true, &stop)?;\n',
    "",
    "package graph legacy bundle",
)
text = replace_once(
    text,
    '    assert!(!path.join("legacy-input").exists());\n',
    "",
    "package graph legacy path assertion",
)
text = cut_once(
    text,
    "    let (legacy_owner, legacy_graph) = owners_from_bundle(legacy_bundle)?;\n",
    "    let input = || -> TestResult<crate::LocalProjectInput> {",
    "    let input = || -> TestResult<crate::LocalProjectInput> {",
    "package graph legacy owner",
)
text = cut_once(
    text,
    '    let legacy_path = path.join("frozen-v6");\n',
    "    drop(legacy);\n    std::fs::remove_dir_all(path)?;",
    "    std::fs::remove_dir_all(path)?;",
    "package graph frozen V6 block",
)
text = cut_once(
    text,
    '    let legacy_path = path.join("legacy-v4");\n',
    "    drop(legacy);\n    std::fs::remove_dir_all(path)?;",
    "    std::fs::remove_dir_all(path)?;",
    "namespace legacy V4 block",
)
save(path, text)

# Direct-service test: remove previous-catalog replay/refusal half of each current scenario.
path = "crates/wow-service/src/live_project/tests/namespaces/direct_service.rs"
text = load(path)
text = replace_once(
    text,
    "        let (direct_publication, _) =\n            request.live_publication_in_namespace(input(&bundle)?, selection.namespace(), &stop)?;\n",
    "",
    "unused direct publication",
)
text = cut_once(
    text,
    '        let legacy_source_path = path.join("legacy-input");\n',
    "        assert_eq!(read.publication_set_id(), old_set);\n",
    "",
    "direct-service frozen V8/V9 block",
)
save(path, text)

# Budget tests: delete the frozen schema13/14/15 matrix and old V10 store-only block.
path = "crates/wow-service/src/local/platform_input/tests/project_budgets.rs"
text = load(path)
start = text.find("#[test]\nfn schema13_profile9_and_report10_schema16_replay_preserve_frozen_store()")
helper = text.find("fn selected_budget_replay_and_frozen_store_preserve_current(", start)
next_test = text.find("\n#[test]\n", helper)
if min(start, helper, next_test) < 0:
    raise SystemExit("frozen budget test group not found")
text = text[:start] + text[next_test + 1:]
text = cut_once(
    text,
    "    // A real previous epoch must refuse the new record without changing Current,\n",
    "    let store_root = root.0.join(\"selected-store\");\n",
    "    let store_root = root.0.join(\"selected-store\");\n",
    "sidecar V10 store block",
)
save(path, text)

# No source file in this slice may retain a removed catalog constant.
for path in [
    "crates/wow-project/tests/primary_callable_replay.rs",
    "crates/wow-service/src/live_project/tests.rs",
    "crates/wow-service/src/live_project/tests/namespaces.rs",
    "crates/wow-service/src/live_project/tests/namespaces/direct_service.rs",
    "crates/wow-service/src/local/platform_input/tests/project_budgets.rs",
]:
    if "STORAGE_SCHEMAS_V" in load(path):
        raise SystemExit(f"legacy catalog reference remains in {path}")
