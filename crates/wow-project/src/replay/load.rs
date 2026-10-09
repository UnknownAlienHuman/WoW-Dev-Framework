//! A selected load recipe and inert documents, never a serialized owner receipt.
use super::{ReplayFile, invalid};
use crate::load::{ProjectLoadPlan, TocLoadContext, read_retained_toc};
use crate::{ProjectInputFile, ProjectResult};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::atomic::AtomicBool};
use wow_core::{CanonicalResult, ContentDigest, ProfileIdentity};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReplayLoad {
    selected_toc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    context: Option<TocLoadContext>,
    documents: Vec<ReplayFile>,
    expected_plan_digest: ContentDigest<CanonicalResult>,
}

impl ReplayLoad {
    pub(super) fn capture(
        plan: &ProjectLoadPlan,
        files: &[ProjectInputFile],
    ) -> ProjectResult<Self> {
        plan.validate_main_files(files)?;
        let documents = plan
            .sources()
            .iter()
            .filter(|source| !source.path.ends_with(".lua"))
            .map(|source| {
                let text = plan.document_text(&source.path).ok_or_else(invalid)?;
                if text.len() as u64 != source.byte_length
                    || crate::identity::source_digest(text.as_bytes()) != source.content_digest
                {
                    return Err(invalid());
                }
                Ok(ReplayFile {
                    path: source.path.clone(),
                    text: text.into(),
                    fixture_ref: None,
                })
            })
            .collect::<ProjectResult<Vec<_>>>()?;
        Ok(Self {
            selected_toc: plan.selected_toc().into(),
            context: plan.load_context().cloned(),
            documents,
            expected_plan_digest: plan.digest(),
        })
    }

    pub(super) fn documents(&self) -> ProjectResult<&[ReplayFile]> {
        if self.documents.is_empty()
            || self
                .documents
                .windows(2)
                .any(|pair| pair[0].path >= pair[1].path)
            || self.documents.iter().any(|file| {
                file.fixture_ref.is_some()
                    || !(file.path.ends_with(".toc") || file.path.ends_with(".xml"))
            })
        {
            return Err(invalid());
        }
        Ok(&self.documents)
    }

    pub(super) fn rebuild(
        &self,
        files: &[ReplayFile],
        profile: &ProfileIdentity,
        stop: &AtomicBool,
    ) -> ProjectResult<ProjectLoadPlan> {
        let mut sources = BTreeMap::new();
        for file in files.iter().chain(self.documents()?) {
            crate::analyzer::checkpoint(stop)?;
            if sources
                .insert(file.path.as_str(), file.text.as_str())
                .is_some()
            {
                return Err(invalid());
            }
        }
        let (rebuilt_files, plan) = read_retained_toc(
            sources,
            &crate::disk::ProjectDiskFile::new(&self.selected_toc),
            profile,
            self.context.as_ref(),
            stop,
        )?
        .into_parts();
        if plan.digest() != self.expected_plan_digest
            || rebuilt_files.len() != files.len()
            || rebuilt_files.iter().zip(files).any(|(actual, expected)| {
                actual.relative_path().as_str() != expected.path
                    || actual.retained_text() != expected.text
            })
        {
            return Err(invalid());
        }
        Ok(plan)
    }
}
