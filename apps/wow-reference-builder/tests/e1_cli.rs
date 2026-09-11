use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static SEQUENCE: AtomicU64 = AtomicU64::new(0);

fn path(label: &str, extension: &str) -> PathBuf {
    let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "wow-reference-builder-{label}-{}-{sequence}.{extension}",
        std::process::id()
    ))
}

#[test]
fn status_uses_only_explicit_database_and_publication_key() {
    let database = path("status", "sqlite3");
    let _ = fs::remove_file(&database);
    let arguments = vec![
        "status".to_owned(),
        "--db".to_owned(),
        database.to_string_lossy().into_owned(),
        "--store-profile".to_owned(),
        "reference-cli-test".to_owned(),
        "--profile".to_owned(),
        "retail-12.1".to_owned(),
        "--channel".to_owned(),
        "stable".to_owned(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        wow_reference_builder::run(arguments, &mut stdout, &mut stderr),
        0
    );
    assert!(stderr.is_empty());
    assert!(stdout.ends_with(b"\n"));
    let value: serde_json::Value =
        serde_json::from_slice(&stdout).unwrap_or_else(|error| panic!("status JSON: {error}"));
    assert_eq!(
        value["publication_profile"],
        serde_json::Value::String("retail-12.1".to_owned())
    );
    assert!(value["current_reference_object_id"].is_null());
    let _ = fs::remove_file(database);
}

#[test]
fn invalid_reference_input_produces_no_stdout_or_publication() {
    let input = path("invalid", "json");
    fs::write(&input, b"{\"not\":\"a-reference-view\"}\n")
        .unwrap_or_else(|error| panic!("fixture write: {error}"));
    let arguments = vec![
        "validate".to_owned(),
        "--store-profile".to_owned(),
        "reference-cli-test".to_owned(),
        "--input".to_owned(),
        input.to_string_lossy().into_owned(),
    ];
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        wow_reference_builder::run(arguments, &mut stdout, &mut stderr),
        4
    );
    assert!(stdout.is_empty());
    assert!(String::from_utf8_lossy(&stderr).contains("reference view"));
    let _ = fs::remove_file(input);
}
