//! Closed inspect-only grammar. Common transport and root decoding stay in the parent.
use super::{Command, set};
use wow_service::context::{
    ContextInspectLevel, ContextInspectRequest, ContextMapCurrentGuard, ContextMapPublication,
    ContextMapReferenceGuard, ContextMapRoot, PRIMARY_L0_PROFILE_ID, PRIMARY_L1_PROFILE_ID,
};

#[derive(Default)]
pub(super) struct Options {
    level: Option<ContextInspectLevel>,
    l0_profile: Option<()>,
    l1_profile: Option<()>,
}

impl Options {
    pub(super) fn accepts(option: &str) -> bool {
        matches!(option, "--level" | "--l0-profile-id" | "--l1-profile-id")
    }

    pub(super) fn option(&mut self, option: &str, value: &str) -> Result<(), &'static str> {
        match option {
            "--level" => {
                let level = match value {
                    "l0" => ContextInspectLevel::L0,
                    "l1" => ContextInspectLevel::L1,
                    "both" => ContextInspectLevel::Both,
                    _ => return Err("unsupported context inspect level"),
                };
                set(&mut self.level, level, Command::Inspect)
            }
            "--l0-profile-id" => {
                if value != PRIMARY_L0_PROFILE_ID {
                    return Err("unsupported exact context L0 profile");
                }
                set(&mut self.l0_profile, (), Command::Inspect)
            }
            "--l1-profile-id" => {
                if value != PRIMARY_L1_PROFILE_ID {
                    return Err("unsupported exact context L1 profile");
                }
                set(&mut self.l1_profile, (), Command::Inspect)
            }
            _ => Err("unknown context inspect option"),
        }
    }

    pub(super) fn request(
        self,
        store: &str,
        publication: ContextMapPublication,
        root: Option<ContextMapRoot>,
        current: Option<ContextMapCurrentGuard>,
        reference: Option<ContextMapReferenceGuard>,
    ) -> Result<ContextInspectRequest, &'static str> {
        let level = self.level.ok_or("context inspect requires --level")?;
        let root = root.ok_or("context inspect requires an explicit --root")?;
        match level {
            ContextInspectLevel::L0 if self.l1_profile.is_some() => {
                return Err("L1 profile conflicts with context inspect level l0");
            }
            ContextInspectLevel::L1 if self.l0_profile.is_some() => {
                return Err("L0 profile conflicts with context inspect level l1");
            }
            _ => {}
        }
        // The service selects the fixed recipes and validates level/root compatibility.
        // No graph kind, owner existence or container selection is inferred here.
        ContextInspectRequest::primary(store, publication, level, root, current, reference)
            .map_err(|_| "invalid context inspect request syntax")
    }
}
