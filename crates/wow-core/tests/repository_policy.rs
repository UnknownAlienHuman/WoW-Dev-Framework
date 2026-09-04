const ROOT_MANIFEST: &str = include_str!("../../../Cargo.toml");
const CRATE_MANIFEST: &str = include_str!("../Cargo.toml");
const TOOLCHAIN: &str = include_str!("../../../rust-toolchain.toml");

#[test]
fn workspace_activates_only_wow_core() {
    assert!(ROOT_MANIFEST.contains("members = [\"crates/wow-core\"]"));
    for forbidden_member in [
        "crates/wow-store",
        "crates/wow-reference",
        "crates/wow-emmy",
        "crates/wow-project",
        "crates/wow-rules",
        "crates/wow-service",
        "apps/wow",
    ] {
        assert!(!ROOT_MANIFEST.contains(forbidden_member));
    }
}

#[test]
fn exact_rust_toolchain_and_edition_are_frozen() {
    assert!(TOOLCHAIN.contains("channel = \"1.98.0\""));
    assert!(ROOT_MANIFEST.contains("edition = \"2024\""));
    assert!(ROOT_MANIFEST.contains("rust-version = \"1.98\""));
    assert!(ROOT_MANIFEST.contains("resolver = \"3\""));
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
fn production_dependencies_are_exactly_pinned() {
    for (name, version) in [
        ("semver", "1.0.28"),
        ("serde", "1.0.228"),
        ("serde_json", "1.0.150"),
        ("sha2", "0.11.0"),
    ] {
        let assignment = format!("{name} =");
        let exact_plain = format!("{name} = \"={version}\"");
        let exact_inline = format!("version = \"={version}\"");
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
                declaration == exact_plain || declaration.contains(&exact_inline),
                "production dependency {name} is not pinned exactly to {version}: {declaration}"
            );
        }
    }
}
