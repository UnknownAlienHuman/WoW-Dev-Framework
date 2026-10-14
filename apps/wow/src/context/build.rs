//! Closed build-only grammar. Shared transport, level/root parsing and output stay in the parent.
use super::{Command, inspect::Selection, set};
use wow_service::context::{
    CONTEXT_CONTINUATION_RETENTION_PROFILE_ID, ContextBuildRequest, ContextL1Profile,
    ContextMapCurrentGuard, ContextMapPublication, ContextMapReferenceGuard,
    ContextRenderSelection, ContextSourceExcerptPolicy, ContextSourceExcerptSelection,
    PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID, PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
    PRIMARY_CONTEXT_PACK_PROFILE_ID, PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE_ID,
    PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
    PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE_ID, PRIMARY_PROGRESSIVE_CONTEXT_JSON_RENDERER_PROFILE_ID,
    PRIMARY_PROGRESSIVE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
    PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE_ID, PRIMARY_SOURCE_CONTEXT_JSON_RENDERER_PROFILE_ID,
    PRIMARY_SOURCE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID, PRIMARY_SOURCE_CONTEXT_PACK_PROFILE_ID,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Family {
    Structural,
    Frontier,
    Progressive,
    SourceExcerpt,
}

#[derive(Default)]
pub(super) struct Options {
    context_profile: Option<Family>,
    json_renderer: Option<Family>,
    markdown_renderer: Option<Family>,
    source_handle: Option<String>,
    source_policy: Option<ContextSourceExcerptPolicy>,
    continuation_retention_profile: Option<&'static str>,
}

impl Options {
    pub(super) fn accepts(option: &str) -> bool {
        matches!(
            option,
            "--context-profile-id"
                | "--renderer-profile-id"
                | "--source-excerpt-handle"
                | "--source-policy"
                | "--continuation-retention-profile-id"
        )
    }

    pub(super) fn has_source_excerpt(&self) -> bool {
        self.source_handle.is_some() || self.source_policy.is_some()
    }

    pub(super) fn option(&mut self, option: &str, value: &str) -> Result<(), &'static str> {
        match option {
            "--context-profile-id" => {
                let family = match value {
                    PRIMARY_CONTEXT_PACK_PROFILE_ID => Family::Structural,
                    PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE_ID => Family::Frontier,
                    PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE_ID => Family::Progressive,
                    PRIMARY_SOURCE_CONTEXT_PACK_PROFILE_ID => Family::SourceExcerpt,
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
                PRIMARY_PROGRESSIVE_CONTEXT_JSON_RENDERER_PROFILE_ID => {
                    set(&mut self.json_renderer, Family::Progressive, Command::Build)
                }
                PRIMARY_PROGRESSIVE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID => set(
                    &mut self.markdown_renderer,
                    Family::Progressive,
                    Command::Build,
                ),
                PRIMARY_SOURCE_CONTEXT_JSON_RENDERER_PROFILE_ID => set(
                    &mut self.json_renderer,
                    Family::SourceExcerpt,
                    Command::Build,
                ),
                PRIMARY_SOURCE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID => set(
                    &mut self.markdown_renderer,
                    Family::SourceExcerpt,
                    Command::Build,
                ),
                _ => Err("unsupported exact context renderer profile"),
            },
            "--source-excerpt-handle" => {
                set(&mut self.source_handle, value.to_owned(), Command::Build)
            }
            "--source-policy" => {
                if value != "local-project-source" {
                    return Err("unsupported source excerpt policy");
                }
                set(
                    &mut self.source_policy,
                    ContextSourceExcerptPolicy::LocalProjectSource,
                    Command::Build,
                )
            }
            "--continuation-retention-profile-id" => {
                if value != CONTEXT_CONTINUATION_RETENTION_PROFILE_ID {
                    return Err("unsupported exact context continuation retention profile");
                }
                set(
                    &mut self.continuation_retention_profile,
                    CONTEXT_CONTINUATION_RETENTION_PROFILE_ID,
                    Command::Build,
                )
            }
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
        let continuation_retention_profile = self.continuation_retention_profile;
        if continuation_retention_profile.is_some() && self.has_source_excerpt() {
            return Err("source excerpts cannot be combined with continuation retention");
        }
        let family = self
            .context_profile
            .ok_or("context build requires --context-profile-id")?;
        let Selection {
            level,
            root,
            l0_profile,
            l1_profile,
            frontier,
            progressive,
        } = selection;
        let source = match (self.source_handle, self.source_policy) {
            (None, None) => None,
            (Some(handle), Some(policy)) => Some((
                ContextSourceExcerptSelection::new(
                    handle
                        .parse()
                        .map_err(|_| "invalid exact source handle ID")?,
                ),
                policy,
            )),
            _ => {
                return Err("source excerpts require both an exact handle and local source policy");
            }
        };
        let selected_family = match (frontier.is_some(), progressive.is_some(), source.is_some()) {
            (false, false, false) => Family::Structural,
            (true, false, false) => Family::Frontier,
            (false, true, false) => Family::Progressive,
            (false, false, true) => Family::SourceExcerpt,
            _ => return Err("context expansion and source excerpt families cannot be mixed"),
        };
        if family != selected_family {
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
            Some(ContextL1Profile::ReferenceFactsV1) => request
                .with_l1_profile(ContextL1Profile::ReferenceFactsV1)
                .map_err(|_| "invalid context build request syntax")?,
            // Entity remains an optional assertion of the existing primary constructor.
            Some(ContextL1Profile::EntityV1) | None => request,
        };
        if let Some((selection, policy)) = source {
            return request
                .with_source_excerpt(selection, policy)
                .map_err(|_| "invalid context source excerpt selection");
        }
        let request = if let Some(rounds) = progressive {
            request
                .with_progressive_expansion(rounds)
                .map_err(|_| "invalid context build progressive expansion syntax")?
        } else {
            match frontier {
                Some(frontier) => request
                    .with_frontier_expansion(frontier)
                    .map_err(|_| "invalid context build expansion syntax")?,
                None => request,
            }
        };
        match continuation_retention_profile {
            Some(profile) => request
                .with_continuation_retention_profile(profile)
                .map_err(|_| "continuation retention requires at least two progressive rounds"),
            None => Ok(request),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wow_service::context::{ContextInspectLevel, ContextMapRoot};

    #[test]
    fn source_excerpt_and_continuation_retention_are_rejected_explicitly()
    -> Result<(), &'static str> {
        let options = Options {
            source_handle: Some("unparsed-source-handle".into()),
            continuation_retention_profile: Some(CONTEXT_CONTINUATION_RETENTION_PROFILE_ID),
            ..Options::default()
        };
        let selection = Selection {
            level: ContextInspectLevel::L1,
            root: ContextMapRoot::graph_entity(
                "graph-node:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            )
            .map_err(|_| "fixture root was invalid")?,
            l0_profile: None,
            l1_profile: None,
            frontier: None,
            progressive: None,
        };

        assert!(matches!(
            options.request(
                "fixture-store",
                ContextMapPublication::Current,
                selection,
                None,
                None,
            ),
            Err("source excerpts cannot be combined with continuation retention")
        ));
        Ok(())
    }
}
