//! Closed inspect grammar and shared typed level/root extraction for build.
use super::{Command, parse_root, set};
use wow_service::context::{
    ContextInspectLevel, ContextInspectRequest, ContextL0Profile, ContextL1Profile,
    ContextMapCurrentGuard, ContextMapPublication, ContextMapReferenceGuard, ContextMapRoot,
    PRIMARY_FRONTIER_EXPANSION_PROFILE_ID, PRIMARY_L0_PROFILE_ID, PRIMARY_L1_PROFILE_ID,
    PRIMARY_PROGRESSIVE_EXPANSION_PROFILE_ID, PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID,
    PRIMARY_STATE_DETAIL_L1_PROFILE_ID, PRIMARY_STATE_ROOT_L0_PROFILE_ID,
};

pub(super) struct Selection {
    pub(super) level: ContextInspectLevel,
    pub(super) root: ContextMapRoot,
    pub(super) l0_profile: Option<ContextL0Profile>,
    pub(super) l1_profile: Option<ContextL1Profile>,
    pub(super) frontier: Option<Vec<ContextMapRoot>>,
    pub(super) progressive: Option<Vec<Vec<ContextMapRoot>>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExpansionProfile {
    Frontier,
    Progressive,
}

#[derive(Default)]
pub(super) struct Options {
    level: Option<ContextInspectLevel>,
    l0_profile: Option<ContextL0Profile>,
    l1_profile: Option<ContextL1Profile>,
    expansion_profile: Option<ExpansionProfile>,
    frontier: Vec<ContextMapRoot>,
    frontier_rounds: Vec<Vec<ContextMapRoot>>,
}

impl Options {
    pub(super) fn accepts(option: &str) -> bool {
        matches!(option, "--level" | "--l0-profile-id" | "--l1-profile-id")
    }

    pub(super) fn accepts_expansion(option: &str) -> bool {
        matches!(
            option,
            "--expansion-profile-id" | "--frontier" | "--frontier-round"
        )
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
                let profile = match value {
                    PRIMARY_L1_PROFILE_ID => ContextL1Profile::EntityV1,
                    PRIMARY_STATE_DETAIL_L1_PROFILE_ID
                        if matches!(command, Command::Inspect | Command::Build) =>
                    {
                        ContextL1Profile::StateDetailV1
                    }
                    PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID
                        if matches!(command, Command::Inspect | Command::Build) =>
                    {
                        ContextL1Profile::ReferenceFactsV1
                    }
                    _ => return Err("unsupported exact context L1 profile"),
                };
                set(&mut self.l1_profile, profile, command)
            }
            "--expansion-profile-id" if matches!(command, Command::Inspect | Command::Build) => {
                let profile = match value {
                    PRIMARY_FRONTIER_EXPANSION_PROFILE_ID => ExpansionProfile::Frontier,
                    PRIMARY_PROGRESSIVE_EXPANSION_PROFILE_ID => ExpansionProfile::Progressive,
                    _ => return Err("unsupported exact context expansion profile"),
                };
                set(&mut self.expansion_profile, profile, command)
            }
            "--frontier" if matches!(command, Command::Inspect | Command::Build) => {
                // Repeatable frontier tokens still require distinct exact IDs.
                if self.frontier.len() >= 32 {
                    return Err(match command {
                        Command::Build => "context build frontier limit exceeded",
                        _ => "context inspect frontier limit exceeded",
                    });
                }
                let root = parse_root(value)?;
                if !matches!(&root, ContextMapRoot::GraphEntity(_)) {
                    return Err(match command {
                        Command::Build => "context build frontier requires graph-entity token",
                        _ => "context inspect frontier requires graph-entity token",
                    });
                }
                if self.frontier.contains(&root) {
                    return Err(match command {
                        Command::Build => "duplicate context build frontier",
                        _ => "duplicate context inspect frontier",
                    });
                }
                self.frontier.push(root);
                Ok(())
            }
            "--frontier-round" if matches!(command, Command::Inspect | Command::Build) => {
                if self.frontier_rounds.len() >= 4 {
                    return Err("context progressive round limit exceeded");
                }
                let mut round = Vec::new();
                for token in value.split(',') {
                    if round.len() >= 32 {
                        return Err("context progressive frontier batch limit exceeded");
                    }
                    let root = parse_root(token)?;
                    if !matches!(&root, ContextMapRoot::GraphEntity(_)) {
                        return Err("context progressive frontier requires graph-entity token");
                    }
                    if round.contains(&root)
                        || self
                            .frontier_rounds
                            .iter()
                            .flatten()
                            .any(|prior| prior == &root)
                    {
                        return Err("duplicate context progressive frontier");
                    }
                    round.push(root);
                }
                // Preserve batch order. The service owns normalization and native admission.
                self.frontier_rounds.push(round);
                Ok(())
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
    ) -> Result<Selection, &'static str> {
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
        if matches!(&self.l1_profile, Some(ContextL1Profile::StateDetailV1)) {
            if level != ContextInspectLevel::L1 || !matches!(&root, ContextMapRoot::GraphEntity(_))
            {
                return Err(match command {
                    Command::Build => {
                        "state-detail L1 requires context build l1 and graph-entity root"
                    }
                    _ => "state-detail L1 requires context inspect l1 and graph-entity root",
                });
            }
            if self.expansion_profile.is_some()
                || !self.frontier.is_empty()
                || !self.frontier_rounds.is_empty()
            {
                return Err(match command {
                    Command::Build => {
                        "state-detail L1 conflicts with context build frontier expansion"
                    }
                    _ => "state-detail L1 conflicts with context frontier expansion",
                });
            }
        }
        if matches!(&self.l1_profile, Some(ContextL1Profile::ReferenceFactsV1)) {
            if level != ContextInspectLevel::L1 || !matches!(&root, ContextMapRoot::GraphEntity(_))
            {
                return Err(match command {
                    Command::Build => {
                        "reference-facts L1 requires context build l1 and graph-entity root"
                    }
                    _ => "reference-facts L1 requires context inspect l1 and graph-entity root",
                });
            }
            if self.expansion_profile.is_some()
                || !self.frontier.is_empty()
                || !self.frontier_rounds.is_empty()
            {
                return Err(match command {
                    Command::Build => {
                        "reference-facts L1 conflicts with context build frontier expansion"
                    }
                    _ => "reference-facts L1 conflicts with context frontier expansion",
                });
            }
        }
        if self.expansion_profile == Some(ExpansionProfile::Progressive)
            || !self.frontier_rounds.is_empty()
        {
            if !self.frontier.is_empty()
                || self.expansion_profile == Some(ExpansionProfile::Frontier)
            {
                return Err("context expansion families cannot be mixed");
            }
            if self.expansion_profile != Some(ExpansionProfile::Progressive)
                || self.frontier_rounds.is_empty()
            {
                return Err("context progressive expansion requires profile and rounds together");
            }
            if level != ContextInspectLevel::L1
                || !matches!(&root, ContextMapRoot::GraphEntity(_))
                || !matches!(&self.l1_profile, None | Some(ContextL1Profile::EntityV1))
            {
                return Err(
                    "context progressive expansion requires l1 entity and graph-entity root",
                );
            }
            if self
                .frontier_rounds
                .iter()
                .flatten()
                .any(|frontier| frontier == &root)
            {
                return Err("context progressive frontier must exclude its initial root");
            }
            return Ok(Selection {
                level,
                root,
                l0_profile: self.l0_profile,
                l1_profile: self.l1_profile,
                frontier: None,
                progressive: Some(self.frontier_rounds),
            });
        }
        if self.expansion_profile.is_some() == self.frontier.is_empty() {
            return Err(match command {
                Command::Build => "context build expansion requires profile and frontier together",
                _ => "context inspect expansion requires profile and frontier together",
            });
        }
        let frontier = if self.expansion_profile.is_some() {
            if level != ContextInspectLevel::L1 || !matches!(&root, ContextMapRoot::GraphEntity(_))
            {
                return Err(match command {
                    Command::Build => "context build expansion requires l1 and graph-entity root",
                    _ => "context inspect expansion requires l1 and graph-entity root",
                });
            }
            if self.frontier.contains(&root) {
                return Err(match command {
                    Command::Build => "context build frontier must exclude its initial root",
                    _ => "context inspect frontier must exclude its initial root",
                });
            }
            Some(self.frontier)
        } else {
            None
        };
        Ok(Selection {
            level,
            root,
            l0_profile: self.l0_profile,
            l1_profile: self.l1_profile,
            frontier,
            progressive: None,
        })
    }

    pub(super) fn request(
        self,
        store: &str,
        publication: ContextMapPublication,
        root: Option<ContextMapRoot>,
        current: Option<ContextMapCurrentGuard>,
        reference: Option<ContextMapReferenceGuard>,
    ) -> Result<ContextInspectRequest, &'static str> {
        let Selection {
            level,
            root,
            l0_profile,
            l1_profile,
            frontier,
            progressive,
        } = self.selection(root, Command::Inspect)?;
        // The service selects the fixed recipes and validates level/root compatibility.
        // No graph kind, owner existence or root selection is inferred here.
        let request =
            ContextInspectRequest::primary(store, publication, level, root, current, reference)
                .map_err(|_| "invalid context inspect request syntax")?;
        let request = match l0_profile {
            Some(profile) => request
                .with_l0_profile(profile)
                .map_err(|_| "invalid context inspect request syntax")?,
            None => request,
        };
        let request = match l1_profile {
            Some(ContextL1Profile::StateDetailV1) => request
                .with_l1_profile(ContextL1Profile::StateDetailV1)
                .map_err(|_| "invalid context inspect request syntax")?,
            Some(ContextL1Profile::ReferenceFactsV1) => request
                .with_l1_profile(ContextL1Profile::ReferenceFactsV1)
                .map_err(|_| "invalid context inspect request syntax")?,
            // Entity is an optional assertion of the existing primary constructor.
            Some(ContextL1Profile::EntityV1) | None => request,
        };
        if let Some(rounds) = progressive {
            return request
                .with_progressive_expansion(rounds)
                .map_err(|_| "invalid context inspect progressive expansion syntax");
        }
        match frontier {
            // Service normalizes the finite frontier and validates native membership later.
            Some(frontier) => request
                .with_frontier_expansion(frontier)
                .map_err(|_| "invalid context inspect expansion syntax"),
            None => Ok(request),
        }
    }
}
