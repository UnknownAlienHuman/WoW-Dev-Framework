use std::error::Error;
use std::io;

use wow_reference::{
    CoverageStatus, LookupResult, LookupUnknownReason, ReferenceConflict, ReferencePartition,
    ReferenceRecord, ReferenceRecordKind, ReferenceView, RestrictionFacet, RestrictionState,
};

fn api_record(key: &str, payload: &str) -> Result<ReferenceRecord, Box<dyn Error>> {
    Ok(ReferenceRecord::new(
        key,
        ReferenceRecordKind::Api,
        payload,
        vec!["source:warcraft-wiki".to_owned(), "source:blizzard-ui".to_owned()],
        vec![RestrictionFacet::new(
            "combat-lockdown",
            RestrictionState::Restricted,
            vec!["evidence:secure-handlers".to_owned()],
        )?],
    )?)
}

fn require_found<'a>(
    result: LookupResult<'a>,
    message: &'static str,
) -> Result<&'a ReferenceRecord, Box<dyn Error>> {
    match result {
        LookupResult::Found(record) => Ok(record),
        _ => Err(io::Error::other(message).into()),
    }
}

#[test]
fn exact_lookup_and_complete_negative_authority_are_distinct() -> Result<(), Box<dyn Error>> {
    let partition = ReferencePartition::new(
        "api:retail",
        CoverageStatus::Complete,
        vec![api_record("C_Spell.GetSpellInfo", "(spell_id)->spell_info")?],
    )?;
    let view = ReferenceView::new("generation:reference:retail-120100", vec![partition], vec![])?;

    let found = require_found(
        view.lookup("api:retail", "C_Spell.GetSpellInfo"),
        "exact record must be found",
    )?;
    assert_eq!(found.kind(), ReferenceRecordKind::Api);
    assert!(view.is_authoritatively_absent("api:retail", "C_Spell.DoesNotExist"));
    assert_eq!(
        view.lookup("missing:partition", "C_Spell.DoesNotExist"),
        LookupResult::Unknown(LookupUnknownReason::PartitionMissing)
    );
    Ok(())
}

#[test]
fn partial_and_not_evaluated_partitions_never_claim_absence() -> Result<(), Box<dyn Error>> {
    let partial = ReferencePartition::new("api:partial", CoverageStatus::Partial, vec![])?;
    let skipped =
        ReferencePartition::new("api:not-evaluated", CoverageStatus::NotEvaluated, vec![])?;
    let view = ReferenceView::new(
        "generation:reference:coverage",
        vec![skipped, partial],
        vec![],
    )?;

    assert_eq!(
        view.lookup("api:partial", "C_Unknown.Call"),
        LookupResult::Unknown(LookupUnknownReason::PartialCoverage)
    );
    assert_eq!(
        view.lookup("api:not-evaluated", "C_Unknown.Call"),
        LookupResult::Unknown(LookupUnknownReason::NotEvaluated)
    );
    assert!(!view.is_authoritatively_absent("api:partial", "C_Unknown.Call"));
    Ok(())
}

#[test]
fn conflicts_take_precedence_over_records_and_negative_authority() -> Result<(), Box<dyn Error>> {
    let first = api_record("C_Map.GetMapInfo", "(map_id)->map_info")?;
    let second = api_record("C_Map.GetMapInfo", "(map_id)->map_info_or_nil")?;
    let first_digest = first.digest()?;
    let second_digest = second.digest()?;
    let partition =
        ReferencePartition::new("api:retail", CoverageStatus::Complete, vec![first])?;
    let conflict = ReferenceConflict::new(
        "api:retail",
        "C_Map.GetMapInfo",
        vec![second_digest, first_digest],
        vec!["source:blizzard-ui".to_owned(), "source:warcraft-wiki".to_owned()],
    )?;
    let view = ReferenceView::new(
        "generation:reference:conflict",
        vec![partition],
        vec![conflict],
    )?;

    match view.lookup("api:retail", "C_Map.GetMapInfo") {
        LookupResult::Conflict(conflict) => {
            assert_eq!(conflict.candidate_digests().len(), 2);
        }
        _ => return Err(io::Error::other("conflict must be preserved").into()),
    }
    Ok(())
}

#[test]
fn constructor_canonicalizes_input_order_and_digest() -> Result<(), Box<dyn Error>> {
    let api = ReferencePartition::new(
        "api:retail",
        CoverageStatus::Complete,
        vec![
            api_record("C_Spell.GetSpellInfo", "(spell_id)->spell_info")?,
            api_record("C_Map.GetMapInfo", "(map_id)->map_info")?,
        ],
    )?;
    let events = ReferencePartition::new(
        "event:retail",
        CoverageStatus::Complete,
        vec![ReferenceRecord::new(
            "PLAYER_LOGIN",
            ReferenceRecordKind::Event,
            "()",
            vec!["source:blizzard-ui".to_owned()],
            vec![],
        )?],
    )?;

    let first = ReferenceView::new(
        "generation:reference:deterministic",
        vec![events.clone(), api.clone()],
        vec![],
    )?;
    let second = ReferenceView::new(
        "generation:reference:deterministic",
        vec![api, events],
        vec![],
    )?;

    assert_eq!(first.self_digest(), second.self_digest());
    assert_eq!(first.canonical_bytes()?, second.canonical_bytes()?);
    Ok(())
}

#[test]
fn serialized_view_round_trips_and_rejects_digest_tampering() -> Result<(), Box<dyn Error>> {
    let partition = ReferencePartition::new(
        "api:retail",
        CoverageStatus::Complete,
        vec![api_record("C_Spell.GetSpellInfo", "(spell_id)->spell_info")?],
    )?;
    let view = ReferenceView::new("generation:reference:round-trip", vec![partition], vec![])?;
    let bytes = view.canonical_bytes()?;
    let reparsed: ReferenceView = serde_json::from_slice(&bytes)?;
    reparsed.validate()?;
    assert_eq!(view, reparsed);

    let mut value = serde_json::to_value(&view)?;
    value["self_digest"] = serde_json::Value::String(format!("sha256:{}", "0".repeat(64)));
    let tampered: ReferenceView = serde_json::from_value(value)?;
    assert!(tampered.validate().is_err());
    Ok(())
}

#[test]
fn duplicate_records_are_rejected_before_publication() -> Result<(), Box<dyn Error>> {
    let first = api_record("C_Spell.GetSpellInfo", "(spell_id)->spell_info")?;
    let second = api_record("C_Spell.GetSpellInfo", "(spell_id)->spell_info")?;
    let result = ReferencePartition::new(
        "api:retail",
        CoverageStatus::Complete,
        vec![first, second],
    );
    assert!(result.is_err());
    Ok(())
}
