use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use sha2::{Digest, Sha256};
use wow_core::{
    CanonicalResult, ContentDigest, CoverageStatus, ProfileIdentityBuilder, ProfileKind,
    ReferenceGenerationId, SchemaVersionEntry, SourceContent, SourceKind, SourceLogicalSnapshot,
};
use wow_project::{
    ProjectErrorCode,
    disk::ProjectInputDirectory,
    platform_source::{
        BlizzardUiSourceProfile, BlizzardUiSourceProfileRequest, PlatformEntryDisposition,
        PlatformFileKind, PlatformInventoryEntry, PlatformInventoryScope, PlatformLicenseRecord,
        PlatformLicenseState, PlatformMaterializer, PlatformRootInventory, PlatformRootSpec,
        PlatformSourceClass, PlatformSourceInventory, PlatformSourceOrigin, PlatformSourceRevision,
        PlatformSpecialEntry, PlatformTarget, PlatformUnevaluatedCapability, ProfileExclusion,
        SourceAdmissionLimits,
    },
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
        100_000,
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
