const ROOT_MANIFEST: &str = include_str!("../../../Cargo.toml");
const CRATE_MANIFEST: &str = include_str!("../Cargo.toml");

#[test]
fn workspace_activates_foundation_crates() {
    for member in ["crates/wow-core", "crates/wow-reference"] {
        assert!(ROOT_MANIFEST.contains(member));
    }
}

#[test]
fn workspace_does_not_require_a_toolchain_patch() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert!(!root.join("rust-toolchain.toml").exists());
    assert!(!root.join("rust-toolchain").exists());
}

#[test]
fn core_has_no_forbidden_runtime_dependency() {
    for forbidden_dependency in [
        "tokio",
        "async-std",
        "smol",
        "uuid",
        "rand",
        "chrono",
        "time =",
        "reqwest",
        "hyper",
        "ureq",
        "rusqlite",
        "sqlx",
        "tracing",
        "log =",
        "anyhow",
        "lsp-types",
        "tower-lsp",
    ] {
        assert!(!CRATE_MANIFEST.contains(forbidden_dependency));
    }
}

#[test]
fn production_dependencies_allow_compatible_updates() {
    for name in ["semver", "serde", "serde_json", "sha2"] {
        let assignment = format!("{name} =");
        let declaration = CRATE_MANIFEST
            .lines()
            .map(str::trim)
            .find(|line| line.starts_with(&assignment));
        assert!(
            declaration.is_some(),
            "missing production dependency {name}"
        );
        if let Some(declaration) = declaration {
            assert!(
                !declaration.contains("\"="),
                "exact version pin: {declaration}"
            );
            assert!(
                !declaration.contains("\"*\""),
                "unbounded version: {declaration}"
            );
        }
    }
}
