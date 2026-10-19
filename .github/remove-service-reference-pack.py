from pathlib import Path


def replace(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one block, found {count}")
    file.write_text(text.replace(old, new))


replace(
    "crates/wow-service/src/lib.rs",
    "pub mod reference_pack;\nmod reference_pack_license;\npub mod reference_pack_materialization;\n",
    "mod reference_pack_license;\n",
)

replace(
    "crates/wow-service/src/local/mod.rs",
    '''pub use native_input::{
    LOCAL_NATIVE_SCHEMA, NativeAnnotationFile, NativeFileIdentity, NativeInputReceipt,
    NativeManifestIdentity, NativeRestrictionIdentity,
};
''',
    '''pub use native_input::{
    LOCAL_NATIVE_SCHEMA, NativeFileIdentity, NativeInputReceipt, NativeManifestIdentity,
    NativeRestrictionIdentity,
};
''',
)

replace(
    "crates/wow-service/src/local/native_input.rs",
    '''#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeAnnotationFile {
    path: Box<str>,
    sha256: Box<str>,
    bytes: Box<[u8]>,
}

impl NativeAnnotationFile {
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn sha256(&self) -> &str {
        &self.sha256
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

''',
    "",
)

replace(
    "crates/wow-service/src/local/native_input.rs",
    '''    annotation_artifact: AnnotationArtifact,
    annotation_files: Box<[NativeAnnotationFile]>,
    annotation_sidecars: NativeProjectionSidecars,
''',
    '''    annotation_artifact: AnnotationArtifact,
    annotation_sidecars: NativeProjectionSidecars,
''',
)

replace(
    "crates/wow-service/src/local/native_input.rs",
    '''    #[must_use]
    pub fn native_annotation_files(&self) -> Option<&[NativeAnnotationFile]> {
        self.native_input
            .as_ref()
            .map(|evidence| evidence.annotation_files.as_ref())
    }

''',
    "",
)

replace(
    "crates/wow-service/src/local/native_input.rs",
    '''        let annotation_files = library
            .files
            .iter()
            .map(|file| NativeAnnotationFile {
                path: file.path.clone().into_boxed_str(),
                sha256: file.sha256.clone().into_boxed_str(),
                bytes: file.text.as_bytes().to_vec().into_boxed_slice(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
''',
    "",
)

replace(
    "crates/wow-service/src/local/native_input.rs",
    '''            annotation_artifact,
            annotation_files,
            annotation_sidecars,
''',
    '''            annotation_artifact,
            annotation_sidecars,
''',
)

replace(
    "crates/wow-service/README.md",
    '''- Graph and search use cases: `src/graph/`, `src/search.rs`
- Builder-only pack materialization still pending replacement:
  `src/reference_pack.rs`, `src/reference_pack_materialization.rs`

Local status/check results are ordinary typed values serialized once at the
application boundary. [#194](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194)
continues only through other result families that still self-identify or journal
ordinary reads. Uncalled Service publication/admin layers are gone; the live
builder pack/materialization path is replaced by native directory generation in
[#196](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/196).
The broader store lifecycle removal is [#111](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/111).
These cuts are pending; this README does not imply those modules are already
removed.
''',
    '''- Graph and search use cases: `src/graph/`, `src/search.rs`

Local status/check results are ordinary typed values serialized once at the
application boundary. [#194](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194)
continues only through other result families that still self-identify or journal
ordinary reads. Builder pack/materialization and its second generated-Lua copy
are removed; supported generation writes an ordinary annotation directory. The
broader store lifecycle removal remains [#111](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/111).
''',
)

replace(
    "docs/IMPLEMENTATION.md",
    '''4. Uncalled admin layers and the builder pack/materialization CLI are removed.
   Delete their now-unconsumed Service implementation and retained Lua copy.
   Native generation requires no pack registry or signed report.

`local/native_input.rs` also copies generated Lua into `NativeAnnotationFile`;
only `reference_pack.rs` consumes that retained copy. Remove the type, field and
accessor with the pack path. The analyzer already receives `library.files`;
this deletion needs no shared-buffer framework or Project input API change.
''',
    '''4. Builder and Service pack/materialization paths are removed. Generated Lua
   enters the analyzer once as ordinary Library files; source maps and callable
   origins remain typed sidecars. Native generation requires no pack registry,
   signed report, second byte copy or compatibility archive.
''',
)

replace(
    "docs/REFACTOR_QUEUE.md",
    '''| Native path | [#196 — annotation delivery](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/196) | Supported native directory generation is live; delete unconsumed Service pack/materialization and its retained Lua copy. |
''',
    '''| Completed | [#196 — annotation delivery](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/196) | Supported generation writes ordinary Library files; builder/Service pack administration and the second Lua copy are removed. |
''',
)
