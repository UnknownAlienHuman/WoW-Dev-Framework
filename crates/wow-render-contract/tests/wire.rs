use wow_render_contract::*;
#[test]
fn strict_wire_rejects_unknown_duplicate_schema_and_null_numeric_values() {
    for text in [
        r#"{"schema":1,"schema":1,"max_output_bytes":100,"input":{"CVars":[]}}"#,
        r#"{"schema":1,"max_output_bytes":100,"input":{"CVars":[]},"extra":0}"#,
        r#"{"schema":2,"max_output_bytes":100,"input":{"CVars":[]}}"#,
        r#"{"schema":1,"max_output_bytes":100,"input":{"Enums":{"enums":[],"constants":[{"name":"X","order":"Name","values":[{"name":"x","value":{"Number":null}}]}]}}}"#,
    ] {
        assert!(
            Request::decode(text.as_bytes()).is_err(),
            "accepted: {text}"
        );
    }
}
#[test]
fn strict_response_cannot_change_schema_or_exceed_requested_budget() {
    assert!(Response::decode(br#"{"schema":2,"result":{"Ok":"text"}}"#, 100).is_err());
    assert!(Response::decode(br#"{"schema":1,"result":{"Ok":"text"}}"#, 3).is_err());
    assert!(Response::decode(br#"{"schema":1,"result":{"Ok":"x"},"extra":1}"#, 100).is_err());
    assert!(Response::decode(br#"{"schema":1,"result":{"Err":"Unrecognized"}}"#, 100).is_err());
}
#[test]
fn typed_wire_keeps_negative_zero_strings_and_order() -> Result<(), LiteralError> {
    let request = Request {
        schema: SCHEMA,
        max_output_bytes: 1000,
        input: LiteralInput::Enums {
            enums: vec![],
            constants: vec![ConstantGroup {
                name: "X".into(),
                order: MemberOrder::Name,
                values: vec![
                    LiteralMember {
                        name: "first".into(),
                        value: LiteralValue::Number(-0.0),
                    },
                    LiteralMember {
                        name: "second".into(),
                        value: LiteralValue::String("9007199254740993".into()),
                    },
                ],
            }],
        },
    };
    let bytes = request.encode()?;
    assert_eq!(request, Request::decode(&bytes)?);
    assert!(String::from_utf8_lossy(&bytes).contains("-0.0"));
    let mut nonfinite = request;
    if let LiteralInput::Enums { constants, .. } = &mut nonfinite.input {
        constants[0].values[0].value = LiteralValue::Number(f64::INFINITY);
    }
    assert!(nonfinite.encode().is_err());
    Ok(())
}
#[test]
fn oversized_counts_and_bytes_reject() {
    assert!(
        Request {
            schema: SCHEMA,
            max_output_bytes: 100,
            input: LiteralInput::CVars(vec!["x".into(); MAX_ITEMS + 1])
        }
        .encode()
        .is_err()
    );
    assert!(Request::decode(&vec![0; MAX_REQUEST_BYTES + 1]).is_err());
}

#[test]
fn selected_module_identity_is_canonical_not_a_moving_selector() {
    use wow_render_contract::SelectedModule;
    let valid = SelectedModule {
        sha256: format!("sha256:{}", "ab".repeat(32)),
        epoch: 0,
    };
    assert!(valid.validate().is_ok());
    for digest in [
        "main".into(),
        "sha256:abc".into(),
        format!("sha256:{}", "AB".repeat(32)),
        format!("sha1:{}", "a".repeat(40)),
    ] {
        assert!(
            SelectedModule {
                sha256: digest,
                epoch: u64::MAX
            }
            .validate()
            .is_err()
        );
    }
}
