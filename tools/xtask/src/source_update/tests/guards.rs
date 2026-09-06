use super::super::{remote::validate_origin, update};
use super::support::{Fixture, put};
use crate::{Result, git};
use std::fs;

#[test]
fn wrong_expected_head_branch_and_detached_state_refuse_before_network() -> Result<()> {
    for case in ["head", "branch", "detached"] {
        let f = Fixture::new("sha1", false)?;
        let remote = f.remote();
        if case == "detached" {
            git::isolated_text(&f.checkout, &["checkout", "--detach"])?;
        }
        let expected = if case == "head" {
            "1".repeat(40)
        } else {
            f.initial.clone()
        };
        let branch = if case == "branch" { "main" } else { "live" };
        assert!(update(&f.checkout, branch, &expected, &remote).is_err());
        assert_eq!(remote.heads.get(), 0);
        assert_eq!(f.head()?, f.initial);
        assert!(!f.lock_exists());
    }
    Ok(())
}
#[test]
fn tracked_staged_untracked_and_hidden_index_edits_are_preserved() -> Result<()> {
    for case in ["tracked", "staged", "untracked", "assume", "skip"] {
        let f = Fixture::new("sha1", false)?;
        match case {
            "untracked" => put(&f.checkout, "my.txt", "my work")?,
            "assume" => {
                git::isolated_text(
                    &f.checkout,
                    &["update-index", "--assume-unchanged", "data.txt"],
                )?;
            }
            "skip" => {
                git::isolated_text(
                    &f.checkout,
                    &["update-index", "--skip-worktree", "data.txt"],
                )?;
            }
            _ => {}
        }
        put(&f.checkout, "data.txt", "do not lose")?;
        if case == "staged" {
            git::isolated_text(&f.checkout, &["add", "data.txt"])?;
        }
        let index = fs::read(f.checkout.join(".git/index"))?;
        let remote = f.remote();
        assert!(
            update(&f.checkout, "live", &f.initial, &remote).is_err(),
            "{case}"
        );
        assert_eq!(remote.heads.get(), 0);
        assert_eq!(f.head()?, f.initial);
        assert_eq!(
            fs::read_to_string(f.checkout.join("data.txt"))?,
            "do not lose"
        );
        assert_eq!(index, fs::read(f.checkout.join(".git/index"))?);
        assert!(!f.lock_exists());
    }
    Ok(())
}
#[test]
fn existing_lock_and_unfinished_git_operations_are_not_deleted() -> Result<()> {
    for entry in [
        "wow-source-update.lock",
        "MERGE_HEAD",
        "CHERRY_PICK_HEAD",
        "REVERT_HEAD",
        "BISECT_LOG",
        "index.lock",
        "rebase-merge",
        "rebase-apply",
        "info/grafts",
    ] {
        let f = Fixture::new("sha1", false)?;
        put(
            &f.checkout,
            &format!(".git/{entry}"),
            "owned by another operation",
        )?;
        let remote = f.remote();
        assert!(update(&f.checkout, "live", &f.initial, &remote).is_err());
        assert_eq!(remote.heads.get(), 0);
        assert_eq!(
            fs::read_to_string(f.checkout.join(".git").join(entry))?,
            "owned by another operation"
        );
    }
    Ok(())
}
#[test]
fn filters_includes_rewrites_credentials_and_sparse_config_are_not_used() -> Result<()> {
    for (key, value) in [
        ("filter.test.smudge", "echo do-not-run"),
        ("include.path", "absent"),
        (
            "url.https://example.invalid/.insteadOf",
            "https://else.invalid/",
        ),
        ("credential.helper", "do-not-run"),
        ("core.askPass", "do-not-run"),
        ("branch.live.mergeOptions", "--autostash"),
        ("http.extraHeader", "Authorization: secret"),
        ("core.sparseCheckout", "true"),
        ("extensions.worktreeConfig", "true"),
    ] {
        let f = Fixture::new("sha1", false)?;
        git::isolated_text(&f.checkout, &["config", key, value])?;
        let remote = f.remote();
        assert!(
            update(&f.checkout, "live", &f.initial, &remote).is_err(),
            "{key}"
        );
        assert_eq!(remote.heads.get(), 0);
        assert_eq!(f.head()?, f.initial);
    }
    Ok(())
}
#[test]
fn origin_must_be_explicit_https_without_embedded_credentials() {
    for origin in [
        "https://",
        "https:///path",
        "http://example.com/x",
        "ssh://example/x",
        "file:///x",
        "https://u:p@example/x",
        "https://example/x?token=x",
        "https://example/x#ref",
        "https://example\\x",
        "https://example/\nx",
    ] {
        assert!(validate_origin(origin).is_err(), "{origin}");
    }
    assert!(validate_origin("https://example.invalid/source.git").is_ok());
}
#[test]
fn missing_or_nested_checkout_is_not_created_or_retargeted() -> Result<()> {
    let f = Fixture::new("sha1", false)?;
    let missing = f.root.join("missing");
    assert!(update(&missing, "live", &f.initial, &f.remote()).is_err());
    assert!(!missing.exists());
    fs::create_dir(f.checkout.join("subdir"))?;
    assert!(update(&f.checkout.join("subdir"), "live", &f.initial, &f.remote()).is_err());
    assert_eq!(f.head()?, f.initial);
    Ok(())
}
#[cfg(unix)]
#[test]
fn configured_hooks_are_never_executed() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new("sha1", false)?;
    put(
        &f.checkout,
        ".git/hooks/post-merge",
        "#!/bin/sh\nprintf ran > unexpected-hook-marker\n",
    )?;
    fs::set_permissions(
        f.checkout.join(".git/hooks/post-merge"),
        fs::Permissions::from_mode(0o755),
    )?;
    f.advance("data.txt", "next")?;
    assert_eq!(update(&f.checkout, "live", &f.initial, &f.remote())?.0, 0);
    assert!(!f.checkout.join("unexpected-hook-marker").exists());
    Ok(())
}

#[cfg(unix)]
#[test]
fn symbolic_git_directory_cannot_redirect_an_update() -> Result<()> {
    let f = Fixture::new("sha1", false)?;
    let original = f.checkout.join(".git");
    let target = f.root.join("foreign-git");
    fs::rename(&original, &target)?;
    std::os::unix::fs::symlink(&target, &original)?;
    let remote = f.remote();
    assert!(update(&f.checkout, "live", &f.initial, &remote).is_err());
    assert_eq!(remote.heads.get(), 0);
    assert!(!target.join("wow-source-update.lock").exists());
    Ok(())
}
