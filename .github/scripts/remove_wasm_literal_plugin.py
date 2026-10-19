from pathlib import Path
import re
import shutil


def read(path: str) -> str:
    return Path(path).read_text()


def write(path: str, text: str) -> None:
    Path(path).write_text(text)


def replace_exact(path: str, old: str, new: str, count: int = 1) -> None:
    text = read(path)
    found = text.count(old)
    if found != count:
        raise SystemExit(f"{path}: expected {count}, found {found}: {old[:80]!r}")
    write(path, text.replace(old, new))


def remove_test(path: str, name: str) -> None:
    text = read(path)
    pos = text.find(f"fn {name}")
    start = text.rfind("#[test]", 0, pos)
    brace = text.find("{", pos)
    if min(pos, start, brace) < 0:
        raise SystemExit(f"{path}: missing test {name}")
    depth = 0
    end = None
    for index in range(brace, len(text)):
        if text[index] == "{":
            depth += 1
        elif text[index] == "}":
            depth -= 1
            if depth == 0:
                end = index + 1
                break
    if end is None:
        raise SystemExit(f"{path}: unterminated test {name}")
    while end < len(text) and text[end] == "\n":
        end += 1
    write(path, text[:start] + text[end:])


shutil.copyfile(
    "crates/wow-render-contract/src/types.rs",
    "crates/wow-ketho-literals/src/types.rs",
)

cargo = read("Cargo.toml")
for member in [
    '    "crates/wow-render-contract",\n',
    '    "modules/ketho-literals",\n',
]:
    if cargo.count(member) != 1:
        raise SystemExit(f"Cargo.toml: unexpected member {member!r}")
    cargo = cargo.replace(member, "")
cargo = cargo.replace('\nexclude = ["bridges/literal-host"]\n', "\n")
profile = cargo.find("\n[profile.wasm-compact]")
if profile < 0:
    raise SystemExit("Cargo.toml: missing Wasm profiles")
write("Cargo.toml", cargo[:profile].rstrip() + "\n")

replace_exact(
    "crates/wow-annotations/Cargo.toml",
    'wow-render-contract = { path = "../wow-render-contract" }\n',
    "",
)
replace_exact(
    "crates/wow-ketho-literals/Cargo.toml",
    '[dependencies]\nwow-render-contract = { path = "../wow-render-contract" }\n',
    '[dependencies]\nserde = { version = "1.0", features = ["derive"] }\n',
)

original = read("crates/wow-ketho-literals/src/lib.rs")
body_start = original.find("const MAX_MEMBERS")
bridge_impl = original.find("\nimpl LiteralBridge for LiteralRenderer")
if body_start < 0 or bridge_impl <= body_start:
    raise SystemExit("wow-ketho-literals: unexpected source layout")
body = original[body_start:bridge_impl].rstrip() + "\n"
header = '''//! Native Ketho literal renderer for events, CVars, enums and constants.
//! No source acquisition, VM, wire protocol, IO or editor state.
mod support;
mod types;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use support::*;
pub use types::*;

pub const MAX_OUTPUT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_TEXT_BYTES: usize = 64 * 1024;
pub const MAX_ITEMS: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LiteralError {
    InvalidIdentifier,
    UnsafeDocumentation,
    DuplicateName,
    InputLimit,
    OutputLimit,
    UnsupportedLiteral,
}

impl std::fmt::Display for LiteralError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "literal rendering failed: {self:?}")
    }
}

impl std::error::Error for LiteralError {}

'''
write("crates/wow-ketho-literals/src/lib.rs", header + body)
replace_exact(
    "crates/wow-ketho-literals/src/support.rs",
    "use wow_render_contract::{LiteralError, MAX_TEXT_BYTES};",
    "use crate::{LiteralError, MAX_TEXT_BYTES};",
)

replace_exact("crates/wow-annotations/src/lib.rs", "pub mod selected_literals;\n", "")
replace_exact(
    "crates/wow-annotations/src/literals.rs",
    "//! Compatibility bridge to the extracted Ketho algorithm. No duplicate emitter.\n//! This facade preserves existing owner APIs and error classifications.\n",
    "//! Thin native facade over the Ketho literal algorithm. No duplicate emitter.\n//! This preserves the annotation owner's errors and public types.\n",
)
literals_path = "crates/wow-annotations/src/literals.rs"
literals = read(literals_path)
literals, changed = re.subn(
    r"\n\s*LiteralError::InvalidWire\n\s*\| LiteralError::IncompatibleSchema\n\s*\| LiteralError::BridgeFailure => RenderError::BridgeFailure,",
    "",
    literals,
    count=1,
)
if changed != 1:
    raise SystemExit("literals.rs: missing bridge-only error arm")
write(literals_path, literals)

native_path = "crates/wow-annotations/src/native.rs"
native = read(native_path)
old_imports = '''use crate::literals::{
    ConstantGroup, EnumDeclaration, EventLiteral, IntegerFormat, LiteralMember, LiteralValue,
    MemberOrder,
};

use crate::selected_literals::{LiteralExecution, SelectedLiterals};
use wow_render_contract::LiteralBridge;
'''
new_imports = '''use crate::literals::{
    ConstantGroup, EnumDeclaration, EventLiteral, IntegerFormat, LiteralMember, LiteralRenderer,
    LiteralValue, MemberOrder,
};
'''
if native.count(old_imports) != 1:
    raise SystemExit("native.rs: unexpected imports")
native = native.replace(old_imports, new_imports)
native = native.replace(
    '''    #[serde(skip_serializing_if = "Option::is_none")]
    pub literal_execution: Option<LiteralExecution>,
''',
    "",
)
old_alias_call = '''    project_with_literal_bridge(
        documents,
        environment,
        corrections,
        aliases,
        None,
        cancelled,
    )
'''
new_alias_call = '''    project_with_alias_catalogs(
        documents,
        environment,
        corrections,
        aliases.as_slice(),
        cancelled,
    )
'''
if native.count(old_alias_call) != 1:
    raise SystemExit("native.rs: missing bridge wrapper call")
native = native.replace(old_alias_call, new_alias_call)
start = native.find("/// Select one retained literal implementation")
end = native.find("/// Compose explicitly selected type resources")
if start < 0 or end <= start:
    raise SystemExit("native.rs: missing bridge wrapper")
native = native[:start] + native[end:]
start = native.find("/// Compose explicitly selected type resources")
end = native.find("/// Multi-resource projection using one retained literal implementation.")
if start < 0 or end <= start:
    raise SystemExit("native.rs: missing forwarding alias wrapper")
native = native[:start] + native[end:]
old_owner = '''/// Multi-resource projection using one retained literal implementation.
/// Catalog resources are sorted by path, must share one exact external revision,
/// and cannot repeat a path. The slice itself is not retained in the result.
/// This adds alias-projection v3 inside the existing native v5/v6 envelope;
/// every resource keeps its own v1/v2 identity, raw bytes and declaration spans.
pub fn project_with_alias_catalogs_and_literal_bridge<'a>(
    documents: &'a [DocumentationDocument],
    environment: &str,
    corrections: Option<&'a ValidatedCorrections>,
    aliases: &[&'a wow_reference::native_aliases::AliasDocument],
    bridge: Option<&dyn LiteralBridge>,
    cancelled: &AtomicBool,
) -> Result<NativeLibrary<'a>, RenderError> {
'''
new_owner = '''/// Compose explicitly selected type resources from one external revision.
/// Catalog resources are sorted by path, must share one exact external revision,
/// and cannot repeat a path. The slice itself is not retained in the result.
/// Alias projection uses the native v5 report; every resource keeps its own
/// identity, raw bytes and declaration spans.
pub fn project_with_alias_catalogs<'a>(
    documents: &'a [DocumentationDocument],
    environment: &str,
    corrections: Option<&'a ValidatedCorrections>,
    aliases: &[&'a wow_reference::native_aliases::AliasDocument],
    cancelled: &AtomicBool,
) -> Result<NativeLibrary<'a>, RenderError> {
'''
if native.count(old_owner) != 1:
    raise SystemExit("native.rs: unexpected implementation signature")
native = native.replace(old_owner, new_owner)
native = native.replace(
    "let literals = SelectedLiterals::new(bridge, MAX_FILE_BYTES, cancelled)?;",
    "let literals = LiteralRenderer::new(MAX_FILE_BYTES)?;",
)
native = native.replace("        literals.check()?;\n", "")
native = native.replace("    literals.check()?;\n", "")
old_schema = '''    let literal_execution = literals.finish(&files)?;
    Ok(NativeLibrary {
        schema: if literal_execution.is_some() {
            "wow-native-annotation-library/6"
        } else if alias_report.is_some() {
            "wow-native-annotation-library/5"
        } else if corrected.is_some() {
'''
new_schema = '''    Ok(NativeLibrary {
        schema: if alias_report.is_some() {
            "wow-native-annotation-library/5"
        } else if corrected.is_some() {
'''
if native.count(old_schema) != 1:
    raise SystemExit("native.rs: unexpected schema selection")
native = native.replace(old_schema, new_schema)
native = native.replace("        literal_execution,\n", "")
for forbidden in [
    "LiteralBridge",
    "LiteralExecution",
    "SelectedLiterals",
    "literal_execution",
    "project_with_literal_bridge",
    "project_with_alias_catalogs_and_literal_bridge",
    "wow-native-annotation-library/6",
]:
    if forbidden in native:
        raise SystemExit(f"native.rs: {forbidden} remains")
write(native_path, native)

support_path = "crates/wow-annotations/examples/support/mod.rs"
replace_exact(
    support_path,
    '''pub fn run(
    args: Vec<OsString>,
    bridge: Option<&dyn wow_render_contract::LiteralBridge>,
) -> Result<bool, Box<dyn std::error::Error>> {
''',
    "pub fn run(args: Vec<OsString>) -> Result<bool, Box<dyn std::error::Error>> {\n",
)
support = read(support_path).replace(
    "wow_annotations::native::project_with_alias_catalogs_and_literal_bridge(",
    "wow_annotations::native::project_with_alias_catalogs(",
)
if support.count("        bridge,\n") != 1:
    raise SystemExit("example support: missing bridge argument")
write(support_path, support.replace("        bridge,\n", ""))
replace_exact(
    "crates/wow-annotations/examples/native_library.rs",
    "driver::run(std::env::args_os().skip(1).collect(), None)",
    "driver::run(std::env::args_os().skip(1).collect())",
)
for path in [
    "crates/wow-annotations/tests/consumer_support/source.rs",
    "crates/wow-annotations/tests/consumer_support/catalog_source.rs",
]:
    replace_exact(path, "        ],\n        None,\n    )?", "        ],\n    )?")

main_path = "tools/xtask/src/main.rs"
main = read(main_path).replace("mod literal_execution;\n", "")
main = main.replace(
    "cargo xtask verify-library OUTPUT [--require-input-complete] [--literal-module SHA256]",
    "cargo xtask verify-library OUTPUT [--require-input-complete]",
)
start = main.find('        ["verify-library", root, "--literal-module", digest] => {')
end = main.find('        ["verify-library", root] =>', start)
if start < 0 or end < 0:
    raise SystemExit("xtask main: missing literal-module arms")
write(main_path, main[:start] + main[end:])

library_path = "tools/xtask/src/library.rs"
library = read(library_path)
old_header = '''pub fn verify(root: &Path, require_input_complete: bool) -> Result<u8> {
    verify_with_module(root, require_input_complete, None)
}
pub fn verify_with_module(
    root: &Path,
    require_input_complete: bool,
    expected_module: Option<&str>,
) -> Result<u8> {
'''
if library.count(old_header) != 1:
    raise SystemExit("xtask library: unexpected verifier header")
library = library.replace(
    old_header,
    "pub fn verify(root: &Path, require_input_complete: bool) -> Result<u8> {\n",
)
library = library.replace("    crate::literal_execution::verify(library, expected_module)?;\n", "")
library = re.sub(r'\s*\|\s*"wow-native-annotation-library/6"', "", library)
write(library_path, library)

aliases_path = "tools/xtask/src/library/aliases.rs"
aliases = read(aliases_path)
special = '''    if library["schema"] == "wow-native-annotation-library/6" && library.get("aliases").is_none() {
        return Ok(CheckedAliases::default());
    }
'''
if aliases.count(special) != 1:
    raise SystemExit("xtask aliases: missing v6 branch")
aliases = aliases.replace(special, "")
aliases = re.sub(r'\s*\|\s*"wow-native-annotation-library/6"', "", aliases)
write(aliases_path, aliases)
remove_test(
    "tools/xtask/src/tests.rs",
    "selected_library_composes_aliases_corrections_and_required_module",
)

for path in [
    "crates/wow-annotations/src/navigation.rs",
    "crates/wow-annotations/src/navigation/sources.rs",
]:
    write(path, re.sub(r'\s*\|\s*"wow-native-annotation-library/6"', "", read(path)))

ci_path = ".github/workflows/ci.yml"
ci = read(ci_path)
start = ci.find("  wasm-bridge:\n")
end = ci.find("  annotation-consumers:\n", start)
if start < 0 or end < 0:
    raise SystemExit("ci.yml: missing Wasm job")
write(ci_path, ci[:start] + ci[end:])

current_path = ".github/workflows/current-source-bundle.yml"
current = read(current_path)
replace = "          rustup target add wasm32-unknown-unknown --toolchain stable\n"
if current.count(replace) != 1:
    raise SystemExit("current-source workflow: missing Wasm target")
current = current.replace(replace, "")
start = current.find("      - name: Build replaceable literal guests\n")
end = current.find("      - name: Retain source and projection evidence\n", start)
if start < 0 or end < 0:
    raise SystemExit("current-source workflow: missing Wasm steps")
current = current[:start] + current[end:]
for line in [
    "            ${{ runner.temp }}/wasm-a/source-report.json\n",
    "            ${{ runner.temp }}/wasm-b/source-report.json\n",
    "            ${{ runner.temp }}/source-wasm-comparison.json\n",
    "            ${{ runner.temp }}/source-wasm-failure.json\n",
]:
    if current.count(line) != 1:
        raise SystemExit(f"current-source workflow: missing artifact {line!r}")
    current = current.replace(line, "")
write(current_path, current)

implementation_path = "docs/IMPLEMENTATION.md"
implementation = read(implementation_path)
implementation = implementation.replace(
    "`wow-annotations/src/{native,ketho,aliases,literals}.rs`; `wow-ketho-literals` is the native literal algorithm, `wow-render-contract` its shared data types.",
    "`wow-annotations/src/{native,ketho,aliases,literals}.rs`; `wow-ketho-literals` owns the typed literal data and native algorithm.",
)
implementation = implementation.replace(
    "`wow-annotations/src/native.rs::project_with_alias_catalogs_and_literal_bridge`",
    "`wow-annotations/src/native.rs::project_with_alias_catalogs`",
)
replace_exact(
    implementation_path,
    "#203 keeps `wow-ketho-literals` callable natively, then removes\n`modules/ketho-literals`, `bridges/literal-host`, exclusive options and CI.\n",
    "Literal generation calls `wow-ketho-literals` directly; no guest, host, wire or runtime selection path exists.\n",
)
implementation = read(implementation_path)
implementation = implementation.replace(
    "`wow-annotations/src/{native,ketho,aliases,literals}.rs`; `wow-ketho-literals` is the native literal algorithm, `wow-render-contract` its shared data types.",
    "`wow-annotations/src/{native,ketho,aliases,literals}.rs`; `wow-ketho-literals` owns the typed literal data and native algorithm.",
).replace(
    "`wow-annotations/src/native.rs::project_with_alias_catalogs_and_literal_bridge`",
    "`wow-annotations/src/native.rs::project_with_alias_catalogs`",
)
write(implementation_path, implementation)

queue_path = "docs/REFACTOR_QUEUE.md"
replace_exact(
    queue_path,
    "| Independent | [#203 — Wasm removal](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/203) | Keep literal algorithm natively; remove guest/host, options, dependencies and exclusive CI. |",
    "| Completed | [#203 — Wasm removal](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/203) | Native literal algorithm retained; guest/host, wire selection and exclusive CI removed. |",
)

replace_exact(
    "docs/KETHO_RUST_PORT.md",
    "| `luasrc/annotate/literals.lua` | `wow-annotations/src/literals.rs` | Event/CVar aliases and enum/constants from explicit typed data. |",
    "| `luasrc/annotate/literals.lua` | `wow-ketho-literals`; `wow-annotations/src/literals.rs` | Native event/CVar aliases and enum/constants from explicit typed data. |",
)

replace_exact(
    "docs/SOURCE_CHECKOUT_UPDATES.md",
    '''Compatible Gethe/Ketho changes update only source/resource data: existing Git
objects are reused, and no Rust host or WASM rebuild is needed to fetch them.
Changed algorithms still use separately built, approved WASM modules through the
[existing bridge](WASM_BRIDGES.md). Network/filesystem authority stays in the
native acquisition adapter and is never given to the guest. This step does not
implement incremental analysis, new type semantics or signed module distribution.
''',
    '''Compatible Gethe/Ketho data changes update only source/resource inputs and reuse
existing Git objects. Literal and annotation algorithms are ordinary native Rust
code and change with the product build; there is no guest module, host runtime or
separate approval/distribution lane. Network/filesystem authority remains in the
native acquisition adapter. This step does not implement incremental analysis or
new type semantics.
''',
)

write(
    "crates/wow-ketho-literals/README.md",
    '''# `wow-ketho-literals`

Native typed Ketho event, CVar, enum and constant rendering used directly by
`wow-annotations`. It owns literal input types, bounds and domain errors; it has
no source parser, VM, wire protocol, IO, editor state or plugin lifecycle.
Focused golden and consumer checks remain with the annotation owner. See the
[third-party notices](THIRD_PARTY_NOTICES.md).
''',
)
