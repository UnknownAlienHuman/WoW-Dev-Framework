//! Internal measurement capacity. It cannot be supplied by wire or exported as
//! a Main/graph/replay capability. Other load entry points retain package64.
use super::{ProjectLoadPlan, budget, census::bounded_encoded_size};
use crate::{ProjectResult, disk::checkpoint, platform_source::AdmittedPlatformSource};
use std::sync::atomic::AtomicBool;

#[derive(Clone, Copy)]
pub(crate) struct PackageLoadCapacity {
    packages: usize,
    census: bool,
}
impl PackageLoadCapacity {
    pub(crate) const fn ordinary() -> Self {
        Self {
            packages: super::package_closure::MAX_PACKAGES,
            census: false,
        }
    }

    pub(crate) fn census(source: &AdmittedPlatformSource) -> ProjectResult<Self> {
        source.profile().validate()?;
        let packages = source
            .profile()
            .roots()
            .iter()
            .try_fold(0usize, |sum, root| {
                sum.checked_add(root.selected_tocs.len()).ok_or_else(budget)
            })?;
        Ok(Self {
            packages,
            census: true,
        })
    }

    pub(crate) const fn packages(self) -> usize {
        self.packages
    }
    pub(crate) const fn is_census(self) -> bool {
        self.census
    }
}

/// Preserve the previous worst-case retained syntax fanout of 64 ordinary
/// packages even when the exact measurement selection contains more packages.
/// The selected source count/bytes remain separately capped by the load owner.
#[derive(Default)]
pub(crate) struct CensusRetention {
    records: usize,
    elements: usize,
    attributes: usize,
    segments: usize,
    plan_json_bytes: u64,
}
impl CensusRetention {
    pub(crate) fn charge(
        &mut self,
        plan: &ProjectLoadPlan,
        stop: &AtomicBool,
    ) -> ProjectResult<()> {
        checkpoint(stop)?;
        let fanout = super::package_closure::MAX_PACKAGES;
        charge(
            &mut self.records,
            plan.records().len(),
            fanout * super::MAX_RECORDS,
        )?;
        for index in plan.xml_documents().values() {
            checkpoint(stop)?;
            charge(
                &mut self.elements,
                index.elements().len(),
                fanout * super::MAX_RECORDS,
            )?;
            charge(
                &mut self.attributes,
                index.attribute_count(),
                fanout * 65_536,
            )?;
            charge(
                &mut self.segments,
                index.map_segment_count(),
                fanout * 65_536,
            )?;
        }
        self.plan_json_bytes = self
            .plan_json_bytes
            .checked_add(bounded_encoded_size(plan, stop)?)
            .filter(|bytes| *bytes <= 64 * 1024 * 1024)
            .ok_or_else(budget)?;
        Ok(())
    }
}

fn charge(total: &mut usize, count: usize, limit: usize) -> ProjectResult<()> {
    *total = total
        .checked_add(count)
        .filter(|sum| *sum <= limit)
        .ok_or_else(budget)?;
    Ok(())
}
