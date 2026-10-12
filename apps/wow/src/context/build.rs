//! Closed build-only grammar. Shared transport, level/root parsing and output stay in the parent.
use super::{Command, inspect::Selection, set};
use wow_service::context::{
    ContextBuildRequest, ContextL1Profile, ContextMapCurrentGuard, ContextMapPublication,
    ContextMapReferenceGuard, ContextRenderSelection, PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
    PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID, PRIMARY_CONTEXT_PACK_PROFILE_ID,
    PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE_ID,
    PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
    PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE_ID,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    Structural,
    Frontier,
}

#[derive(Default)]
pub(super) struct Options {
    context_profile: Option<Family>,
    json_renderer: Option<Family>,
    markdown_renderer: Option<Family>,
}

impl Options {
    pub(super) fn accepts(option: &str) -> bool {
        matches!(option, "--context-profile-id" | "--renderer-profile-id")
    }

    pub(super) fn option(&mut self, option: &str, value: &str) -> Result<(), &'static str> {
        match option {
            "--context-profile-id" => {
                let family = match value {
                    PRIMARY_CONTEXT_PACK_PROFILE_ID => Family::Structural,
                    PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE_ID => Family::Frontier,
                    _ => return Err("unsupported exact context pack profile"),
                };
                set(&mut self.context_profile, family, Command::Build)
            }
            "--renderer-profile-id" => match value {
                PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID => {
                    set(&mut self.json_renderer, Family::Structural, Command::Build)
                }
                PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID => set(
                    &mut self.markdown_renderer,
                    Family::Structural,
                    Command::Build,
                ),
                PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE_ID => {
                    set(&mut self.json_renderer, Family::Frontier, Command::Build)
                }
                PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID => set(
                    &mut self.markdown_renderer,
                    Family::Frontier,
                    Command::Build,
                ),
                _ => Err("unsupported exact context renderer profile"),
            },
            _ => Err("unknown context build option"),
        }
    }

    pub(super) fn request(
        self,
        store: &str,
        publication: ContextMapPublication,
        selection: Selection,
        current: Option<ContextMapCurrentGuard>,
        reference: Option<ContextMapReferenceGuard>,
    ) -> Result<ContextBuildRequest, &'static str> {
        let family = self
            .context_profile
            .ok_or("context build requires --context-profile-id")?;
        let Selection {
            level,
            root,
            l0_profile,
            l1_profile,
            frontier,
        } = selection;
        if (family == Family::Frontier) != frontier.is_some() {
            return Err("context build pack profile conflicts with expansion selection");
        }
        if self
            .json_renderer
            .is_some_and(|selected| selected != family)
            || self
                .markdown_renderer
                .is_some_and(|selected| selected != family)
        {
            return Err("context build renderer profile conflicts with context pack profile");
        }
        // The enum normalizes the reviewed JSON/Markdown pair independent of argv order.
        let render = match (
            self.json_renderer.is_some(),
            self.markdown_renderer.is_some(),
        ) {
            (false, false) => ContextRenderSelection::None,
            (true, false) => ContextRenderSelection::Json,
            (false, true) => ContextRenderSelection::Markdown,
            (true, true) => ContextRenderSelection::Both,
        };
        // The service owns fixed profiles, root compatibility and artifact production.
        let request = ContextBuildRequest::primary(
            store,
            publication,
            level,
            root,
            render,
            current,
            reference,
        )
        .map_err(|_| "invalid context build request syntax")?;
        let request = match l0_profile {
            Some(profile) => request
                .with_l0_profile(profile)
                .map_err(|_| "invalid context build request syntax")?,
            None => request,
        };
        let request = match l1_profile {
            Some(ContextL1Profile::StateDetailV1) => request
                .with_l1_profile(ContextL1Profile::StateDetailV1)
                .map_err(|_| "invalid context build request syntax")?,
            Some(ContextL1Profile::ReferenceRecordsV1) => request
                .with_l1_profile(ContextL1Profile::ReferenceRecordsV1)
                .map_err(|_| "invalid context build request syntax")?,
            // Entity remains an optional assertion of the existing primary constructor.
            Some(ContextL1Profile::EntityV1) | None => request,
        };
        match frontier {
            Some(frontier) => request
                .with_frontier_expansion(frontier)
                .map_err(|_| "invalid context build expansion syntax"),
            None => Ok(request),
        }
    }
}
