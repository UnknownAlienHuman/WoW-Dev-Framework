//! Useful checks recovered from the retired E0 prototype, expressed through the
//! current contract rather than restoring its incompatible IDs and envelopes.
use std::error::Error;

use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use wow_core::{
    ClaimScope, ContentDigest, CoreErrorCode, CoreResult, CoverageRecord, CoverageStatus,
    EvidenceConfidence, EvidenceRecord, Finding, FindingDraft, GenerationContext,
    GenerationContextBuilder, MergeMode, NegativeAuthorityOutcome, NegativeAuthorityReason,
    ProjectGenerationId, ProvenanceClass, Remediation, RemediationClass, RolloutPolicy, Severity,
    SourceContent, SourceHandle, TruncationState, combine_coverage, evaluate_negative_authority,
    verify_source_handle_content,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn fixture<T: DeserializeOwned>(field: &str) -> Result<T> {
    let value: Value = serde_json::from_str(include_str!("../examples/e0-findings-result.json"))?;
    Ok(serde_json::from_value(
        value.get(field).ok_or("missing fixture field")?.clone(),
    )?)
}

fn context(project: Option<ProjectGenerationId>) -> Result<GenerationContext> {
    let base: GenerationContext = fixture("context")?;
    let builder =
        GenerationContextBuilder::new(base.profile_identity().clone(), base.reference_generation())
            .schema_versions(base.schema_versions().to_vec())
            .producer_versions(base.producer_versions().to_vec())
            .external_generations(base.external_generations().to_vec());
    Ok(match project {
        Some(id) => builder.project_generation(id).build()?,
        None => builder.build()?,
    })
}

fn code<T>(result: CoreResult<T>) -> Result<CoreErrorCode> {
    match result {
        Err(error) => Ok(error.code()),
        Ok(_) => Err("expected a contract rejection".into()),
    }
}

#[test]
fn generation_optional_extension_requires_explicit_mode() -> Result<()> {
    let with: GenerationContext = fixture("context")?;
    let without = context(None)?;
    assert!(without.merge(&with, MergeMode::Strict).is_err());
    assert_eq!(
        without.merge(&with, MergeMode::ExtendMissingOptional)?,
        with
    );
    assert_eq!(
        with.merge(&without, MergeMode::ExtendMissingOptional)?,
        with
    );
    assert!(without.require_same_generation(&with).is_err());
    Ok(())
}

#[test]
fn distinct_project_generations_cannot_be_merged_in_any_mode() -> Result<()> {
    let left: GenerationContext = fixture("context")?;
    let right = context(Some(
        format!("generation:project:sha256:{}", "ab".repeat(32)).parse()?,
    ))?;
    for mode in [
        MergeMode::Strict,
        MergeMode::ExtendMissingOptional,
        MergeMode::ExternalUnion,
    ] {
        assert_eq!(
            code(left.merge(&right, mode))?,
            CoreErrorCode::GenerationMismatch
        );
    }
    assert_eq!(
        code(left.require_same_generation(&right))?,
        CoreErrorCode::GenerationMismatch
    );
    Ok(())
}

#[test]
fn partial_coverage_denies_absence_and_duplicate_partitions_are_rejected() -> Result<()> {
    let ctx: GenerationContext = fixture("context")?;
    let records: Vec<CoverageRecord> = fixture("coverage_records")?;
    let complete = records.first().ok_or("empty coverage fixture")?;
    assert_eq!(complete.status(), CoverageStatus::Complete);
    let partial = CoverageRecord::new(
        ctx.context_id(),
        complete.capability_id().clone(),
        complete.partition_id().clone(),
        CoverageStatus::Partial,
        complete.producer_id().clone(),
        complete.producer_version().clone(),
        vec!["fixture:missing".into()],
        None,
        vec![],
        vec![],
    )?;
    let summarize = |records: &[CoverageRecord]| {
        combine_coverage(
            ctx.context_id(),
            complete.capability_id().clone(),
            complete.producer_id().clone(),
            complete.producer_version().clone(),
            records,
        )
    };
    let good = summarize(std::slice::from_ref(complete))?;
    assert_eq!(
        evaluate_negative_authority(
            true,
            true,
            &[good],
            &[],
            vec![],
            None,
            &TruncationState::NotTruncated
        )
        .outcome(),
        NegativeAuthorityOutcome::AuthoritativeAbsent
    );
    let incomplete = summarize(std::slice::from_ref(&partial))?;
    let decision = evaluate_negative_authority(
        true,
        true,
        &[incomplete],
        &[],
        vec![],
        None,
        &TruncationState::NotTruncated,
    );
    assert_eq!(
        decision.outcome(),
        NegativeAuthorityOutcome::NotAuthoritative
    );
    assert!(
        decision
            .reasons()
            .contains(&NegativeAuthorityReason::PartitionPartial)
    );
    assert_eq!(
        code(summarize(&[complete.clone(), partial]))?,
        CoreErrorCode::DuplicateCoverageRecord
    );
    Ok(())
}

fn draft(ctx: &GenerationContext, source: &SourceHandle) -> Result<FindingDraft> {
    Ok(FindingDraft::new(
        ctx.context_id(),
        "fixture.recovered".parse()?,
        "0.1.0".parse()?,
        "fixture.recovered.finding".parse()?,
        Severity::Warning,
        RolloutPolicy::Advisory,
        source.handle_id(),
        CoverageStatus::Complete,
    ))
}

#[test]
fn finding_context_guard_survives_binding_and_deserialization() -> Result<()> {
    let ctx: GenerationContext = fixture("context")?;
    let other = context(None)?;
    let sources: Vec<SourceHandle> = fixture("source_handles")?;
    let source = sources.first().ok_or("empty source fixture")?;
    assert_eq!(
        code(draft(&ctx, source)?.bind(other.context_id(), &sources, &[]))?,
        CoreErrorCode::FindingContextMismatch
    );
    let finding = draft(&ctx, source)?.bind(ctx.context_id(), &sources, &[])?;
    let decoded: Finding = serde_json::from_slice(&serde_json::to_vec(&finding)?)?;
    decoded.validate(ctx.context_id(), &sources, &[])?;
    assert_eq!(
        code(decoded.validate(other.context_id(), &sources, &[]))?,
        CoreErrorCode::FindingContextMismatch
    );
    Ok(())
}

#[test]
fn candidate_cannot_be_promoted_or_authorize_an_exact_edit() -> Result<()> {
    let ctx: GenerationContext = fixture("context")?;
    let sources: Vec<SourceHandle> = fixture("source_handles")?;
    let source = sources.first().ok_or("empty source fixture")?;
    let evidence = |confidence| {
        EvidenceRecord::new(
            ctx.context_id(),
            ProvenanceClass::SemanticCandidate,
            confidence,
            ClaimScope::CandidateRelation,
            "fixture.history".parse()?,
            "0.1.0".parse()?,
            vec![],
            vec![],
            vec![],
        )
    };
    assert_eq!(
        code(evidence(EvidenceConfidence::Proven))?,
        CoreErrorCode::EvidenceAuthorityViolation
    );
    let candidate = evidence(EvidenceConfidence::Candidate)?;
    let unbound = draft(&ctx, source)?.evidence_ids(vec![candidate.evidence_id()]);
    let recipe = Remediation::new(
        RemediationClass::ExactEdit,
        Some("fixture.recipe".parse()?),
        None,
    )?;
    assert_eq!(
        code(unbound.clone().remediation(recipe).bind(
            ctx.context_id(),
            &sources,
            std::slice::from_ref(&candidate)
        ))?,
        CoreErrorCode::RemediationAuthorityViolation
    );
    let plan = unbound
        .remediation(Remediation::new(RemediationClass::PlanOnly, None, None)?)
        .bind(ctx.context_id(), &sources, std::slice::from_ref(&candidate))?;
    let mut wire = serde_json::to_value(plan)?;
    wire["remediation"] = json!({"class":"exact_edit", "recipe_id":"fixture.recipe"});
    let decoded: Finding = serde_json::from_value(wire)?;
    assert_eq!(
        code(decoded.validate(ctx.context_id(), &sources, &[candidate]))?,
        CoreErrorCode::RemediationAuthorityViolation
    );
    Ok(())
}

#[test]
fn source_content_and_supplied_handle_id_are_independently_checked() -> Result<()> {
    let sources: Vec<SourceHandle> = fixture("source_handles")?;
    let source = sources.first().ok_or("empty source fixture")?;
    verify_source_handle_content(source, source.content_digest())?;
    assert!(
        verify_source_handle_content(source, &ContentDigest::<SourceContent>::from_bytes([0; 32]))
            .is_err()
    );
    let mut wire = serde_json::to_value(source)?;
    wire["handle_id"] = json!(format!("handle:sha256:{}", "00".repeat(32)));
    let decoded: SourceHandle = serde_json::from_value(wire)?;
    assert!(decoded.validate().is_err());
    Ok(())
}
