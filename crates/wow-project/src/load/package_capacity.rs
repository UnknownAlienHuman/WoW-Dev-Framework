//! Internal load capacity. Census and explicitly selected native retention are
//! separate purposes; neither constructs an owner. Ordinary loads keep package64.
use super::{ProjectLoadPlan, budget, budget_at, census::bounded_encoded_size};
use crate::platform_source::PlatformPackageRetentionLimits;
use crate::{ProjectResult, disk::checkpoint, platform_source::AdmittedPlatformSource};
use std::sync::atomic::AtomicBool;

#[derive(Clone, Copy)]
pub(crate) struct PackageLoadCapacity {
    packages: usize,
    purpose: Purpose,
}
#[derive(Clone, Copy)]
enum Purpose {
    Main,
    Census,
    Retained(PlatformPackageRetentionLimits),
}
impl PackageLoadCapacity {
    pub(crate) const fn ordinary() -> Self {
        Self {
            packages: super::package_closure::MAX_PACKAGES,
            purpose: Purpose::Main,
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
            purpose: Purpose::Census,
        })
    }

    pub(crate) const fn packages(self) -> usize {
        self.packages
    }
    pub(crate) const fn is_census(self) -> bool {
        matches!(self.purpose, Purpose::Census)
    }
    pub(crate) fn retained(limits: PlatformPackageRetentionLimits) -> ProjectResult<Self> {
        limits.validate()?;
        Ok(Self {
            packages: limits.max_packages(),
            purpose: Purpose::Retained(limits),
        })
    }
    pub(crate) const fn retention_limits(self) -> Option<PlatformPackageRetentionLimits> {
        match self.purpose {
            Purpose::Retained(limits) => Some(limits),
            _ => None,
        }
    }
}

/// Preserve the previous finite syntax-work fanout even when completed syntax
/// indexes are dropped between packages. Each encoded plan remains bounded.
#[derive(Default)]
pub(crate) struct CensusRetention {
    records: usize,
    elements: usize,
    attributes: usize,
    segments: usize,
    retained_plan_bytes: u64,
}
impl CensusRetention {
    pub(crate) fn charge_retained(
        &mut self,
        plan: &ProjectLoadPlan,
        limits: PlatformPackageRetentionLimits,
        stop: &AtomicBool,
    ) -> ProjectResult<()> {
        let encoded = self.charge(plan, stop)?;
        self.retained_plan_bytes = self
            .retained_plan_bytes
            .checked_add(encoded)
            .filter(|bytes| *bytes <= limits.max_total_plan_encoding_bytes())
            .ok_or_else(|| budget_at("native package retained-plan encoding limit exceeded"))?;
        Ok(())
    }
    pub(crate) fn charge(
        &mut self,
        plan: &ProjectLoadPlan,
        stop: &AtomicBool,
    ) -> ProjectResult<u64> {
        checkpoint(stop)?;
        let fanout = super::package_closure::MAX_PACKAGES;
        charge(
            &mut self.records,
            plan.records().len(),
            fanout * super::MAX_RECORDS,
            "package census processed-record count limit exceeded",
        )?;
        for index in plan.xml_documents().values() {
            checkpoint(stop)?;
            charge(
                &mut self.elements,
                index.elements().len(),
                fanout * super::MAX_RECORDS,
                "package census processed-element count limit exceeded",
            )?;
            charge(
                &mut self.attributes,
                index.attribute_count(),
                fanout * 65_536,
                "package census processed-attribute count limit exceeded",
            )?;
            charge(
                &mut self.segments,
                index.map_segment_count(),
                fanout * 65_536,
                "package census processed-map-segment count limit exceeded",
            )?;
        }
        bounded_encoded_size(plan, stop)
    }
}

fn charge(
    total: &mut usize,
    count: usize,
    limit: usize,
    message: &'static str,
) -> ProjectResult<()> {
    *total = total
        .checked_add(count)
        .filter(|sum| *sum <= limit)
        .ok_or_else(|| budget_at(message))?;
    Ok(())
}
