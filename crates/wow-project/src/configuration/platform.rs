//! Private capability retention for an owner-bound platform configuration.

use std::{fmt, sync::Arc};

use wow_core::{ProfileIdentity, ReferenceGenerationId};

use crate::{
    ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult,
    load::{ProjectPackageLoadPlan, ProjectPackageMainPlan},
    platform_source::{PlatformPackageBinding, PlatformPackageSpecialization},
};

#[derive(Clone)]
pub(super) struct RetainedPlatformPackages(Arc<PlatformPackageSpecialization>);

impl RetainedPlatformPackages {
    pub(super) const fn new(owner: Arc<PlatformPackageSpecialization>) -> Self {
        Self(owner)
    }

    pub(super) fn owner(&self) -> &PlatformPackageSpecialization {
        &self.0
    }

    pub(super) fn binding(&self) -> &PlatformPackageBinding {
        self.0.binding()
    }

    pub(super) fn validate(
        &self,
        profile: &ProfileIdentity,
        reference_generation: ReferenceGenerationId,
        load: Option<&ProjectPackageLoadPlan>,
        main: Option<&ProjectPackageMainPlan>,
    ) -> ProjectResult<()> {
        let owner = self.owner();
        let target = owner.binding().target();
        if &target.reference_profile != profile
            || target.reference_generation != reference_generation
            || load != Some(owner.load_plan())
            || main != Some(owner.main_plan())
        {
            return Err(invalid());
        }
        owner
            .binding()
            .validate(owner.source(), owner.load_plan(), owner.main_plan())?;
        owner.main_plan().validate_main_files(owner.files())
    }
}

impl PartialEq for RetainedPlatformPackages {
    fn eq(&self, other: &Self) -> bool {
        self.binding() == other.binding()
            && self.owner().load_plan() == other.owner().load_plan()
            && self.owner().main_plan() == other.owner().main_plan()
            && self.owner().files() == other.owner().files()
    }
}
impl Eq for RetainedPlatformPackages {}

impl fmt::Debug for RetainedPlatformPackages {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RetainedPlatformPackages")
            .field("binding_digest", &self.binding().binding_digest())
            .finish_non_exhaustive()
    }
}

pub(super) fn invalid() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::InvalidConfiguration,
        ProjectPhase::Configuration,
        "platform configuration requires matching admitted source, target and native package owners",
    )
}
