//! Closed build-only grammar. Shared transport, level/root parsing and output stay in the parent.
use super::{Command, set};
use wow_service::context::{
    ContextBuildRequest, ContextInspectLevel, ContextMapCurrentGuard, ContextMapPublication,
    ContextMapReferenceGuard, ContextMapRoot, ContextRenderSelection,
    PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID, PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
    PRIMARY_CONTEXT_PACK_PROFILE_ID,
};

#[derive(Default)]
pub(super) struct Options {
    context_profile: Option<()>,
    json_renderer: Option<()>,
    markdown_renderer: Option<()>,
}

impl Options {
    pub(super) fn accepts(option: &str) -> bool {
        matches!(option, "--context-profile-id" | "--renderer-profile-id")
    }

    pub(super) fn option(&mut self, option: &str, value: &str) -> Result<(), &'static str> {
        match option {
            "--context-profile-id" => {
                if value != PRIMARY_CONTEXT_PACK_PROFILE_ID {
                    return Err("unsupported exact context pack profile");
                }
                set(&mut self.context_profile, (), Command::Build)
            }
            "--renderer-profile-id" => {
                if value == PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID {
                    set(&mut self.json_renderer, (), Command::Build)
                } else if value == PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID {
                    set(&mut self.markdown_renderer, (), Command::Build)
                } else {
                    Err("unsupported exact context renderer profile")
                }
            }
            _ => Err("unknown context build option"),
        }
    }

    pub(super) fn request(
        self,
        store: &str,
        publication: ContextMapPublication,
        selection: (ContextInspectLevel, ContextMapRoot),
        current: Option<ContextMapCurrentGuard>,
        reference: Option<ContextMapReferenceGuard>,
    ) -> Result<ContextBuildRequest, &'static str> {
        self.context_profile
            .ok_or("context build requires --context-profile-id")?;
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
        let (level, root) = selection;
        // The service owns fixed profiles, root compatibility and artifact production.
        ContextBuildRequest::primary(store, publication, level, root, render, current, reference)
            .map_err(|_| "invalid context build request syntax")
    }
}
