use std::collections::BTreeMap;

use wow_core::*;

fn fixture_profile() -> ProfileIdentity {
    ProfileIdentity::fixture(
        ProfileId::parse_exact("retail-12-1-0-69497-fixture").unwrap(),
        SchemaVersion::parse("1").unwrap(),
    )
    .unwrap()
    .with_fixture_platform(
        FlavorId::parse("retail").unwrap(),
        120_100,
        "12.1.0.69497",
        "027d26c3406d3de2cbd2b1f67d468fe033a1bcd4",
        ContentDigest::from_bytes(b"fixture-blizzard-ui"),
    )
    .unwrap()
}

fn context(project: &str) -> GenerationContext {
    GenerationContext::new(
        fixture_profile(),
        ReferenceGenerationId::parse("reference:e0-fixture").unwrap(),
        Some(ProjectGenerationId::parse(project).unwrap()),
    )
    .unwrap()
    .with_schema(
        ProducerId::parse("wow-core").unwrap(),
        SchemaVersion::parse("1").unwrap(),
    )
    .unwrap()
    .with_tool(
        ProducerId::parse("wow-core").unwrap(),
        ToolVersion::parse("0.1.0").unwrap(),
    )
    .unwrap()
}

fn reference_source(context: &GenerationContext, path: &str) -> SourceHandle {
    build_source_handle(SourceHandleInput {
        owner: SourceOwner::ReferencePack {
            profile: context.profile().id().clone(),
            reference_generation: context.reference().clone(),
        },
        path: path.to_owned(),
        byte_span: Some(ByteSpan::new(10, 40).unwrap()),
        line_span: Some(
            LineSpan::new(
                LinePosition::new(2, 0).unwrap(),
                LinePosition::new(4, 8).unwrap(),
            )
            .unwrap(),
        ),
        digest: ContentDigest::from_bytes(path.as_bytes()),
        symbol: Some(EntityKey::parse("C_Test.ValidApi").unwrap()),
    })
    .unwrap()
}

fn complete_coverage(context: &GenerationContext, capability: &str) -> CoverageRecord {
    CoverageRecord::new(CoverageRecordInput {
        partition: CoveragePartitionId::parse("apidoc:fixture").unwrap(),
        capability: CapabilityId::parse(capability).unwrap(),
        status: CoverageStatus::Complete,
        missing_inputs: Vec::new(),
        missing_capabilities: Vec::new(),
        producer: ProducerId::parse("wow-reference").unwrap(),
        generation: context.clone(),
        conflicts: Vec::new(),
        reasons: Vec::new(),
    })
    .unwrap()
}

fn platform_evidence(context: &GenerationContext, path: &str) -> EvidenceRecord {
    EvidenceRecord::new(
        ProvenanceClass::PlatformSource,
        EvidenceLevel::Proven,
        Some(reference_source(context, path)),
        ProducerId::parse("wow-reference").unwrap(),
        ToolVersion::parse("0.1.0").unwrap(),
        context.clone(),
        None,
    )
    .unwrap()
}

fn finding(context: &GenerationContext, symbol: &str, path: &str) -> Finding {
    let mut arguments = MessageArguments::new();
    arguments.insert("symbol", symbol).unwrap();
    Finding::new(FindingInput {
        rule_id: RuleId::parse("wow.api.exists").unwrap(),
        rule_version: ToolVersion::parse("0.1.0").unwrap(),
        severity: FindingSeverity::Error,
        policy: FindingPolicy::Advisory,
        message_key: MessageKey::parse("wow.api.missing").unwrap(),
        message_arguments: arguments,
        primary_source: Some(reference_source(context, path)),
        related_sources: Vec::new(),
        evidence: vec![platform_evidence(context, path)],
        required_capabilities: vec![CapabilityId::parse("apidoc.functions.complete").unwrap()],
        coverage: vec![complete_coverage(context, "apidoc.functions.complete")],
        context: context.clone(),
        root_cause: Some(RootCauseKey::parse("api:missing").unwrap()),
        remediation: Some(RemediationClass::PlanOnly),
    })
    .unwrap()
}

#[test]
fn identifiers_are_canonical_and_exact() {
    assert!(ProfileId::parse_exact("retail-12-1-0-69497").is_ok());
    assert!(ProfileId::parse_exact("current").is_err());
    assert!(RuleId::parse("WoW.Api.Exists").is_err());
    assert_eq!(
        RepositoryId::parse("UnknownAlienHuman/WoW-Dev-Framework")
            .unwrap()
            .as_str(),
        "unknownalienhuman/wow-dev-framework"
    );

    let digest = ContentDigest::from_bytes(b"same logical input");
    assert_eq!(ContentDigest::parse(digest.to_string()).unwrap(), digest);
}

#[test]
fn profile_deserialization_rejects_contradictory_source_identity() {
    let invalid = serde_json::json!({
        "id": "retail-12-1-fixture",
        "kind": "fixture",
        "flavor": "retail",
        "interface": 120100,
        "client_build": "12.1.0.69497",
        "source_revision": "027d26c3406d",
        "source_digest": null,
        "builder_version": null,
        "schema_version": "1",
        "correction_set_digest": null
    });
    assert!(serde_json::from_value::<ProfileIdentity>(invalid).is_err());
}

#[test]
fn source_handles_normalize_paths_and_reject_escape() {
    assert_eq!(
        normalize_source_path("Interface\\AddOns//Example/./Core.lua").unwrap(),
        "Interface/AddOns/Example/Core.lua"
    );
    assert!(normalize_source_path("../outside.lua").is_err());
    assert!(normalize_source_path("C:\\outside.lua").is_err());

    let context = context("project:e0-a");
    let handle = reference_source(&context, "Interface/AddOns/Example/Core.lua");
    assert!(verify_source_handle_content(&handle, handle.digest()).is_ok());
    assert!(verify_source_handle_content(&handle, ContentDigest::from_bytes(b"other")).is_err());

    let mut serialized = serde_json::to_value(&handle).unwrap();
    serialized["id"] = serde_json::Value::String(format!("source:{}", "0".repeat(64)));
    assert!(serde_json::from_value::<SourceHandle>(serialized).is_err());
}

#[test]
fn generation_contexts_merge_only_when_compatible() {
    let profile = fixture_profile();
    let reference = ReferenceGenerationId::parse("reference:e0-fixture").unwrap();
    let without_project = GenerationContext::new(profile.clone(), reference.clone(), None).unwrap();
    let with_project = GenerationContext::new(
        profile.clone(),
        reference.clone(),
        Some(ProjectGenerationId::parse("project:e0-a").unwrap()),
    )
    .unwrap();
    assert_eq!(
        merge_generation_context(&without_project, &with_project)
            .unwrap()
            .project()
            .unwrap()
            .as_str(),
        "project:e0-a"
    );

    let other_project = GenerationContext::new(
        profile,
        reference,
        Some(ProjectGenerationId::parse("project:e0-b").unwrap()),
    )
    .unwrap();
    assert!(merge_generation_context(&with_project, &other_project).is_err());
    assert!(require_same_generation(&with_project, &other_project).is_err());
}

#[test]
fn coverage_controls_negative_authority_conservatively() {
    let context = context("project:e0-a");
    let complete = complete_coverage(&context, "apidoc.functions.complete");
    let complete_summary = combine_coverage(&[complete.clone()]).unwrap();
    assert_eq!(
        evaluate_negative_authority(&complete_summary).status,
        NegativeAuthorityStatus::Authoritative
    );

    let partial = CoverageRecord::new(CoverageRecordInput {
        partition: CoveragePartitionId::parse("apidoc:fixture").unwrap(),
        capability: CapabilityId::parse("apidoc.functions.complete").unwrap(),
        status: CoverageStatus::Partial,
        missing_inputs: vec!["Unsupported.lua".to_owned()],
        missing_capabilities: Vec::new(),
        producer: ProducerId::parse("wow-reference").unwrap(),
        generation: context.clone(),
        conflicts: Vec::new(),
        reasons: vec!["unsupported declarative construct".to_owned()],
    })
    .unwrap();
    let partial_summary = combine_coverage(&[partial.clone()]).unwrap();
    assert_eq!(
        evaluate_negative_authority(&partial_summary).status,
        NegativeAuthorityStatus::Partial
    );

    let conflicting = combine_coverage(&[complete, partial]).unwrap();
    assert!(conflicting.conflicted);
    assert_eq!(
        evaluate_negative_authority(&conflicting).status,
        NegativeAuthorityStatus::Conflict
    );
}

#[test]
fn candidate_evidence_cannot_be_promoted_to_proven() {
    let context = context("project:e0-a");
    assert!(EvidenceRecord::new(
        ProvenanceClass::SemanticCandidate,
        EvidenceLevel::Proven,
        None,
        ProducerId::parse("codebase-memory").unwrap(),
        ToolVersion::parse("0.1.0").unwrap(),
        context,
        None,
    )
    .is_err());
    assert_eq!(
        combine_evidence_levels([EvidenceLevel::Proven, EvidenceLevel::Candidate]),
        Some(EvidenceLevel::Candidate)
    );
}

#[test]
fn exact_edit_requires_proof_and_complete_coverage() {
    let context = context("project:e0-a");
    let mut arguments = MessageArguments::new();
    arguments.insert("symbol", "C_Test.Missing").unwrap();

    let missing_coverage = Finding::new(FindingInput {
        rule_id: RuleId::parse("wow.api.exists").unwrap(),
        rule_version: ToolVersion::parse("0.1.0").unwrap(),
        severity: FindingSeverity::Error,
        policy: FindingPolicy::Advisory,
        message_key: MessageKey::parse("wow.api.missing").unwrap(),
        message_arguments: arguments.clone(),
        primary_source: None,
        related_sources: Vec::new(),
        evidence: vec![platform_evidence(&context, "Api.lua")],
        required_capabilities: Vec::new(),
        coverage: Vec::new(),
        context: context.clone(),
        root_cause: None,
        remediation: Some(RemediationClass::ExactEdit),
    });
    assert!(missing_coverage.is_err());

    let proven = Finding::new(FindingInput {
        rule_id: RuleId::parse("wow.api.exists").unwrap(),
        rule_version: ToolVersion::parse("0.1.0").unwrap(),
        severity: FindingSeverity::Error,
        policy: FindingPolicy::Advisory,
        message_key: MessageKey::parse("wow.api.missing").unwrap(),
        message_arguments: arguments,
        primary_source: None,
        related_sources: Vec::new(),
        evidence: vec![platform_evidence(&context, "Api.lua")],
        required_capabilities: vec![CapabilityId::parse("apidoc.functions.complete").unwrap()],
        coverage: vec![complete_coverage(&context, "apidoc.functions.complete")],
        context,
        root_cause: None,
        remediation: Some(RemediationClass::ExactEdit),
    });
    assert!(proven.is_ok());
}

#[test]
fn result_envelopes_are_deterministic_and_round_trip() {
    let context = context("project:e0-a");
    let finding_a = finding(&context, "C_Test.MissingA", "A.lua");
    let finding_b = finding(&context, "C_Test.MissingB", "B.lua");

    let capability = CapabilityState {
        capability: CapabilityId::parse("apidoc.functions.complete").unwrap(),
        status: CoverageStatus::Complete,
        partitions: vec![CoveragePartitionId::parse("apidoc:fixture").unwrap()],
        reasons: Vec::new(),
    };
    let not_evaluated = NotEvaluatedRecord {
        subject: OperationId::parse("wow.secret.local_operation").unwrap(),
        missing_capabilities: vec![CapabilityId::parse("emmy.control_flow").unwrap()],
        partitions: vec![CoveragePartitionId::parse("project:file:secret.lua").unwrap()],
        reasons: vec!["control-flow facts are unavailable".to_owned()],
        context: context.clone(),
    };
    let mut warning_arguments = MessageArguments::new();
    warning_arguments.insert("lane", "secret").unwrap();
    let warning = ResultWarning {
        message_key: MessageKey::parse("operation.not_evaluated").unwrap(),
        arguments: warning_arguments,
    };

    let envelope_a = ResultEnvelope::new(ResultEnvelopeInput {
        schema_version: SchemaVersion::parse("1").unwrap(),
        operation: OperationId::parse("wow.check").unwrap(),
        context: context.clone(),
        capabilities: vec![capability.clone()],
        findings: vec![finding_b.clone(), finding_a.clone()],
        not_evaluated: vec![not_evaluated.clone()],
        warnings: vec![warning.clone()],
        truncation: TruncationStatus::Complete,
    })
    .unwrap();
    let envelope_b = ResultEnvelope::new(ResultEnvelopeInput {
        schema_version: SchemaVersion::parse("1").unwrap(),
        operation: OperationId::parse("wow.check").unwrap(),
        context,
        capabilities: vec![capability],
        findings: vec![finding_a, finding_b],
        not_evaluated: vec![not_evaluated],
        warnings: vec![warning],
        truncation: TruncationStatus::Complete,
    })
    .unwrap();

    assert_eq!(envelope_a.canonical_json().unwrap(), envelope_b.canonical_json().unwrap());
    assert_eq!(envelope_a.canonical_digest().unwrap(), envelope_b.canonical_digest().unwrap());

    let json = envelope_a.canonical_json().unwrap();
    let decoded: ResultEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, envelope_a);
    assert_eq!(decoded.canonical_json().unwrap(), json);
}

#[test]
fn envelope_rejects_cross_generation_findings() {
    let context_a = context("project:e0-a");
    let context_b = context("project:e0-b");
    let result = ResultEnvelope::new(ResultEnvelopeInput {
        schema_version: SchemaVersion::parse("1").unwrap(),
        operation: OperationId::parse("wow.check").unwrap(),
        context: context_a,
        capabilities: Vec::new(),
        findings: vec![finding(&context_b, "C_Test.Missing", "Mismatch.lua")],
        not_evaluated: Vec::new(),
        warnings: Vec::new(),
        truncation: TruncationStatus::Complete,
    });
    assert!(result.is_err());
}

#[test]
fn message_arguments_use_sorted_map_serialization() {
    let mut first = BTreeMap::new();
    first.insert("zeta".to_owned(), "2".to_owned());
    first.insert("alpha".to_owned(), "1".to_owned());
    let arguments = MessageArguments::from_map(first).unwrap();
    assert_eq!(
        serde_json::to_string(&arguments).unwrap(),
        r#"{"alpha":"1","zeta":"2"}"#
    );
}
