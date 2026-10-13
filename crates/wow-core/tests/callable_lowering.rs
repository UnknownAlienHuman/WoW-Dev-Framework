use wow_core::{
    CallableFieldLowering, CallableLoweringLane, CallableLoweringObservation, SourceSpan,
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
    let skipped = [field(1, Some(span(14, 15)?))?];
    assert!(
        observation("library/Fixture.lua", &skipped).is_err(),
        "lane ordinal must start at zero"
    );
    Ok(())
}
