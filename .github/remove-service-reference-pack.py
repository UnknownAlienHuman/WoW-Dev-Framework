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
    "",
)

replace(
    "crates/wow-service/src/local/mod.rs",
    "mod native_distribution;\n",
    "",
)
replace(
    "crates/wow-service/src/local/mod.rs",
    "pub use native_distribution::NativeDistributionSelection;\n",
    "",
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
    "crates/wow-service/src/local/platform_input/tests.rs",
    "mod builder_attempts;\n",
    "",
)

replace(
    "crates/wow-service/src/local/native_input.rs",
    "use super::native_distribution::{NativeDistributionInputs, NativeDistributionSelection};\n",
    "",
)
replace(
    "crates/wow-service/src/local/native_input.rs",
    "use crate::reference_pack_license::{NativeDistributionBinding, NativeDistributionManifest};\n",
    "",
)
replace(
    "crates/wow-service/src/local/native_input.rs",
    '''    #[serde(default, deserialize_with = "present")]
    distribution_evidence: Option<NativeDistributionInputs>,
''',
    "",
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
    '''    #[serde(skip_serializing_if = "Option::is_none")]
    pub distribution_evidence: Option<NativeDistributionSelection>,
''',
    "",
)
replace(
    "crates/wow-service/src/local/native_input.rs",
    '''    annotation_artifact: AnnotationArtifact,
    annotation_files: Box<[NativeAnnotationFile]>,
    annotation_sidecars: NativeProjectionSidecars,
    annotation_callable_bindings: NativeCallableBindings,
    annotation_compatibility: Option<NativeCompatibilityEvidence>,
    distribution_evidence: Option<NativeDistributionManifest>,
''',
    '''    annotation_artifact: AnnotationArtifact,
    annotation_sidecars: NativeProjectionSidecars,
    annotation_callable_bindings: NativeCallableBindings,
    annotation_compatibility: Option<NativeCompatibilityEvidence>,
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
    '''    #[must_use]
    pub(crate) fn native_distribution_evidence(&self) -> Option<&NativeDistributionManifest> {
        self.native_input
            .as_ref()
            .and_then(|evidence| evidence.distribution_evidence.as_ref())
    }

''',
    "",
)
replace(
    "crates/wow-service/src/local/native_input.rs",
    '''        let distribution_evidence = input
            .distribution_evidence
            .as_ref()
            .map(|selected| {
                let source_manifest = source_manifest.as_ref().ok_or_else(|| {
                    invalid("native distribution evidence requires manifested source input")
                })?;
                let binding = NativeDistributionBinding::new(
                    profile.profile_id().as_str(),
                    reference.view.generation_id(),
                    source_manifest.manifest_sha256.as_str(),
                    reference.view.self_digest(),
                    annotation_artifact.artifact_id(),
                    annotation_sidecars.source_map().source_map_id(),
                    annotation_sidecars.loss_report().report_id(),
                    annotation_compatibility
                        .as_ref()
                        .map(|loaded| loaded.evidence.evidence_id()),
                )
                .map_err(|_| invalid("native distribution binding rejected"))?;
                selected.read(directory, binding, stop)
            })
            .transpose()?;
        let distribution_selection = distribution_evidence
            .as_ref()
            .map(|loaded| loaded.selection.clone());
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
    '''            #[serde(skip_serializing_if = "Option::is_none")]
            compatibility_evidence: Option<&'a NativeCompatibilitySelection>,
            #[serde(skip_serializing_if = "Option::is_none")]
            distribution_evidence: Option<&'a NativeDistributionSelection>,
            semantic_consumer_acceptance: &'static str,
''',
    '''            #[serde(skip_serializing_if = "Option::is_none")]
            compatibility_evidence: Option<&'a NativeCompatibilitySelection>,
            semantic_consumer_acceptance: &'static str,
''',
)
replace(
    "crates/wow-service/src/local/native_input.rs",
    '''                schema: if distribution_selection.is_some() {
                    "wow-service/native-input-report/7"
                } else if compatibility_selection.is_some() {
''',
    '''                schema: if compatibility_selection.is_some() {
''',
)
replace(
    "crates/wow-service/src/local/native_input.rs",
    '''                restriction_facts: restriction_facts.as_ref(),
                compatibility_evidence: compatibility_selection.as_ref(),
                distribution_evidence: distribution_selection.as_ref(),
                semantic_consumer_acceptance,
''',
    '''                restriction_facts: restriction_facts.as_ref(),
                compatibility_evidence: compatibility_selection.as_ref(),
                semantic_consumer_acceptance,
''',
)
replace(
    "crates/wow-service/src/local/native_input.rs",
    '''            schema: if distribution_selection.is_some() {
                "wow-service/native-input-receipt/7"
            } else if compatibility_selection.is_some() {
''',
    '''            schema: if compatibility_selection.is_some() {
''',
)
replace(
    "crates/wow-service/src/local/native_input.rs",
    '''            restriction_facts,
            compatibility_evidence: compatibility_selection,
            distribution_evidence: distribution_selection,
            semantic_consumer_acceptance,
''',
    '''            restriction_facts,
            compatibility_evidence: compatibility_selection,
            semantic_consumer_acceptance,
''',
)
replace(
    "crates/wow-service/src/local/native_input.rs",
    '''            annotation_artifact,
            annotation_files,
            annotation_sidecars,
            annotation_callable_bindings,
            annotation_compatibility: annotation_compatibility.map(|loaded| loaded.evidence),
            distribution_evidence: distribution_evidence.map(|loaded| loaded.evidence),
''',
    '''            annotation_artifact,
            annotation_sidecars,
            annotation_callable_bindings,
            annotation_compatibility: annotation_compatibility.map(|loaded| loaded.evidence),
''',
)

replace(
    "apps/wow/NATIVE_INPUT.md",
    '''| `compatibility_evidence` | `parity_report` plus 1–8 `consumer_probe_results`, each a pinned JSON file. The annotation compatibility owner validates the reports; this is optional evidence, not an ordinary check prerequisite. |
| `distribution_evidence` | A pinned `manifest` JSON file validated by the [distribution/license owner](../../crates/wow-service/src/reference_pack_license.rs). It records the selected distribution evidence; it is not required to analyze addon Lua. |
''',
    '''| `compatibility_evidence` | `parity_report` plus 1–8 `consumer_probe_results`, each a pinned JSON file. The annotation compatibility owner validates the reports; this is optional evidence, not an ordinary check prerequisite. |
''',
)
replace(
    "apps/wow/NATIVE_INPUT.md",
    '''`native_api_uses`/`state_projection` select static facts; witness and evidence
fields are current optional machinery targeted by #195/#196/#118, not setup
requirements to carry into #200.
''',
    '''`native_api_uses`/`state_projection` select static facts; witness and
compatibility fields are optional machinery tracked by #195/#118, not setup
requirements to carry into #200.
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
ordinary reads. Builder pack/materialization, its distribution manifest option
and the second generated-Lua copy are removed; supported generation writes an
ordinary annotation directory. The broader store lifecycle removal remains
[#111](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/111).
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
   distribution manifest gate, signed report, second byte copy or compatibility
   archive.
''',
)
replace(
    "docs/REFACTOR_QUEUE.md",
    '''| Native path | [#196 — annotation delivery](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/196) | Supported native directory generation is live; delete unconsumed Service pack/materialization and its retained Lua copy. |
''',
    '''| Completed | [#196 — annotation delivery](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/196) | Supported generation writes ordinary Library files; builder/Service pack administration, distribution gate and second Lua copy are removed. |
''',
)
