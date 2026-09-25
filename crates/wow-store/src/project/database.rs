use super::model::*;
use crate::{StoreError, StoreErrorCode, StoreResult};
use rusqlite::{Connection, OpenFlags, OptionalExtension, config::DbConfig};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    rc::Rc,
    time::Duration,
};

const APPLICATION_ID: i64 = 0x5744_5031;
const SCHEMA: &str = r#"
CREATE TABLE epoch_metadata (id INTEGER PRIMARY KEY CHECK(id=1), manifest BLOB NOT NULL) STRICT;
CREATE TABLE partition_versions (
    version TEXT PRIMARY KEY, logical_key TEXT NOT NULL, schema_id TEXT NOT NULL,
    byte_length INTEGER NOT NULL CHECK(byte_length>=0 AND byte_length<=33554432),
    payload BLOB NOT NULL CHECK(length(payload)=byte_length)
) STRICT;
CREATE TABLE generations (generation_id TEXT PRIMARY KEY, manifest BLOB NOT NULL) STRICT;
CREATE TABLE membership (
    generation_id TEXT NOT NULL REFERENCES generations(generation_id), logical_key TEXT NOT NULL,
    version TEXT NOT NULL REFERENCES partition_versions(version), PRIMARY KEY(generation_id,logical_key)
) STRICT;
CREATE TABLE operations (
    operation_id TEXT PRIMARY KEY, request_digest TEXT NOT NULL, manifest BLOB NOT NULL, record BLOB NOT NULL
) STRICT;
CREATE TABLE validations (
    validation_id TEXT PRIMARY KEY, generation_id TEXT NOT NULL REFERENCES generations(generation_id), record BLOB NOT NULL
) STRICT;
CREATE TABLE publication_history (
    record_id TEXT PRIMARY KEY, generation_id TEXT NOT NULL REFERENCES generations(generation_id),
    validation_id TEXT NOT NULL REFERENCES validations(validation_id), record BLOB NOT NULL
) STRICT;
CREATE TABLE current_publication (
    id INTEGER PRIMARY KEY CHECK(id=1), record_id TEXT NOT NULL REFERENCES publication_history(record_id)
) STRICT;
CREATE INDEX membership_version ON membership(version);
"#;

pub(super) struct Lifetime {
    // The OS-held writer lease survives the writer while any read snapshot lives.
    pub _lock: File,
    pub leases: RefCell<BTreeMap<StoreGenerationId, usize>>,
}
pub(super) struct Database {
    pub connection: Connection,
    pub path: PathBuf,
    pub epoch: EpochManifest,
    pub life: Rc<Lifetime>,
}

impl Database {
    pub fn create(root: &Path, owner: &str, catalog: RecordCatalog) -> StoreResult<Self> {
        let schema_digest = expected_schema()?;
        let runtime = runtime_id()?;
        let epoch = EpochManifest::new(owner, catalog, runtime, schema_digest)?;
        // Creation never adopts a pre-existing directory or SQLite database.
        create_private_directory(root).map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        let root = admitted_root(root)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(root.join("writer.lock"))
            .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        lock.try_lock()
            .map_err(|_| failure(StoreErrorCode::WriterBusy))?;
        fs::create_dir(root.join("epochs"))
            .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        let dir = epoch_directory(&root, &epoch)?;
        fs::create_dir(&dir).map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        let path = dir.join("project.sqlite");
        let file =
            File::create_new(&path).map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        file.sync_all()
            .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        drop(file);
        let connection = connect(&path, false)?;
        connection
            .execute_batch(
                "PRAGMA page_size=4096; PRAGMA auto_vacuum=NONE; PRAGMA encoding=\"UTF-8\";",
            )
            .map_err(StoreError::database)?;
        connection
            .pragma_update(None, "application_id", APPLICATION_ID)
            .map_err(StoreError::database)?;
        connection
            .pragma_update(None, "user_version", 1)
            .map_err(StoreError::database)?;
        connection
            .execute_batch(SCHEMA)
            .map_err(StoreError::database)?;
        let bytes = encode(&epoch, 65536)?;
        connection
            .execute(
                "INSERT INTO epoch_metadata(id,manifest) VALUES(1,?1)",
                [&bytes],
            )
            .map_err(StoreError::database)?;
        enable_writer(&connection)?;
        validate_header(&connection, &epoch)?;
        // The outer selector is published only after a valid epoch exists. An
        // interrupted new root is not silently repaired/adopted on the next open.
        let mut epoch_file =
            File::create_new(dir.join("epoch-manifest.json")).map_err(|_| invalid())?;
        epoch_file
            .write_all(&bytes)
            .and_then(|()| epoch_file.sync_all())
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        let mut registry = File::create_new(root.join("project-store-registry.json"))
            .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        registry
            .write_all(&bytes)
            .and_then(|()| registry.sync_all())
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        Ok(Self {
            connection,
            path,
            epoch,
            life: Rc::new(Lifetime {
                _lock: lock,
                leases: RefCell::new(BTreeMap::new()),
            }),
        })
    }
    pub fn open(root: &Path, catalog: &RecordCatalog) -> StoreResult<Self> {
        let root = admitted_root(root)?;
        let lock_path = root.join("writer.lock");
        regular(&lock_path, 0)?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .open(lock_path)
            .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        lock.try_lock()
            .map_err(|_| failure(StoreErrorCode::WriterBusy))?;
        let registry_path = root.join("project-store-registry.json");
        regular(&registry_path, 65536)?;
        let mut bytes = Vec::new();
        File::open(registry_path)
            .map_err(|_| invalid())?
            .take(65537)
            .read_to_end(&mut bytes)
            .map_err(|_| invalid())?;
        if bytes.len() > 65536 {
            return Err(invalid());
        }
        let epoch: EpochManifest = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
        let expected = EpochManifest::new(
            &epoch.owner,
            catalog.clone(),
            runtime_id()?,
            expected_schema()?,
        )?;
        if epoch != expected || encode(&epoch, 65536)? != bytes {
            return Err(invalid());
        }
        directory(&root.join("epochs"))?;
        let dir = epoch_directory(&root, &epoch)?;
        directory(&dir)?;
        let epoch_path = dir.join("epoch-manifest.json");
        regular(&epoch_path, 65536)?;
        let mut epoch_bytes = Vec::new();
        File::open(epoch_path)
            .map_err(|_| invalid())?
            .take(65537)
            .read_to_end(&mut epoch_bytes)
            .map_err(|_| invalid())?;
        if epoch_bytes != bytes {
            return Err(invalid());
        }
        let path = dir.join("project.sqlite");
        regular(&path, 1024 * 1024 * 1024)?;
        for name in [
            "project.sqlite-wal",
            "project.sqlite-shm",
            "project.sqlite-journal",
        ] {
            let sidecar = dir.join(name);
            match fs::symlink_metadata(&sidecar) {
                Ok(_) => regular(&sidecar, 128 * 1024 * 1024)?,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err(invalid()),
            }
        }
        // Inspect an existing database read-only before any writable open or DDL.
        let inspect = connect(&path, true)?;
        validate_header(&inspect, &epoch)?;
        drop(inspect);
        let connection = connect(&path, false)?;
        validate_header(&connection, &epoch)?;
        enable_writer(&connection)?;
        Ok(Self {
            connection,
            path,
            epoch,
            life: Rc::new(Lifetime {
                _lock: lock,
                leases: RefCell::new(BTreeMap::new()),
            }),
        })
    }
    pub fn read_connection(&self) -> StoreResult<Connection> {
        let c = connect(&self.path, true)?;
        validate_header(&c, &self.epoch)?;
        c.execute_batch("BEGIN DEFERRED")
            .map_err(StoreError::database)?;
        Ok(c)
    }
    pub fn write_budget(&self, bytes: usize) -> StoreResult<()> {
        // Reserve WAL growth conservatively before entering any transaction. A
        // pinned reader can cause an explicit refusal, never unbounded growth.
        let mut wal = self.path.as_os_str().to_os_string();
        wal.push("-wal");
        let size = match fs::metadata(PathBuf::from(wal)) {
            Ok(m) => m.len(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => 0,
            Err(_) => return Err(invalid()),
        };
        let reserve = (bytes as u64).saturating_mul(3).saturating_add(1024 * 1024);
        if size.saturating_add(reserve) > 128 * 1024 * 1024 {
            return Err(failure(StoreErrorCode::BudgetExceeded));
        }
        Ok(())
    }
}

fn create_private_directory(root: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new().mode(0o700).create(root)
    }
    #[cfg(not(unix))]
    {
        fs::create_dir(root)
    }
}
fn admitted_root(root: &Path) -> StoreResult<PathBuf> {
    directory(root)?;
    fs::canonicalize(root).map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))
}
fn directory(path: &Path) -> StoreResult<()> {
    let m = fs::symlink_metadata(path).map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
    if m.file_type().is_symlink() || !m.is_dir() || reparse(&m) {
        return Err(invalid());
    }
    Ok(())
}
fn regular(path: &Path, max: u64) -> StoreResult<()> {
    let m = fs::symlink_metadata(path).map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
    if m.file_type().is_symlink() || !m.is_file() || reparse(&m) || m.len() > max {
        return Err(invalid());
    }
    Ok(())
}
#[cfg(windows)]
fn reparse(m: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    m.file_attributes() & 0x400 != 0
}
#[cfg(not(windows))]
fn reparse(_: &fs::Metadata) -> bool {
    false
}
fn epoch_directory(root: &Path, epoch: &EpochManifest) -> StoreResult<PathBuf> {
    let id = epoch
        .epoch_id
        .as_str()
        .strip_prefix("project-epoch:sha256:")
        .ok_or_else(invalid)?;
    Ok(root.join("epochs").join(id))
}
fn runtime_id() -> StoreResult<String> {
    let c = Connection::open_in_memory().map_err(StoreError::database)?;
    let source: String = c
        .query_row("SELECT sqlite_source_id()", [], |r| r.get(0))
        .map_err(StoreError::database)?;
    let mut statement = c
        .prepare("PRAGMA compile_options")
        .map_err(StoreError::database)?;
    let mut options = statement
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(StoreError::database)?
        .take(257)
        .collect::<Result<Vec<_>, _>>()
        .map_err(StoreError::database)?;
    if options.len() > 256 {
        return Err(invalid());
    }
    options.sort_unstable();
    Ok(digest(
        "sqlite-runtime",
        &encode(&(source, options), 65536)?,
    ))
}
pub(super) fn connect(path: &Path, readonly: bool) -> StoreResult<Connection> {
    let mode = if readonly {
        OpenFlags::SQLITE_OPEN_READ_ONLY
    } else {
        OpenFlags::SQLITE_OPEN_READ_WRITE
    };
    let c = Connection::open_with_flags(path, mode | OpenFlags::SQLITE_OPEN_NO_MUTEX)
        .map_err(StoreError::database)?;
    c.busy_timeout(Duration::from_millis(500))
        .map_err(StoreError::database)?;
    c.set_db_config(DbConfig::SQLITE_DBCONFIG_DEFENSIVE, true)
        .map_err(StoreError::database)?;
    c.set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_TRIGGER, false)
        .map_err(StoreError::database)?;
    c.execute_batch("PRAGMA trusted_schema=OFF; PRAGMA foreign_keys=ON;")
        .map_err(StoreError::database)?;
    if readonly {
        c.execute_batch("PRAGMA query_only=ON")
            .map_err(StoreError::database)?;
    }
    Ok(c)
}
fn enable_writer(c: &Connection) -> StoreResult<()> {
    let mode: String = c
        .query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))
        .map_err(StoreError::database)?;
    if mode != "wal" {
        return Err(failure(StoreErrorCode::ConfigurationInvalid));
    }
    c.execute_batch("PRAGMA synchronous=FULL; PRAGMA wal_autocheckpoint=256; PRAGMA journal_size_limit=8388608; PRAGMA max_page_count=262144;")
        .map_err(StoreError::database)?;
    let sync: i64 = c
        .query_row("PRAGMA synchronous", [], |r| r.get(0))
        .map_err(StoreError::database)?;
    let fk: i64 = c
        .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
        .map_err(StoreError::database)?;
    let page: i64 = c
        .query_row("PRAGMA page_size", [], |r| r.get(0))
        .map_err(StoreError::database)?;
    if sync != 2 || fk != 1 || page != 4096 {
        return Err(invalid());
    }
    Ok(())
}
fn expected_schema() -> StoreResult<String> {
    let c = Connection::open_in_memory().map_err(StoreError::database)?;
    c.execute_batch(SCHEMA).map_err(StoreError::database)?;
    schema_digest(&c)
}
fn schema_digest(c: &Connection) -> StoreResult<String> {
    let mut s = c.prepare("SELECT CASE WHEN length(type)<=64 THEN type END,CASE WHEN length(name)<=256 THEN name END,CASE WHEN length(tbl_name)<=256 THEN tbl_name END,CASE WHEN length(sql)<=8192 THEN sql END FROM sqlite_schema WHERE substr(name,1,7)!='sqlite_' ORDER BY type,name LIMIT 33").map_err(StoreError::database)?;
    let entries = s
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .map_err(StoreError::database)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(StoreError::database)?;
    if entries.len() > 32 {
        return Err(invalid());
    }
    Ok(digest("schema", &encode(&entries, 65536)?))
}
fn validate_header(c: &Connection, epoch: &EpochManifest) -> StoreResult<()> {
    let app: i64 = c
        .query_row("PRAGMA application_id", [], |r| r.get(0))
        .map_err(StoreError::database)?;
    let version: i64 = c
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(StoreError::database)?;
    let mode: String = c
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .map_err(StoreError::database)?;
    let page: i64 = c
        .query_row("PRAGMA page_size", [], |r| r.get(0))
        .map_err(StoreError::database)?;
    let vacuum: i64 = c
        .query_row("PRAGMA auto_vacuum", [], |r| r.get(0))
        .map_err(StoreError::database)?;
    let encoding: String = c
        .query_row("PRAGMA encoding", [], |r| r.get(0))
        .map_err(StoreError::database)?;
    if page != 4096
        || vacuum != 0
        || encoding != "UTF-8"
        || app != APPLICATION_ID
        || version != 1
        || mode != "wal"
        || schema_digest(c)? != epoch.schema_digest
    {
        return Err(invalid());
    }
    let bytes = blob(
        c,
        "SELECT CASE WHEN length(manifest)<=?2 THEN manifest END FROM epoch_metadata WHERE id=?1",
        "1",
        65536,
    )?
    .ok_or_else(invalid)?;
    if bytes != encode(epoch, 65536)? {
        return Err(invalid());
    }
    Ok(())
}
/// SQL is always one of this module's repository-owned statements.
pub(super) fn blob(
    c: &Connection,
    sql: &'static str,
    key: &str,
    max: usize,
) -> StoreResult<Option<Vec<u8>>> {
    let result: Option<Option<Vec<u8>>> = c
        .query_row(sql, rusqlite::params![key, max as i64], |r| r.get(0))
        .optional()
        .map_err(StoreError::database)?;
    match result {
        Some(Some(bytes)) => Ok(Some(bytes)),
        Some(None) => Err(failure(StoreErrorCode::BudgetExceeded)),
        None => Ok(None),
    }
}
