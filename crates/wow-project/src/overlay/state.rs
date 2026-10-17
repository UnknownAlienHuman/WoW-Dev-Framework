use super::coordinates::OverlayLineIndex;
use super::*;
use std::sync::Weak;
use wow_emmy::{
    EmmyBackendIdentity, LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot,
    LuaWorkspaceUniverse,
};

pub(super) struct Snapshot {
    pub(super) descriptor: OverlaySnapshotDescriptor,
    pub(super) content: Arc<str>,
    pub(super) index: OverlayLineIndex,
    pub(super) retained_bytes: usize,
    prior: Option<Box<str>>,
    change_digest: Option<Box<str>>,
    leases: AtomicUsize,
}
/// Exact private bytes stay owned until this operation lease is dropped.
pub struct OverlaySnapshotLease {
    snapshot: Arc<Snapshot>,
    total: Arc<AtomicUsize>,
}
impl Drop for OverlaySnapshotLease {
    fn drop(&mut self) {
        self.snapshot.leases.fetch_sub(1, Ordering::AcqRel);
        self.total.fetch_sub(1, Ordering::AcqRel);
    }
}
impl OverlaySnapshotLease {
    pub fn descriptor(&self) -> &OverlaySnapshotDescriptor {
        &self.snapshot.descriptor
    }
}
pub struct OverlayAnalyzerInput {
    workspace: LuaWorkspaceSnapshot,
    _lease: OverlaySnapshotLease,
    _bytes: AnalyzerBytesLease,
}
// Dropped after the owned analyzer workspace frees its private source copy.
struct AnalyzerBytesLease {
    bytes: usize,
    total: Arc<AtomicUsize>,
}
impl Drop for AnalyzerBytesLease {
    fn drop(&mut self) {
        self.total.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
impl OverlayAnalyzerInput {
    pub fn workspace(&self) -> &LuaWorkspaceSnapshot {
        &self.workspace
    }
}

impl OverlayWorkspace {
    fn by_document(&self, document: &str) -> OverlayResult<Arc<Snapshot>> {
        self.current
            .values()
            .find(|snapshot| snapshot.descriptor.document_id.as_ref() == document)
            .cloned()
            .ok_or_else(|| error(OverlayErrorCode::DocumentMissing))
    }
    fn index(
        &self,
        content: Arc<str>,
        retained: usize,
        budget: &mut OverlayBudget<'_>,
    ) -> OverlayResult<OverlayLineIndex> {
        // Two working strings and conservative Vec growth remain inside the
        // selected private-source/index allowance before allocation.
        let available = self
            .descriptor
            .limits
            .max_private_bytes
            .checked_sub(retained)
            .and_then(|n| n.checked_sub(content.len().checked_mul(2)?))
            .ok_or_else(|| error(OverlayErrorCode::Limit))?;
        let lines = self.descriptor.limits.max_lines.min(available / 64);
        if lines == 0 {
            return Err(error(OverlayErrorCode::Limit));
        }
        OverlayLineIndex::build(content, lines, budget)
    }
    fn snapshot(
        &self,
        path: &str,
        version: u64,
        content: Arc<str>,
        index: OverlayLineIndex,
        change: Option<(Box<str>, Box<str>)>,
    ) -> OverlayResult<Arc<Snapshot>> {
        let (prior, change_digest) = match change {
            Some((prior, change)) => (Some(prior), Some(change)),
            None => (None, None),
        };
        let content_digest = digest_bytes(content.as_bytes());
        let document_id = identity(
            "private-overlay-document",
            &(&self.descriptor.registration_id, path),
        )?;
        #[derive(Serialize)]
        struct Scope<'a> {
            registration: &'a str,
            path: &'a str,
            version: u64,
            content: ContentDigest<SourceContent>,
            #[serde(skip_serializing_if = "Option::is_none")]
            prior: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            change: Option<&'a str>,
        }
        let snapshot_id = identity(
            "private-overlay-snapshot",
            &Scope {
                registration: &self.descriptor.registration_id,
                path,
                version,
                content: content_digest,
                prior: prior.as_deref(),
                change: change_digest.as_deref(),
            },
        )?;
        let descriptor = OverlaySnapshotDescriptor {
            schema: "wow-project/private-document-overlay/1".into(),
            snapshot_id,
            registration_id: self.descriptor.registration_id.clone(),
            document_id,
            relative_path: path.into(),
            client_version: version,
            content_digest,
            byte_length: content.len(),
            line_count: index.line_count(),
            base_state: "no_published_base".into(),
            synchronization_profile: "sequential_intermediate_atomic_utf8/1".into(),
        };
        let retained_bytes = content
            .len()
            .checked_add(index.retained_bytes())
            .ok_or_else(|| error(OverlayErrorCode::Limit))?;
        Ok(Arc::new(Snapshot {
            descriptor,
            content,
            index,
            retained_bytes,
            prior,
            change_digest,
            leases: AtomicUsize::new(0),
        }))
    }
    pub fn open_document(
        &mut self,
        scope: &OverlaySessionScope,
        request: OverlayOpenRequest<'_>,
        stop: &AtomicBool,
    ) -> OverlayResult<OverlaySnapshotDescriptor> {
        let OverlayOpenRequest {
            relative_path: path,
            uri,
            client_version: version,
            content: text,
        } = request;
        self.require(scope)?;
        self.require_uri(path, uri)?;
        let key = self.path_key(path)?;
        let limits = self.descriptor.limits;
        let mut budget = OverlayBudget::new(limits.max_work, stop);
        budget.work(text.len())?;
        if text.len() > limits.max_document_bytes {
            return Err(error(OverlayErrorCode::Limit));
        }
        if let Some(current) = self.current.get(&key) {
            return if current.descriptor.relative_path.as_ref() == path
                && current.descriptor.client_version == version
                && current.content.as_ref() == text
            {
                Ok(current.descriptor.clone())
            } else {
                Err(error(OverlayErrorCode::Conflict))
            };
        }
        if self.current.len() >= limits.max_documents {
            return Err(error(OverlayErrorCode::Limit));
        }
        let (count, retained) = self.retained()?;
        if count >= limits.max_live_snapshots {
            return Err(error(OverlayErrorCode::Limit));
        }
        // String -> Arc conversion briefly retains both copies.
        if text
            .len()
            .checked_mul(2)
            .and_then(|n| n.checked_add(retained))
            .is_none_or(|n| n > limits.max_private_bytes)
        {
            return Err(error(OverlayErrorCode::Limit));
        }
        let content: Arc<str> = Arc::from(text);
        let index = self.index(content.clone(), retained, &mut budget)?;
        let snapshot = self.snapshot(path, version, content, index, None)?;
        if retained
            .checked_add(snapshot.retained_bytes)
            .is_none_or(|n| n > limits.max_private_bytes)
        {
            return Err(error(OverlayErrorCode::Limit));
        }
        budget.work(1)?;
        let descriptor = snapshot.descriptor.clone();
        self.snapshots.push(Arc::downgrade(&snapshot));
        self.current.insert(key, snapshot);
        Ok(descriptor)
    }
    pub fn change_document(
        &mut self,
        scope: &OverlaySessionScope,
        request: OverlayChangeRequest<'_>,
        stop: &AtomicBool,
    ) -> OverlayResult<OverlaySnapshotDescriptor> {
        let OverlayChangeRequest {
            document_id: document,
            expected_snapshot,
            expected_content,
            client_version: version,
            encoding,
            edits,
        } = request;
        self.require(scope)?;
        let limits = self.descriptor.limits;
        let mut budget = OverlayBudget::new(limits.max_work, stop);
        budget.work(1)?;
        if edits.is_empty() || edits.len() > limits.max_edits {
            return Err(error(OverlayErrorCode::Limit));
        }
        for edit in edits {
            budget.work(edit.text.len())?;
            if edit.text.len() > limits.max_document_bytes {
                return Err(error(OverlayErrorCode::Limit));
            }
        }
        #[derive(Serialize)]
        struct EditIdentity {
            #[serde(skip_serializing_if = "Option::is_none")]
            range: Option<OverlayRange>,
            text_digest: ContentDigest<SourceContent>,
            byte_length: usize,
        }
        // Change identity carries exact digests rather than serializing/copying
        // private replacement bodies into a second canonical JSON buffer.
        let manifest: Vec<_> = edits
            .iter()
            .map(|edit| {
                budget.work(edit.text.len())?;
                Ok(EditIdentity {
                    range: edit.range,
                    text_digest: digest_bytes(edit.text.as_bytes()),
                    byte_length: edit.text.len(),
                })
            })
            .collect::<OverlayResult<_>>()?;
        let change = identity(
            "private-overlay-change",
            &(
                document,
                expected_snapshot,
                expected_content,
                version,
                encoding,
                manifest,
            ),
        )?;
        let current = self.by_document(document)?;
        if current.descriptor.client_version == version
            && current.prior.as_deref() == Some(expected_snapshot)
            && current.change_digest.as_deref() == Some(change.as_ref())
        {
            return Ok(current.descriptor.clone());
        }
        if current.descriptor.snapshot_id.as_ref() != expected_snapshot
            || current.descriptor.content_digest != expected_content
            || current.descriptor.client_version.checked_add(1) != Some(version)
        {
            return Err(error(OverlayErrorCode::ResynchronizationRequired));
        }
        let (count, retained) = self.retained()?;
        if count >= limits.max_live_snapshots {
            return Err(error(OverlayErrorCode::Limit));
        }
        let mut content = current.content.clone();
        let mut index = self.index(content.clone(), retained, &mut budget)?;
        for edit in edits {
            let (start, end) = if let Some(range) = edit.range {
                (
                    index.byte_offset(range.start, encoding, &mut budget)?,
                    index.byte_offset(range.end, encoding, &mut budget)?,
                )
            } else {
                (0, content.len())
            };
            if start > end {
                return Err(error(OverlayErrorCode::InvalidRange));
            }
            let size = content
                .len()
                .checked_sub(end - start)
                .and_then(|n| n.checked_add(edit.text.len()))
                .ok_or_else(|| error(OverlayErrorCode::Limit))?;
            if size > limits.max_document_bytes
                || retained
                    .checked_add(
                        size.checked_mul(2)
                            .and_then(|n| n.checked_add(content.len()))
                            .ok_or_else(|| error(OverlayErrorCode::Limit))?,
                    )
                    .is_none_or(|n| n > limits.max_private_bytes)
            {
                return Err(error(OverlayErrorCode::Limit));
            }
            // Ranges are resolved; release the old transient index before
            // allocating replacement bytes/index. The original snapshot stays
            // included in `retained` while a previous lease can still read it.
            drop(index);
            budget.work(size)?;
            let mut next = String::new();
            next.try_reserve_exact(size)
                .map_err(|_| error(OverlayErrorCode::Limit))?;
            next.push_str(&content[..start]);
            next.push_str(&edit.text);
            next.push_str(&content[end..]);
            content = Arc::from(next);
            index = self.index(content.clone(), retained, &mut budget)?;
        }
        let snapshot = self.snapshot(
            &current.descriptor.relative_path,
            version,
            content,
            index,
            Some((current.descriptor.snapshot_id.clone(), change)),
        )?;
        if retained
            .checked_add(snapshot.retained_bytes)
            .is_none_or(|n| n > limits.max_private_bytes)
        {
            return Err(error(OverlayErrorCode::Limit));
        }
        budget.work(1)?;
        let descriptor = snapshot.descriptor.clone();
        let key = self.path_key(&descriptor.relative_path)?;
        self.snapshots.push(Arc::downgrade(&snapshot));
        self.current.insert(key, snapshot);
        Ok(descriptor)
    }
    pub fn get(
        &self,
        scope: &OverlaySessionScope,
        document: &str,
        expected_snapshot: &str,
        stop: &AtomicBool,
    ) -> OverlayResult<OverlaySnapshotLease> {
        self.require(scope)?;
        let mut budget = OverlayBudget::new(self.descriptor.limits.max_work, stop);
        budget.work(self.current.len())?;
        let snapshot = self.by_document(document)?;
        if snapshot.descriptor.snapshot_id.as_ref() != expected_snapshot {
            return Err(error(OverlayErrorCode::ResynchronizationRequired));
        }
        self.leases
            .try_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                if count < self.descriptor.limits.max_leases {
                    count.checked_add(1)
                } else {
                    None
                }
            })
            .map_err(|_| error(OverlayErrorCode::Limit))?;
        snapshot.leases.fetch_add(1, Ordering::AcqRel);
        Ok(OverlaySnapshotLease {
            snapshot,
            total: self.leases.clone(),
        })
    }
    fn require_lease(
        &self,
        scope: &OverlaySessionScope,
        lease: &OverlaySnapshotLease,
    ) -> OverlayResult<()> {
        self.require(scope)?;
        if lease.snapshot.descriptor.registration_id != self.descriptor.registration_id
            || !Arc::ptr_eq(&lease.total, &self.leases)
        {
            return Err(error(OverlayErrorCode::ScopeMismatch));
        }
        Ok(())
    }
    pub fn positions(
        &self,
        scope: &OverlaySessionScope,
        lease: &OverlaySnapshotLease,
        offsets: &[usize],
        encoding: OverlayEncoding,
        stop: &AtomicBool,
    ) -> OverlayResult<Vec<OverlayPosition>> {
        self.require_lease(scope, lease)?;
        if offsets.len() > 4096 {
            return Err(error(OverlayErrorCode::Limit));
        }
        let mut budget = OverlayBudget::new(self.descriptor.limits.max_work, stop);
        let mut output = Vec::with_capacity(offsets.len());
        for offset in offsets {
            budget.work(1)?;
            output.push(
                lease
                    .snapshot
                    .index
                    .position(*offset, encoding, &mut budget)?,
            );
        }
        Ok(output)
    }
    pub fn analyzer_input(
        &self,
        scope: &OverlaySessionScope,
        lease: OverlaySnapshotLease,
        backend: EmmyBackendIdentity,
        stop: &AtomicBool,
    ) -> OverlayResult<OverlayAnalyzerInput> {
        self.require_lease(scope, &lease)?;
        let mut budget = OverlayBudget::new(self.descriptor.limits.max_work, stop);
        budget.work(lease.snapshot.content.len())?;
        let retained = self.snapshots.iter().filter_map(Weak::upgrade).try_fold(
            0_usize,
            |bytes, snapshot| {
                bytes
                    .checked_add(snapshot.retained_bytes)
                    .ok_or_else(|| error(OverlayErrorCode::Limit))
            },
        )?;
        let bytes = lease
            .snapshot
            .content
            .len()
            .checked_mul(2)
            .ok_or_else(|| error(OverlayErrorCode::Limit))?;
        self.analyzer_bytes
            .try_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                current.checked_add(bytes).filter(|total| {
                    total
                        .checked_add(retained)
                        .is_some_and(|n| n <= self.descriptor.limits.max_private_bytes)
                })
            })
            .map_err(|_| error(OverlayErrorCode::Limit))?;
        let copy_lease = AnalyzerBytesLease {
            bytes,
            total: self.analyzer_bytes.clone(),
        };
        let limits = LuaWorkspaceLimits::new(
            1,
            4096,
            self.descriptor.limits.max_document_bytes as u64,
            self.descriptor.limits.max_private_bytes as u64,
        )
        .map_err(|_| error(OverlayErrorCode::Analyzer))?;
        let input = LuaWorkspaceFileInput::new(
            lease.snapshot.descriptor.relative_path.as_ref(),
            lease.snapshot.content.to_string(),
        );
        let workspace = LuaWorkspaceSnapshot::build(
            backend,
            LuaWorkspaceUniverse::Project,
            vec![input],
            limits,
        )
        .map_err(|_| error(OverlayErrorCode::Analyzer))?;
        budget.work(1)?;
        Ok(OverlayAnalyzerInput {
            workspace,
            _lease: lease,
            _bytes: copy_lease,
        })
    }
    pub fn save_acknowledge(
        &self,
        scope: &OverlaySessionScope,
        lease: &OverlaySnapshotLease,
    ) -> OverlayResult<OverlaySaveObservation> {
        self.require_lease(scope, lease)?;
        let current = self.by_document(&lease.snapshot.descriptor.document_id)?;
        if current.descriptor.snapshot_id != lease.snapshot.descriptor.snapshot_id {
            return Err(error(OverlayErrorCode::ResynchronizationRequired));
        }
        Ok(OverlaySaveObservation {
            snapshot_id: current.descriptor.snapshot_id.clone(),
            client_version: current.descriptor.client_version,
            state: OverlaySaveState::ContentAcknowledgedOnly,
        })
    }
    pub fn reconcile_disk(
        &self,
        scope: &OverlaySessionScope,
        lease: &OverlaySnapshotLease,
        stop: &AtomicBool,
    ) -> OverlayResult<OverlaySaveObservation> {
        let mut observation = self.save_acknowledge(scope, lease)?;
        let descriptor = &lease.snapshot.descriptor;
        let files = self
            .directory
            .as_ref()
            .ok_or_else(|| error(OverlayErrorCode::Closed))?
            .read_lua_inventory(
                ".",
                &[crate::disk::ProjectDiskFile::new(
                    descriptor.relative_path.as_ref(),
                )],
                crate::ProjectFileRole::FirstPartyMain,
                stop,
            )
            .map_err(|_| {
                if stop.load(Ordering::Acquire) {
                    error(OverlayErrorCode::Cancelled)
                } else {
                    error(OverlayErrorCode::SourceFailure)
                }
            })?;
        let file = files
            .first()
            .ok_or_else(|| error(OverlayErrorCode::SourceFailure))?;
        observation.state = if file.manifest().content_digest() == descriptor.content_digest {
            OverlaySaveState::DiskReconciled
        } else {
            OverlaySaveState::ConflictWithDisk
        };
        Ok(observation)
    }
    pub fn close_document(
        &mut self,
        scope: &OverlaySessionScope,
        document: &str,
    ) -> OverlayResult<()> {
        self.require(scope)?;
        let current = self.by_document(document)?;
        if self
            .snapshots
            .iter()
            .filter_map(Weak::upgrade)
            .any(|snapshot| {
                snapshot.descriptor.document_id.as_ref() == document
                    && snapshot.leases.load(Ordering::Acquire) != 0
            })
        {
            return Err(error(OverlayErrorCode::Busy));
        }
        let key = self.path_key(&current.descriptor.relative_path)?;
        self.current.remove(&key);
        self.retained()?;
        Ok(())
    }
}
