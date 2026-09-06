use super::super::{remote::Remote, update};
use super::support::{Fixture, Local, put};
use crate::{Result, git};
use std::path::Path;
struct Changed<'a> {
    local: Local<'a>,
    field: &'a str,
    on_head: bool,
}
impl Changed<'_> {
    fn change(&self, root: &Path) -> Result<()> {
        match self.field {
            "file" => put(root, "data.txt", "changed while fetching")?,
            "origin" => {
                git::isolated_text(
                    root,
                    &[
                        "remote",
                        "set-url",
                        "origin",
                        "https://other.invalid/source.git",
                    ],
                )?;
            }
            "branch" => {
                git::isolated_text(root, &["checkout", "-b", "another-operation"])?;
            }
            "config" => {
                git::isolated_text(
                    root,
                    &["config", "user.name", "changed by concurrent operation"],
                )?;
            }
            _ => return Err("invalid test case".into()),
        }
        Ok(())
    }
}
impl Remote for Changed<'_> {
    fn head(&self, root: &Path, origin: &str, branch: &str) -> Result<String> {
        let head = self.local.head(root, origin, branch)?;
        if self.on_head {
            self.change(root)?;
        }
        Ok(head)
    }
    fn fetch(&self, root: &Path, origin: &str, revision: &str) -> Result<()> {
        self.local.fetch(root, origin, revision)?;
        if !self.on_head {
            self.change(root)?;
        }
        Ok(())
    }
}
#[test]
fn changes_during_head_observation_or_fetch_refuse_checkout_effect() -> Result<()> {
    for field in ["file", "origin", "branch", "config"] {
        for on_head in [true, false] {
            let f = Fixture::new("sha1", false)?;
            f.advance("new.txt", "not to be checked out")?;
            let remote = Changed {
                local: f.remote(),
                field,
                on_head,
            };
            assert!(
                update(&f.checkout, "live", &f.initial, &remote).is_err(),
                "{field}/{on_head}"
            );
            assert_eq!(f.head()?, f.initial);
            assert!(!f.checkout.join("new.txt").exists());
            assert!(!f.lock_exists());
        }
    }
    Ok(())
}
struct Moving<'a> {
    fixture: &'a Fixture,
    local: Local<'a>,
}
impl Remote for Moving<'_> {
    fn head(&self, root: &Path, origin: &str, branch: &str) -> Result<String> {
        let selected = self.local.head(root, origin, branch)?;
        self.fixture.advance("later.txt", "next operation only")?;
        Ok(selected)
    }
    fn fetch(&self, root: &Path, origin: &str, revision: &str) -> Result<()> {
        self.local.fetch(root, origin, revision)
    }
}
#[test]
fn moving_remote_cannot_mix_selected_generations() -> Result<()> {
    let f = Fixture::new("sha1", false)?;
    let selected = f.advance("first.txt", "selected")?;
    let moving = Moving {
        fixture: &f,
        local: f.remote(),
    };
    let (code, report) = update(&f.checkout, "live", &f.initial, &moving)?;
    assert_eq!(code, 0);
    assert_eq!(f.head()?, selected);
    assert_eq!(report["selected_remote_revision"], selected);
    assert!(!f.checkout.join("later.txt").exists());
    assert_eq!(update(&f.checkout, "live", &selected, &f.remote())?.0, 0);
    assert!(f.checkout.join("later.txt").exists());
    Ok(())
}
