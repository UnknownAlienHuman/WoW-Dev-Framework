from pathlib import Path
import re


def read(path: str) -> str:
    return Path(path).read_text()


def write(path: str, text: str) -> None:
    Path(path).write_text(text)


def replace_exact(path: str, old: str, new: str, count: int = 1) -> None:
    text = read(path)
    found = text.count(old)
    if found != count:
        raise SystemExit(f"{path}: expected {count}, found {found}: {old[:100]!r}")
    write(path, text.replace(old, new))


def remove_function(path: str, signature: str) -> None:
    text = read(path)
    start = text.find(signature)
    if start < 0:
        raise SystemExit(f"{path}: missing {signature}")
    brace = text.find("{", start)
    if brace < 0:
        raise SystemExit(f"{path}: missing function body for {signature}")
    depth = 0
    end = None
    for index in range(brace, len(text)):
        char = text[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                end = index + 1
                break
    if end is None:
        raise SystemExit(f"{path}: unterminated {signature}")
    while end < len(text) and text[end] == "\n":
        end += 1
    write(path, text[:start] + text[end:])


# CLI: remove the standalone two-generation command without touching ordinary
# Current/Exact single-generation reads.
replace_exact("apps/wow/src/project.rs", "mod pair;\n", "")
replace_exact(
    "apps/wow/src/project.rs",
    '    "wow project pair --store-root <existing-ordinary-store> --before-generation <exact-id> --after-generation <exact-id> [--max-output-bytes <bound>] [--format json]\\n",\n',
    "",
)
replace_exact(
    "apps/wow/src/project.rs",
    '''    if values.get(1).is_some_and(|value| value == "pair") {
        return pair::run(values);
    }
''',
    "",
)

# Service transport and its exclusive test module.
replace_exact("crates/wow-service/src/live_project.rs", "mod generation_pair;\n", "")
replace_exact(
    "crates/wow-service/src/live_project.rs",
    '''pub use generation_pair::{
    PROJECT_GENERATION_PAIR_MAX_OUTPUT_BYTES, ProjectGenerationPairRequest,
    ProjectGenerationPairResult, read_live_project_generation_pair,
};
''',
    "",
)
replace_exact(
    "crates/wow-service/src/live_project/tests.rs",
    "mod generation_pair;\n",
    "",
)

# Project pair owners and their source-only sibling were never consumed by the
# retained single-generation reader.
publication = "crates/wow-project/src/replay/publication.rs"
replace_exact(
    publication,
    '''mod generation_pair;
mod source_generation_pair;
pub use source_generation_pair::*;
''',
    "",
)
replace_exact(
    publication,
    '''pub use generation_pair::{
    RETAINED_GENERATION_PAIR_PROFILE, RetainedGenerationPair, RetainedGenerationPairDescriptor,
};
''',
    "",
)
replace_exact(
    publication,
    "    read_origin: wow_store::project::ReadSnapshotOrigin,\n",
    "",
)
replace_exact(
    publication,
    "                read_origin: read.origin(),\n",
    "",
)
remove_function(publication, "fn validate_read_native_parts(")

replace_exact(
    "crates/wow-project/src/error.rs",
    "    IncompatibleGenerationPair,\n",
    "",
)

# Remove the process-local read-custody proof that existed solely to bind two
# simultaneous historical reads. The actual transaction and lease remain.
read_path = "crates/wow-store/src/project/read.rs"
store_read = read(read_path)
start = store_read.find("/// Nonserializable origin of a native read.")
end = store_read.find("#[derive(Debug, Clone)]\npub enum ReadSelector", start)
if start < 0 or end <= start:
    raise SystemExit("read.rs: missing pair-only custody block")
store_read = store_read[:start] + store_read[end:]
store_read = store_read.replace(
    "    sync::{Arc, Weak, atomic::AtomicBool},\n",
    "    sync::atomic::AtomicBool,\n",
)
store_read = store_read.replace("    custody: Arc<()>,\n", "")
store_read = store_read.replace("            custody: Arc::new(()),\n", "")
write(read_path, store_read)
remove_function(read_path, "    pub fn origin(")
remove_function(read_path, "    pub fn custody(")
replace_exact(
    "crates/wow-store/src/project/mod.rs",
    '''pub use read::{
    ReadSelector, ReadSnapshot, ReadSnapshotCustody, ReadSnapshotOrigin, ValidatedRead,
};
''',
    "pub use read::{ReadSelector, ReadSnapshot, ValidatedRead};\n",
)
replace_exact(
    "crates/wow-store/src/project/database.rs",
    "    pub custody: std::sync::Arc<()>,\n",
    "",
)

for path in Path("crates/wow-store/src/project").rglob("*.rs"):
    text = path.read_text()
    new, count = re.subn(
        r"^\s*custody: std::sync::Arc::new\(\(\)\),\n",
        "",
        text,
        flags=re.MULTILINE,
    )
    if count:
        path.write_text(new)

# Remove the user-facing manual section, leaving the selected/current routes.
live_doc = "apps/wow/LIVE_PROJECT.md"
text = read(live_doc)
start = text.find("## Exact retained generation pair\n")
end = text.find("## Selected native candidate and first Skeleton profile\n", start)
if start < 0 or end <= start:
    raise SystemExit("LIVE_PROJECT.md: missing generation-pair section")
write(live_doc, text[:start] + text[end:])
