use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use sha2::{Digest, Sha256};
use wow_core::{
    CanonicalResult, ContentDigest, CoverageStatus, ProfileIdentityBuilder, ProfileKind,
    ReferenceGenerationId, SchemaVersionEntry, SourceContent, SourceKind, SourceLogicalSnapshot,
    SourceOriginKind, SourceSpan,
};
use wow_emmy::{
    EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity,
    EmmyMemberCallErrorCode, LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot,
    LuaWorkspaceUniverse,
};
use wow_project::{
    AnalyzerBindingDeclaration, ProjectBudgetPolicy, ProjectCapabilityPolicy,
    ProjectConfigurationBuilder, ProjectErrorCode, ProjectId, ProjectInputBundle, ProjectKind,
    ProjectPublisher, ProjectSourceOriginId, ProjectSourceOriginKind, ProjectWorkspaceId,
    disk::ProjectInputDirectory,
    load::{ProjectPackageInput, ProjectPackageVariantInput},
    platform_source::{
        BlizzardUiSourceProfile, BlizzardUiSourceProfileRequest, PlatformEntryDisposition,
        PlatformFileKind, PlatformInventoryEntry, PlatformInventoryScope, PlatformLicenseRecord,
        PlatformLicenseState, PlatformMaterializer, PlatformRootInventory, PlatformRootSpec,
        PlatformSourceClass, PlatformSourceInventory, PlatformSourceOrigin, PlatformSourceRevision,
        PlatformSpecialEntry, PlatformTarget, PlatformUnevaluatedCapability, ProfileExclusion,
        SourceAdmissionLimits,
    },
    replay::ProjectReplay,
};

type TestResult = Result<(), Box<dyn Error>>;
static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);
struct FixtureRoot(PathBuf);
impl FixtureRoot {
    fn new() -> Result<Self, Box<dyn Error>> {
        let root = std::env::temp_dir().join(format!(
            "wow-platform-source-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root)?;
        std::fs::create_dir(root.join("UI"))?;
        let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/xml-facts");
        for name in ["Fixture.toc", "frames.xml", "defs.lua"] {
            std::fs::copy(corpus.join(name), root.join("UI").join(name))?;
        }
        std::fs::write(root.join("UI/opaque.bin"), [0xff, 0xfe, 0, 1])?;
        Ok(Self(root))
    }
    fn directory(&self) -> Result<ProjectInputDirectory, Box<dyn Error>> {
        Ok(ProjectInputDirectory::open(&self.0)?)
    }
}
impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn raw_digest(bytes: &[u8]) -> ContentDigest<SourceContent> {
    ContentDigest::from_bytes(Sha256::digest(bytes).into())
}
fn target() -> Result<PlatformTarget, Box<dyn Error>> {
    let reference_profile = ProfileIdentityBuilder::new(
        "profile:fixture:platform-source-native-v1".parse()?,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "platform-source-handwritten-native-fixture-v1",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([2; 32]),
    )
    .schema_versions(vec![SchemaVersionEntry::new(
        "schema:wow:platform-source-fixture".parse()?,
        "1.0.0".parse()?,
    )])
    .fixture_scope("platform-source-admission-local-fixture")
    .build()?;
    Ok(PlatformTarget {
        product: "fixture".into(),
        channel: "fixture".into(),
        reference_profile,
        reference_generation: ReferenceGenerationId::from_hash([3; 32]),
    })
}
fn profile_request() -> Result<BlizzardUiSourceProfileRequest, Box<dyn Error>> {
    Ok(BlizzardUiSourceProfileRequest {
        profile_id: "profile:fixture:platform-source-admission-v1".parse()?,
        source_class: PlatformSourceClass::SyntheticFixture,
        target: target()?,
        roots: vec![PlatformRootSpec {
            root: "UI".into(),
            selected_tocs: vec!["UI/Fixture.toc".into()],
        }],
        exclusions: vec![ProfileExclusion {
            path: "UI/omitted.txt".into(),
        }],
        limits: SourceAdmissionLimits {
            max_entries: 16,
            max_total_bytes: 1024 * 1024,
            max_file_bytes: 1024 * 1024,
            max_manifest_bytes: 32 * 1024,
        },
    })
}
fn inventory(
    root: &FixtureRoot,
    profile: &BlizzardUiSourceProfile,
) -> Result<PlatformSourceInventory, Box<dyn Error>> {
    let mut entries = Vec::new();
    for (name, kind) in [
        ("Fixture.toc", PlatformFileKind::Toc),
        ("frames.xml", PlatformFileKind::Xml),
        ("defs.lua", PlatformFileKind::Lua),
        ("opaque.bin", PlatformFileKind::Unknown),
    ] {
        let bytes = std::fs::read(root.0.join("UI").join(name))?;
        entries.push(PlatformInventoryEntry {
            path: format!("UI/{name}"),
            kind,
            disposition: PlatformEntryDisposition::Included {
                digest: raw_digest(&bytes),
                byte_length: bytes.len() as u64,
                object_id: None,
            },
        });
    }
    entries.push(PlatformInventoryEntry {
        path: "UI/omitted.txt".into(),
        kind: PlatformFileKind::Unknown,
        disposition: PlatformEntryDisposition::Excluded {
            rule_path: "UI/omitted.txt".into(),
        },
    });
    entries.push(PlatformInventoryEntry {
        path: "UI/linked.lua".into(),
        kind: PlatformFileKind::Lua,
        disposition: PlatformEntryDisposition::Unsupported {
            kind: PlatformSpecialEntry::Symlink,
            evidence_digest: raw_digest(b"explicit unsupported fixture record"),
        },
    });
    Ok(PlatformSourceInventory {
        schema: "wow-project/platform-source-inventory/1".into(),
        profile_digest: profile.digest(),
        target: profile.target().clone(),
        origin: PlatformSourceOrigin {
            provider: "handwritten-fixture".into(),
            repository: "native-platform-input".into(),
            revision: PlatformSourceRevision::Fixture {
                digest: raw_digest(b"handwritten-fixture-revision"),
            },
        },
        materializer: PlatformMaterializer {
            producer: "wow.fixture_materializer".parse()?,
            version: "1.0.0".parse()?,
            configuration_digest: ContentDigest::<CanonicalResult>::from_bytes([4; 32]),
            report_digest: raw_digest(b"fixture materialization declaration, not attestation"),
        },
        roots: vec![PlatformRootInventory {
            root: "UI".into(),
            declared_entries: entries.len() as u64,
            scope: PlatformInventoryScope::DeclaredPartial,
            evidence_digest: raw_digest(b"fixture root accounting assertion"),
        }],
        entries,
        license: PlatformLicenseRecord {
            state: PlatformLicenseState::Unknown,
            attribution: "project-owned synthetic test input".into(),
            evidence_digest: raw_digest(b"local-only fixture notice"),
        },
        compatibility_evidence: raw_digest(b"caller fixture compatibility assertion"),
    })
}

fn fixture_packages(name: &str) -> Vec<ProjectPackageInput> {
    vec![ProjectPackageInput::new(
        name,
        "UI",
        true,
        vec![ProjectPackageVariantInput::new(
            wow_project::disk::ProjectDiskFile::new("Fixture.toc"),
            true,
        )],
    )]
}

fn configuration_builder(
    kind: ProjectKind,
    target: &PlatformTarget,
) -> Result<ProjectConfigurationBuilder, Box<dyn Error>> {
    let backend = EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(EMMYLUA_CODE_ANALYSIS_VERSION),
        EMMYLUA_REVISION,
        EMMYLUA_TREE,
        format!("sha256:{}", "1".repeat(64)),
        format!("sha256:{}", "2".repeat(64)),
    )?;
    let analyzer = AnalyzerBindingDeclaration::new(
        "wow-emmy/e0-c/1",
        format!("emmy-pin:{EMMYLUA_REVISION}"),
        backend.compatibility_report_sha256(),
        ContentDigest::<CanonicalResult>::from_bytes([3; 32]),
        "wow-emmy/e0-c/1",
        "wow-emmy-e0-c-library-v1",
        backend.clone(),
    )?;
    Ok(ProjectConfigurationBuilder::new(
        ProjectId::new("platform-source-lifecycle-fixture")?,
        kind,
        target.reference_profile.clone(),
        target.reference_generation,
        analyzer,
    )
    .workspace_id(ProjectWorkspaceId::new(
        "workspace:main:platform-source-fixture",
    )?)
    .source_origin_id(ProjectSourceOriginId::new(
        "project-origin:platform-source-fixture",
    )?)
    .logical_root("fixtures/platform-source/UI")
    .capability_policy(ProjectCapabilityPolicy::strict_e0()?)
    .budget_policy(ProjectBudgetPolicy::fixture_e0()?))
}

#[test]
fn native_selected_schema_binds_components_and_refuses_foreign_source() -> TestResult {
    use std::collections::{BTreeMap, BTreeSet};
    use wow_project::{
        ProjectPhase, ProjectResult,
        load::{
            XmlSourceSpan,
            schema::{
                XML_SCHEMA_POLICY_PROFILE, XML_SCHEMA_PROFILE, XmlSchemaAttributeValue,
                XmlSchemaComponentKind as Kind, XmlSchemaComponentState as State,
                XmlSchemaIssueKind, XmlSchemaMemberSelection, XmlSchemaQNameState,
                XmlSchemaReferenceKind as RefKind, XmlSchemaReferenceState, XmlSchemaSelection,
                admit_xml_schema,
            },
        },
    };

    // Exact project-owned body, LF with one trailing LF; not a live vendor schema.
    const XSD: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns="http://www.blizzard.com/wow/ui/" xmlns:ui="http://www.blizzard.com/wow/ui/" targetNamespace="http://www.blizzard.com/wow/ui/" elementFormDefault="qualified" attributeFormDefault="unqualified">
<xs:complexType name="LayoutFrameRefType"/>
<xs:element name="LayoutFrameRef" type="LayoutFrameRefType" abstract="true"/>
<xs:complexType name="FrameRefType"><xs:complexContent><xs:extension base="ui:LayoutFrameRefType"/></xs:complexContent></xs:complexType>
<xs:element name="FrameRef" type="ui:FrameRefType" abstract="true" substitutionGroup="LayoutFrameRef"/>
<xs:complexType name="FrameType"><xs:complexContent><xs:extension base="FrameRefType"/></xs:complexContent></xs:complexType>
<xs:element name="Frame" type="FrameType" substitutionGroup="ui:FrameRef"/>
<xs:complexType name="TextureType"><xs:complexContent><xs:extension base="LayoutFrameRefType"/></xs:complexContent></xs:complexType>
<xs:element name="Texture" type="ui:TextureType" substitutionGroup="LayoutFrameRef"/>
<xs:element name="Anonymous" substitutionGroup="ui:FrameRef"><xs:complexType><xs:complexContent><xs:extension base="FrameRefType"/></xs:complexContent></xs:complexType></xs:element>
<xs:complexType name="OwnerAType"><xs:sequence><xs:element name="Slot" type="ui:FrameType"/></xs:sequence></xs:complexType>
<xs:complexType name="OwnerBType"><xs:sequence><xs:element name="Slot" type="TextureType"/></xs:sequence></xs:complexType>
<xs:element name="OwnerA" type="OwnerAType"/>
<xs:element name="OwnerB" type="ui:OwnerBType"/>
</xs:schema>
"#;
    const UI: &str = "http://www.blizzard.com/wow/ui/";
    const MISLABELED_UI: &str = "<Ui><Script>\n</Script></Ui>";
    const NESTED_TYPE: &str = r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:ui="http://www.blizzard.com/wow/ui/" targetNamespace="http://www.blizzard.com/wow/ui/">
<xs:complexType name="T"/>
<xs:element name="Holder"><xs:complexType name="T"/></xs:element>
<xs:element name="Uses" type="ui:T"/>
<xs:annotation><xs:appinfo><xs:schema><xs:complexType name="T"/></xs:schema></xs:appinfo></xs:annotation>
<xs:element name="Conflicted" type="ui:T"><xs:complexType><xs:complexContent><xs:extension base="ui:T"/></xs:complexContent></xs:complexType></xs:element>
</xs:schema>
"#;

    fn span_text<'a>(text: &'a str, span: &XmlSourceSpan) -> Result<&'a str, Box<dyn Error>> {
        assert!(span.byte_start < span.byte_end);
        Ok(text
            .get(usize::try_from(span.byte_start)?..usize::try_from(span.byte_end)?)
            .ok_or("schema span escapes its original UTF-8 member")?)
    }
    fn refuses<T>(
        result: ProjectResult<T>,
        code: ProjectErrorCode,
        phase: ProjectPhase,
    ) -> TestResult {
        let error = result.err().ok_or("schema guard unexpectedly succeeded")?;
        assert_eq!(error.code(), code);
        assert_eq!(error.phase(), phase);
        Ok(())
    }

    assert_eq!(XSD.len(), 1_575);
    let schema_digest: ContentDigest<SourceContent> =
        "sha256:021ee75d13ed27ef8f327a1e9a3f3c4f4cf7e62c8f0e2138a2e41d103432f3c3".parse()?;
    assert_eq!(raw_digest(XSD.as_bytes()), schema_digest);
    let root = FixtureRoot::new()?;
    let stop = AtomicBool::new(false);
    let profile = BlizzardUiSourceProfile::new(profile_request()?)?;
    std::fs::write(root.0.join("UI/schema.xsd"), XSD)?;
    // Only this test extends inventory accounting; no TOC/Main registration.
    let declare = |revision: &[u8]| -> Result<PlatformSourceInventory, Box<dyn Error>> {
        let mut declared = inventory(&root, &profile)?;
        let bytes = std::fs::read(root.0.join("UI/schema.xsd"))?;
        declared.entries.push(PlatformInventoryEntry {
            path: "UI/schema.xsd".into(),
            kind: PlatformFileKind::Schema,
            disposition: PlatformEntryDisposition::Included {
                digest: raw_digest(&bytes),
                byte_length: bytes.len() as u64,
                object_id: None,
            },
        });
        declared
            .roots
            .first_mut()
            .ok_or("fixture root missing")?
            .declared_entries = declared.entries.len() as u64;
        declared.origin.revision = PlatformSourceRevision::Fixture {
            digest: raw_digest(revision),
        };
        Ok(declared)
    };
    let source_a = Arc::new(root.directory()?.admit_platform_source(
        &profile,
        declare(b"schema-owner-source-a")?,
        &stop,
    )?);
    let mut changed_lua = std::fs::read(root.0.join("UI/defs.lua"))?;
    changed_lua.extend_from_slice(b"\n-- schema-owner foreign source witness\n");
    std::fs::write(root.0.join("UI/defs.lua"), changed_lua)?;
    let source_b = Arc::new(root.directory()?.admit_platform_source(
        &profile,
        declare(b"schema-owner-source-b")?,
        &stop,
    )?);
    std::fs::write(root.0.join("UI/schema.xsd"), MISLABELED_UI)?;
    let source_c = Arc::new(root.directory()?.admit_platform_source(
        &profile,
        declare(b"schema-owner-source-c")?,
        &stop,
    )?);
    std::fs::write(root.0.join("UI/schema.xsd"), NESTED_TYPE)?;
    let source_d = Arc::new(root.directory()?.admit_platform_source(
        &profile,
        declare(b"schema-owner-source-d")?,
        &stop,
    )?);
    std::fs::remove_dir_all(&root.0)?;
    assert!(!root.0.exists());

    let original = source_a.raw_member("UI/schema.xsd", &stop)?;
    let foreign = source_b.raw_member("UI/schema.xsd", &stop)?;
    assert_eq!(original.kind(), PlatformFileKind::Schema);
    assert_eq!(original.kind(), foreign.kind());
    assert_eq!(original.path(), foreign.path());
    assert_eq!(original.bytes(), XSD.as_bytes());
    assert_eq!(original.bytes(), foreign.bytes());
    assert_eq!(original.content_digest(), schema_digest);
    assert_eq!(original.content_digest(), foreign.content_digest());
    assert_eq!(original.byte_length(), foreign.byte_length());
    assert_eq!(
        source_a.receipt().profile_digest(),
        source_b.receipt().profile_digest()
    );
    assert_ne!(
        source_a.receipt().source_snapshot_id(),
        source_b.receipt().source_snapshot_id()
    );
    assert_ne!(
        source_a.receipt().content_manifest_digest(),
        source_b.receipt().content_manifest_digest()
    );
    assert_ne!(
        source_a.receipt().admission_digest(),
        source_b.receipt().admission_digest()
    );

    let loaded =
        Arc::new(source_a.specialize_packages(&fixture_packages("Fixture"), None, &stop)?);
    assert!(Arc::ptr_eq(loaded.source(), &source_a));
    assert_eq!(loaded.files().len(), 1);
    assert_eq!(
        loaded.files()[0].relative_path().as_str(),
        "packages/Fixture/defs.lua"
    );
    let plan = loaded
        .load_plan()
        .package_plan("Fixture")
        .ok_or("package plan missing")?;
    assert!(
        !plan
            .sources()
            .iter()
            .any(|source| source.path == "schema.xsd")
    );
    assert_eq!(
        loaded.main_plan().resolve_source("Fixture", "schema.xsd"),
        None
    );
    let configuration =
        configuration_builder(ProjectKind::BlizzardUiPlatformSource, profile.target())?
            .platform_packages(Arc::clone(&loaded))?
            .build()?;
    let library = LuaWorkspaceSnapshot::build(
        configuration.analyzer_binding().backend().clone(),
        LuaWorkspaceUniverse::Fixture,
        vec![LuaWorkspaceFileInput::new(
            "library/schema-owner-fixture.lua",
            "---@meta _\nSchemaOwnerLibraryFixture = {}\n",
        )],
        LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
    )?;
    let mut publisher = ProjectPublisher::new();
    let snapshot = publisher.publish_initial_cancellable(
        ProjectInputBundle::closed(configuration, loaded.files().to_vec(), vec![library])?,
        &stop,
    )?;
    snapshot.validate()?;
    let view = snapshot.open_view();
    assert_eq!(view.file_manifest().len(), 1);
    assert_eq!(
        view.file_manifest()[0].relative_path(),
        loaded.files()[0].relative_path()
    );
    assert!(view.source_artifact("UI/schema.xsd")?.is_none());
    assert!(
        view.source_artifact("packages/Fixture/schema.xsd")?
            .is_none()
    );
    let main_before = view.file_manifest().to_vec();
    let snapshot_before = view.snapshot_id().to_owned();
    let load_before = loaded.load_plan().digest();

    let selection = XmlSchemaSelection::for_source(&source_a, &["UI/schema.xsd"], &stop)?;
    let schema = admit_xml_schema(&source_a, &selection, &stop)?;
    assert!(std::ptr::eq(schema.source(), source_a.as_ref()));
    schema.validate_source(&source_a, &stop)?;
    let receipt = schema.receipt();
    assert_eq!(receipt.profile(), XML_SCHEMA_PROFILE);
    assert_eq!(receipt.policy_profile(), XML_SCHEMA_POLICY_PROFILE);
    assert_eq!(
        receipt.source_snapshot_id(),
        source_a.receipt().source_snapshot_id()
    );
    assert_eq!(
        receipt.source_profile_digest(),
        source_a.receipt().profile_digest()
    );
    assert_eq!(
        receipt.content_manifest_digest(),
        source_a.receipt().content_manifest_digest()
    );
    assert_eq!(
        receipt.admission_digest(),
        source_a.receipt().admission_digest()
    );
    assert_eq!(receipt.members(), selection.members);
    assert_eq!(receipt.component_count(), schema.components().len());
    assert_eq!(receipt.reference_count(), schema.references().len());
    assert_eq!(receipt.issue_count(), schema.issues().len());
    assert_eq!(receipt.members().len(), 1);
    assert_eq!(receipt.members()[0].path, original.path());
    assert_eq!(
        receipt.members()[0].content_digest,
        original.content_digest()
    );
    assert_eq!(receipt.members()[0].byte_length, original.byte_length());
    assert_eq!(schema.documents().len(), 1);
    let document = schema
        .documents()
        .first()
        .ok_or("schema document missing")?;
    assert_eq!(document.document(), original.path());
    assert_eq!(document.source_digest(), schema_digest);
    assert!(document.scripts().next().is_none());
    let by_id: BTreeMap<_, _> = schema
        .components()
        .iter()
        .map(|component| (component.id(), component))
        .collect();
    assert_eq!(by_id.len(), schema.components().len());
    let mut elements = BTreeMap::new();
    let mut types = BTreeMap::new();
    let mut anonymous = Vec::new();
    let mut locals = Vec::new();
    for component in schema.components() {
        assert_eq!(component.document(), original.path());
        assert_eq!(component.content_digest(), schema_digest);
        let native = document
            .element(component.occurrence())
            .ok_or("component occurrence missing")?;
        assert_eq!(component.span(), &native.span);
        assert!(span_text(XSD, component.span())?.starts_with("<xs:"));
        match component.parent() {
            Some(parent) => {
                let parent = by_id.get(&parent).ok_or("component parent missing")?;
                assert_eq!(
                    native.parent_occurrence_id.as_deref(),
                    Some(parent.occurrence())
                );
                assert!(parent.span().byte_start < component.span().byte_start);
                assert!(component.span().byte_end <= parent.span().byte_end);
            }
            None => {
                assert_eq!(component.kind(), Kind::Schema);
                assert!(native.parent_occurrence_id.is_none());
            }
        }
        assert_eq!(component.attributes().len(), native.attributes.len());
        for attribute in component.attributes() {
            let native_attribute = native
                .attributes
                .iter()
                .find(|value| value.qualified_name == attribute.qualified_name())
                .ok_or("component attribute missing from native index")?;
            assert_eq!(attribute.span(), &native_attribute.span);
            assert_eq!(attribute.value_span(), &native_attribute.value_span);
            assert_eq!(attribute.value(), native_attribute.value());
            assert_eq!(span_text(XSD, attribute.value_span())?, attribute.value());
            assert_eq!(
                attribute.decoded_value_digest(),
                native_attribute.decoded_value_digest
            );
            assert_eq!(
                attribute.decoded_value_digest(),
                raw_digest(attribute.value().as_bytes())
            );
        }
        match component.kind() {
            Kind::GlobalElement | Kind::NamedComplexType | Kind::LocalElement => {
                assert_eq!(component.state(), State::Observed);
                let name = component
                    .name()
                    .ok_or("named component has no expanded name")?;
                assert_eq!(name.namespace(), Some(UI));
                match component.kind() {
                    Kind::GlobalElement => {
                        assert!(elements.insert(name.local_name(), component).is_none());
                    }
                    Kind::NamedComplexType => {
                        assert!(types.insert(name.local_name(), component).is_none());
                    }
                    _ => locals.push(component),
                }
            }
            Kind::AnonymousComplexType => {
                assert_eq!(component.state(), State::Observed);
                assert!(component.name().is_none());
                anonymous.push(component);
            }
            _ => {}
        }
    }
    assert_eq!(
        elements.keys().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "LayoutFrameRef",
            "FrameRef",
            "Frame",
            "Texture",
            "Anonymous",
            "OwnerA",
            "OwnerB",
        ])
    );
    assert_eq!(
        types.keys().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "LayoutFrameRefType",
            "FrameRefType",
            "FrameType",
            "TextureType",
            "OwnerAType",
            "OwnerBType",
        ])
    );
    for (name, element) in &elements {
        let is_abstract = matches!(*name, "LayoutFrameRef" | "FrameRef");
        assert_eq!(
            element
                .attribute("abstract")
                .map(|attribute| attribute.interpretation()),
            is_abstract.then_some(&XmlSchemaAttributeValue::Boolean(true))
        );
    }
    assert_eq!(anonymous.len(), 1);
    let anonymous_type = anonymous.first().ok_or("anonymous type missing")?;
    let anonymous_element = elements
        .get("Anonymous")
        .ok_or("Anonymous element missing")?;
    assert_eq!(anonymous_type.parent(), Some(anonymous_element.id()));
    assert!(anonymous_element.attribute("type").is_none());
    assert_eq!(locals.len(), 2);
    let mut local_types = BTreeMap::new();
    for local in &locals {
        assert_eq!(
            local.name().ok_or("local name missing")?.local_name(),
            "Slot"
        );
        let sequence = by_id
            .get(&local.parent().ok_or("local parent missing")?)
            .ok_or("sequence missing")?;
        assert_eq!(sequence.kind(), Kind::Sequence);
        let owner = by_id
            .get(&sequence.parent().ok_or("sequence parent missing")?)
            .ok_or("local type owner missing")?;
        assert_eq!(owner.kind(), Kind::NamedComplexType);
        assert!(
            local_types
                .insert(
                    owner.name().ok_or("owner type name missing")?.local_name(),
                    local.id()
                )
                .is_none()
        );
    }
    assert_eq!(
        local_types.keys().copied().collect::<BTreeSet<_>>(),
        BTreeSet::from(["OwnerAType", "OwnerBType"])
    );
    assert_ne!(locals[0].id(), locals[1].id());
    assert!(locals[0].span().byte_end <= locals[1].span().byte_start);

    let mut type_refs = BTreeSet::new();
    let mut base_refs = BTreeSet::new();
    let mut substitution_refs = BTreeSet::new();
    for reference in schema.references() {
        assert_eq!(reference.state(), XmlSchemaReferenceState::Unique);
        assert_eq!(reference.target().state(), XmlSchemaQNameState::Expanded);
        assert_eq!(reference.candidates().len(), 1);
        let source = by_id
            .get(&reference.source())
            .ok_or("reference source missing")?;
        let target = by_id
            .get(
                reference
                    .candidates()
                    .first()
                    .ok_or("reference target missing")?,
            )
            .ok_or("target component missing")?;
        assert_eq!(reference.target().name(), target.name());
        let attribute = source
            .attribute(reference.attribute())
            .ok_or("reference attribute missing")?;
        assert_eq!(reference.value_span(), attribute.value_span());
        assert_eq!(
            reference.decoded_value_digest(),
            attribute.decoded_value_digest()
        );
        assert_eq!(
            reference.target().lexical(),
            span_text(XSD, reference.value_span())?
        );
        assert_eq!(
            attribute.interpretation(),
            &XmlSchemaAttributeValue::QName(reference.target().clone())
        );
        let binding = reference
            .target()
            .binding()
            .ok_or("QName binding witness missing")?;
        assert_eq!(binding.document(), original.path());
        assert_eq!(binding.namespace(), UI);
        assert_eq!(
            binding.prefix(),
            reference
                .target()
                .lexical()
                .split_once(':')
                .map(|(prefix, _)| prefix)
        );
        let root_record = document
            .element(binding.occurrence())
            .ok_or("namespace occurrence missing")?;
        assert!(root_record.parent_occurrence_id.is_none());
        let namespace_attribute = root_record
            .attributes
            .iter()
            .find(|attribute| {
                attribute.qualified_name
                    == binding
                        .prefix()
                        .map_or_else(|| "xmlns".to_owned(), |prefix| format!("xmlns:{prefix}"))
            })
            .ok_or("namespace declaration missing")?;
        assert_eq!(binding.span(), &namespace_attribute.span);
        assert_eq!(binding.value_span(), &namespace_attribute.value_span);
        assert_eq!(
            binding.decoded_value_digest(),
            namespace_attribute.decoded_value_digest
        );
        assert_eq!(span_text(XSD, binding.value_span())?, UI);
        match reference.kind() {
            RefKind::Type => {
                assert_eq!(reference.attribute(), "type");
                assert_eq!(target.kind(), Kind::NamedComplexType);
                assert!(type_refs.insert((source.id(), target.id())));
            }
            RefKind::Base => {
                assert_eq!(reference.attribute(), "base");
                assert_eq!(source.kind(), Kind::Extension);
                let content = by_id
                    .get(&source.parent().ok_or("extension parent missing")?)
                    .ok_or("complex content missing")?;
                assert_eq!(content.kind(), Kind::ComplexContent);
                let owner = by_id
                    .get(&content.parent().ok_or("base owner missing")?)
                    .ok_or("base type owner missing")?;
                assert!(matches!(
                    owner.kind(),
                    Kind::NamedComplexType | Kind::AnonymousComplexType
                ));
                assert!(base_refs.insert((owner.id(), target.id())));
            }
            RefKind::SubstitutionGroup => {
                assert_eq!(reference.attribute(), "substitutionGroup");
                assert_eq!(source.kind(), Kind::GlobalElement);
                assert_eq!(target.kind(), Kind::GlobalElement);
                assert!(substitution_refs.insert((source.id(), target.id())));
            }
            _ => return Err("fixture has an unexpected schema reference kind".into()),
        }
    }
    let mut expected_types = BTreeSet::new();
    for (element, ty) in [
        ("LayoutFrameRef", "LayoutFrameRefType"),
        ("FrameRef", "FrameRefType"),
        ("Frame", "FrameType"),
        ("Texture", "TextureType"),
        ("OwnerA", "OwnerAType"),
        ("OwnerB", "OwnerBType"),
    ] {
        expected_types.insert((
            elements
                .get(element)
                .ok_or("expected element missing")?
                .id(),
            types.get(ty).ok_or("expected type missing")?.id(),
        ));
    }
    for (owner, ty) in [("OwnerAType", "FrameType"), ("OwnerBType", "TextureType")] {
        expected_types.insert((
            *local_types.get(owner).ok_or("expected local missing")?,
            types.get(ty).ok_or("local target type missing")?.id(),
        ));
    }
    assert_eq!(type_refs, expected_types);
    let mut expected_bases = BTreeSet::new();
    for (source, target) in [
        ("FrameRefType", "LayoutFrameRefType"),
        ("FrameType", "FrameRefType"),
        ("TextureType", "LayoutFrameRefType"),
    ] {
        expected_bases.insert((
            types.get(source).ok_or("derived type missing")?.id(),
            types.get(target).ok_or("base type missing")?.id(),
        ));
    }
    expected_bases.insert((
        anonymous_type.id(),
        types
            .get("FrameRefType")
            .ok_or("anonymous base missing")?
            .id(),
    ));
    assert_eq!(base_refs, expected_bases);
    let mut expected_substitutions = BTreeSet::new();
    for (source, target) in [
        ("FrameRef", "LayoutFrameRef"),
        ("Frame", "FrameRef"),
        ("Texture", "LayoutFrameRef"),
        ("Anonymous", "FrameRef"),
    ] {
        expected_substitutions.insert((
            elements
                .get(source)
                .ok_or("substitution element missing")?
                .id(),
            elements
                .get(target)
                .ok_or("substitution head missing")?
                .id(),
        ));
    }
    assert_eq!(substitution_refs, expected_substitutions);
    assert_eq!(
        schema.references().len(),
        type_refs.len() + base_refs.len() + substitution_refs.len()
    );
    // Issues remain explicit observations; do not turn this subset into full XSD validity.
    for issue in schema.issues() {
        let component = by_id
            .get(&issue.component())
            .ok_or("issue component missing")?;
        assert_eq!(issue.document(), original.path());
        assert_eq!(issue.occurrence(), component.occurrence());
        span_text(XSD, issue.span())?;
    }

    refuses(
        admit_xml_schema(&source_b, &selection, &stop),
        ProjectErrorCode::SourceRegistryInvalid,
        ProjectPhase::Inventory,
    )?;
    refuses(
        schema.validate_source(&source_b, &stop),
        ProjectErrorCode::SourceRegistryInvalid,
        ProjectPhase::Inventory,
    )?;
    refuses(
        XmlSchemaSelection::for_source(&source_a, &["UI/frames.xml"], &stop),
        ProjectErrorCode::InvalidFileLanguage,
        ProjectPhase::Inventory,
    )?;
    let xml_member = source_a.raw_member("UI/frames.xml", &stop)?;
    assert_eq!(xml_member.kind(), PlatformFileKind::Xml);
    let wrong_kind = XmlSchemaSelection {
        members: vec![XmlSchemaMemberSelection {
            path: xml_member.path().to_owned(),
            content_digest: xml_member.content_digest(),
            byte_length: xml_member.byte_length(),
        }],
        ..selection.clone()
    };
    refuses(
        admit_xml_schema(&source_a, &wrong_kind, &stop),
        ProjectErrorCode::InvalidFileLanguage,
        ProjectPhase::Inventory,
    )?;
    let ui_member = source_c.raw_member("UI/schema.xsd", &stop)?;
    assert_eq!(ui_member.kind(), PlatformFileKind::Schema);
    assert_eq!(ui_member.bytes(), MISLABELED_UI.as_bytes());
    assert_eq!(
        ui_member.content_digest(),
        raw_digest(MISLABELED_UI.as_bytes())
    );
    let ui_selection = XmlSchemaSelection::for_source(&source_c, &[ui_member.path()], &stop)?;
    // No capability containing the native inline Script payload may escape admission.
    refuses(
        admit_xml_schema(&source_c, &ui_selection, &stop),
        ProjectErrorCode::SourceRegistryInvalid,
        ProjectPhase::Inventory,
    )?;

    let nested_selection = XmlSchemaSelection::for_source(&source_d, &["UI/schema.xsd"], &stop)?;
    let nested = admit_xml_schema(&source_d, &nested_selection, &stop)?;
    nested.validate_source(&source_d, &stop)?;
    let nested_ids: BTreeMap<_, _> = nested
        .components()
        .iter()
        .map(|component| (component.id(), component))
        .collect();
    let schema_parent = nested
        .components()
        .iter()
        .find(|component| component.kind() == Kind::Schema && component.parent().is_none())
        .ok_or("nested fixture root missing")?;
    let declarations: Vec<_> = nested
        .components()
        .iter()
        .filter(|component| {
            component.kind() == Kind::NamedComplexType
                && component
                    .name()
                    .is_some_and(|name| name.local_name() == "T")
        })
        .collect();
    assert_eq!(declarations.len(), 3);
    let global = declarations
        .iter()
        .find(|component| component.parent() == Some(schema_parent.id()))
        .ok_or("global T missing")?;
    let illegal = declarations
        .iter()
        .find(|component| component.state() == State::Invalid)
        .ok_or("nested T missing")?;
    let unsupported = declarations
        .iter()
        .find(|component| component.state() == State::Unsupported)
        .ok_or("nested-Schema T missing")?;
    assert_eq!(global.state(), State::Observed);
    assert_eq!(illegal.state(), State::Invalid);
    assert_eq!(global.name(), illegal.name());
    assert_eq!(global.name(), unsupported.name());
    assert_ne!(global.id(), illegal.id());
    assert_ne!(global.id(), unsupported.id());
    let nested_schema = nested_ids
        .get(&unsupported.parent().ok_or("nested Schema parent missing")?)
        .ok_or("nested Schema missing")?;
    assert_eq!(nested_schema.kind(), Kind::Schema);
    assert!(nested_schema.parent().is_some());
    assert_eq!(nested_schema.state(), State::Invalid);
    let holder = nested_ids
        .get(&illegal.parent().ok_or("illegal type parent missing")?)
        .ok_or("Holder missing")?;
    assert_eq!(holder.kind(), Kind::GlobalElement);
    assert_eq!(
        holder.name().ok_or("Holder name missing")?.local_name(),
        "Holder"
    );
    let invalid_context = nested
        .issues()
        .iter()
        .find(|issue| {
            issue.component() == illegal.id() && issue.kind() == XmlSchemaIssueKind::InvalidContext
        })
        .ok_or("nested T lost its InvalidContext observation")?;
    assert_eq!(invalid_context.document(), "UI/schema.xsd");
    assert_eq!(invalid_context.occurrence(), illegal.occurrence());
    assert_eq!(invalid_context.span(), illegal.span());
    assert!(
        !nested
            .issues()
            .iter()
            .any(|issue| issue.kind() == XmlSchemaIssueKind::ConflictingDeclarations)
    );
    let uses = nested
        .components()
        .iter()
        .find(|component| {
            component.kind() == Kind::GlobalElement
                && component
                    .name()
                    .is_some_and(|name| name.local_name() == "Uses")
        })
        .ok_or("Uses element missing")?;
    assert_eq!(nested.references().len(), 3);
    let reference = nested
        .references()
        .iter()
        .find(|reference| reference.source() == uses.id())
        .ok_or("Uses type reference missing")?;
    assert_eq!(reference.source(), uses.id());
    assert_eq!(reference.kind(), RefKind::Type);
    assert_eq!(reference.state(), XmlSchemaReferenceState::Unique);
    assert_eq!(reference.candidates(), &[global.id()]);
    assert_eq!(reference.target().name(), global.name());
    assert_eq!(reference.target().lexical(), "ui:T");
    let uses_attribute = uses
        .attribute("type")
        .ok_or("Uses type attribute missing")?;
    assert_eq!(reference.value_span(), uses_attribute.value_span());
    assert_eq!(
        reference.decoded_value_digest(),
        uses_attribute.decoded_value_digest()
    );
    assert_eq!(span_text(NESTED_TYPE, reference.value_span())?, "ui:T");
    let conflicted = nested
        .components()
        .iter()
        .find(|component| {
            component.kind() == Kind::GlobalElement
                && component
                    .name()
                    .is_some_and(|name| name.local_name() == "Conflicted")
        })
        .ok_or("Conflicted element missing")?;
    assert_eq!(conflicted.state(), State::Invalid);
    let anonymous = nested
        .components()
        .iter()
        .find(|component| {
            component.kind() == Kind::AnonymousComplexType
                && component.parent() == Some(conflicted.id())
        })
        .ok_or("conflicted anonymous type missing")?;
    let content = nested
        .components()
        .iter()
        .find(|component| {
            component.kind() == Kind::ComplexContent && component.parent() == Some(anonymous.id())
        })
        .ok_or("conflicted complex content missing")?;
    let extension = nested
        .components()
        .iter()
        .find(|component| {
            component.kind() == Kind::Extension && component.parent() == Some(content.id())
        })
        .ok_or("conflicted extension missing")?;
    for component in [anonymous, content, extension] {
        assert_eq!(component.state(), State::Unsupported);
    }
    for (source, kind) in [(conflicted, RefKind::Type), (extension, RefKind::Base)] {
        let reference = nested
            .references()
            .iter()
            .find(|reference| reference.source() == source.id())
            .ok_or("conflicted reference missing")?;
        assert_eq!(reference.kind(), kind);
        assert_eq!(
            reference.state(),
            XmlSchemaReferenceState::UnsupportedContext
        );
        assert_eq!(reference.candidates(), &[global.id()]);
        assert_eq!(reference.target().name(), global.name());
        assert_eq!(span_text(NESTED_TYPE, reference.value_span())?, "ui:T");
    }
    let nested_document = nested
        .documents()
        .first()
        .ok_or("nested fixture index missing")?;
    for component in nested.components() {
        assert_eq!(component.document(), "UI/schema.xsd");
        assert_eq!(
            component.content_digest(),
            raw_digest(NESTED_TYPE.as_bytes())
        );
        assert_eq!(
            component.span(),
            &nested_document
                .element(component.occurrence())
                .ok_or("nested component index missing")?
                .span
        );
        span_text(NESTED_TYPE, component.span())?;
    }

    let cancelled = AtomicBool::new(true);
    let raw_cancel = source_a
        .raw_member("UI/schema.xsd", &cancelled)
        .err()
        .ok_or("cancelled raw read succeeded")?;
    assert_eq!(raw_cancel.code(), ProjectErrorCode::SourceReadCancelled);
    assert_eq!(raw_cancel.phase(), ProjectPhase::Inventory);
    refuses(
        XmlSchemaSelection::for_source(&source_a, &["UI/schema.xsd"], &cancelled),
        raw_cancel.code(),
        raw_cancel.phase(),
    )?;
    refuses(
        admit_xml_schema(&source_a, &selection, &cancelled),
        raw_cancel.code(),
        raw_cancel.phase(),
    )?;
    refuses(
        schema.validate_source(&source_a, &cancelled),
        raw_cancel.code(),
        raw_cancel.phase(),
    )?;
    schema.validate_source(&source_a, &stop)?;
    snapshot.validate()?;
    assert_eq!(view.snapshot_id(), snapshot_before);
    assert_eq!(view.file_manifest(), main_before);
    assert_eq!(loaded.load_plan().digest(), load_before);
    assert!(view.source_artifact("UI/schema.xsd")?.is_none());
    Ok(())
}

#[test]
fn native_admission_retains_bytes_and_separate_identity_scopes() -> TestResult {
    let root = FixtureRoot::new()?;
    let copied = FixtureRoot::new()?;
    let stop = AtomicBool::new(false);
    let profile = BlizzardUiSourceProfile::new(profile_request()?)?;
    let declared = inventory(&root, &profile)?;
    let admitted = root
        .directory()?
        .admit_platform_source(&profile, declared.clone(), &stop)?;
    let mut reordered = declared.clone();
    reordered.entries.reverse();
    let independent = copied
        .directory()?
        .admit_platform_source(&profile, reordered, &stop)?;
    assert_eq!(admitted.receipt(), independent.receipt());
    assert_eq!(admitted.source_bytes("UI/opaque.bin")?, &[0xff, 0xfe, 0, 1]);
    assert_eq!(admitted.receipt().coverage().verified_files(), 4);
    assert_eq!(
        admitted.receipt().coverage().inventory(),
        CoverageStatus::Partial
    );
    assert_eq!(
        admitted.receipt().coverage().declared_included_bytes(),
        CoverageStatus::Complete
    );
    assert!(
        admitted
            .receipt()
            .coverage()
            .unevaluated()
            .contains(&PlatformUnevaluatedCapability::Runtime)
    );
    assert!(
        admitted
            .receipt()
            .coverage()
            .unevaluated()
            .contains(&PlatformUnevaluatedCapability::RootCompleteness)
    );
    assert_eq!(admitted.receipt().inventory().entries.len(), 6);
    assert_eq!(
        admitted
            .source_bytes("UI/linked.lua")
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::FileNotPresent)
    );
    let original = admitted.source_bytes("UI/defs.lua")?.to_vec();
    std::fs::write(root.0.join("UI/defs.lua"), b"changed after admission")?;
    assert_eq!(admitted.source_bytes("UI/defs.lua")?, original);
    let mut renamed = declared.clone();
    renamed.origin.provider = "renamed-display-provider".into();
    let display_rename = copied
        .directory()?
        .admit_platform_source(&profile, renamed, &stop)?;
    assert_eq!(
        admitted.receipt().source_snapshot_id(),
        display_rename.receipt().source_snapshot_id()
    );
    assert_eq!(
        admitted.receipt().content_manifest_digest(),
        display_rename.receipt().content_manifest_digest()
    );
    assert_ne!(
        admitted.receipt().admission_digest(),
        display_rename.receipt().admission_digest()
    );
    let decoded =
        PlatformSourceInventory::from_json(&serde_json::to_vec(&declared)?, &profile, &stop)?;
    assert_eq!(decoded.entries, independent.receipt().inventory().entries);
    assert!(
        root.directory()?
            .admit_platform_source(&profile, declared, &stop)
            .is_err()
    );
    Ok(())
}

#[test]
fn native_raw_inventory_borrows_included_bytes_and_keeps_terminal_cancellation() -> TestResult {
    let root = FixtureRoot::new()?;
    let stop = AtomicBool::new(false);
    let profile = BlizzardUiSourceProfile::new(profile_request()?)?;
    let admitted =
        root.directory()?
            .admit_platform_source(&profile, inventory(&root, &profile)?, &stop)?;
    let receipt = admitted.receipt().clone();
    let included: Vec<_> = receipt
        .inventory()
        .entries
        .iter()
        .filter_map(|entry| match &entry.disposition {
            PlatformEntryDisposition::Included {
                digest,
                byte_length,
                ..
            } => Some((entry.path.as_str(), entry.kind, *digest, *byte_length)),
            _ => None,
        })
        .collect();
    assert_eq!(
        included.iter().map(|entry| entry.0).collect::<Vec<_>>(),
        [
            "UI/Fixture.toc",
            "UI/defs.lua",
            "UI/frames.xml",
            "UI/opaque.bin",
        ]
    );
    assert_eq!(receipt.inventory().entries.len(), 6);
    assert_eq!(receipt.coverage().inventory(), CoverageStatus::Partial);
    assert_eq!(receipt.coverage().verified_files(), included.len());
    // Admission owns the bytes; neither constructing nor draining a cursor can reread disk.
    std::fs::remove_dir_all(&root.0)?;
    assert!(!root.0.exists());

    stop.store(true, Ordering::Relaxed);
    let constructor_error = admitted
        .raw_inventory(&stop)
        .err()
        .ok_or("cancelled raw cursor constructor succeeded")?;
    assert_eq!(
        constructor_error.code(),
        ProjectErrorCode::SourceReadCancelled
    );
    stop.store(false, Ordering::Relaxed);

    let mut interrupted = admitted.raw_inventory(&stop)?;
    assert!(std::ptr::eq(interrupted.receipt(), admitted.receipt()));
    assert_eq!(
        interrupted
            .next(&stop)?
            .ok_or("raw cursor omitted its first Included member")?
            .path(),
        included.first().ok_or("fixture has no Included members")?.0
    );
    stop.store(true, Ordering::Relaxed);
    let cancellation = interrupted
        .next(&stop)
        .err()
        .ok_or("mid-cursor cancellation succeeded")?;
    assert_eq!(cancellation.code(), ProjectErrorCode::SourceReadCancelled);
    stop.store(false, Ordering::Relaxed);
    for _ in 0..2 {
        assert_eq!(
            interrupted
                .next(&stop)
                .err()
                .ok_or("failed raw cursor resumed after cancellation was cleared")?,
            cancellation
        );
    }
    assert_eq!(interrupted.receipt(), &receipt);

    let mut cursor = admitted.raw_inventory(&stop)?;
    let mut yielded_paths = Vec::new();
    let mut yielded_bytes = 0;
    for &(path, kind, digest, byte_length) in &included {
        let member = cursor
            .next(&stop)?
            .ok_or("raw cursor exhausted before all Included members")?;
        assert_eq!(member.path(), path);
        assert_eq!(member.kind(), kind);
        assert_eq!(member.content_digest(), digest);
        assert_eq!(raw_digest(member.bytes()), digest);
        assert_eq!(member.byte_length(), byte_length);
        assert_eq!(u64::try_from(member.bytes().len())?, byte_length);
        assert!(std::ptr::eq(
            member.bytes(),
            admitted.source_bytes(member.path())?
        ));
        if path == "UI/opaque.bin" {
            assert_eq!(member.kind(), PlatformFileKind::Unknown);
            assert_eq!(member.bytes(), &[0xff, 0xfe, 0, 1]);
        }
        yielded_paths.push(member.path());
        yielded_bytes += member.byte_length();
    }
    for _ in 0..3 {
        assert!(cursor.next(&stop)?.is_none());
    }
    assert_eq!(yielded_paths.len(), receipt.coverage().verified_files());
    assert_eq!(yielded_bytes, receipt.coverage().verified_bytes());
    for entry in &receipt.inventory().entries {
        if !matches!(
            &entry.disposition,
            PlatformEntryDisposition::Included { .. }
        ) {
            assert!(!yielded_paths.contains(&entry.path.as_str()));
            assert_eq!(
                admitted
                    .source_bytes(&entry.path)
                    .err()
                    .ok_or("omitted raw inventory entry yielded bytes")?
                    .code(),
                ProjectErrorCode::FileNotPresent
            );
        }
    }
    assert_eq!(cursor.receipt(), &receipt);
    assert_eq!(admitted.receipt(), &receipt);
    assert_eq!(admitted.profile().digest(), profile.digest());
    Ok(())
}

#[test]
fn native_admission_refuses_stale_binding_invalid_inventory_and_unresolved_content() -> TestResult {
    let root = FixtureRoot::new()?;
    let directory = root.directory()?;
    let stop = AtomicBool::new(false);
    let profile = BlizzardUiSourceProfile::new(profile_request()?)?;
    let declared = inventory(&root, &profile)?;
    let mut wrong = declared.clone();
    wrong.target.channel = "other-fixture".into();
    assert!(
        directory
            .admit_platform_source(&profile, wrong, &stop)
            .is_err()
    );
    let mut wrong = declared.clone();
    wrong.profile_digest = ContentDigest::from_bytes([9; 32]);
    assert!(
        directory
            .admit_platform_source(&profile, wrong, &stop)
            .is_err()
    );
    let mut collision = declared.clone();
    collision.entries[0].path = "UI/defs.lua".into();
    assert!(
        directory
            .admit_platform_source(&profile, collision, &stop)
            .is_err()
    );
    let mut escape = declared.clone();
    escape.entries[0].path = "UI/../outside.toc".into();
    assert!(
        directory
            .admit_platform_source(&profile, escape, &stop)
            .is_err()
    );
    let mut missing_accounting = declared.clone();
    missing_accounting.entries.pop();
    assert!(
        directory
            .admit_platform_source(&profile, missing_accounting, &stop)
            .is_err()
    );
    let mut limit_request = profile_request()?;
    limit_request.limits.max_entries = 1;
    let limited = BlizzardUiSourceProfile::new(limit_request)?;
    let mut too_many = declared.clone();
    too_many.profile_digest = limited.digest();
    assert_eq!(
        directory
            .admit_platform_source(&limited, too_many, &stop)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::SourceBudgetExceeded)
    );
    let cancelled = AtomicBool::new(true);
    assert_eq!(
        directory
            .admit_platform_source(&profile, declared.clone(), &cancelled)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::SourceReadCancelled)
    );
    std::fs::remove_file(root.0.join("UI/defs.lua"))?;
    assert_eq!(
        directory
            .admit_platform_source(&profile, declared.clone(), &stop)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::MissingDeclaredFile)
    );
    let pointer = b"version https://git-lfs.github.com/spec/v1\noid sha256:fixture\nsize 1\n";
    std::fs::write(root.0.join("UI/defs.lua"), pointer)?;
    let mut unresolved = declared;
    let entry = unresolved
        .entries
        .iter_mut()
        .find(|entry| entry.path == "UI/defs.lua")
        .ok_or_else(|| std::io::Error::other("fixture Lua member missing"))?;
    entry.disposition = PlatformEntryDisposition::Included {
        digest: raw_digest(pointer),
        byte_length: pointer.len() as u64,
        object_id: None,
    };
    assert_eq!(
        directory
            .admit_platform_source(&profile, unresolved, &stop)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::InvalidInputInventory)
    );
    Ok(())
}

#[test]
fn platform_packages_reuse_native_load_from_retained_bytes_and_refuse_declared_omissions()
-> TestResult {
    let root = FixtureRoot::new()?;
    let stop = AtomicBool::new(false);
    let profile = BlizzardUiSourceProfile::new(profile_request()?)?;
    let declared = inventory(&root, &profile)?;
    let source = Arc::new(
        root.directory()?
            .admit_platform_source(&profile, declared, &stop)?,
    );
    let packages = fixture_packages("Fixture");
    // No filesystem input remains for the native TOC/XML/package owners to reread.
    std::fs::remove_dir_all(&root.0)?;
    let loaded = source.specialize_packages(&packages, None, &stop)?;
    assert!(Arc::ptr_eq(loaded.source(), &source));
    assert_eq!(loaded.files().len(), 1);
    assert_eq!(
        loaded.files()[0].relative_path().as_str(),
        "packages/Fixture/defs.lua"
    );
    assert_eq!(
        loaded.binding().source_snapshot_id(),
        source.receipt().source_snapshot_id()
    );
    assert_eq!(loaded.binding().load_digest(), loaded.load_plan().digest());
    assert_eq!(loaded.binding().main_digest(), loaded.main_plan().digest());
    assert_eq!(
        loaded.source().source_bytes("UI/opaque.bin")?,
        &[0xff, 0xfe, 0, 1]
    );
    assert_eq!(
        loaded.source().receipt().coverage().inventory(),
        CoverageStatus::Partial
    );
    assert!(
        loaded
            .binding()
            .universe_id()
            .starts_with("blizzard_ui_source:")
    );

    let wrong_pin = vec![ProjectPackageInput::new(
        "Fixture",
        "UI",
        true,
        vec![ProjectPackageVariantInput::new(
            wow_project::disk::ProjectDiskFile::new("Fixture.toc").with_identity(
                ContentDigest::from_bytes([9; 32]),
                source.source_bytes("UI/Fixture.toc")?.len() as u64,
            ),
            true,
        )],
    )];
    assert_eq!(
        source
            .specialize_packages(&wrong_pin, None, &stop)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::FileDigestMismatch)
    );
    assert_eq!(
        source
            .specialize_packages(&packages, None, &AtomicBool::new(true))
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::SourceReadCancelled)
    );

    let excluded_root = FixtureRoot::new()?;
    let mut exclusion_request = profile_request()?;
    exclusion_request.exclusions.push(ProfileExclusion {
        path: "UI/defs.lua".into(),
    });
    let exclusion_profile = BlizzardUiSourceProfile::new(exclusion_request)?;
    let mut excluded_inventory = inventory(&excluded_root, &exclusion_profile)?;
    let entry = excluded_inventory
        .entries
        .iter_mut()
        .find(|entry| entry.path == "UI/defs.lua")
        .ok_or_else(|| std::io::Error::other("fixture Lua record missing"))?;
    entry.disposition = PlatformEntryDisposition::Excluded {
        rule_path: "UI/defs.lua".into(),
    };
    let excluded = Arc::new(excluded_root.directory()?.admit_platform_source(
        &exclusion_profile,
        excluded_inventory,
        &stop,
    )?);
    let refused = excluded
        .specialize_packages(&packages, None, &stop)
        .err()
        .ok_or_else(|| std::io::Error::other("excluded load target was accepted"))?;
    assert_eq!(refused.code(), ProjectErrorCode::PackageTargetExcluded);
    assert_eq!(refused.relative_path(), Some("UI/defs.lua"));

    let unsupported_root = FixtureRoot::new()?;
    let toc_path = unsupported_root.0.join("UI/Fixture.toc");
    let mut toc = std::fs::read(&toc_path)?;
    toc.extend_from_slice(b"\nlinked.lua\n");
    std::fs::write(&toc_path, toc)?;
    let unsupported_inventory = inventory(&unsupported_root, &profile)?;
    let unsupported = Arc::new(unsupported_root.directory()?.admit_platform_source(
        &profile,
        unsupported_inventory,
        &stop,
    )?);
    assert_eq!(
        unsupported
            .specialize_packages(&packages, None, &stop)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::InvalidFileLanguage)
    );
    Ok(())
}

#[test]
fn platform_configuration_publishes_native_main_and_replays_without_source() -> TestResult {
    let root = FixtureRoot::new()?;
    let stop = AtomicBool::new(false);
    let profile = BlizzardUiSourceProfile::new(profile_request()?)?;
    let declared = inventory(&root, &profile)?;
    let source = Arc::new(
        root.directory()?
            .admit_platform_source(&profile, declared, &stop)?,
    );
    std::fs::remove_dir_all(&root.0)?;
    let loaded = Arc::new(source.specialize_packages(&fixture_packages("Fixture"), None, &stop)?);
    let competing =
        Arc::new(source.specialize_packages(&fixture_packages("Other"), None, &stop)?);

    let builder = configuration_builder(ProjectKind::BlizzardUiPlatformSource, profile.target())?;
    assert_eq!(
        builder.clone().build().err().map(|error| error.code()),
        Some(ProjectErrorCode::InvalidConfiguration)
    );
    assert_eq!(
        builder
            .clone()
            .package_load_plan(loaded.load_plan(), loaded.main_plan())?
            .build()
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::InvalidConfiguration)
    );
    for kind in [ProjectKind::Fixture, ProjectKind::Repository] {
        assert_eq!(
            configuration_builder(kind, profile.target())?
                .platform_packages(Arc::clone(&loaded))
                .err()
                .map(|error| error.code()),
            Some(ProjectErrorCode::InvalidConfiguration)
        );
    }
    let mut wrong_target = profile.target().clone();
    wrong_target.reference_generation = ReferenceGenerationId::from_hash([9; 32]);
    assert_eq!(
        configuration_builder(ProjectKind::BlizzardUiPlatformSource, &wrong_target)?
            .platform_packages(Arc::clone(&loaded))
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::InvalidConfiguration)
    );
    let builder = builder.platform_packages(Arc::clone(&loaded))?;
    let configuration = builder.clone().build()?;
    assert_eq!(
        builder
            .clone()
            .package_load_plan(loaded.load_plan(), loaded.main_plan())?
            .build()?,
        configuration
    );
    assert_eq!(
        builder
            .clone()
            .platform_packages(Arc::clone(&loaded))?
            .build()?,
        configuration
    );
    assert_eq!(
        builder
            .clone()
            .package_load_plan(competing.load_plan(), competing.main_plan())
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::InvalidConfiguration)
    );
    assert_eq!(
        builder
            .clone()
            .platform_packages(competing)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::InvalidConfiguration)
    );
    let single_plan = loaded
        .load_plan()
        .package_plan("Fixture")
        .ok_or_else(|| std::io::Error::other("fixture package receipt missing"))?;
    assert_eq!(
        builder
            .load_plan(single_plan)
            .err()
            .map(|error| error.code()),
        Some(ProjectErrorCode::InvalidConfiguration)
    );

    configuration.validate()?;
    assert_eq!(
        configuration.project_kind(),
        ProjectKind::BlizzardUiPlatformSource
    );
    assert_eq!(
        configuration.platform_package_binding(),
        Some(loaded.binding())
    );
    assert_eq!(
        configuration.platform_package_binding_digest(),
        Some(loaded.binding().binding_digest())
    );
    let retained = configuration
        .platform_packages()
        .ok_or_else(|| std::io::Error::other("platform owner missing"))?;
    assert!(Arc::ptr_eq(retained.source(), &source));
    assert_eq!(retained.files(), loaded.files());
    assert_eq!(configuration.package_load_plan(), Some(loaded.load_plan()));
    assert_eq!(configuration.package_main_plan(), Some(loaded.main_plan()));

    let library = LuaWorkspaceSnapshot::build(
        configuration.analyzer_binding().backend().clone(),
        LuaWorkspaceUniverse::BlizzardUi,
        vec![LuaWorkspaceFileInput::new(
            "library/platform-fixture.lua",
            "---@meta _\n---@class C_PlatformFixture\n---@field KnownApi fun(): boolean\nC_PlatformFixture = {}\n",
        )],
        LuaWorkspaceLimits::new(8, 16_384, 256 * 1024, 512 * 1024)?,
    )?;
    let mut publisher = ProjectPublisher::with_function_call_facts();
    let snapshot = publisher.publish_initial_cancellable(
        ProjectInputBundle::closed(
            configuration.clone(),
            loaded.files().to_vec(),
            vec![library.clone()],
        )?,
        &stop,
    )?;
    snapshot.validate()?;
    assert_eq!(snapshot.configuration(), &configuration);
    let analyzer = snapshot.analyzer_binding();
    assert_eq!(
        analyzer.main_workspace().universe(),
        LuaWorkspaceUniverse::BlizzardUiMain
    );
    assert_eq!(
        analyzer.library_snapshot_ids().collect::<Vec<_>>(),
        vec![library.snapshot_id()]
    );
    let xml = analyzer
        .xml_lua_analysis()
        .ok_or_else(|| std::io::Error::other("XML virtual analysis missing"))?;
    assert!(!xml.units().is_empty());
    let mut virtual_files = Vec::new();
    for unit in xml.units() {
        assert_eq!(unit.package.as_deref(), Some("Fixture"));
        let index = single_plan
            .xml_documents()
            .iter()
            .find_map(|(path, index)| {
                (loaded.load_plan().source_path("Fixture", path).as_deref()
                    == Some(unit.document.as_str()))
                .then_some(index)
            })
            .ok_or_else(|| std::io::Error::other("virtual unit document missing"))?;
        let body = index
            .element(&unit.script_occurrence_id)
            .and_then(|element| element.script.as_ref())
            .and_then(|script| script.inline_lua.as_ref())
            .ok_or_else(|| std::io::Error::other("virtual unit source missing"))?;
        assert_eq!(unit.extracted_unit_id, body.unit_id);
        assert_eq!(unit.content_digest, body.content_digest);
        virtual_files.push(LuaWorkspaceFileInput::new(&unit.virtual_path, body.text()));
    }
    // Workspace identity includes the universe; use native unwrapped retained units.
    let virtual_limits = LuaWorkspaceLimits::new(64, 16_384, 256 * 1024, 2 * 1024 * 1024)?;
    let mismatched_virtual = LuaWorkspaceSnapshot::build(
        configuration.analyzer_binding().backend().clone(),
        LuaWorkspaceUniverse::Project,
        virtual_files.clone(),
        virtual_limits,
    )?;
    let expected_virtual = LuaWorkspaceSnapshot::build(
        configuration.analyzer_binding().backend().clone(),
        LuaWorkspaceUniverse::BlizzardUiMain,
        virtual_files,
        virtual_limits,
    )?;
    assert_eq!(
        wow_emmy::analyze_member_calls(analyzer.main_workspace(), &[analyzer.main_workspace()])
            .err()
            .map(|error| error.code()),
        Some(EmmyMemberCallErrorCode::InvalidLibraryWorkspace)
    );
    assert_eq!(
        wow_emmy::references::analyze_member_call_session_with_virtual(
            analyzer.main_workspace(),
            &[&library],
            &mismatched_virtual,
            snapshot.project_generation(),
            &[],
            false,
            &stop,
        )
        .err()
        .map(|error| error.code()),
        Some(EmmyMemberCallErrorCode::InvalidMainWorkspace)
    );
    let semantics = xml
        .semantic_report()
        .ok_or_else(|| std::io::Error::other("XML virtual semantics missing"))?;
    assert_eq!(
        semantics.virtual_snapshot_id(),
        expected_virtual.snapshot_id()
    );
    assert_eq!(
        semantics.main_snapshot_id(),
        analyzer.main_workspace().snapshot_id()
    );
    assert_eq!(
        semantics.project_generation(),
        snapshot.project_generation()
    );
    assert_eq!(
        semantics.library_snapshot_ids().collect::<Vec<_>>(),
        vec![library.snapshot_id()]
    );

    let registry = snapshot.source_registry();
    let origin = registry.source_origin();
    assert_eq!(
        origin.origin_kind(),
        ProjectSourceOriginKind::BlizzardUiPlatformSource
    );
    assert_eq!(
        origin.revision_identity(),
        loaded.binding().source_snapshot_id()
    );
    assert_eq!(origin.project_generation(), snapshot.project_generation());
    let view = publisher.open_current()?;
    for file in snapshot.file_manifest() {
        registry.validate_source_handle(file.source_handle_base())?;
        assert_eq!(
            file.source_handle_base().origin_kind(),
            SourceOriginKind::Fixture
        );
        assert_eq!(
            file.source_handle_base().revision(),
            loaded.binding().source_snapshot_id()
        );
        assert_eq!(
            file.source_handle_base().project_generation(),
            Some(snapshot.project_generation())
        );
        assert_eq!(
            view.source_handle(
                file.relative_path().as_str(),
                SourceSpan::whole_file(),
                file.source_handle_base().entity_key().cloned()
            )?,
            *file.source_handle_base()
        );
    }
    let (registry, batch, coverage, provenance, limits) =
        wow_project::graph::build_source_graph_proposals(&view, &stop)?.into_parts();
    assert_eq!(batch.universe().as_str(), loaded.binding().universe_id());
    for kind in ["toc_manifest", "xml_object"] {
        let definition = registry
            .entity_kind(kind)
            .ok_or_else(|| std::io::Error::other("platform structural registry kind missing"))?;
        assert!(definition.allows_universe(batch.universe()));
    }
    assert_eq!(provenance.context(), snapshot.generation_context());
    assert!(
        provenance
            .source_handles()
            .values()
            .any(|handle| handle.path().as_str() == "packages/Fixture/frames.xml")
    );
    for handle in provenance.source_handles().values() {
        assert_eq!(handle.origin_kind(), SourceOriginKind::Fixture);
        assert_eq!(
            handle.origin_id(),
            configuration.source_origin_id().as_str()
        );
        assert_eq!(handle.revision(), loaded.binding().source_snapshot_id());
        assert_eq!(
            handle.project_generation(),
            Some(snapshot.project_generation())
        );
        assert_eq!(
            handle.reference_generation(),
            Some(configuration.reference_generation())
        );
        assert_eq!(
            view.source_handle(
                handle.path().as_str(),
                handle.span(),
                handle.entity_key().cloned()
            )?,
            *handle
        );
    }
    let foundation = wow_graph::GraphSnapshot::build(
        batch.universe().clone(),
        batch.generation().clone(),
        limits,
        Vec::new(),
        Vec::new(),
        coverage.clone(),
    )?;
    let owner = wow_graph::GraphPartitionSnapshot::new(
        registry,
        foundation,
        batch.source_context_id(),
        &stop,
    )?;
    let replacement = owner.prepare_replacement(
        wow_graph::GraphPartitionReplacement {
            expected_snapshot_id: owner.snapshot().snapshot_id().clone(),
            expected_partition_digest: None,
            producer_version: env!("CARGO_PKG_VERSION").into(),
            batch,
            coverage,
        },
        &stop,
    )?;
    replacement.candidate().validate(&stop)?;
    assert!(!replacement.candidate().snapshot().nodes().is_empty());
    assert!(!root.0.exists());
    let replay = ProjectReplay::capture(&publisher, &stop)?;
    let archive = serde_json::to_value(&replay)?;
    assert_eq!(archive["schema"], "wow-project/native-project-replay/5");
    assert_eq!(archive["generation_schema_version"], 2);
    assert_eq!(archive["function_calls"], true);
    assert!(archive["files"].as_array().is_some_and(Vec::is_empty));
    assert!(archive.get("load").is_none());
    assert!(archive.get("packages").is_none());
    let raw_files = archive["platform"]["files"]
        .as_array()
        .ok_or_else(|| std::io::Error::other("platform raw archive missing"))?;
    assert_eq!(raw_files.len(), 4);
    let opaque_index = raw_files
        .iter()
        .position(|file| file["path"] == "UI/opaque.bin")
        .ok_or_else(|| std::io::Error::other("archived binary member missing"))?;
    assert_eq!(
        raw_files[opaque_index]["bytes"],
        serde_json::json!([255, 254, 0, 1])
    );
    let encoded = serde_json::to_vec(&replay)?;
    let decoded = ProjectReplay::from_json(&encoded, &stop)?;
    assert_eq!(decoded, replay);
    let wire = std::str::from_utf8(&encoded)?;
    for replacement in [
        r#""bytes":[255,254,0,1],"bytes":[255,254,0,1]"#,
        r#""bytes":[255,254,0,1],"unexpected":true"#,
    ] {
        let malformed = wire.replacen(r#""bytes":[255,254,0,1]"#, replacement, 1);
        assert_ne!(malformed, wire);
        assert!(ProjectReplay::from_json(malformed.as_bytes(), &stop).is_err());
    }
    let replayed = decoded.hydrate(&stop)?;
    assert_eq!(replayed.snapshot(), &snapshot);
    let replayed_packages = replayed
        .configuration()
        .platform_packages()
        .ok_or_else(|| std::io::Error::other("replayed platform owner missing"))?;
    assert_eq!(replayed_packages.binding(), loaded.binding());
    assert_eq!(replayed_packages.source().receipt(), source.receipt());
    assert_eq!(
        replayed_packages.source().source_bytes("UI/opaque.bin")?,
        &[0xff, 0xfe, 0, 1]
    );
    assert_eq!(replayed_packages.load_plan(), loaded.load_plan());
    assert_eq!(replayed_packages.main_plan(), loaded.main_plan());
    let mut recaptured = ProjectPublisher::with_function_call_facts();
    recaptured.publish_initial_cancellable(
        ProjectInputBundle::closed(
            replayed.configuration().clone(),
            replayed_packages.files().to_vec(),
            vec![library.clone()],
        )?,
        &stop,
    )?;
    assert_eq!(ProjectReplay::capture(&recaptured, &stop)?, replay);
    let mut mixed = archive.clone();
    mixed["files"] = mixed["libraries"][0]["files"].clone();
    assert!(
        ProjectReplay::from_json(&serde_json::to_vec(&mixed)?, &stop)
            .and_then(|replay| replay.hydrate(&stop))
            .is_err()
    );
    let mut wrong_library = archive.clone();
    wrong_library["libraries"][0]["universe"] = "blizzard_ui_main".into();
    assert!(
        ProjectReplay::from_json(&serde_json::to_vec(&wrong_library)?, &stop)
            .and_then(|replay| replay.hydrate(&stop))
            .is_err()
    );
    let changed_context = serde_json::to_value(wow_project::load::TocLoadContext {
        game_types: std::collections::BTreeMap::new(),
        family: Some("retail".into()),
        game: None,
        text_locale: None,
        location: None,
        environment: None,
    })?;
    for mutation in [
        "duplicate raw member",
        "surplus raw member",
        "missing raw member",
        "altered raw bytes",
        "altered package root",
        "altered TOC pin",
        "altered load context",
        "ordinary replay schema",
    ] {
        let mut changed = archive.clone();
        match mutation {
            "duplicate raw member" | "surplus raw member" => {
                let files = changed["platform"]["files"]
                    .as_array_mut()
                    .ok_or_else(|| std::io::Error::other("raw archive missing"))?;
                let mut extra = files[opaque_index].clone();
                if mutation == "surplus raw member" {
                    extra["path"] = "UI/surplus.bin".into();
                }
                files.push(extra);
            }
            "missing raw member" => {
                changed["platform"]["files"]
                    .as_array_mut()
                    .ok_or_else(|| std::io::Error::other("raw archive missing"))?
                    .remove(opaque_index);
            }
            "altered raw bytes" => {
                changed["platform"]["files"][opaque_index]["bytes"][3] = 2.into();
            }
            "altered package root" => {
                changed["platform"]["package_request"]["packages"][0]["root"] = "UI/other".into();
            }
            "altered TOC pin" => {
                changed["platform"]["package_request"]["packages"][0]["variants"][0]["toc"]["content_digest"] =
                    serde_json::to_value(raw_digest(b"wrong pin"))?;
            }
            "altered load context" => {
                changed["platform"]["package_request"]["context"] = changed_context.clone();
            }
            _ => changed["schema"] = "wow-project/native-project-replay/4".into(),
        }
        assert!(
            ProjectReplay::from_json(&serde_json::to_vec(&changed)?, &stop)
                .and_then(|replay| replay.hydrate(&stop))
                .is_err(),
            "platform replay accepted {mutation}"
        );
    }
    assert!(!root.0.exists());
    assert_eq!(
        publisher.open_current()?.snapshot_id(),
        snapshot.snapshot_id()
    );

    // A retained inventory-only byte change must bypass NoChange even when
    // every parsed load/Main input and Library is identical.
    let changed_root = FixtureRoot::new()?;
    std::fs::write(changed_root.0.join("UI/opaque.bin"), [0xff, 0xfe, 0, 2])?;
    let changed_inventory = inventory(&changed_root, &profile)?;
    let changed_source = Arc::new(changed_root.directory()?.admit_platform_source(
        &profile,
        changed_inventory,
        &stop,
    )?);
    std::fs::remove_dir_all(&changed_root.0)?;
    let changed_packages =
        Arc::new(changed_source.specialize_packages(&fixture_packages("Fixture"), None, &stop)?);
    assert_eq!(changed_packages.files(), loaded.files());
    assert_eq!(changed_packages.load_plan(), loaded.load_plan());
    assert_eq!(changed_packages.main_plan(), loaded.main_plan());
    let changed_configuration =
        configuration_builder(ProjectKind::BlizzardUiPlatformSource, profile.target())?
            .platform_packages(changed_packages)?
            .build()?;
    assert_ne!(
        changed_configuration.configuration_digest(),
        configuration.configuration_digest()
    );
    let request = wow_project::ProjectUpdateRequest::new(changed_configuration, Vec::new())
        .expected_generation(snapshot.project_generation())
        .expected_snapshot_digest(snapshot.canonical_snapshot_digest());
    let wow_project::ProjectUpdateOutcome::Published(changed_snapshot) =
        publisher.apply_update_cancellable(request, &stop)?
    else {
        return Err(std::io::Error::other("changed raw inventory was treated as NoChange").into());
    };
    assert_ne!(
        changed_snapshot.project_generation(),
        snapshot.project_generation()
    );
    assert_eq!(
        changed_snapshot
            .analyzer_binding()
            .main_workspace()
            .snapshot_id(),
        analyzer.main_workspace().snapshot_id()
    );
    snapshot.validate()?;
    Ok(())
}
