use wow_core::{
    CallableAliasDefinition, CallableFieldLowering, CallableLoweringLane,
    CallableLoweringObservation, CallableTypeAtom, CallableTypeDeclaration,
    CallableTypeDeclarationKind, SourceSpan,
};

fn span(start: u64, end: u64) -> wow_core::CoreResult<SourceSpan> {
    SourceSpan::byte_range(start, end)
}

fn field(
    ordinal: usize,
    marker: Option<SourceSpan>,
) -> wow_core::CoreResult<CallableFieldLowering> {
    CallableFieldLowering::admit(
        CallableLoweringLane::Parameter,
        ordinal,
        span(50, 60)?,
        span(0, 32)?,
        span(16, 23)?,
        marker,
        "boolean".into(),
        true,
    )
}

#[test]
fn callable_lowering_admission_enforces_ranges_and_identity_shape()
-> Result<(), Box<dyn std::error::Error>> {
    let valid = field(0, Some(span(14, 15)?))?;
    let atom = CallableTypeAtom::admit("boolean", "boolean", span(16, 23)?)?;
    let with_atom = valid
        .clone()
        .with_type_atoms(vec![atom].into_boxed_slice())?;
    assert_eq!(with_atom.type_atoms().len(), 1);
    assert!(
        valid
            .clone()
            .with_type_atoms(
                vec![CallableTypeAtom::admit(
                    "boolean",
                    "boolean",
                    span(24, 31)?
                )?]
                .into_boxed_slice()
            )
            .is_err(),
        "type atom range must remain inside the emitted token"
    );
    let old_field_json = serde_json::to_value(&valid)?;
    let old_field_bytes = wow_core::canonical_json_bytes(&valid)?;
    let empty_atoms = valid.clone().with_type_atoms(Box::default())?;
    assert!(
        old_field_json.get("type_atoms").is_none(),
        "omitted atoms preserve the old wire shape"
    );
    assert!(
        serde_json::to_value(&empty_atoms)?
            .get("type_atoms")
            .is_none(),
        "zero atoms are absent, not null"
    );
    assert_eq!(
        wow_core::canonical_json_bytes(&empty_atoms)?,
        old_field_bytes
    );
    assert!(
        field(0, None).is_err(),
        "optional marker is mandatory when optional"
    );
    assert!(
        field(0, Some(span(16, 17)?)).is_err(),
        "marker cannot overlap type token"
    );
    assert!(
        field(0, Some(span(14, 16)?)).is_err(),
        "marker is exactly one byte"
    );
    assert!(
        field(0, Some(span(32, 33)?)).is_err(),
        "marker is inside its member"
    );
    let fields = [valid];
    let digest = format!("sha256:{}", "0".repeat(64));
    let generation = format!("generation:reference:{digest}");
    let observation = |path, fields| {
        CallableLoweringObservation::admit(
            "profile:wow:fixture-callable-lowering",
            &generation,
            &digest,
            "function:C_Fixture.Read",
            &digest,
            "native-source:fixture",
            "UI/API.lua",
            &digest,
            "annotation-artifact:fixture",
            "source-map:fixture",
            path,
            &digest,
            span(0, 40)?,
            fields,
        )
    };
    let admitted = observation("library/Fixture.lua", &fields)?;
    let old_observation_json = serde_json::to_value(admitted)?;
    let old_observation_bytes = wow_core::canonical_json_bytes(&admitted)?;
    let empty_definitions = admitted.with_alias_definitions(&[])?;
    assert!(
        old_observation_json.get("alias_definitions").is_none(),
        "omitted definitions are absent"
    );
    assert!(
        serde_json::to_value(empty_definitions)?
            .get("alias_definitions")
            .is_none(),
        "zero definitions are absent, not null"
    );
    assert_eq!(
        wow_core::canonical_json_bytes(&empty_definitions)?,
        old_observation_bytes
    );
    assert_eq!(admitted.consumer_semantics(), "NotEvaluated");
    assert_eq!(admitted.fields()[0].lowered_type(), "boolean");
    let required = CallableFieldLowering::admit(
        CallableLoweringLane::Return,
        0,
        span(70, 80)?,
        span(32, 64)?,
        span(42, 49)?,
        None,
        "boolean".into(),
        false,
    )?;
    let encoded = wow_core::canonical_json_bytes(&required)?;
    assert!(!String::from_utf8(encoded)?.contains("optional_marker_span"));
    assert!(observation("../Fixture.lua", &fields).is_err());
    assert!(observation("C:/Fixture.lua", &fields).is_err());
    let lowered = "boolean";
    let token_start = 21;
    let token_end = token_start + lowered.len() as u64;
    let alias = CallableAliasDefinition::admit(
        "FixtureType",
        &"a".repeat(40),
        "docs/aliases.lua",
        &digest,
        span(0, 12)?,
        "library/Fixture.lua",
        &digest,
        span(0, 40)?,
        span(token_start, token_end)?,
        lowered,
        Vec::new(),
    )?;
    assert_eq!(alias.lowered_definition(), "boolean");
    assert_eq!(alias.declaration_span(), span(0, 40)?);
    assert_eq!(alias.type_token_span(), span(21, 28)?);
    assert!(
        CallableAliasDefinition::admit(
            "FixtureType",
            &"a".repeat(40),
            "docs/aliases.lua",
            &digest,
            span(0, 12)?,
            "library/Fixture.lua",
            &digest,
            span(0, 40)?,
            span(token_start, token_end + 1)?,
            lowered,
            Vec::new(),
        )
        .is_err(),
        "expression bytes must match the projected token span length"
    );
    let bad_digest = CallableAliasDefinition::admit(
        "FixtureType",
        &"a".repeat(40),
        "docs/aliases.lua",
        "sha256:BAD",
        span(0, 12)?,
        "library/Fixture.lua",
        &digest,
        span(0, lowered.len() as u64)?,
        span(token_start, token_end)?,
        lowered,
        Vec::new(),
    );
    assert!(
        bad_digest.is_err(),
        "source digests require canonical typed SourceContent spelling"
    );
    let empty_declarations = admitted.with_type_declarations(&[])?;
    assert_eq!(
        wow_core::canonical_json_bytes(&empty_declarations)?,
        old_observation_bytes,
        "empty declaration selection preserves bytes"
    );
    let declaration = CallableTypeDeclaration::admit(
        CallableTypeDeclarationKind::Structure,
        "FixtureType",
        "FixtureType",
        &"a".repeat(40),
        "docs/API.lua",
        &digest,
        span(0, 12)?,
        "library/Types.lua",
        &digest,
        span(48, 70)?,
        span(50, 61)?,
    )?;
    let declarations = [declaration.clone()];
    let linked = admitted.with_type_declarations(&declarations)?;
    assert_eq!(linked.type_declarations()[0].rendered_name(), "FixtureType");
    assert!(
        serde_json::to_value(linked)?
            .get("type_declarations")
            .is_some()
    );
    assert!(
        admitted
            .with_type_declarations(&[declaration.clone(), declaration])
            .is_err(),
        "duplicate emitted names are not exact links"
    );
    assert!(
        CallableTypeDeclaration::admit(
            CallableTypeDeclarationKind::Structure,
            "FixtureType",
            "FixtureType",
            &"a".repeat(40),
            "docs/API.lua",
            &digest,
            span(0, 12)?,
            "library/Types.lua",
            &digest,
            span(48, 70)?,
            span(71, 82)?,
        )
        .is_err(),
        "name outside its declaration is rejected"
    );
    let skipped = [field(1, Some(span(14, 15)?))?];
    assert!(
        observation("library/Fixture.lua", &skipped).is_err(),
        "lane ordinal must start at zero"
    );
    Ok(())
}
