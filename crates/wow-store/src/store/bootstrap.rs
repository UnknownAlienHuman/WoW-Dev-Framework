//! Own a private bootstrap attempt; publish only a closed, validated database.
use super::*;
use std::{
    fs::{DirBuilder, File, OpenOptions},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

struct Staging {
    directory: PathBuf,
    path: PathBuf,
    file: Option<File>,
}

impl Staging {
    fn create(final_path: &Path) -> StoreResult<Self> {
        let parent = final_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let sequence = SEQUENCE
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| failure("bootstrap attempt sequence is exhausted"))?;
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| failure("bootstrap clock is unavailable"))?
            .as_nanos();
        let directory = parent.join(format!(
            ".wow-store-bootstrap-{}-{time}-{sequence}",
            std::process::id()
        ));
        let builder = DirBuilder::new();
        #[cfg(unix)]
        let mut builder = builder;
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder
            .create(&directory)
            .map_err(|_| failure("private bootstrap directory could not be created"))?;
        let path = directory.join("store.sqlite");
        let mut staging = Self {
            directory,
            path,
            file: None,
        };
        let mut options = OpenOptions::new();
        options.read(true).write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(3);
        }
        staging.file = Some(
            options
                .open(&staging.path)
                .map_err(|_| failure("private bootstrap file could not be created"))?,
        );
        Ok(staging)
    }
}

impl Drop for Staging {
    fn drop(&mut self) {
        drop(self.file.take());
        // Only these two exclusively created names belong to this live attempt.
        // No recursive scan/delete or adoption of another attempt's remnants.
        let _ = fs::remove_file(&self.path);
        let _ = fs::remove_dir(&self.directory);
    }
}

pub(super) fn create(final_path: &Path, configuration: StoreConfiguration) -> StoreResult<Store> {
    let staging = Staging::create(final_path)?;
    staging
        .file
        .as_ref()
        .ok_or_else(|| failure("bootstrap handle is unavailable"))?
        .sync_all()
        .map_err(|_| failure("bootstrap file could not be synchronized"))?;
    #[cfg(test)]
    interrupt_at("created");
    let connection = Connection::open_with_flags(&staging.path, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .map_err(StoreError::database)?;
    let store = Store::initialize_connection(connection, configuration.clone())?;
    if !store
        .validate_integrity(configuration.limits().max_manifest_records)?
        .complete()
    {
        return Err(failure("bootstrap validation is incomplete"));
    }
    let Store {
        connection,
        configuration: _,
    } = store;
    connection.close().map_err(|(connection, error)| {
        drop(connection);
        StoreError::database(error)
    })?;
    staging
        .file
        .as_ref()
        .ok_or_else(|| failure("bootstrap handle is unavailable"))?
        .sync_all()
        .map_err(|_| failure("ready bootstrap file could not be synchronized"))?;
    #[cfg(test)]
    interrupt_at("ready");
    // The link operation fails if a winner already owns the final name. Unlike
    // rename on Unix, it never replaces that winner. Unsupported publication is
    // a typed refusal with the requested pathname still unoccupied.
    match fs::hard_link(&staging.path, final_path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(_) => {
            return Err(failure(
                "atomic no-replace bootstrap publication is unavailable",
            ));
        }
    }
    drop(staging);
    super::admit_file(final_path, configuration)
}

fn failure(message: &'static str) -> StoreError {
    StoreError::new(StoreErrorCode::DatabaseUnavailable, message)
}

#[cfg(test)]
pub(super) fn interrupt_at(phase: &str) {
    if std::env::var("WOW_STORE_BOOTSTRAP_TEST_PHASE").as_deref() == Ok(phase) {
        std::process::exit(71);
    }
}

#[cfg(test)]
mod tests;
