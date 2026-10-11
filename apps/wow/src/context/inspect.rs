//! Closed inspect grammar and shared typed level/root extraction for build.
use super::{Command, set};
use wow_service::context::{
    ContextInspectLevel, ContextInspectRequest, ContextL0Profile, ContextMapCurrentGuard,
    ContextMapPublication, ContextMapReferenceGuard, ContextMapRoot, PRIMARY_L0_PROFILE_ID,
    PRIMARY_L1_PROFILE_ID, PRIMARY_STATE_ROOT_L0_PROFILE_ID,
};

#[derive(Default)]
pub(super) struct Options {
    level: Option<ContextInspectLevel>,
    l0_profile: Option<ContextL0Profile>,
    l1_profile: Option<()>,
}

impl Options {
    pub(super) fn accepts(option: &str) -> bool {
        matches!(option, "--level" | "--l0-profile-id" | "--l1-profile-id")
    }

    pub(super) fn option(
        &mut self,
        option: &str,
        value: &str,
        command: Command,
    ) -> Result<(), &'static str> {
        match option {
            "--level" => {
                let level = match value {
                    "l0" => ContextInspectLevel::L0,
                    "l1" => ContextInspectLevel::L1,
                    "both" => ContextInspectLevel::Both,
                    _ => {
                        return Err(match command {
                            Command::Build => "unsupported context build level",
                            _ => "unsupported context inspect level",
                        });
                    }
                };
                set(&mut self.level, level, command)
            }
            "--l0-profile-id" => {
                let profile = match value {
                    PRIMARY_L0_PROFILE_ID => ContextL0Profile::ContainersV1,
                    PRIMARY_STATE_ROOT_L0_PROFILE_ID => ContextL0Profile::StateRootV1,
                    _ => return Err("unsupported exact context L0 profile"),
                };
                set(&mut self.l0_profile, profile, command)
            }
            "--l1-profile-id" => {
                if value != PRIMARY_L1_PROFILE_ID {
                    return Err("unsupported exact context L1 profile");
                }
                set(&mut self.l1_profile, (), command)
            }
            _ => Err(match command {
                Command::Build => "unknown context build option",
                _ => "unknown context inspect option",
            }),
        }
    }

    pub(super) fn selection(
        self,
        root: Option<ContextMapRoot>,
        command: Command,
    ) -> Result<
        (
            ContextInspectLevel,
            ContextMapRoot,
            Option<ContextL0Profile>,
        ),
        &'static str,
    > {
        let level = self.level.ok_or(match command {
            Command::Build => "context build requires --level",
            _ => "context inspect requires --level",
        })?;
        let root = root.ok_or(match command {
            Command::Build => "context build requires an explicit --root",
            _ => "context inspect requires an explicit --root",
        })?;
        match level {
            ContextInspectLevel::L0 if self.l1_profile.is_some() => {
                return Err(match command {
                    Command::Build => "L1 profile conflicts with context build level l0",
                    _ => "L1 profile conflicts with context inspect level l0",
                });
            }
            ContextInspectLevel::L1 if self.l0_profile.is_some() => {
                return Err(match command {
                    Command::Build => "L0 profile conflicts with context build level l1",
                    _ => "L0 profile conflicts with context inspect level l1",
                });
            }
            _ => {}
        }
        Ok((level, root, self.l0_profile))
    }

    pub(super) fn request(
        self,
        store: &str,
        publication: ContextMapPublication,
        root: Option<ContextMapRoot>,
        current: Option<ContextMapCurrentGuard>,
        reference: Option<ContextMapReferenceGuard>,
    ) -> Result<ContextInspectRequest, &'static str> {
        let (level, root, l0_profile) = self.selection(root, Command::Inspect)?;
        // The service selects the fixed recipes and validates level/root compatibility.
        // No graph kind, owner existence or root selection is inferred here.
        let request =
            ContextInspectRequest::primary(store, publication, level, root, current, reference)
                .map_err(|_| "invalid context inspect request syntax")?;
        match l0_profile {
            Some(profile) => request
                .with_l0_profile(profile)
                .map_err(|_| "invalid context inspect request syntax"),
            None => Ok(request),
        }
    }
}
