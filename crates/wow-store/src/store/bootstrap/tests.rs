use super::*;
use crate::StoreLimits;
use std::{error::Error, process::Command};

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn configuration() -> StoreResult<StoreConfiguration> {
    StoreConfiguration::new("fixture.bootstrap", StoreLimits::default())
}

fn root(label: &str) -> TestResult<PathBuf> {
    let root = std::env::temp_dir().join(format!(
        "wow-bootstrap-{label}-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir(&root)?;
    Ok(root)
}

#[test]
fn bootstrap_child() -> TestResult {
    let Some(path) = std::env::var_os("WOW_STORE_BOOTSTRAP_TEST_PATH") else {
        return Ok(());
    };
    drop(Store::open(PathBuf::from(path), configuration()?)?);
    Err("the requested interruption did not occur".into())
}

#[test]
fn real_process_interruption_never_strands_the_final_journal() -> TestResult {
    for phase in ["created", "schema", "ready"] {
        let root = root(phase)?;
        let final_path = root.join("journal.sqlite");
        let old_pack = root.join("previous.pack");
        fs::write(&old_pack, b"retained previous pack")?;
        let child = Command::new(std::env::current_exe()?)
            .args([
                "--exact",
                "store::bootstrap::tests::bootstrap_child",
                "--nocapture",
            ])
            .env("WOW_STORE_BOOTSTRAP_TEST_PATH", &final_path)
            .env("WOW_STORE_BOOTSTRAP_TEST_PHASE", phase)
            .output()?;
        assert_eq!(
            child.status.code(),
            Some(71),
            "bootstrap phase {phase} was not reached"
        );
        assert!(
            !final_path.exists(),
            "interruption published a partial final journal"
        );
        let remnants = fs::read_dir(&root)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".wow-store-bootstrap-")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            remnants.len(),
            1,
            "owned interrupted attempt was not retained"
        );
        let orphan = remnants[0].path().join("store.sqlite");
        let before = fs::read(&orphan)?;
        let store = Store::open(&final_path, configuration()?)?;
        assert!(store.validate_integrity(32)?.complete());
        drop(store);
        assert_eq!(
            fs::read(&orphan)?,
            before,
            "retry adopted or changed an old attempt"
        );
        drop(Store::open(&final_path, configuration()?)?);
        assert_eq!(fs::read(&old_pack)?, b"retained previous pack");
        eprintln!(
            "native SQLite {}: process stopped at {phase}; final path retry succeeded",
            rusqlite::version()
        );
        fs::remove_dir_all(root)?;
    }
    Ok(())
}

#[test]
fn publication_never_replaces_an_existing_foreign_winner() -> TestResult {
    let root = root("winner")?;
    let final_path = root.join("journal.sqlite");
    // Exercise the publication helper's race branch, not only Store::open's
    // pre-existing-file check. The fully prepared private file cannot replace it.
    fs::write(&final_path, b"foreign winner")?;
    let before = fs::read(&final_path)?;
    let result = create(&final_path, configuration()?);
    assert!(result.is_err());
    assert_eq!(fs::read(&final_path)?, before);
    fs::remove_dir_all(root)?;
    Ok(())
}
