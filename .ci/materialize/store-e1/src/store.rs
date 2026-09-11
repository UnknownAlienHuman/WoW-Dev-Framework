use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use fs2::FileExt;
use serde::Serialize;
use tempfile::NamedTempFile;
use wow_core::canonical_json_bytes;

use crate::identity::{canonical_id, sha256_id};
use crate::model::{
    REF_RECORD_SCHEMA, RefRecord, RefRecordIdentity, SNAPSHOT_SCHEMA, validate_entry_path,
};
use crate::{
    BlobId, RefName, RefUpdate, SnapshotEntry, SnapshotEntryInput, SnapshotId, StoreError,
    StoreErrorCode, StoreLimits, StoreResult, StoreSnapshot,
};

#[derive(Debug)]
pub struct Store {
    root: PathBuf,
    objects: PathBuf,
    snapshots: PathBuf,
    refs: PathBuf,
    locks: PathBuf,
    temporary: PathBuf,
    limits: StoreLimits,
}

impl Store {
    pub fn open(root: impl AsRef<Path>, limits: StoreLimits) -> StoreResult<Self> {
        let root = root.as_ref();
        fs::create_dir_all(root).map_err(|error| io_error("create store root", error, None))?;
        let root = root
            .canonicalize()
            .map_err(|error| io_error("canonicalize store root", error, None))?;
        ensure_directory(&root, "store root")?;
        let store = Self {
            objects: root.join("objects/sha256"),
            snapshots: root.join("snapshots/sha256"),
            refs: root.join("refs"),
            locks: root.join("locks"),
            temporary: root.join("tmp"),
            root,
            limits,
        };
        for (path, name) in [
            (&store.objects, "objects"),
            (&store.snapshots, "snapshots"),
            (&store.refs, "refs"),
            (&store.locks, "locks"),
            (&store.temporary, "temporary"),
        ] {
            fs::create_dir_all(path)
                .map_err(|error| io_error("create store directory", error, Some(name)))?;
            ensure_directory(path, name)?;
        }
        Ok(store)
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub const fn limits(&self) -> StoreLimits {
        self.limits
    }

    pub fn put_blob(&self, bytes: &[u8]) -> StoreResult<BlobId> {
        let length = u64::try_from(bytes.len()).map_err(|_| limit_error("blob"))?;
        if length > self.limits.max_blob_bytes {
            return Err(limit_error("blob"));
        }
        let id = BlobId::from_verified(sha256_id(bytes));
        let target = self.object_path(&id);
        persist_noclobber(
            &self.temporary,
            &target,
            bytes,
            StoreErrorCode::ObjectCorrupt,
        )?;
        Ok(id)
    }

    pub fn read_blob(&self, id: &BlobId) -> StoreResult<Vec<u8>> {
        let path = self.object_path(id);
        let bytes = read_bounded_file(
            &path,
            self.limits.max_blob_bytes,
            StoreErrorCode::ObjectMissing,
            id.as_str(),
        )?;
        if sha256_id(&bytes).as_ref() != id.as_str() {
            return Err(StoreError::new(
                StoreErrorCode::ObjectCorrupt,
                "blob bytes do not match their content identity",
                Some(id.as_str()),
            ));
        }
        Ok(bytes)
    }

    pub fn commit_snapshot<I>(&self, entries: I) -> StoreResult<StoreSnapshot>
    where
        I: IntoIterator<Item = SnapshotEntryInput>,
    {
        let mut inputs = entries
            .into_iter()
            .map(SnapshotEntryInput::into_parts)
            .collect::<Vec<_>>();
        if inputs.is_empty() || inputs.len() > self.limits.max_snapshot_entries {
            return Err(limit_error("snapshot entries"));
        }
        inputs.sort_by(|left, right| left.0.cmp(&right.0));
        if inputs.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(StoreError::new(
                StoreErrorCode::InvalidPath,
                "snapshot contains a duplicate logical path",
                None,
            ));
        }
        let mut total_bytes = 0_u64;
        let mut verified = Vec::with_capacity(inputs.len());
        for (path, blob_id) in inputs {
            validate_entry_path(&path)?;
            let bytes = self.read_blob(&blob_id)?;
            let length = u64::try_from(bytes.len()).map_err(|_| limit_error("snapshot bytes"))?;
            total_bytes = total_bytes
                .checked_add(length)
                .ok_or_else(|| limit_error("snapshot bytes"))?;
            if total_bytes > self.limits.max_snapshot_bytes {
                return Err(limit_error("snapshot bytes"));
            }
            verified.push(SnapshotEntry::new(path, blob_id, length));
        }
        #[derive(Serialize)]
        struct ManifestIdentity<'a> {
            schema: &'static str,
            total_bytes: u64,
            entries: &'a [SnapshotEntry],
        }
        let identity = ManifestIdentity {
            schema: SNAPSHOT_SCHEMA,
            total_bytes,
            entries: &verified,
        };
        let snapshot_id = SnapshotId::from_verified(canonical_id(
            "store-snapshot:sha256:",
            &identity,
        )?);
        let snapshot = StoreSnapshot::new(snapshot_id.clone(), total_bytes, verified);
        let bytes = canonical_json_bytes(&snapshot).map_err(|error| {
            StoreError::new(
                StoreErrorCode::CanonicalizationFailed,
                format!("snapshot serialization failed: {error}"),
                Some(snapshot_id.as_str()),
            )
        })?;
        if u64::try_from(bytes.len()).map_err(|_| limit_error("snapshot manifest"))?
            > self.limits.max_blob_bytes
        {
            return Err(limit_error("snapshot manifest"));
        }
        persist_noclobber(
            &self.temporary,
            &self.snapshot_path(&snapshot_id),
            &bytes,
            StoreErrorCode::SnapshotCorrupt,
        )?;
        Ok(snapshot)
    }

    pub fn load_snapshot(&self, id: &SnapshotId) -> StoreResult<StoreSnapshot> {
        let bytes = read_bounded_file(
            &self.snapshot_path(id),
            self.limits.max_blob_bytes,
            StoreErrorCode::ObjectMissing,
            id.as_str(),
        )?;
        let snapshot: StoreSnapshot = serde_json::from_slice(&bytes).map_err(|error| {
            StoreError::new(
                StoreErrorCode::SnapshotCorrupt,
                format!("snapshot manifest is invalid JSON: {error}"),
                Some(id.as_str()),
            )
        })?;
        self.verify_snapshot(id, &snapshot)?;
        Ok(snapshot)
    }

    pub fn read_ref(&self, name: &RefName) -> StoreResult<Option<SnapshotId>> {
        let _lock = self.lock_refs(false)?;
        self.read_ref_unlocked(name).map(|state| state.current)
    }

    pub fn update_ref(
        &self,
        name: &RefName,
        expected: Option<&SnapshotId>,
        next: &SnapshotId,
    ) -> StoreResult<RefUpdate> {
        self.load_snapshot(next)?;
        let lock = self.lock_refs(true)?;
        let state = self.read_ref_unlocked(name)?;
        if state.current.as_ref() != expected {
            return Err(StoreError::new(
                StoreErrorCode::RefConflict,
                "ref compare-and-swap expectation does not match current state",
                Some(name.as_str()),
            ));
        }
        if state.current.as_ref() == Some(next) {
            return Ok(RefUpdate::NoChange);
        }
        let sequence = state
            .sequence
            .checked_add(1)
            .ok_or_else(|| limit_error("ref sequence"))?;
        let identity = RefRecordIdentity {
            schema: REF_RECORD_SCHEMA,
            sequence,
            previous: state.current.as_ref(),
            next,
        };
        let record_id = canonical_id("store-ref-record:sha256:", &identity)?;
        let record = RefRecord {
            schema: REF_RECORD_SCHEMA.into(),
            sequence,
            previous: state.current,
            next: next.clone(),
            record_id,
        };
        let mut line = canonical_json_bytes(&record).map_err(|error| {
            StoreError::new(
                StoreErrorCode::CanonicalizationFailed,
                format!("ref record serialization failed: {error}"),
                Some(name.as_str()),
            )
        })?;
        line.push(b'\n');
        let path = self.ref_path(name);
        let existing = match fs::metadata(&path) {
            Ok(metadata) => metadata.len(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
            Err(error) => {
                return Err(io_error(
                    "inspect ref journal",
                    error,
                    Some(name.as_str()),
                ));
            }
        };
        let line_length = u64::try_from(line.len()).map_err(|_| limit_error("ref journal"))?;
        if existing
            .checked_add(line_length)
            .is_none_or(|length| length > self.limits.max_ref_journal_bytes)
        {
            return Err(limit_error("ref journal"));
        }
        let mut journal = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|error| io_error("open ref journal", error, Some(name.as_str())))?;
        journal
            .write_all(&line)
            .and_then(|()| journal.sync_data())
            .map_err(|error| io_error("append ref journal", error, Some(name.as_str())))?;
        fs2::FileExt::unlock(&lock)
            .map_err(|error| io_error("unlock ref journal", error, Some(name.as_str())))?;
        Ok(if expected.is_some() {
            RefUpdate::Advanced
        } else {
            RefUpdate::Created
        })
    }

    fn verify_snapshot(&self, expected: &SnapshotId, snapshot: &StoreSnapshot) -> StoreResult<()> {
        if snapshot.schema() != SNAPSHOT_SCHEMA || snapshot.snapshot_id() != expected {
            return Err(snapshot_corrupt(
                expected,
                "snapshot schema or embedded identity mismatch",
            ));
        }
        if snapshot.entries().is_empty()
            || snapshot.entries().len() > self.limits.max_snapshot_entries
        {
            return Err(snapshot_corrupt(
                expected,
                "snapshot entry count is invalid",
            ));
        }
        let mut previous = None;
        let mut total = 0_u64;
        for entry in snapshot.entries() {
            validate_entry_path(entry.path())?;
            if previous.is_some_and(|value| value >= entry.path()) {
                return Err(snapshot_corrupt(
                    expected,
                    "snapshot paths are duplicate or unordered",
                ));
            }
            previous = Some(entry.path());
            let bytes = self.read_blob(entry.blob_id())?;
            let length = u64::try_from(bytes.len()).map_err(|_| limit_error("snapshot bytes"))?;
            if length != entry.bytes() {
                return Err(snapshot_corrupt(
                    expected,
                    "snapshot entry length mismatch",
                ));
            }
            total = total
                .checked_add(length)
                .ok_or_else(|| limit_error("snapshot bytes"))?;
        }
        if total != snapshot.total_bytes() || total > self.limits.max_snapshot_bytes {
            return Err(snapshot_corrupt(
                expected,
                "snapshot total byte count mismatch",
            ));
        }
        #[derive(Serialize)]
        struct ManifestIdentity<'a> {
            schema: &'static str,
            total_bytes: u64,
            entries: &'a [SnapshotEntry],
        }
        let identity = ManifestIdentity {
            schema: SNAPSHOT_SCHEMA,
            total_bytes: snapshot.total_bytes(),
            entries: snapshot.entries(),
        };
        let derived = canonical_id("store-snapshot:sha256:", &identity)?;
        if derived.as_ref() != expected.as_str() {
            return Err(snapshot_corrupt(
                expected,
                "snapshot content identity mismatch",
            ));
        }
        Ok(())
    }

    fn read_ref_unlocked(&self, name: &RefName) -> StoreResult<RefState> {
        let path = self.ref_path(name);
        if !path.exists() {
            return Ok(RefState::default());
        }
        let bytes = read_bounded_file(
            &path,
            self.limits.max_ref_journal_bytes,
            StoreErrorCode::RefCorrupt,
            name.as_str(),
        )?;
        let complete_length = bytes
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |index| index + 1);
        let mut state = RefState::default();
        for line in bytes[..complete_length]
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
        {
            let record: RefRecord = serde_json::from_slice(line).map_err(|error| {
                StoreError::new(
                    StoreErrorCode::RefCorrupt,
                    format!("ref journal contains invalid JSON: {error}"),
                    Some(name.as_str()),
                )
            })?;
            let expected_sequence = state.sequence.checked_add(1).ok_or_else(|| {
                StoreError::new(
                    StoreErrorCode::RefCorrupt,
                    "ref journal sequence overflow",
                    Some(name.as_str()),
                )
            })?;
            if record.schema.as_ref() != REF_RECORD_SCHEMA
                || record.sequence != expected_sequence
                || record.previous != state.current
            {
                return Err(StoreError::new(
                    StoreErrorCode::RefCorrupt,
                    "ref journal sequence or predecessor is invalid",
                    Some(name.as_str()),
                ));
            }
            SnapshotId::parse(record.next.as_str())?;
            self.load_snapshot(&record.next)?;
            let identity = RefRecordIdentity {
                schema: REF_RECORD_SCHEMA,
                sequence: record.sequence,
                previous: record.previous.as_ref(),
                next: &record.next,
            };
            if canonical_id("store-ref-record:sha256:", &identity)? != record.record_id {
                return Err(StoreError::new(
                    StoreErrorCode::RefCorrupt,
                    "ref journal checksum does not match its record",
                    Some(name.as_str()),
                ));
            }
            state.sequence = record.sequence;
            state.current = Some(record.next);
        }
        Ok(state)
    }

    fn object_path(&self, id: &BlobId) -> PathBuf {
        shard_path(&self.objects, id.hex())
    }

    fn snapshot_path(&self, id: &SnapshotId) -> PathBuf {
        shard_path(&self.snapshots, id.hex()).with_extension("json")
    }

    fn ref_path(&self, name: &RefName) -> PathBuf {
        self.refs.join(format!("{}.jsonl", name.as_str()))
    }

    fn lock_refs(&self, exclusive: bool) -> StoreResult<File> {
        let path = self.locks.join("refs.lock");
        let lock = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)
            .map_err(|error| io_error("open ref lock", error, None))?;
        let result = if exclusive {
            fs2::FileExt::lock_exclusive(&lock)
        } else {
            fs2::FileExt::lock_shared(&lock)
        };
        result.map_err(|error| {
            StoreError::new(
                StoreErrorCode::LockFailed,
                format!("cannot lock ref journal: {error}"),
                None,
            )
        })?;
        Ok(lock)
    }
}

#[derive(Default)]
struct RefState {
    sequence: u64,
    current: Option<SnapshotId>,
}

fn shard_path(root: &Path, hex: &str) -> PathBuf {
    root.join(&hex[..2]).join(&hex[2..])
}

fn persist_noclobber(
    temporary: &Path,
    target: &Path,
    bytes: &[u8],
    corruption: StoreErrorCode,
) -> StoreResult<()> {
    let parent = target.parent().ok_or_else(|| {
        StoreError::new(StoreErrorCode::InvalidPath, "target has no parent", None)
    })?;
    fs::create_dir_all(parent)
        .map_err(|error| io_error("create object shard", error, None))?;
    ensure_directory(parent, "object shard")?;
    let mut staged = NamedTempFile::new_in(temporary)
        .map_err(|error| io_error("create staged object", error, None))?;
    staged
        .write_all(bytes)
        .and_then(|()| staged.flush())
        .and_then(|()| staged.as_file().sync_all())
        .map_err(|error| io_error("write staged object", error, None))?;
    match staged.persist_noclobber(target) {
        Ok(file) => file
            .sync_all()
            .map_err(|error| io_error("sync persisted object", error, None)),
        Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
            let existing = read_bounded_file(
                target,
                u64::try_from(bytes.len()).unwrap_or(u64::MAX),
                corruption,
                "existing object",
            )?;
            if existing == bytes {
                Ok(())
            } else {
                Err(StoreError::new(
                    corruption,
                    "existing content-addressed object has different bytes",
                    None,
                ))
            }
        }
        Err(error) => Err(io_error("persist staged object", error.error, None)),
    }
}

fn read_bounded_file(
    path: &Path,
    maximum: u64,
    missing_code: StoreErrorCode,
    subject: &str,
) -> StoreResult<Vec<u8>> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        let code = if error.kind() == std::io::ErrorKind::NotFound {
            missing_code
        } else {
            StoreErrorCode::IoFailed
        };
        StoreError::new(
            code,
            format!("cannot inspect stored file: {error}"),
            Some(subject),
        )
    })?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(StoreError::new(
            missing_code,
            "stored path is not a regular file",
            Some(subject),
        ));
    }
    if metadata.len() > maximum {
        return Err(limit_error(subject));
    }
    let mut file = File::open(path)
        .map_err(|error| io_error("open stored file", error, Some(subject)))?;
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).unwrap_or(0));
    file.take(maximum.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| io_error("read stored file", error, Some(subject)))?;
    if u64::try_from(bytes.len()).map_or(true, |length| length > maximum) {
        return Err(limit_error(subject));
    }
    Ok(bytes)
}

fn ensure_directory(path: &Path, subject: &str) -> StoreResult<()> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| io_error("inspect store directory", error, Some(subject)))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(StoreError::new(
            StoreErrorCode::InvalidPath,
            "store directory is not a real directory",
            Some(subject),
        ));
    }
    Ok(())
}

fn io_error(operation: &str, error: std::io::Error, subject: Option<&str>) -> StoreError {
    StoreError::new(
        StoreErrorCode::IoFailed,
        format!("{operation} failed: {error}"),
        subject,
    )
}

fn limit_error(subject: &str) -> StoreError {
    StoreError::new(
        StoreErrorCode::InputLimitExceeded,
        "store input exceeds a configured bound",
        Some(subject),
    )
}

fn snapshot_corrupt(id: &SnapshotId, message: &str) -> StoreError {
    StoreError::new(
        StoreErrorCode::SnapshotCorrupt,
        message,
        Some(id.as_str()),
    )
}
