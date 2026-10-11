use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};

use wow_core::{
    ContentDigest, ProfileIdentity, ProfileIdentityBuilder, ProfileKind, SchemaVersionEntry,
    SourceKind, SourceLogicalSnapshot,
};

use super::check_counts;
use crate::disk::ProjectDiskFile;
use crate::graph::{SourceGraphBudget, SourceGraphXmlCountBudget};
use crate::load::xml_references::{
    XmlReferenceIssueKind, XmlReferenceKind, XmlReferenceResolution,
};
use crate::load::{LoadRecordKind, ProjectLoadPlan, read_retained_package_toc};
use crate::{ProjectErrorCode, ProjectPhase, ProjectResult};

type FixtureResult<T> = Result<T, Box<dyn Error>>;

fn fixture_profile() -> FixtureResult<ProfileIdentity> {
    Ok(ProfileIdentityBuilder::new(
        "profile:fixture:xml-count-retail-120100".parse()?,
        ProfileKind::Fixture,
        "retail",
        120_100,
        SourceKind::SyntheticFixture,
        "fixture:xml-counts-v1",
        ContentDigest::<SourceLogicalSnapshot>::from_bytes([7_u8; 32]),
    )
    .schema_versions(vec![SchemaVersionEntry::new(
        "schema:wow:xml-count-fixture".parse()?,
        "1.0.0".parse()?,
    )])
    .fixture_scope("native XML graph count preflight only")
    .build()?)
}

fn native_plan(
    package: &str,
    xml: &str,
    repeated: bool,
    profile: &ProfileIdentity,
) -> FixtureResult<ProjectLoadPlan> {
    let toc_path = format!("{package}/Fixture.toc");
    let xml_path = format!("{package}/frames.xml");
    let toc = if repeated {
        "## Interface: 120100\nframes.xml\nframes.xml\n"
    } else {
        "## Interface: 120100\nframes.xml\n"
    };
    let sources = BTreeMap::from([(toc_path.as_str(), toc), (xml_path.as_str(), xml)]);
    let (files, plan) = read_retained_package_toc(
        sources,
        &ProjectDiskFile::new(toc_path.as_str()),
        profile,
        None,
        &AtomicBool::new(false),
    )?
    .into_parts();
    assert!(files.is_empty());
    assert_eq!(plan.xml_documents().len(), 1);
    assert_eq!(
        plan.records()
            .iter()
            .filter(|record| record.kind == LoadRecordKind::XmlFile)
            .count(),
        if repeated { 2 } else { 1 },
    );
    Ok(plan)
}

fn declaration_xml(count: usize) -> FixtureResult<String> {
    let mut xml = String::from("<Ui>");
    for index in 0..count {
        write!(xml, r#"<Frame name="F{index:05}"/>"#)?;
    }
    xml.push_str("</Ui>");
    Ok(xml)
}

fn inheritance_xml() -> FixtureResult<String> {
    let names = (0..64)
        .map(|index| format!("Missing{index}"))
        .collect::<Vec<_>>()
        .join(",");
    let mut xml = String::from("<Ui>");
    for index in 0..128 {
        write!(xml, r#"<Frame name="I{index}" inherits="{names}""#)?;
        if index == 0 {
            xml.push_str(r#" parent="ParentMissing""#);
        }
        xml.push_str("/>");
    }
    xml.push_str("</Ui>");
    Ok(xml)
}

fn assert_count_refusal(
    result: ProjectResult<()>,
    declarations: u64,
    inheritance: u64,
    limits: (u64, u64),
) -> FixtureResult<()> {
    let error = result
        .err()
        .ok_or("XML count preflight unexpectedly succeeded")?;
    assert_eq!(error.code(), ProjectErrorCode::SourceBudgetExceeded);
    assert_eq!(error.phase(), ProjectPhase::View);
    assert_eq!(
        error.message(),
        format!(
            "source graph XML counts exceed their bounded profile: declarations {declarations}/{}, inheritance references {inheritance}/{}",
            limits.0, limits.1,
        ),
    );
    Ok(())
}

#[test]
fn native_xml_count_preflight_preserves_exact_boundaries() -> FixtureResult<()> {
    let profile = fixture_profile()?;
    let stop = AtomicBool::new(false);
    let metadata_only = SourceGraphBudget::new(4 * 1024 * 1024)?;
    assert_eq!(metadata_only.xml_counts(), None);
    let counts = SourceGraphXmlCountBudget::new(32_768, 8192)?;
    assert_eq!(counts.max_declarations(), 32_768);
    assert_eq!(counts.max_inheritance_references(), 8192);
    let selected = metadata_only.with_xml_counts(counts).xml_counts();
    assert_eq!(selected, Some(counts));

    // One genuine extra plan supplies +1 without rebuilding the large plans.
    let extra = native_plan("Extra", &declaration_xml(1)?, false, &profile)?;
    assert_eq!(extra.xml_references().declarations().len(), 1);
    {
        let xml = declaration_xml(2048)?;
        let alpha = native_plan("Alpha", &xml, false, &profile)?;
        let beta = native_plan("Beta", &xml, false, &profile)?;
        assert_eq!(alpha.xml_references().declarations().len(), 2048);
        assert_eq!(beta.xml_references().declarations().len(), 2048);
        // Equal names/content across distinct plans still count in both scopes.
        check_counts([&alpha, &beta], metadata_only.xml_counts(), &stop)?;
        assert_count_refusal(
            check_counts([&alpha, &beta, &extra], None, &stop),
            4097,
            0,
            (4096, 8192),
        )?;
        check_counts([&alpha, &beta, &extra], selected, &stop)?;
    }
    {
        // Each package stays below its own element/expanded-record ceilings.
        let xml = declaration_xml(16_384)?;
        let alpha = native_plan("Alpha", &xml, false, &profile)?;
        let beta = native_plan("Beta", &xml, false, &profile)?;
        assert_eq!(alpha.xml_references().declarations().len(), 16_384);
        assert_eq!(beta.xml_references().declarations().len(), 16_384);
        check_counts([&alpha, &beta], selected, &stop)?;
        assert_count_refusal(
            check_counts([&alpha, &beta, &extra], selected, &stop),
            32_769,
            0,
            (32_768, 8192),
        )?;

        // Cancellation wins on a nonempty, otherwise overlimit scope.
        stop.store(true, Ordering::Relaxed);
        let error = check_counts([&alpha, &beta, &extra], selected, &stop)
            .err()
            .ok_or("cancelled XML count preflight unexpectedly succeeded")?;
        assert_eq!(error.code(), ProjectErrorCode::AnalysisCancelled);
        assert_eq!(error.phase(), ProjectPhase::Analyzer);
        stop.store(false, Ordering::Relaxed);
        check_counts([&alpha, &beta], selected, &stop)?;
    }
    {
        let plan = native_plan("Inheritance", &inheritance_xml()?, false, &profile)?;
        let report = plan.xml_references();
        assert_eq!(report.declarations().len(), 128);
        assert_eq!(report.references().len(), 8193);
        assert_eq!(
            report
                .references()
                .iter()
                .filter(|r| r.kind == XmlReferenceKind::Inherits)
                .count(),
            8192,
        );
        assert_eq!(
            report
                .references()
                .iter()
                .filter(|r| r.kind == XmlReferenceKind::Parent)
                .count(),
            1,
        );
        assert!(
            report
                .references()
                .iter()
                .all(|r| { matches!(r.resolution, XmlReferenceResolution::NotInCapturedScope) })
        );
        check_counts([&plan], None, &stop)?;
        check_counts([&plan], selected, &stop)?;
        let extra = native_plan(
            "InheritanceExtra",
            r#"<Ui><Frame name="Extra" inherits="AnotherMissing"/></Ui>"#,
            false,
            &profile,
        )?;
        assert_eq!(extra.xml_references().declarations().len(), 1);
        assert_eq!(extra.xml_references().references().len(), 1);
        assert_count_refusal(
            check_counts([&plan, &extra], None, &stop),
            129,
            8193,
            (4096, 8192),
        )?;
        assert_count_refusal(
            check_counts([&plan, &extra], selected, &stop),
            129,
            8193,
            (32_768, 8192),
        )?;
    }
    {
        let xml = concat!(
            r#"<Ui><Frame name="Same"/><Frame name="Same"/>"#,
            r#"<Frame name="Consumer" inherits="Same,Same,Missing" parent="ParentMissing"/></Ui>"#,
        );
        let plan = native_plan("Duplicates", xml, true, &profile)?;
        let report = plan.xml_references();
        assert_eq!(report.declarations().len(), 3);
        assert_eq!(report.names().get("Same").map(Vec::len), Some(2));
        assert!(
            report
                .declarations()
                .values()
                .all(|site| site.load_ordinals.len() == 2)
        );
        assert_eq!(report.references().len(), 4);
        assert_eq!(
            report
                .references()
                .iter()
                .filter(|r| {
                    r.kind == XmlReferenceKind::Inherits
                        && matches!(r.resolution, XmlReferenceResolution::AmbiguousName { .. })
                })
                .count(),
            2,
        );
        assert!(
            report
                .issues()
                .iter()
                .any(|issue| issue.kind == XmlReferenceIssueKind::DuplicateInheritance)
        );
        // Count duplicate Inherits occurrences, exclude Parent, do not double
        // the retained declarations/references for the repeated XML load.
        check_counts([&plan], Some(SourceGraphXmlCountBudget::new(3, 3)?), &stop)?;
        assert_count_refusal(
            check_counts([&plan], Some(SourceGraphXmlCountBudget::new(3, 2)?), &stop),
            3,
            3,
            (3, 2),
        )?;
    }
    Ok(())
}
