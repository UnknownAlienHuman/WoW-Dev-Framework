use std::{error::Error, fs, path::PathBuf};

use rusqlite::{Connection, MAIN_DB, params};
use sha2::{Digest, Sha256};
use wow_store::{
    PendingObject, SealedStore, Store, StoreConfiguration, StoreError, StoreErrorCode, StoreLimits,
    WriteBatch,
};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;
const IMAGE_LIMIT: u64 = 1024 * 1024;

struct Fixture {
    root: PathBuf,
    database: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> TestResult<Self> {
        let root = std::env::temp_dir().join(format!(
            "wow-store-admission-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        fs::create_dir(&root)?;
        let database = root.join("store.sqlite");
        Ok(Self { root, database })
    }

    fn bytes_digest(&self) -> TestResult<[u8; 32]> {
        Ok(Sha256::digest(fs::read(&self.database)?).into())
    }

    fn image(&self) -> TestResult<Vec<u8>> {
        let connection = Connection::open(&self.database)?;
        Ok(connection.serialize(MAIN_DB)?.to_vec())
    }

    fn close(self) -> TestResult {
        fs::remove_dir_all(self.root)?;
        Ok(())
    }
}

fn configuration() -> Result<StoreConfiguration, StoreError> {
    StoreConfiguration::new("fixture.admission", StoreLimits::new(32, 8, 8, 32, 8)?)
}

#[test]
fn existing_only_open_never_initializes_missing_or_foreign_files() -> TestResult {
    let fixture = Fixture::new("existing-only")?;
    refused(
        Store::open_existing(&fixture.database, configuration()?),
        StoreErrorCode::DatabaseUnavailable,
    )?;
    assert!(!fixture.database.exists());
    drop(Store::open(&fixture.database, configuration()?)?);
    drop(Store::open_existing(&fixture.database, configuration()?)?);
    fs::write(&fixture.database, b"foreign existing bytes")?;
    let before = fixture.bytes_digest()?;
    assert!(Store::open_existing(&fixture.database, configuration()?).is_err());
    assert_eq!(fixture.bytes_digest()?, before);
    fixture.close()
}

fn refused<T>(result: Result<T, StoreError>, expected: StoreErrorCode) -> TestResult {
    let error = result.err().ok_or("unexpected successful admission")?;
    assert_eq!(error.code(), expected);
    Ok(())
}

fn assert_file_and_image_refused(fixture: &Fixture) -> TestResult {
    let before = fixture.bytes_digest()?;
    let image = fixture.image()?;
    let image_digest: [u8; 32] = Sha256::digest(&image).into();
    refused(
        Store::open(&fixture.database, configuration()?),
        StoreErrorCode::IntegrityViolation,
    )?;
    assert_eq!(fixture.bytes_digest()?, before, "refusal mutated the file");
    refused(
        SealedStore::open_serialized(&image, configuration()?, IMAGE_LIMIT),
        StoreErrorCode::IntegrityViolation,
    )?;
    assert_eq!(<[u8; 32]>::from(Sha256::digest(&image)), image_digest);
    assert_eq!(fixture.bytes_digest()?, before);
    Ok(())
}

#[test]
fn foreign_and_zero_header_databases_remain_unchanged() -> TestResult {
    for application_id in [0, 123] {
        let fixture = Fixture::new(&format!("foreign-{application_id}"))?;
        let connection = Connection::open(&fixture.database)?;
        connection.pragma_update(None, "application_id", application_id)?;
        connection.execute_batch(
            "CREATE TABLE unrelated (marker TEXT NOT NULL);
             INSERT INTO unrelated VALUES('keep');",
        )?;
        drop(connection);
        assert_file_and_image_refused(&fixture)?;
        let connection = Connection::open(&fixture.database)?;
        let marker: String =
            connection.query_row("SELECT marker FROM unrelated", [], |row| row.get(0))?;
        assert_eq!(marker, "keep");
        let count: i64 = connection.query_row(
            "SELECT count(*) FROM sqlite_schema WHERE name GLOB 'store_*'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(count, 0, "foreign file acquired owned schema objects");
        drop(connection);
        fixture.close()?;
    }
    Ok(())
}

#[test]
fn an_existing_empty_file_is_not_a_creator_capability() -> TestResult {
    let fixture = Fixture::new("empty")?;
    fs::File::create_new(&fixture.database)?;
    refused(
        Store::open(&fixture.database, configuration()?),
        StoreErrorCode::IntegrityViolation,
    )?;
    assert_eq!(fs::metadata(&fixture.database)?.len(), 0);
    fixture.close()
}

#[test]
fn substituted_views_constraints_indexes_and_extra_objects_are_refused() -> TestResult {
    let mutations = [
        "DROP TABLE store_meta;
         CREATE VIEW store_meta AS SELECT key,value FROM missing_tripwire_table;",
        "DROP TABLE store_objects;
         CREATE TABLE store_objects (
             object_id TEXT PRIMARY KEY NOT NULL, kind TEXT NOT NULL,
             schema_version INTEGER NOT NULL, content_sha256 TEXT NOT NULL,
             canonical_json BLOB NOT NULL
         ) STRICT;",
        "DROP INDEX store_catalog_object_idx;
         CREATE INDEX store_catalog_object_idx ON store_catalog(path);",
        "CREATE TABLE sqliteXshadow (marker TEXT);",
        "INSERT INTO store_meta(key,value) VALUES('fixture_marker','keep');
         CREATE TRIGGER admission_tripwire BEFORE INSERT ON store_meta BEGIN
             UPDATE store_meta SET value='ran' WHERE key='fixture_marker';
         END;",
    ];
    for (index, mutation) in mutations.iter().enumerate() {
        let fixture = Fixture::new(&format!("schema-{index}"))?;
        let store = Store::open(&fixture.database, configuration()?)?;
        drop(store);
        let connection = Connection::open(&fixture.database)?;
        connection.execute_batch(mutation)?;
        drop(connection);
        assert_file_and_image_refused(&fixture)?;
        if index == 4 {
            let connection = Connection::open(&fixture.database)?;
            let marker: String = connection.query_row(
                "SELECT value FROM store_meta WHERE key='fixture_marker'",
                [],
                |row| row.get(0),
            )?;
            assert_eq!(marker, "keep", "a refused trigger executed");
        }
        fixture.close()?;
    }
    Ok(())
}

#[test]
fn exact_object_ceiling_round_trips_and_correct_hash_oversize_is_refused() -> TestResult {
    let fixture = Fixture::new("object-ceiling")?;
    let configuration = configuration()?;
    let exact =
        PendingObject::from_json("fixture.object", 1, &"x".repeat(30), configuration.limits())?;
    assert_eq!(exact.canonical_json().len(), 32);
    let exact_id = exact.object_id().clone();
    let mut store = Store::open(&fixture.database, configuration.clone())?;
    let mut batch = WriteBatch::new();
    batch.add_object(exact)?;
    store.commit(batch)?;
    assert_eq!(
        store
            .object(&exact_id)?
            .ok_or("missing exact object")?
            .canonical_json()
            .len(),
        32
    );
    let image = store.serialize_database(IMAGE_LIMIT)?;
    let sealed = SealedStore::open_serialized(&image, configuration.clone(), IMAGE_LIMIT)?;
    assert_eq!(
        sealed
            .object(&exact_id)?
            .ok_or("missing sealed object")?
            .canonical_json()
            .len(),
        32
    );
    assert!(sealed.validate_integrity(32)?.complete());
    drop(sealed);
    drop(store);

    // The oversized row has its genuine canonical bytes, object ID and hash.
    // It is inserted through disposable fixture SQL because the public writer
    // correctly refuses a caller's over-budget object before persistence.
    let large =
        PendingObject::from_json("fixture.object", 1, &"y".repeat(31), StoreLimits::default())?;
    assert_eq!(large.canonical_json().len(), 33);
    let large_id = large.object_id().clone();
    let connection = Connection::open(&fixture.database)?;
    connection.execute(
        "INSERT INTO store_objects(object_id,kind,schema_version,content_sha256,canonical_json)
         VALUES(?1,?2,?3,?4,?5)",
        params![
            large.object_id().as_str(),
            large.kind(),
            large.schema_version(),
            large.content_sha256(),
            large.canonical_json()
        ],
    )?;
    drop(connection);
    let before = fixture.bytes_digest()?;
    let store = Store::open(&fixture.database, configuration.clone())?;
    refused(store.object(&large_id), StoreErrorCode::ObjectTooLarge)?;
    refused(store.validate_integrity(32), StoreErrorCode::ObjectTooLarge)?;
    assert_eq!(
        store
            .object(&exact_id)?
            .ok_or("exact object lost")?
            .canonical_json()
            .len(),
        32
    );
    let missing =
        PendingObject::from_json("fixture.object", 1, &"missing", configuration.limits())?;
    assert!(store.object(missing.object_id())?.is_none());
    drop(store);
    let image = fixture.image()?;
    refused(
        SealedStore::open_serialized(&image, configuration, IMAGE_LIMIT),
        StoreErrorCode::ObjectTooLarge,
    )?;
    assert_eq!(fixture.bytes_digest()?, before);
    eprintln!(
        "native SQLite {}: exact 32 bytes accepted, valid-hash 33 bytes refused",
        rusqlite::version()
    );
    fixture.close()
}

#[test]
fn configuration_mismatch_stays_typed_and_does_not_mutate() -> TestResult {
    let fixture = Fixture::new("configuration")?;
    drop(Store::open(&fixture.database, configuration()?)?);
    let image = fixture.image()?;
    let before = fixture.bytes_digest()?;
    let other = StoreConfiguration::new("fixture.other", configuration()?.limits())?;
    refused(
        Store::open(&fixture.database, other.clone()),
        StoreErrorCode::ConfigurationInvalid,
    )?;
    refused(
        SealedStore::open_serialized(&image, other, IMAGE_LIMIT),
        StoreErrorCode::ConfigurationInvalid,
    )?;
    assert_eq!(fixture.bytes_digest()?, before);
    fixture.close()
}
