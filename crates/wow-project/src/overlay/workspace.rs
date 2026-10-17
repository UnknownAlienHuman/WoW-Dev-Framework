use super::state::Snapshot;
use super::*;
use crate::disk::ProjectInputDirectory;
use std::{
    collections::BTreeMap,
    path::{Component, PathBuf, Prefix},
    sync::Weak,
};

/// Actual directory registration and session-private current states. No raw
/// directory capability or private source bytes cross this owner port.
pub struct OverlayWorkspace {
    pub(super) scope: Arc<Scope>,
    pub(super) descriptor: OverlayWorkspaceDescriptor,
    pub(super) directory: Option<ProjectInputDirectory>,
    pub(super) root_uri: Box<str>,
    pub(super) current: BTreeMap<Box<str>, Arc<Snapshot>>,
    pub(super) snapshots: Vec<Weak<Snapshot>>,
    pub(super) leases: Arc<AtomicUsize>,
    pub(super) analyzer_bytes: Arc<AtomicUsize>,
}
impl OverlayWorkspace {
    pub fn register(
        scope: &OverlaySessionScope,
        root: &Path,
        workspace_id: &str,
        limits: OverlayLimits,
        stop: &AtomicBool,
    ) -> OverlayResult<Self> {
        limits.validate()?;
        let mut budget = OverlayBudget::new(limits.max_work, stop);
        budget.work(1)?;
        if workspace_id.is_empty()
            || workspace_id.len() > 256
            || !workspace_id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b':' | b'_' | b'-' | b'.'))
        {
            return Err(error(OverlayErrorCode::InvalidRoot));
        }
        let text = root
            .to_str()
            .ok_or_else(|| error(OverlayErrorCode::InvalidRoot))?;
        budget.work(text.len())?;
        if text.len() > 32768
            || !root.is_absolute()
            || text.ends_with(['.', ' '])
            || text.chars().any(char::is_control)
            || text
                .split(['\\', '/'])
                .any(|part| matches!(part, "." | ".."))
        {
            return Err(error(OverlayErrorCode::InvalidRoot));
        }
        for part in root.components() {
            match part {
                Component::Prefix(prefix) if !matches!(prefix.kind(), Prefix::Disk(_)) => {
                    return Err(error(OverlayErrorCode::InvalidRoot));
                }
                Component::CurDir | Component::ParentDir => {
                    return Err(error(OverlayErrorCode::InvalidRoot));
                }
                _ => {}
            }
        }
        // Components normalize separators once. Use this same lexical root for
        // both the nofollow capability walk and the private canonical URI.
        let lexical_root: PathBuf = root.components().collect();
        let text = lexical_root
            .to_str()
            .ok_or_else(|| error(OverlayErrorCode::InvalidRoot))?;
        let directory =
            ProjectInputDirectory::open_workspace(&lexical_root, stop).map_err(|_| {
                if stop.load(Ordering::Acquire) {
                    error(OverlayErrorCode::Cancelled)
                } else {
                    error(OverlayErrorCode::InvalidRoot)
                }
            })?;
        let profile = if cfg!(windows) {
            "windows-local-canonical-nofollow/1"
        } else {
            "unix-local-canonical-nofollow/1"
        };
        let descriptor = OverlayWorkspaceDescriptor {
            schema: "wow-project/private-workspace/1".into(),
            registration_id: identity(
                "private-workspace",
                &(
                    scope.0.id,
                    next(&NEXT_REGISTRATION)?,
                    workspace_id,
                    profile,
                    limits,
                ),
            )?,
            session_scope: scope.0.id,
            workspace_id: workspace_id.into(),
            state: "registered_unmaterialized".into(),
            path_profile: profile.into(),
            base_state: "no_published_base".into(),
            privacy: "session_private_memory_only".into(),
            limits,
        };
        let mut slash = text.replace('\\', "/");
        if cfg!(windows) {
            let first = slash
                .get(..1)
                .ok_or_else(|| error(OverlayErrorCode::InvalidRoot))?
                .to_ascii_uppercase();
            slash.replace_range(..1, &first);
        }
        while slash.ends_with('/') {
            slash.pop();
        }
        let root_uri = if cfg!(windows) {
            format!("file:///{}", uri_path(&slash))
        } else {
            format!("file://{}", uri_path(&slash))
        };
        Ok(Self {
            scope: scope.0.clone(),
            descriptor,
            directory: Some(directory),
            root_uri: root_uri.into(),
            current: BTreeMap::new(),
            snapshots: Vec::new(),
            leases: Arc::new(AtomicUsize::new(0)),
            analyzer_bytes: Arc::new(AtomicUsize::new(0)),
        })
    }
    pub fn descriptor(&self) -> &OverlayWorkspaceDescriptor {
        &self.descriptor
    }
    pub(super) fn require(&self, scope: &OverlaySessionScope) -> OverlayResult<()> {
        if !Arc::ptr_eq(&self.scope, &scope.0) {
            return Err(error(OverlayErrorCode::ScopeMismatch));
        }
        if self.directory.is_none() {
            return Err(error(OverlayErrorCode::Closed));
        }
        Ok(())
    }
    pub(super) fn path_key(&self, path: &str) -> OverlayResult<Box<str>> {
        crate::disk::validate_path(path).map_err(|_| error(OverlayErrorCode::InvalidPath))?;
        if !path.ends_with(".lua") {
            return Err(error(OverlayErrorCode::InvalidPath));
        }
        Ok(if cfg!(windows) {
            path.to_lowercase().into()
        } else {
            path.into()
        })
    }
    pub fn document_uri(&self, scope: &OverlaySessionScope, path: &str) -> OverlayResult<Box<str>> {
        self.require(scope)?;
        self.path_key(path)?;
        Ok(format!("{}/{}", self.root_uri, uri_path(path)).into())
    }
    pub(super) fn require_uri(&self, path: &str, uri: &str) -> OverlayResult<()> {
        if uri.len() > 65536 || uri != format!("{}/{}", self.root_uri, uri_path(path)) {
            return Err(error(OverlayErrorCode::InvalidPath));
        }
        Ok(())
    }
    pub(super) fn retained(&mut self) -> OverlayResult<(usize, usize)> {
        self.snapshots
            .retain(|snapshot| snapshot.strong_count() > 0);
        let mut bytes = self.analyzer_bytes.load(Ordering::Acquire);
        for snapshot in &self.snapshots {
            if let Some(snapshot) = snapshot.upgrade() {
                bytes = bytes
                    .checked_add(snapshot.retained_bytes)
                    .ok_or_else(|| error(OverlayErrorCode::Limit))?;
            }
        }
        Ok((self.snapshots.len(), bytes))
    }
    pub fn close(
        &mut self,
        scope: &OverlaySessionScope,
    ) -> OverlayResult<OverlayWorkspaceDescriptor> {
        if !Arc::ptr_eq(&self.scope, &scope.0) {
            return Err(error(OverlayErrorCode::ScopeMismatch));
        }
        if self.leases.load(Ordering::Acquire) != 0 {
            return Err(error(OverlayErrorCode::Busy));
        }
        self.current.clear();
        self.snapshots.clear();
        self.directory.take();
        self.descriptor.state = "closed".into();
        Ok(self.descriptor.clone())
    }
}
fn uri_path(path: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut output = String::with_capacity(path.len());
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/' | b':') {
            output.push(char::from(byte));
        } else {
            output.push('%');
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 15)]));
        }
    }
    output
}
