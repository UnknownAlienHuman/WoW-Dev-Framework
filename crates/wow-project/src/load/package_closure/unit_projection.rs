//! Exact per-package native load-unit identities before universe ordering.
use super::*;

/// One projected load unit before the builder binds universe-level fields.
///
/// `ordinal`, `package`, `order_group`, `reachability` and `phase` remain the
/// native builder's responsibility and are intentionally not here.
#[derive(Debug, Clone, Serialize)]
pub(super) struct PackageUnitDraft {
    pub(super) unit_digest: ContentDigest<CanonicalResult>,
    pub(super) source_ordinal: u64,
    pub(super) document: String,
    pub(super) target: String,
    pub(super) kind: LoadRecordKind,
    pub(super) bootstrap: bool,
}

/// Capture each included native load-unit identity before universe ordering.
pub(super) fn project_units(
    package: &str,
    plan: &ProjectLoadPlan,
    stop: &AtomicBool,
) -> ProjectResult<Vec<PackageUnitDraft>> {
    checkpoint(stop)?;
    let mut units = Vec::new();
    for record in plan.records().iter().filter(|record| {
        record.selection == LoadSelection::Included
            && matches!(
                record.kind,
                LoadRecordKind::LuaFile | LoadRecordKind::XmlFile
            )
            && record.target.is_some()
    }) {
        checkpoint(stop)?;
        let target = record
            .target
            .as_ref()
            .ok_or_else(|| invalid("load unit has no target"))?;
        #[derive(Serialize)]
        struct UnitIdentity<'a> {
            profile: &'static str,
            package: &'a str,
            plan: ContentDigest<CanonicalResult>,
            source_ordinal: u64,
            document: &'a str,
            target: &'a str,
            kind: LoadRecordKind,
            bootstrap: bool,
        }
        let unit_digest = crate::identity::canonical_digest(
            "wow-project/package-load-unit/1",
            &UnitIdentity {
                profile: PACKAGE_LOAD_PROFILE,
                package,
                plan: plan.digest(),
                source_ordinal: record.ordinal,
                document: &record.document,
                target,
                kind: record.kind,
                bootstrap: record.bootstrap,
            },
            ProjectPhase::Inventory,
        )?;
        units.push(PackageUnitDraft {
            unit_digest,
            source_ordinal: record.ordinal,
            document: record.document.clone(),
            target: target.clone(),
            kind: record.kind,
            bootstrap: record.bootstrap,
        });
    }
    Ok(units)
}
