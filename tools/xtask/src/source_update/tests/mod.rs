mod guards;
mod races;
mod support;
use super::{remote::Remote, update};
use crate::{Result, git};
use std::fs;
use std::path::Path;
use support::{Fixture, Local, put};

#[test]
fn current_checkout_is_idempotent_without_fetch() -> Result<()> {
    let f = Fixture::new("sha1", false)?;
    let remote = f.remote();
    let before = git::isolated_text(&f.checkout, &["show-ref"])?;
    let (code, report) = update(&f.checkout, "live", &f.initial, &remote)?;
    assert_eq!(code, 0);
    assert_eq!(report["status"], "current");
    assert_eq!(remote.fetches.get(), 0);
    assert_eq!(before, git::isolated_text(&f.checkout, &["show-ref"])?);
    assert!(!f.lock_exists());
    Ok(())
}
#[test]
fn fast_forward_preserves_old_objects_and_other_refs_for_both_formats() -> Result<()> {
    for format in ["sha1", "sha256"] {
        let f = Fixture::new(format, false)?;
        git::isolated_text(&f.checkout, &["tag", "retained"])?;
        let remote_refs =
            git::isolated_text(&f.checkout, &["for-each-ref", "refs/remotes", "refs/tags"])?;
        let selected = f.advance("new.txt", "new source data\n")?;
        assert!(git::isolated_text(&f.checkout, &["cat-file", "-e", &selected]).is_err());
        let remote = f.remote();
        let (code, report) = update(&f.checkout, "live", &f.initial, &remote)?;
        assert_eq!(code, 0);
        assert_eq!(report["status"], "updated");
        assert_eq!(report["after_revision"], selected);
        assert_eq!(report["semantic_compatibility"], "not_evaluated");
        assert_eq!(f.head()?, selected);
        assert_eq!(remote.fetches.get(), 1);
        assert_eq!(
            remote_refs,
            git::isolated_text(&f.checkout, &["for-each-ref", "refs/remotes", "refs/tags"])?
        );
        assert_eq!(
            git::isolated_text(&f.checkout, &["show", &format!("{}:data.txt", f.initial)])?,
            "initial"
        );
        assert_eq!(
            fs::read_to_string(f.checkout.join("new.txt"))?,
            "new source data\n"
        );
        assert!(!f.checkout.join(".git/FETCH_HEAD").exists());
        assert!(!f.lock_exists());
        assert_eq!(
            update(&f.checkout, "live", &selected, &remote)?.1["status"],
            "current"
        );
    }
    Ok(())
}
#[test]
fn shallow_checkout_can_fast_forward_without_full_reclone() -> Result<()> {
    let f = Fixture::new("sha1", true)?;
    let selected = f.advance("data.txt", "next\n")?;
    assert_eq!(update(&f.checkout, "live", &f.initial, &f.remote())?.0, 0);
    assert_eq!(f.head()?, selected);
    Ok(())
}
#[test]
fn divergence_is_reported_without_merge_reset_stash_or_new_branch() -> Result<()> {
    let f = Fixture::new("sha1", false)?;
    put(&f.checkout, "local.txt", "my work")?;
    let before = support::commit(&f.checkout)?;
    f.advance("remote.txt", "remote work")?;
    let refs = git::isolated_text(&f.checkout, &["show-ref"])?;
    let (code, report) = update(&f.checkout, "live", &before, &f.remote())?;
    assert_eq!(code, 3);
    assert_eq!(report["status"], "not_fast_forward_or_incomplete_history");
    assert_eq!(f.head()?, before);
    assert_eq!(refs, git::isolated_text(&f.checkout, &["show-ref"])?);
    assert!(!f.checkout.join("remote.txt").exists());
    assert!(!f.lock_exists());
    Ok(())
}
#[test]
fn ignored_file_collision_retains_exact_reconciliation_lock() -> Result<()> {
    let f = Fixture::new("sha1", false)?;
    put(&f.checkout, "ignored", "private local bytes")?;
    put(&f.upstream, "ignored", "new tracked content")?;
    git::isolated_text(&f.upstream, &["add", "-f", "ignored"])?;
    let selected = support::commit(&f.upstream)?;
    let (code, report) = update(&f.checkout, "live", &f.initial, &f.remote())?;
    assert_eq!(code, 5);
    assert_eq!(report["status"], "reconciliation_required");
    assert_eq!(report["lock_retained"], true);
    assert_eq!(f.head()?, f.initial);
    assert_eq!(
        fs::read_to_string(f.checkout.join("ignored"))?,
        "private local bytes"
    );
    let journal = fs::read_to_string(f.checkout.join(".git/wow-source-update.lock"))?;
    assert!(journal.contains(&format!("expected_head={}", f.initial)));
    assert!(journal.contains(&format!("selected_revision={selected}")));
    assert!(update(&f.checkout, "live", &f.initial, &f.remote()).is_err());
    Ok(())
}
struct Unavailable<'a> {
    local: Local<'a>,
    head_fails: bool,
}
impl Remote for Unavailable<'_> {
    fn head(&self, root: &Path, origin: &str, branch: &str) -> Result<String> {
        if self.head_fails {
            Err("private-network-detail".into())
        } else {
            self.local.head(root, origin, branch)
        }
    }
    fn fetch(&self, _: &Path, _: &str, _: &str) -> Result<()> {
        Err("private-network-detail".into())
    }
}
#[test]
fn network_failures_never_claim_current_or_expose_provider_details() -> Result<()> {
    for head_fails in [true, false] {
        let f = Fixture::new("sha1", false)?;
        f.advance("data.txt", "next")?;
        let remote = Unavailable {
            local: f.remote(),
            head_fails,
        };
        let (code, report) = update(&f.checkout, "live", &f.initial, &remote)?;
        assert_eq!(code, 4);
        assert_eq!(f.head()?, f.initial);
        assert!(!report.to_string().contains("private-network-detail"));
        assert!(!report.to_string().contains("example.invalid"));
        assert!(!f.lock_exists());
    }
    Ok(())
}

#[test]
fn source_update_keeps_old_manifest_verifiable_and_marks_local_drift() -> Result<()> {
    let f = Fixture::new("sha256", false)?;
    let old = crate::manifest::build(&f.checkout, &f.initial, "synthetic-selector")?;
    let path = f.root.join("old-manifest.json");
    crate::manifest::write_new(&path, &old)?;
    let selected = f.advance("version.txt", "99.0.0.2")?;
    assert_eq!(update(&f.checkout, "live", &f.initial, &f.remote())?.0, 0);
    assert_eq!(crate::manifest::verify(&path, &f.checkout, None)?, 0);
    assert_eq!(
        crate::manifest::verify(&path, &f.checkout, Some("HEAD"))?,
        3
    );
    let current = crate::manifest::build(&f.checkout, &selected, "synthetic-selector")?;
    assert_ne!(old["manifest_sha256"], current["manifest_sha256"]);
    assert_eq!(old["source"]["version"], "99.0.0.1");
    assert_eq!(current["source"]["version"], "99.0.0.2");
    Ok(())
}

#[test]
fn branch_names_and_source_files_are_not_provider_specific() -> Result<()> {
    let f = Fixture::new("sha1", false)?;
    for root in [&f.upstream, &f.checkout] {
        git::isolated_text(root, &["branch", "-m", "resource-updates"])?;
    }
    let selected = f.advance("annotations/new-type.lua", "-- synthetic new resource")?;
    let (code, result) = update(&f.checkout, "resource-updates", &f.initial, &f.remote())?;
    assert_eq!(code, 0);
    assert_eq!(result["after_revision"], selected);
    assert!(f.checkout.join("annotations/new-type.lua").exists());
    Ok(())
}

struct InvalidHead(&'static str);
impl Remote for InvalidHead {
    fn head(&self, _: &Path, _: &str, _: &str) -> Result<String> {
        Ok(self.0.into())
    }
    fn fetch(&self, _: &Path, _: &str, _: &str) -> Result<()> {
        Err("fetch must never be called for invalid identity".into())
    }
}
#[test]
fn malformed_remote_identity_cannot_claim_verified_freshness() -> Result<()> {
    let f = Fixture::new("sha1", false)?;
    for value in [
        "main",
        "private-text",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "",
    ] {
        let (code, report) = update(&f.checkout, "live", &f.initial, &InvalidHead(value))?;
        assert_eq!(code, 4);
        assert_eq!(report["freshness"], "unverified_current");
        assert!(report["selected_remote_revision"].is_null());
        assert_eq!(f.head()?, f.initial);
        assert!(!f.lock_exists());
    }
    Ok(())
}
