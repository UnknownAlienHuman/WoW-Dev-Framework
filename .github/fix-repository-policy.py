from pathlib import Path

path = Path("crates/wow-core/tests/repository_policy.rs")
source = path.read_text(encoding="utf-8")
old = '''#[test]
fn production_dependencies_are_exactly_pinned() {
    for required in [
        "semver = { version = \"=1.0.28\"",
        "serde = { version = \"=1.0.228\"",
        "serde_json = { version = \"=1.0.150\"",
        "sha2 = { version = \"=0.11.0\"",
    ] {
        assert!(ROOT_MANIFEST.contains(required));
    }
}
'''
new = '''#[test]
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
'''
count = source.count(old)
if count != 1:
    raise SystemExit(f"repository policy block: expected one match, found {count}")
path.write_text(source.replace(old, new, 1), encoding="utf-8")
