use std::str::FromStr;

use wow_core::{
    ContentDigest, CoveragePartitionId, EntityKey, ProfileId, RuleId, SourceContent,
    SourceHandleBuilder, SourceOriginKind, SourceSpan,
};

#[test]
fn identifier_families_remain_distinct() -> Result<(), Box<dyn std::error::Error>> {
    let profile = ProfileId::from_str("profile:fixture:e0-retail-120100")?;
    let rule = RuleId::from_str("wow.api.exists")?;
    let entity = EntityKey::from_str("entity:api:C_UnitAuras.GetAuraDataByIndex")?;
    let partition = CoveragePartitionId::from_str("partition:project.file:Core%2FInit.lua")?;

    assert_eq!(profile.to_string(), "profile:fixture:e0-retail-120100");
    assert_eq!(rule.to_string(), "wow.api.exists");
    assert_eq!(entity.key(), "C_UnitAuras.GetAuraDataByIndex");
    assert_eq!(partition.key(), Some("Core/Init.lua"));
    Ok(())
}

#[test]
fn repository_handle_rejects_generation_binding() -> Result<(), Box<dyn std::error::Error>> {
    let reference = "generation:reference:sha256:8e56faa8b5c7efae0e0c8468c48101ed8d2cfef206b40e6e96106993096d2786"
        .parse()?;
    let result = SourceHandleBuilder::new(
        SourceOriginKind::Repository,
        "UnknownAlienHuman/example",
        "0123456789abcdef0123456789abcdef01234567",
        "Core/Init.lua",
        SourceSpan::whole_file(),
        ContentDigest::<SourceContent>::from_bytes([7_u8; 32]),
    )
    .reference_generation(reference)
    .build();
    assert!(result.is_err());
    Ok(())
}

#[test]
fn source_path_normalization_never_accepts_escape() {
    let hostile = [
        "../Core.lua",
        "Core/../Core.lua",
        "/Core.lua",
        "C:\\Core.lua",
        "\\\\server\\share\\Core.lua",
        "file://host/Core.lua",
    ];
    for candidate in hostile {
        assert!(wow_core::NormalizedSourcePath::parse(candidate).is_err());
    }
}
