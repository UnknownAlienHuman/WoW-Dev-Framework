//! Exact source descriptor keys. Overlap is never interpreted as source identity.
use super::{GeneratedLocation, LookupError, check_cancelled};
use crate::native::SourceLink;
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

pub(super) type Descriptors<'a> = BTreeMap<(usize, usize), Vec<GeneratedLocation<'a>>>;
type Files<'a> = BTreeMap<&'a str, Descriptors<'a>>;

#[derive(Default)]
pub(super) struct Reverse<'a> {
    blizzard: Files<'a>,
    aliases: Files<'a>,
}

impl<'a> Reverse<'a> {
    pub(super) fn insert(&mut self, location: GeneratedLocation<'a>) -> Result<(), LookupError> {
        let source = location.source;
        let files = match source.scope {
            None => &mut self.blizzard,
            Some("annotation_alias_catalog") => &mut self.aliases,
            _ => return Err(LookupError::UnsupportedProfile),
        };
        files
            .entry(source.path.as_str())
            .or_default()
            .entry((source.span.start, source.span.end))
            .or_default()
            .push(location);
        Ok(())
    }

    pub(super) fn get(&self, source: &SourceLink) -> Result<&[GeneratedLocation<'a>], LookupError> {
        Ok(self
            .descriptors(source.scope, &source.path)?
            .and_then(|descriptors| descriptors.get(&(source.span.start, source.span.end)))
            .map(Vec::as_slice)
            .unwrap_or(&[]))
    }

    pub(super) fn descriptors(
        &self,
        scope: Option<&str>,
        path: &str,
    ) -> Result<Option<&Descriptors<'a>>, LookupError> {
        let files = match scope {
            None => &self.blizzard,
            Some("annotation_alias_catalog") => &self.aliases,
            _ => return Err(LookupError::UnsupportedProfile),
        };
        Ok(files.get(path))
    }

    pub(super) fn order(&mut self, cancelled: &AtomicBool) -> Result<(), LookupError> {
        for files in [&mut self.blizzard, &mut self.aliases] {
            for descriptors in files.values_mut() {
                for locations in descriptors.values_mut() {
                    check_cancelled(cancelled)?;
                    locations.sort_by_key(|location| {
                        (
                            location.path,
                            location.generated.start,
                            location.generated.end,
                            location.granularity,
                        )
                    });
                }
            }
        }
        check_cancelled(cancelled)
    }
}
