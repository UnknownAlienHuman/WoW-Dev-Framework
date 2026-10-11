//! Derived exact-location lookup. It never deduplicates source observations.
use super::*;
use crate::global_access::SourceGlobalAccess;

type Spans = BTreeMap<SourceSpan, Vec<usize>>;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(super) struct AccessLocationIndex {
    files: BTreeMap<String, BTreeMap<String, Spans>>,
}

impl AccessLocationIndex {
    pub(super) fn build(
        accesses: &[SourceGlobalAccess],
        limit: u64,
        stop: &AtomicBool,
    ) -> EmmyMemberCallResult<Self> {
        if accesses.len() as u128 > u128::from(limit) {
            return Err(budget());
        }
        let mut index = Self::default();
        for (position, access) in accesses.iter().enumerate() {
            checkpoint(stop)?;
            index
                .files
                .entry(access.path().to_owned())
                .or_default()
                .entry(access.function_id().to_owned())
                .or_default()
                .entry(access.span())
                .or_default()
                .push(position);
        }
        Ok(index)
    }

    pub(super) fn at<'a>(
        &'a self,
        accesses: &'a [SourceGlobalAccess],
        path: &str,
        function: &str,
        span: SourceSpan,
    ) -> impl Iterator<Item = &'a SourceGlobalAccess> {
        self.files
            .get(path)
            .and_then(|functions| functions.get(function))
            .and_then(|spans| spans.get(&span))
            .into_iter()
            .flatten()
            .filter_map(|position| accesses.get(*position))
    }

    // Every position must occur exactly once in ascending insertion order under
    // its actual immutable location. This also catches stale private clones.
    pub(super) fn validate(&self, accesses: &[SourceGlobalAccess]) -> EmmyMemberCallResult<()> {
        let mut count = 0usize;
        for (path, functions) in &self.files {
            for (function, spans) in functions {
                for (span, positions) in spans {
                    if positions.is_empty() || positions.windows(2).any(|p| p[0] >= p[1]) {
                        return Err(invalid());
                    }
                    for position in positions {
                        let access = accesses.get(*position).ok_or_else(invalid)?;
                        if access.path() != path
                            || access.function_id() != function
                            || access.span() != *span
                        {
                            return Err(invalid());
                        }
                        count = count.checked_add(1).ok_or_else(budget)?;
                    }
                }
            }
        }
        if count != accesses.len() {
            return Err(invalid());
        }
        Ok(())
    }
}
