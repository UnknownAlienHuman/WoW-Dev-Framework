use super::{
    model::*,
    namespace::ProjectStoreNamespace,
    registry::{self, RegistrySelection},
};
use crate::{StoreError, StoreErrorCode, StoreResult};
use rusqlite::{Connection, OpenFlags, OptionalExtension, config::DbConfig};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    rc::Rc,
    time::Duration,
};

const APPLICATION_ID: i64 = 0x5744_5031;
const MAX_EPOCH_BYTES: usize = 65536;
const MAX_DATABASE_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_SIDECAR_BYTES: u64 = 128 * 1024 * 1024;
const BUSY_TIMEOUT_MS: u64 = 500;
const DEFENSIVE: bool = true;
const ENABLE_TRIGGERS: bool = false;
const CREATE_POLICY: &str =
    "PRAGMA page_size=4096; PRAGMA auto_vacuum=NONE; PRAGMA encoding=\"UTF-8\";";
const CONNECT_POLICY: &str = "PRAGMA trusted_schema=OFF; PRAGMA foreign_keys=ON;";
const READ_POLICY: &str = "PRAGMA query_only=ON";
const WRITER_POLICY: &str = "PRAGMA synchronous=FULL; PRAGMA wal_autocheckpoint=256; PRAGMA journal_size_limit=8388608; PRAGMA max_page_count=262144;";
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

const RETENTION_SCHEMA: &str = r#"
CREATE TABLE retention_roots (
    root_id TEXT PRIMARY KEY,
    generation_id TEXT NOT NULL REFERENCES generations(generation_id),
    record BLOB NOT NULL CHECK(length(record)<=65536)
) STRICT;
CREATE INDEX retention_generation ON retention_roots(generation_id);
"#;

const GC_SCHEMA: &str = r#"
CREATE TABLE gc_policy (
    id INTEGER PRIMARY KEY CHECK(id=1),
    policy_digest TEXT NOT NULL,
    record BLOB NOT NULL CHECK(length(record)<=262144)
) STRICT;
CREATE TABLE gc_operations (
    operation_id TEXT PRIMARY KEY,
    request_digest TEXT NOT NULL,
    record BLOB NOT NULL CHECK(length(record)<=2097152)
) STRICT;
"#;

pub(super) struct Lifetime {
    // The OS-held writer lease survives the writer while any read snapshot lives.
    pub _lock: Rc<File>,
    pub _instance_lock: Option<Rc<File>>,
    pub leases: RefCell<BTreeMap<StoreGenerationId, usize>>,
    pub lease_revision: Cell<u64>,
    pub reader_admissions: Rc<Cell<usize>>,
}
pub(super) struct Database {
    pub connection: Connection,
    pub path: PathBuf,
    pub epoch: EpochManifest,
    pub life: Rc<Lifetime>,
    pub root: PathBuf,
    pub selection: Option<RegistrySelection>,
}

impl Database {
    pub fn create(root: &Path, owner: &str, catalog: RecordCatalog) -> StoreResult<Self> {
        Self::create_profile(root, owner, catalog, PHYSICAL_PROFILE)
    }
    pub fn create_with_retention(
        root: &Path,
        owner: &str,
        catalog: RecordCatalog,
    ) -> StoreResult<Self> {
        Self::create_profile(root, owner, catalog, RETAINED_PHYSICAL_PROFILE)
    }
    pub fn create_with_gc(root: &Path, owner: &str, catalog: RecordCatalog) -> StoreResult<Self> {
        Self::create_profile(root, owner, catalog, GC_PHYSICAL_PROFILE)
    }
    pub fn create_with_namespace(
        root: &Path,
        namespace: &ProjectStoreNamespace,
        catalog: RecordCatalog,
    ) -> StoreResult<Self> {
        let epoch = EpochManifest::with_namespace(
            namespace,
            catalog,
            runtime_id()?,
            expected_schema(GC_PHYSICAL_PROFILE)?,
            security_limit_digest()?,
        )?;
        Self::select_initial(Self::create_unselected_epoch(root, epoch)?)
    }
    pub(super) fn create_inactive_with_gc(
        root: &Path,
        owner: &str,
        catalog: RecordCatalog,
    ) -> StoreResult<Self> {
        let db = Self::create_unselected_profile(root, owner, catalog, GC_PHYSICAL_PROFILE)?;
        write_epoch_file(
            &db.root.join("epoch-manifest.json"),
            &encode(&db.epoch, 65536)?,
        )?;
        Ok(db)
    }
    fn create_profile(
        root: &Path,
        owner: &str,
        catalog: RecordCatalog,
        profile: &str,
    ) -> StoreResult<Self> {
        Self::select_initial(Self::create_unselected_profile(
            root, owner, catalog, profile,
        )?)
    }
    fn select_initial(mut db: Self) -> StoreResult<Self> {
        let bytes = encode(&db.epoch, 65536)?;
        // The outer selector is published only after a valid epoch exists. An
        // interrupted new root is not silently repaired/adopted on the next open.
        let mut registry = File::create_new(db.root.join("project-store-registry.json"))
            .map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
        registry
            .write_all(&bytes)
            .and_then(|()| registry.sync_all())
            .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))?;
        db.selection = Some(RegistrySelection::from_bytes(&bytes, &db.epoch, 0, None));
        Ok(db)
    }
    fn create_unselected_profile(
        root: &Path,
        owner: &str,
        catalog: RecordCatalog,
        profile: &str,
    ) -> StoreResult<Self> {
        let schema_digest = expected_schema(profile)?;
        let runtime = runtime_id()?;
        let epoch = if profile == PHYSICAL_PROFILE {
            EpochManifest::new(owner, catalog, runtime, schema_digest)?
        } else {
            EpochManifest::with_physical_profile(owner, catalog, runtime, schema_digest, profile)?
        };
        Self::create_unselected_epoch(root, epoch)
    }
    fn create_unselected_epoch(root: &Path, epoch: EpochManifest) -> StoreResult<Self> {
        let profile = epoch.physical_profile.as_str();
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
            .execute_batch(CREATE_POLICY)
            .map_err(StoreError::database)?;
        connection
            .pragma_update(None, "application_id", APPLICATION_ID)
            .map_err(StoreError::database)?;
        connection
            .pragma_update(None, "user_version", physical_version(profile)?)
            .map_err(StoreError::database)?;
        connection
            .execute_batch(SCHEMA)
            .map_err(StoreError::database)?;
        if matches!(profile, RETAINED_PHYSICAL_PROFILE | GC_PHYSICAL_PROFILE) {
            connection
                .execute_batch(RETENTION_SCHEMA)
                .map_err(StoreError::database)?;
        }
        if profile == GC_PHYSICAL_PROFILE {
            connection
                .execute_batch(GC_SCHEMA)
                .map_err(StoreError::database)?;
        }
        let bytes = encode(&epoch, 65536)?;
        connection
            .execute(
                "INSERT INTO epoch_metadata(id,manifest) VALUES(1,?1)",
                [&bytes],
            )
            .map_err(StoreError::database)?;
        enable_writer(&connection)?;
        validate_header(&connection, &epoch)?;
        write_epoch_file(&dir.join("epoch-manifest.json"), &bytes)?;
        Ok(Self {
            connection,
            path,
            selection: None,
            root,
            epoch,
            life: Rc::new(Lifetime {
                _lock: Rc::new(lock),
                _instance_lock: None,
                leases: RefCell::new(BTreeMap::new()),
                lease_revision: Cell::new(0),
                reader_admissions: Rc::new(Cell::new(0)),
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
        let admitted = registry::read(&root, catalog)?;
        if admitted.quarantine.is_some() {
            return Err(failure(StoreErrorCode::Quarantined));
        }
        let epoch = admitted.epoch;
        let dir = admitted.selection.directory(&root, &epoch)?;
        let instance_lock = if let Some(instance) = admitted.selection.instance_root(&root) {
            let path = instance.join("writer.lock");
            regular(&path, 0)?;
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .map_err(|_| invalid())?;
            file.try_lock()
                .map_err(|_| failure(StoreErrorCode::WriterBusy))?;
            Some(Rc::new(file))
        } else {
            None
        };
        let (connection, path) = Self::open_epoch(&dir, &epoch)?;
        Ok(Self {
            connection,
            path,
            epoch,
            root,
            selection: Some(admitted.selection),
            life: Rc::new(Lifetime {
                _lock: Rc::new(lock),
                _instance_lock: instance_lock,
                leases: RefCell::new(BTreeMap::new()),
                lease_revision: Cell::new(0),
                reader_admissions: Rc::new(Cell::new(0)),
            }),
        })
    }
    pub(super) fn open_inactive(root: &Path, epoch: &EpochManifest) -> StoreResult<Self> {
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
        match fs::symlink_metadata(root.join(registry::REGISTRY_FILE)) {
            Ok(_) => return Err(invalid()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(invalid()),
        }
        let bytes = read_epoch_file(&root.join("epoch-manifest.json"))?;
        let admitted = admit_epoch(&bytes, &epoch.catalog)?;
        if &admitted != epoch {
            return Err(invalid());
        }
        directory(&root.join("epochs"))?;
        let dir = epoch_directory(&root, &admitted)?;
        directory(&dir)?;
        let (connection, path) = Self::open_epoch(&dir, &admitted)?;
        Ok(Self {
            connection,
            path,
            epoch: admitted,
            root,
            selection: None,
            life: Rc::new(Lifetime {
                _lock: Rc::new(lock),
                _instance_lock: None,
                leases: RefCell::new(BTreeMap::new()),
                lease_revision: Cell::new(0),
                reader_admissions: Rc::new(Cell::new(0)),
            }),
        })
    }
    fn open_epoch(dir: &Path, epoch: &EpochManifest) -> StoreResult<(Connection, PathBuf)> {
        if read_epoch_file(&dir.join("epoch-manifest.json"))? != encode(epoch, 65536)? {
            return Err(invalid());
        }
        let path = dir.join("project.sqlite");
        regular(&path, MAX_DATABASE_BYTES)?;
        for name in [
            "project.sqlite-wal",
            "project.sqlite-shm",
            "project.sqlite-journal",
        ] {
            let sidecar = dir.join(name);
            match fs::symlink_metadata(&sidecar) {
                Ok(_) => regular(&sidecar, MAX_SIDECAR_BYTES)?,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err(invalid()),
            }
        }
        // Inspect an existing database read-only before any writable open or DDL.
        let inspect = connect(&path, true)?;
        validate_header(&inspect, epoch)?;
        drop(inspect);
        let connection = connect(&path, false)?;
        validate_header(&connection, epoch)?;
        enable_writer(&connection)?;
        Ok((connection, path))
    }
    pub fn read_connection(&self) -> StoreResult<Connection> {
        self.ensure_idle()?;
        let c = connect(&self.path, true)?;
        validate_header(&c, &self.epoch)?;
        c.execute_batch("BEGIN DEFERRED")
            .map_err(StoreError::database)?;
        // Bind schema and epoch admission to the same snapshot as subsequent
        // closure reads, rather than only the pre-transaction observation.
        validate_header(&c, &self.epoch)?;
        Ok(c)
    }
    pub fn write_budget(&self, bytes: usize) -> StoreResult<()> {
        self.ensure_idle()?;
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
    pub fn ensure_idle(&self) -> StoreResult<()> {
        if !self.connection.is_autocommit() {
            return Err(failure(StoreErrorCode::OutcomeUnknown));
        }
        if let Some(selection) = &self.selection {
            let observed = registry::read_file(
                &self.root.join(registry::REGISTRY_FILE),
                registry::MAX_REGISTRY,
            )?;
            if digest("project-registry", &observed) != selection.digest() {
                if registry::read(&self.root, &self.epoch.catalog)?
                    .quarantine
                    .is_some()
                {
                    return Err(failure(StoreErrorCode::Quarantined));
                }
                return Err(failure(StoreErrorCode::OutcomeUnknown));
            }
        }
        Ok(())
    }
}

fn write_epoch_file(path: &Path, bytes: &[u8]) -> StoreResult<()> {
    let mut file = File::create_new(path).map_err(|_| invalid())?;
    file.write_all(bytes)
        .and_then(|()| file.sync_all())
        .map_err(|_| failure(StoreErrorCode::OutcomeUnknown))
}

fn read_epoch_file(path: &Path) -> StoreResult<Vec<u8>> {
    regular(path, 65536)?;
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(|_| invalid())?
        .take(65537)
        .read_to_end(&mut bytes)
        .map_err(|_| invalid())?;
    Ok(bytes)
}

pub(super) fn admit_epoch(bytes: &[u8], catalog: &RecordCatalog) -> StoreResult<EpochManifest> {
    if bytes.len() > MAX_EPOCH_BYTES {
        return Err(invalid());
    }
    let epoch: EpochManifest = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    let expected = match epoch.schema.as_str() {
        RECORD_PROFILE => EpochManifest::with_physical_profile(
            &epoch.owner,
            catalog.clone(),
            runtime_id()?,
            expected_schema(&epoch.physical_profile)?,
            &epoch.physical_profile,
        )?,
        NAMESPACE_EPOCH_SCHEMA => EpochManifest::with_namespace(
            epoch.namespace.as_ref().ok_or_else(invalid)?,
            catalog.clone(),
            runtime_id()?,
            expected_schema(GC_PHYSICAL_PROFILE)?,
            security_limit_digest()?,
        )?,
        _ => return Err(invalid()),
    };
    if epoch != expected || encode(&epoch, MAX_EPOCH_BYTES)? != bytes {
        return Err(invalid());
    }
    Ok(epoch)
}

pub(super) fn create_private_directory(root: &Path) -> std::io::Result<()> {
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
pub(super) fn admitted_root(root: &Path) -> StoreResult<PathBuf> {
    directory(root)?;
    fs::canonicalize(root).map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))
}
pub(super) fn directory(path: &Path) -> StoreResult<()> {
    let m = fs::symlink_metadata(path).map_err(|_| failure(StoreErrorCode::DatabaseUnavailable))?;
    if m.file_type().is_symlink() || !m.is_dir() || reparse(&m) {
        return Err(invalid());
    }
    Ok(())
}
pub(super) fn regular(path: &Path, max: u64) -> StoreResult<()> {
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
pub(super) fn epoch_directory(root: &Path, epoch: &EpochManifest) -> StoreResult<PathBuf> {
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
fn security_limit_digest() -> StoreResult<String> {
    let limits = BTreeMap::from([
        ("identifier_bytes", MAX_IDENTIFIER_BYTES as u64),
        ("catalog_entries", MAX_CATALOG_ENTRIES as u64),
        ("record_bytes", MAX_RECORD_BYTES as u64),
        ("generation_bytes", MAX_GENERATION_BYTES as u64),
        ("partitions", MAX_PARTITIONS as u64),
        ("generations", MAX_GENERATIONS as u64),
        ("partition_versions", MAX_VERSIONS as u64),
        ("readers", MAX_READERS as u64),
        ("epoch_bytes", MAX_EPOCH_BYTES as u64),
        ("database_bytes", MAX_DATABASE_BYTES),
        ("sidecar_bytes", MAX_SIDECAR_BYTES),
        ("busy_timeout_ms", BUSY_TIMEOUT_MS),
    ]);
    Ok(digest(
        "project-security-limits",
        &encode(
            &(
                "wow-store/project-security-limits/1",
                limits,
                DEFENSIVE,
                ENABLE_TRIGGERS,
                CREATE_POLICY,
                CONNECT_POLICY,
                READ_POLICY,
                WRITER_POLICY,
            ),
            MAX_EPOCH_BYTES,
        )?,
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
    c.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))
        .map_err(StoreError::database)?;
    c.set_db_config(DbConfig::SQLITE_DBCONFIG_DEFENSIVE, DEFENSIVE)
        .map_err(StoreError::database)?;
    c.set_db_config(DbConfig::SQLITE_DBCONFIG_ENABLE_TRIGGER, ENABLE_TRIGGERS)
        .map_err(StoreError::database)?;
    c.execute_batch(CONNECT_POLICY)
        .map_err(StoreError::database)?;
    if readonly {
        c.execute_batch(READ_POLICY).map_err(StoreError::database)?;
    }
    Ok(c)
}
pub(super) fn enable_writer(c: &Connection) -> StoreResult<()> {
    let mode: String = c
        .query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))
        .map_err(StoreError::database)?;
    if mode != "wal" {
        return Err(failure(StoreErrorCode::ConfigurationInvalid));
    }
    c.execute_batch(WRITER_POLICY)
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
fn physical_version(profile: &str) -> StoreResult<i64> {
    match profile {
        PHYSICAL_PROFILE => Ok(1),
        RETAINED_PHYSICAL_PROFILE => Ok(2),
        GC_PHYSICAL_PROFILE => Ok(3),
        _ => Err(failure(StoreErrorCode::ConfigurationInvalid)),
    }
}
fn expected_schema(profile: &str) -> StoreResult<String> {
    physical_version(profile)?;
    let c = Connection::open_in_memory().map_err(StoreError::database)?;
    c.execute_batch(SCHEMA).map_err(StoreError::database)?;
    if matches!(profile, RETAINED_PHYSICAL_PROFILE | GC_PHYSICAL_PROFILE) {
        c.execute_batch(RETENTION_SCHEMA)
            .map_err(StoreError::database)?;
    }
    if profile == GC_PHYSICAL_PROFILE {
        c.execute_batch(GC_SCHEMA).map_err(StoreError::database)?;
    }
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
pub(super) fn validate_header(c: &Connection, epoch: &EpochManifest) -> StoreResult<()> {
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
        || version != physical_version(&epoch.physical_profile)?
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
