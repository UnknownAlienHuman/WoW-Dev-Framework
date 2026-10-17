use super::*;
use std::path::PathBuf;
type TestResult = Result<(), Box<dyn std::error::Error>>;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let root = std::env::temp_dir().join(format!(
            "wow-private-overlay-{}-{}",
            std::process::id(),
            next(&NEXT_REGISTRATION)?
        ));
        std::fs::create_dir(&root)?;
        Ok(Self(root))
    }
    fn register(
        &self,
        scope: &OverlaySessionScope,
        stop: &AtomicBool,
    ) -> OverlayResult<OverlayWorkspace> {
        OverlayWorkspace::register(scope, &self.0, "workspace:private-test", limits(), stop)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn limits() -> OverlayLimits {
    OverlayLimits {
        max_documents: 4,
        max_document_bytes: 65536,
        max_private_bytes: 1048576,
        max_live_snapshots: 8,
        max_leases: 8,
        max_lines: 4096,
        max_edits: 16,
        max_work: 1048576,
    }
}
fn open(
    workspace: &mut OverlayWorkspace,
    scope: &OverlaySessionScope,
    text: &str,
    stop: &AtomicBool,
) -> OverlayResult<OverlaySnapshotDescriptor> {
    let uri = workspace.document_uri(scope, "main.lua")?;
    workspace.open_document(
        scope,
        OverlayOpenRequest {
            relative_path: "main.lua",
            uri: &uri,
            client_version: 0,
            content: text.into(),
        },
        stop,
    )
}
fn change<'a>(
    prior: &'a OverlaySnapshotDescriptor,
    edits: &'a [OverlayEdit],
) -> OverlayChangeRequest<'a> {
    OverlayChangeRequest {
        document_id: &prior.document_id,
        expected_snapshot: &prior.snapshot_id,
        expected_content: prior.content_digest,
        client_version: prior.client_version + 1,
        encoding: OverlayEncoding::Utf16,
        edits,
    }
}
fn range(line: u32, start: u32, end: u32, text: &str) -> OverlayEdit {
    OverlayEdit {
        range: Some(OverlayRange {
            start: OverlayPosition {
                line,
                character: start,
            },
            end: OverlayPosition {
                line,
                character: end,
            },
        }),
        text: text.into(),
    }
}

#[test]
fn sequential_unicode_edits_exact_replay_and_stale_guards() -> TestResult {
    let fixture = Fixture::new()?;
    let scope = OverlaySessionScope::begin()?;
    let stop = AtomicBool::new(false);
    let mut workspace = fixture.register(&scope, &stop)?;
    let original = open(&mut workspace, &scope, "a😀b\r\n中\n", &stop)?;
    let old = workspace.get(&scope, &original.document_id, &original.snapshot_id, &stop)?;
    let edits = [range(0, 1, 3, "XY"), range(0, 3, 4, "Z")];
    let changed = workspace.change_document(&scope, change(&original, &edits), &stop)?;
    assert_eq!(
        changed.content_digest,
        digest_bytes("aXYZ\r\n中\n".as_bytes())
    );
    assert_eq!(
        workspace.change_document(&scope, change(&original, &edits), &stop)?,
        changed
    );
    assert_eq!(old.descriptor(), &original);
    assert!(
        workspace
            .snapshots
            .iter()
            .filter_map(std::sync::Weak::upgrade)
            .any(|snapshot| snapshot.content.as_ref() == "a😀b\r\n中\n")
    );
    assert_eq!(
        workspace
            .change_document(&scope, change(&original, &[range(0, 1, 3, "Q")]), &stop)
            .err()
            .map(|e| e.code()),
        Some(OverlayErrorCode::ResynchronizationRequired)
    );
    assert_eq!(
        workspace
            .close_document(&scope, &changed.document_id)
            .err()
            .map(|e| e.code()),
        Some(OverlayErrorCode::Busy)
    );
    assert_eq!(
        workspace.close(&scope).err().map(|e| e.code()),
        Some(OverlayErrorCode::Busy)
    );
    drop(old);
    workspace.close_document(&scope, &changed.document_id)?;
    workspace.close(&scope)?;
    assert!(workspace.current.is_empty());
    assert!(workspace.snapshots.is_empty());
    assert!(workspace.directory.is_none());
    Ok(())
}

#[test]
fn invalid_later_change_cancel_and_limit_preserve_exact_current() -> TestResult {
    let fixture = Fixture::new()?;
    let scope = OverlaySessionScope::begin()?;
    let stop = AtomicBool::new(false);
    let mut workspace = fixture.register(&scope, &stop)?;
    let original = open(&mut workspace, &scope, "a😀b\n", &stop)?;
    let edits = [range(0, 0, 1, "A"), range(0, 2, 3, "bad")];
    assert_eq!(
        workspace
            .change_document(&scope, change(&original, &edits), &stop)
            .err()
            .map(|e| e.code()),
        Some(OverlayErrorCode::InvalidEncodingBoundary)
    );
    let edits = [OverlayEdit {
        range: None,
        text: "x".repeat(65537),
    }];
    assert_eq!(
        workspace
            .change_document(&scope, change(&original, &edits), &stop)
            .err()
            .map(|e| e.code()),
        Some(OverlayErrorCode::Limit)
    );
    stop.store(true, Ordering::Release);
    assert_eq!(
        workspace
            .change_document(&scope, change(&original, &[range(0, 0, 1, "A")]), &stop)
            .err()
            .map(|e| e.code()),
        Some(OverlayErrorCode::Cancelled)
    );
    stop.store(false, Ordering::Release);
    let current = workspace.get(&scope, &original.document_id, &original.snapshot_id, &stop)?;
    assert_eq!(current.descriptor(), &original);
    assert!(
        workspace
            .current
            .values()
            .any(|snapshot| snapshot.content.as_ref() == "a😀b\n")
    );
    drop(current);
    workspace.close(&scope)?;
    Ok(())
}

#[test]
fn session_and_registration_tokens_do_not_authorize_other_private_bytes() -> TestResult {
    let fixture = Fixture::new()?;
    let first = OverlaySessionScope::begin()?;
    let second = OverlaySessionScope::begin()?;
    let stop = AtomicBool::new(false);
    let mut one = fixture.register(&first, &stop)?;
    let mut two = fixture.register(&second, &stop)?;
    let a = open(&mut one, &first, "local a = 1", &stop)?;
    let b = open(&mut two, &second, "local a = 2", &stop)?;
    assert_ne!(a.document_id, b.document_id);
    assert_ne!(a.snapshot_id, b.snapshot_id);
    assert_eq!(
        one.get(&second, &a.document_id, &a.snapshot_id, &stop)
            .err()
            .map(|e| e.code()),
        Some(OverlayErrorCode::ScopeMismatch)
    );
    let held = one.get(&first, &a.document_id, &a.snapshot_id, &stop)?;
    assert_eq!(
        two.save_acknowledge(&second, &held).err().map(|e| e.code()),
        Some(OverlayErrorCode::ScopeMismatch)
    );
    drop(held);
    one.close(&first)?;
    two.close(&second)?;
    Ok(())
}

#[test]
fn explicit_paths_uri_collision_and_disk_reconciliation() -> TestResult {
    let fixture = Fixture::new()?;
    std::fs::write(fixture.0.join("main.lua"), b"local a = 1\n")?;
    let scope = OverlaySessionScope::begin()?;
    let stop = AtomicBool::new(false);
    let mut workspace = fixture.register(&scope, &stop)?;
    for path in [
        "../escape.lua",
        "a/../escape.lua",
        "C:/escape.lua",
        "main.lua:ads",
        "NUL.lua",
        "bad. /x.lua",
    ] {
        assert!(workspace.document_uri(&scope, path).is_err(), "{path}");
    }
    let doubled = fixture.0.to_str().ok_or("fixture path encoding")?.replace(
        std::path::MAIN_SEPARATOR_STR,
        &std::path::MAIN_SEPARATOR_STR.repeat(2),
    );
    let mut normalized = OverlayWorkspace::register(
        &scope,
        Path::new(&doubled),
        "workspace:normalized",
        limits(),
        &stop,
    )?;
    assert_eq!(
        workspace.document_uri(&scope, "main.lua")?,
        normalized.document_uri(&scope, "main.lua")?
    );
    normalized.close(&scope)?;
    assert!(
        workspace
            .open_document(
                &scope,
                OverlayOpenRequest {
                    relative_path: "main.lua",
                    uri: "file:///wrong/main.lua",
                    client_version: 0,
                    content: "secret".into()
                },
                &stop
            )
            .is_err()
    );
    let current = open(&mut workspace, &scope, "local a = 1\n", &stop)?;
    assert_eq!(
        open(&mut workspace, &scope, "local a = 1\n", &stop)?,
        current
    );
    assert!(open(&mut workspace, &scope, "other", &stop).is_err());
    if cfg!(windows) {
        let uri = workspace.document_uri(&scope, "MAIN.lua")?;
        assert!(
            workspace
                .open_document(
                    &scope,
                    OverlayOpenRequest {
                        relative_path: "MAIN.lua",
                        uri: &uri,
                        client_version: 0,
                        content: "other".into()
                    },
                    &stop
                )
                .is_err()
        );
    }
    let lease = workspace.get(&scope, &current.document_id, &current.snapshot_id, &stop)?;
    assert_eq!(
        workspace.save_acknowledge(&scope, &lease)?.state,
        OverlaySaveState::ContentAcknowledgedOnly
    );
    assert_eq!(
        workspace.reconcile_disk(&scope, &lease, &stop)?.state,
        OverlaySaveState::DiskReconciled
    );
    std::fs::write(fixture.0.join("main.lua"), b"external update\n")?;
    assert_eq!(
        workspace.reconcile_disk(&scope, &lease, &stop)?.state,
        OverlaySaveState::ConflictWithDisk
    );
    assert_eq!(
        std::fs::read(fixture.0.join("main.lua"))?,
        b"external update\n"
    );
    drop(lease);
    workspace.close(&scope)?;
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlink_root_and_descendant_are_never_followed() -> TestResult {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new()?;
    let target = fixture.0.join("target");
    let link = fixture.0.join("link");
    std::fs::create_dir(&target)?;
    std::fs::write(target.join("main.lua"), b"local a = 1\n")?;
    symlink(&target, &link)?;
    let scope = OverlaySessionScope::begin()?;
    let stop = AtomicBool::new(false);
    assert!(OverlayWorkspace::register(&scope, &link, "workspace:link", limits(), &stop).is_err());
    let mut workspace = fixture.register(&scope, &stop)?;
    let uri = workspace.document_uri(&scope, "link/main.lua")?;
    let current = workspace.open_document(
        &scope,
        OverlayOpenRequest {
            relative_path: "link/main.lua",
            uri: &uri,
            client_version: 0,
            content: "local a = 1\n".into(),
        },
        &stop,
    )?;
    let lease = workspace.get(&scope, &current.document_id, &current.snapshot_id, &stop)?;
    assert!(workspace.reconcile_disk(&scope, &lease, &stop).is_err());
    drop(lease);
    workspace.close(&scope)?;
    std::fs::remove_file(&link)?;
    Ok(())
}

#[test]
fn analyzer_input_private_copies_are_charged_until_actual_drop() -> TestResult {
    let fixture = Fixture::new()?;
    let scope = OverlaySessionScope::begin()?;
    let stop = AtomicBool::new(false);
    let mut selected = limits();
    selected.max_document_bytes = 4096;
    selected.max_private_bytes = 4096;
    let mut workspace =
        OverlayWorkspace::register(&scope, &fixture.0, "workspace:copy-budget", selected, &stop)?;
    let original = open(&mut workspace, &scope, &"x".repeat(1024), &stop)?;
    // Synthetic report/surface identity for the actual compiled workspace owner;
    // this fixture is not an upstream compatibility or donor qualification.
    let backend = wow_emmy::EmmyBackendIdentity::new(
        "emmylua_code_analysis",
        Some(wow_emmy::EMMYLUA_CODE_ANALYSIS_VERSION),
        wow_emmy::EMMYLUA_REVISION,
        wow_emmy::EMMYLUA_TREE,
        format!("sha256:{}", "1".repeat(64)),
        format!("sha256:{}", "2".repeat(64)),
    )?;
    let lease = workspace.get(&scope, &original.document_id, &original.snapshot_id, &stop)?;
    let input = workspace.analyzer_input(&scope, lease, backend.clone(), &stop)?;
    assert_eq!(input.workspace().files()[0].text(), "x".repeat(1024));
    let lease = workspace.get(&scope, &original.document_id, &original.snapshot_id, &stop)?;
    assert_eq!(
        workspace
            .analyzer_input(&scope, lease, backend.clone(), &stop)
            .err()
            .map(|e| e.code()),
        Some(OverlayErrorCode::Limit)
    );
    assert_eq!(
        workspace.close(&scope).err().map(|e| e.code()),
        Some(OverlayErrorCode::Busy)
    );
    drop(input);
    let lease = workspace.get(&scope, &original.document_id, &original.snapshot_id, &stop)?;
    let input = workspace.analyzer_input(&scope, lease, backend, &stop)?;
    drop(input);
    workspace.close(&scope)?;
    assert_eq!(workspace.analyzer_bytes.load(Ordering::Acquire), 0);
    Ok(())
}

#[test]
fn canonical_uri_inverse_is_scoped_and_rejects_encoded_escape_and_aliases() -> TestResult {
    let fixture = Fixture::new()?;
    let scope = OverlaySessionScope::begin()?;
    let other = OverlaySessionScope::begin()?;
    let stop = AtomicBool::new(false);
    let mut workspace = fixture.register(&scope, &stop)?;
    let path = "a b/中%.lua";
    let uri = workspace.document_uri(&scope, path)?;
    assert_eq!(
        workspace.relative_path_for_uri(&scope, &uri)?.as_ref(),
        path
    );
    assert!(workspace.relative_path_for_uri(&other, &uri).is_err());
    let parent = workspace.document_uri(&scope, "main.lua")?;
    let base = parent
        .strip_suffix("main.lua")
        .ok_or("missing URI suffix")?;
    workspace.validate_workspace_uri(&scope, base.trim_end_matches('/'))?;
    assert!(
        workspace
            .validate_workspace_uri(&scope, &format!("{base}other"))
            .is_err()
    );
    for tail in [
        "../main.lua",
        "%2E%2E/main.lua",
        "a%2Fmain.lua",
        "a%5Cmain.lua",
        "main.lua%00",
        "%FF.lua",
        "%xx.lua",
        "%",
        "C%3A/main.lua",
    ] {
        assert!(
            workspace
                .relative_path_for_uri(&scope, &format!("{base}{tail}"))
                .is_err(),
            "{tail}"
        );
    }
    assert!(
        workspace
            .relative_path_for_uri(&scope, "https://example.invalid/main.lua")
            .is_err()
    );
    workspace.close(&scope)?;
    assert!(workspace.relative_path_for_uri(&scope, &uri).is_err());
    Ok(())
}
