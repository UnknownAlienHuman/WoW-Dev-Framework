//! Native child termination after durable operation boundaries. This is process
//! loss evidence, not interruption inside an OS call or power-loss certification.
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{atomic::AtomicBool, mpsc},
    time::Duration,
};
use wow_store::{
    OperationId,
    project::{
        CurrentPublication, PartitionRecord, ProjectStore, PublicationRequest, PublicationState,
        ReadSelector, RecordCatalog, RegistrySelection, ValidatedRead, VerifiedBackup,
    },
};

type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;
const ROOT_ENV: &str = "WOW_STORE_REPLACEMENT_CHILD_ROOT";
const PHASE_ENV: &str = "WOW_STORE_REPLACEMENT_CHILD_PHASE";
const READY: &str = "WOW_STORE_REPLACEMENT_READY";
const REPLACE: &str = "fixture:process-replacement";
const BACKUP: &str = "fixture:process-backup";

fn catalog() -> Result<RecordCatalog> {
    Ok(RecordCatalog::new(
        &["fixture.process.partition.v1"],
        &["fixture.process.owner.v1"],
    )?)
}
fn request(store: &ProjectStore, operation: &str, value: u32) -> Result<PublicationRequest> {
    Ok(PublicationRequest::new(
        store.epoch(),
        OperationId::new(operation)?,
        store.current()?.map(|c| c.record_id),
        [(
            "fixture.process.owner".into(),
            format!("fixture:value-{value}"),
        )]
        .into(),
        vec![PartitionRecord::new(
            "fixture.process.data",
            "fixture.process.partition.v1",
            &vec![value],
        )?],
    )?)
}
fn publish(
    store: &mut ProjectStore,
    operation: &str,
    value: u32,
    stop: &AtomicBool,
) -> Result<CurrentPublication> {
    let request = request(store, operation, value)?;
    store.prepare(&request, stop)?;
    let read = store.read(
        &ReadSelector::Exact(request.generation().generation_id.clone()),
        stop,
    )?;
    check(&read, value, stop)?;
    let validation = read.owner_validation(&["fixture.process.owner.v1"])?;
    drop(read);
    store.validate_inactive(
        request.operation_id(),
        request.request_digest(),
        validation,
        stop,
    )?;
    store.activate(request.operation_id(), request.request_digest(), stop)?;
    Ok(store.current()?.ok_or("missing fixture current")?)
}
fn check(read: &wow_store::project::ReadSnapshot, value: u32, stop: &AtomicBool) -> Result {
    assert_eq!(
        read.record("fixture.process.data", stop)?
            .ok_or("missing fixture partition")?
            .decode::<Vec<u32>>()?,
        vec![value]
    );
    Ok(())
}
fn owners(backup: &VerifiedBackup, stop: &AtomicBool) -> Result<Vec<ValidatedRead>> {
    let mut checks = Vec::new();
    for generation in backup.manifest().generations() {
        let read = backup.read(&ReadSelector::Exact(generation.clone()), stop)?;
        check(&read, 11, stop)?;
        checks.push(read.owner_validation(&["fixture.process.owner.v1"])?);
    }
    Ok(checks)
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Evidence {
    first: CurrentPublication,
    second: CurrentPublication,
    selection: RegistrySelection,
    snapshot: String,
    replacement_request: Option<String>,
}
fn ready(root: &Path, evidence: &Evidence) -> Result {
    let mut file = fs::File::create_new(root.join("evidence.json"))?;
    file.write_all(&serde_json::to_vec(evidence)?)?;
    file.sync_all()?;
    // libtest prints its progress prefix without a newline under --nocapture.
    // Frame the handshake on its own line before the parent kills the process.
    println!("\n{READY}");
    std::io::stdout().flush()?;
    // The parent forcibly terminates the child while all DB/lock handles live.
    // A missed kill times out rather than leaving an unbounded helper process.
    std::thread::sleep(Duration::from_secs(30));
    Err("parent did not terminate the ready child".into())
}

/// Entry point used only by the parent below through this compiled test binary.
#[test]
fn replacement_child() -> Result {
    let Some(root) = std::env::var_os(ROOT_ENV) else {
        return Ok(());
    };
    let root = PathBuf::from(root);
    let phase = std::env::var(PHASE_ENV)?;
    let stop = AtomicBool::new(false);
    let mut store = ProjectStore::create_with_gc(root.join("live"), "fixture.process", catalog()?)?;
    let first = publish(&mut store, "fixture:process-first", 11, &stop)?;
    let backup = store.backup_to_new(root.join("backup"), &OperationId::new(BACKUP)?, &stop)?;
    let second = publish(&mut store, "fixture:process-second", 22, &stop)?;
    let mut evidence = Evidence {
        first,
        second,
        selection: store.registry_selection()?,
        snapshot: backup.manifest().snapshot_digest().to_owned(),
        replacement_request: None,
    };
    if phase == "prepared" {
        let request = request(&store, "fixture:process-inactive", 33)?;
        store.prepare(&request, &stop)?;
        return ready(&root, &evidence);
    }
    let candidate = store.stage_replacement(
        &backup,
        &OperationId::new(REPLACE)?,
        &evidence.selection,
        Some(evidence.second.record_id.clone()),
        &stop,
    )?;
    evidence.replacement_request = Some(candidate.request_digest()?);
    if phase == "staged" {
        return ready(&root, &evidence);
    }
    let checks = owners(candidate.backup(), &stop)?;
    if phase == "validated" {
        return ready(&root, &evidence);
    }
    if phase != "activated" {
        return Err("unknown child phase".into());
    }
    store.activate_replacement(candidate, checks, &stop)?;
    ready(&root, &evidence)
}

struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn terminate_at(root: &Path, phase: &str) -> Result<Evidence> {
    let mut child = ChildGuard(
        Command::new(std::env::current_exe()?)
            .args([
                "--exact",
                "replacement_child",
                "--nocapture",
                "--test-threads=1",
            ])
            .env(ROOT_ENV, root)
            .env(PHASE_ENV, phase)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?,
    );
    let stdout = child.0.stdout.take().ok_or("missing child stdout")?;
    let (sender, receiver) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().take(16) {
            match line {
                Ok(line) if line == READY => {
                    let _ = sender.send(true);
                    return;
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
        let _ = sender.send(false);
    });
    let observed = receiver.recv_timeout(Duration::from_secs(20));
    if !matches!(observed, Ok(true)) {
        child.0.kill()?;
        child.0.wait()?;
        reader.join().map_err(|_| "child output reader failed")?;
        return Err(format!("child did not reach exact {phase} boundary").into());
    }
    child.0.kill()?;
    let status = child.0.wait()?;
    reader.join().map_err(|_| "child output reader failed")?;
    assert!(
        !status.success(),
        "ready child exited normally instead of being terminated"
    );
    let bytes = fs::read(root.join("evidence.json"))?;
    assert!(bytes.len() <= 65536);
    Ok(serde_json::from_slice(&bytes)?)
}

#[test]
fn process_termination_reconciles_four_exact_boundaries_without_repeating_copy_or_selection()
-> Result {
    let stop = AtomicBool::new(false);
    let root = std::env::temp_dir().join(format!(
        "wow-store-replacement-process-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir(&root)?;
    for phase in ["prepared", "staged", "validated", "activated"] {
        let directory = root.join(phase);
        fs::create_dir(&directory)?;
        let evidence = terminate_at(&directory, phase)?;
        let mut store = ProjectStore::open(directory.join("live"), &catalog()?)?;
        let epoch = store.epoch().clone();
        let before = if phase == "activated" {
            &evidence.first
        } else {
            &evidence.second
        };
        assert_eq!(store.current()?.as_ref(), Some(before));
        let report = store.recovery_report(&stop)?;
        assert!(report.incidents().is_empty());
        assert_eq!(report.current(), Some(before));
        let replacement = OperationId::new(REPLACE)?;
        if phase == "prepared" {
            let operation = store
                .operation(&OperationId::new("fixture:process-inactive")?)?
                .ok_or("missing committed inactive receipt")?;
            assert_ne!(operation.state, PublicationState::Activated);
            assert!(operation.activation.is_none());
            let read = store.read(&ReadSelector::Exact(operation.generation_id), &stop)?;
            check(&read, 33, &stop)?;
            assert_eq!(store.registry_selection()?, evidence.selection);
        } else if phase == "activated" {
            let request = evidence
                .replacement_request
                .as_deref()
                .ok_or("missing replacement request")?;
            let receipt = store
                .replacement_receipt(&replacement, request)?
                .ok_or("missing committed replacement receipt")?;
            assert_eq!(receipt.previous(), &evidence.selection);
            assert_eq!(receipt.activated_current(), Some(&evidence.first));
            assert_eq!(receipt.selected(), &store.registry_selection()?);
            assert_eq!(serde_json::to_value(&receipt)?["acknowledgment"], "unknown");
        } else {
            assert_eq!(store.registry_selection()?, evidence.selection);
            let candidate = store.reopen_replacement(
                &replacement,
                &evidence.selection,
                Some(evidence.second.record_id.clone()),
                &evidence.snapshot,
                &stop,
            )?;
            assert_eq!(
                Some(candidate.request_digest()?),
                evidence.replacement_request
            );
            let checks = owners(candidate.backup(), &stop)?;
            let receipt = store.activate_replacement(candidate, checks, &stop)?;
            assert_eq!(receipt.activated_current(), Some(&evidence.first));
            assert_eq!(store.current()?.as_ref(), Some(&evidence.first));
            assert_eq!(store.epoch(), &epoch);
        }
        let read = store.read(&ReadSelector::Current, &stop)?;
        check(&read, if phase == "prepared" { 22 } else { 11 }, &stop)?;
        drop(read);
        drop(store);
    }
    fs::remove_dir_all(root)?;
    Ok(())
}
