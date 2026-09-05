use wow_annotations::ketho::RenderError;
use wow_annotations::literals::{
    ConstantGroup, EnumDeclaration, EventLiteral, IntegerFormat, LiteralMember, LiteralRenderer,
    LiteralValue, MemberOrder,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn renderer() -> Result<LiteralRenderer, RenderError> {
    LiteralRenderer::new(1024 * 1024)
}

fn member(name: &str, value: LiteralValue) -> LiteralMember {
    LiteralMember {
        name: name.into(),
        value,
    }
}

fn number(name: &str, value: i64) -> LiteralMember {
    member(name, LiteralValue::Integer(value))
}

fn enumeration(name: &str, values: Vec<LiteralMember>) -> EnumDeclaration {
    EnumDeclaration {
        name: name.into(),
        values,
        integer_format: IntegerFormat::Decimal,
    }
}

fn group(name: &str, values: Vec<LiteralMember>, order: MemberOrder) -> ConstantGroup {
    ConstantGroup {
        name: name.into(),
        values,
        order,
    }
}

fn events() -> Vec<EventLiteral> {
    vec![
        EventLiteral {
            name: "Z_TEST".into(),
            payload: "unitTarget: string".into(),
        },
        EventLiteral {
            name: "A_TEST".into(),
            payload: String::new(),
        },
        EventLiteral {
            name: "B_TEST".into(),
            payload: "id: number, value: boolean".into(),
        },
    ]
}

fn sample() -> (Vec<EnumDeclaration>, Vec<ConstantGroup>) {
    let mut bits = enumeration(
        "BBits",
        vec![number("High", 8), number("Low", 2), number("Mid", 4)],
    );
    bits.integer_format = IntegerFormat::Hexadecimal;
    (
        vec![
            enumeration(
                "ZWide",
                vec![
                    member("Huge", LiteralValue::String("18446744073709551615".into())),
                    member("Small", LiteralValue::String("00017".into())),
                ],
            ),
            enumeration(
                "AMode",
                vec![
                    number("Negative", -2),
                    number("Zero", 0),
                    number("First", 1),
                    number("Alias", 1),
                ],
            ),
            bits,
            enumeration(
                "CBoolean",
                vec![
                    member("On", LiteralValue::Boolean(true)),
                    member("Off", LiteralValue::Boolean(false)),
                ],
            ),
        ],
        vec![
            group(
                "Values",
                vec![
                    number("Z", 3),
                    member("A", LiteralValue::String("test".into())),
                    member("B", LiteralValue::Boolean(false)),
                ],
                MemberOrder::Name,
            ),
            group(
                "UICharacterClasses",
                vec![number("Zulu", 1), number("Alpha", 10)],
                MemberOrder::Value,
            ),
        ],
    )
}

#[test]
fn donor_event_golden() -> TestResult {
    assert_eq!(
        renderer()?.render_events(&events())?,
        include_str!("golden/events.lua")
    );
    Ok(())
}

#[test]
fn donor_cvar_golden() -> TestResult {
    assert_eq!(
        renderer()?.render_cvars(&["zeta".into(), "alpha".into(), "Test_Name".into()])?,
        include_str!("golden/cvars.lua")
    );
    Ok(())
}

#[test]
fn donor_enum_and_constant_golden() -> TestResult {
    let (enums, constants) = sample();
    assert_eq!(
        renderer()?.render_enums(&enums, &constants)?,
        include_str!("golden/enum-constants.lua")
    );
    Ok(())
}

#[test]
fn shuffled_literals_are_deterministic_and_do_not_mutate_input() -> TestResult {
    let (mut enums, mut constants) = sample();
    let before = (enums.clone(), constants.clone());
    let expected = renderer()?.render_enums(&enums, &constants)?;
    assert_eq!((&enums, &constants), (&before.0, &before.1));
    enums.reverse();
    constants.reverse();
    for value in &mut enums {
        value.values.reverse();
    }
    for value in &mut constants {
        value.values.reverse();
    }
    assert_eq!(renderer()?.render_enums(&enums, &constants)?, expected);
    let mut input = events();
    let expected = renderer()?.render_events(&input)?;
    input.reverse();
    assert_eq!(renderer()?.render_events(&input)?, expected);
    Ok(())
}

#[test]
fn empty_aliases_retain_open_string_base_without_runtime_whitelist_claim() -> TestResult {
    assert_eq!(
        renderer()?.render_events(&[])?,
        "---@meta _\n---@alias FrameEvent string\n"
    );
    assert_eq!(
        renderer()?.render_cvars(&[])?,
        "---@meta _\n---@alias CVar string\n"
    );
    assert_eq!(
        renderer()?.render_enums(&[], &[])?,
        "---@meta _\nEnum = {}\n\nConstants = {\n}\n"
    );
    Ok(())
}

#[test]
fn literal_names_are_escaped_not_interpolated() -> TestResult {
    let input = ["x\"; evil() --".into(), "a\\b".into()];
    let output = renderer()?.render_cvars(&input)?;
    assert!(output.contains("---|\"x\\\"; evil() --\"\n"));
    assert!(output.contains("---|\"a\\\\b\"\n"));
    assert_eq!(output.lines().count(), 4);
    Ok(())
}

#[test]
fn payload_cannot_add_physical_lines_or_break_its_delimiter() -> TestResult {
    for payload in [
        "\n---@class Injected",
        "ok\r---@meta",
        "`bad`",
        "a\u{2028}b",
        "\0",
    ] {
        let event = EventLiteral {
            name: "VALID".into(),
            payload: payload.into(),
        };
        assert_eq!(
            renderer()?.render_events(&[event]),
            Err(RenderError::UnsafeDocumentation)
        );
    }
    Ok(())
}

#[test]
fn control_bearing_or_empty_event_and_cvar_names_fail() -> TestResult {
    for name in ["", "a\nb", "a\u{2029}b"] {
        assert!(renderer()?.render_cvars(&[name.into()]).is_err());
        assert!(
            renderer()?
                .render_events(&[EventLiteral {
                    name: name.into(),
                    payload: String::new()
                }])
                .is_err()
        );
    }
    Ok(())
}

#[test]
fn duplicate_event_or_cvar_is_not_silently_discarded() -> TestResult {
    assert_eq!(
        renderer()?.render_cvars(&["x".into(), "x".into()]),
        Err(RenderError::DuplicateName)
    );
    let mut input = events();
    input.push(input[0].clone());
    assert_eq!(
        renderer()?.render_events(&input),
        Err(RenderError::DuplicateName)
    );
    Ok(())
}

#[test]
fn duplicate_declaration_group_or_member_is_rejected() -> TestResult {
    let (mut enums, mut constants) = sample();
    enums.push(enums[0].clone());
    assert_eq!(
        renderer()?.render_enums(&enums, &[]),
        Err(RenderError::DuplicateName)
    );
    constants.push(constants[0].clone());
    assert_eq!(
        renderer()?.render_enums(&[], &constants),
        Err(RenderError::DuplicateName)
    );
    for order in [MemberOrder::Name, MemberOrder::Value] {
        let duplicate = vec![number("A", 0), number("A", 1)];
        assert_eq!(
            renderer()?.render_enums(&[enumeration("Test", duplicate.clone())], &[]),
            Err(RenderError::DuplicateName)
        );
        assert_eq!(
            renderer()?.render_enums(&[], &[group("Test", duplicate, order)]),
            Err(RenderError::DuplicateName)
        );
    }
    Ok(())
}

#[test]
fn enum_and_constant_names_live_in_separate_roots() -> TestResult {
    let output = renderer()?.render_enums(
        &[enumeration("Same", vec![number("X", 1)])],
        &[group("Same", vec![number("X", 2)], MemberOrder::Name)],
    )?;
    assert!(output.contains("Enum.Same = {\n\tX = 1,"));
    assert!(output.contains("Constants = {\n\tSame = {\n\t\tX = 2,"));
    Ok(())
}

#[test]
fn enum_type_and_value_sorting_has_stable_boolean_ties() -> TestResult {
    let values = vec![
        member("ZZ", LiteralValue::Boolean(true)),
        member("AA", LiteralValue::Boolean(true)),
        member("Off", LiteralValue::Boolean(false)),
        number("Minus", -1),
        number("A", 0),
        number("B", 0),
        member("Text", LiteralValue::String("0".into())),
    ];
    let output = renderer()?.render_enums(&[enumeration("Mixed", values)], &[])?;
    assert!(output.contains("\tAA = true,\n\tZZ = true,\n\tOff = false,\n\tMinus = -1,\n\tA = 0,\n\tB = 0,\n\tText = \"0\","));
    Ok(())
}

#[test]
fn formatting_is_explicit_not_selected_by_known_names_or_value_heuristics() -> TestResult {
    let output = renderer()?.render_enums(
        &[enumeration(
            "Damageclass",
            vec![number("A", 2), number("B", 4), number("C", 8)],
        )],
        &[group(
            "UICharacterClasses",
            vec![number("Zulu", 1), number("Alpha", 10)],
            MemberOrder::Name,
        )],
    )?;
    assert!(!output.contains("0x"));
    assert!(output.contains("\t\tAlpha = 10,\n\t\tZulu = 1,"));
    Ok(())
}

#[test]
fn negative_or_boolean_members_do_not_change_later_hex_formatting() -> TestResult {
    let mut value = enumeration(
        "Bits",
        vec![
            member("Flag", LiteralValue::Boolean(true)),
            number("Negative", -1),
            number("Mask", 16),
        ],
    );
    value.integer_format = IntegerFormat::Hexadecimal;
    let output = renderer()?.render_enums(&[value], &[])?;
    assert!(output.contains("\tFlag = true,\n\tNegative = -1,\n\tMask = 0x10,"));
    Ok(())
}

#[test]
fn wide_enum_text_keeps_type_and_leading_zeroes() -> TestResult {
    let output = renderer()?.render_enums(
        &[enumeration(
            "Wide",
            vec![member(
                "Value",
                LiteralValue::String("00018446744073709551615".into()),
            )],
        )],
        &[],
    )?;
    assert!(output.contains("Value = \"00018446744073709551615\""));
    Ok(())
}

#[test]
fn numeric_inputs_outside_lua51_exact_integer_interval_are_not_coerced() -> TestResult {
    for value in [
        i64::MIN,
        i64::MAX,
        9_007_199_254_740_992,
        -9_007_199_254_740_992,
    ] {
        assert_eq!(
            renderer()?.render_enums(&[enumeration("Large", vec![number("V", value)])], &[]),
            Err(RenderError::UnsupportedLiteral)
        );
    }
    let result = renderer()?.render_enums(
        &[enumeration(
            "Exact",
            vec![
                number("Max", 9_007_199_254_740_991),
                number("Min", -9_007_199_254_740_991),
            ],
        )],
        &[],
    )?;
    assert!(result.contains("Min = -9007199254740991"));
    assert!(result.contains("Max = 9007199254740991"));
    Ok(())
}

#[test]
fn finite_constants_sort_numerically_and_preserve_negative_zero() -> TestResult {
    let values = vec![
        number("C", 2),
        member("B", LiteralValue::Number(0.25)),
        member("A", LiteralValue::Number(-0.0)),
        number("Minus", -1),
    ];
    let result = renderer()?.render_enums(&[], &[group("Values", values, MemberOrder::Value)])?;
    assert!(result.contains("\t\tMinus = -1,\n\t\tA = -0.0,\n\t\tB = 0.25,\n\t\tC = 2,"));
    Ok(())
}

#[test]
fn nan_infinity_and_unprobed_enum_number_forms_fail() -> TestResult {
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            renderer()?.render_enums(
                &[],
                &[group(
                    "Values",
                    vec![member("V", LiteralValue::Number(value))],
                    MemberOrder::Value
                )]
            ),
            Err(RenderError::UnsupportedLiteral)
        );
    }
    assert_eq!(
        renderer()?.render_enums(
            &[enumeration(
                "Values",
                vec![member("V", LiteralValue::Number(0.25))]
            )],
            &[]
        ),
        Err(RenderError::UnsupportedLiteral)
    );
    Ok(())
}

#[test]
fn scalar_strings_escape_bytes_without_creating_code_or_directives() -> TestResult {
    let value = "\"\\\0\n\r\u{2028}é---@class Pwned";
    let output = renderer()?.render_enums(
        &[],
        &[group(
            "Strings",
            vec![member("V", LiteralValue::String(value.into()))],
            MemberOrder::Name,
        )],
    )?;
    assert!(
        output.contains("V = \"\\\"\\\\\\000\\010\\013\\226\\128\\168\\195\\169---@class Pwned\"")
    );
    assert_eq!(output.lines().count(), 8);
    assert!(
        !output
            .lines()
            .any(|line| line.starts_with("---@class Pwned"))
    );
    Ok(())
}

#[test]
fn declaration_and_field_identifiers_cannot_be_expressions() -> TestResult {
    for name in ["end", "A.B", "x = {}; evil()", "bad-name", "", "a\nb"] {
        assert_eq!(
            renderer()?.render_enums(&[enumeration(name, vec![])], &[]),
            Err(RenderError::InvalidIdentifier)
        );
        assert_eq!(
            renderer()?.render_enums(
                &[],
                &[group("Safe", vec![number(name, 1)], MemberOrder::Name)]
            ),
            Err(RenderError::InvalidIdentifier)
        );
    }
    Ok(())
}

#[test]
fn exact_output_budget_includes_expanding_escapes() -> TestResult {
    let values = [group(
        "Safe",
        vec![member("V", LiteralValue::String("é\0\\".into()))],
        MemberOrder::Name,
    )];
    let expected = renderer()?.render_enums(&[], &values)?;
    assert_eq!(
        LiteralRenderer::new(expected.len())?.render_enums(&[], &values)?,
        expected
    );
    assert_eq!(
        LiteralRenderer::new(expected.len() - 1)?.render_enums(&[], &values),
        Err(RenderError::OutputLimit)
    );
    assert_eq!(
        LiteralRenderer::new(0)?.render_events(&[]),
        Err(RenderError::OutputLimit)
    );
    assert!(LiteralRenderer::new(8 * 1024 * 1024 + 1).is_err());
    Ok(())
}

#[test]
fn text_and_item_limits_are_enforced() -> TestResult {
    assert_eq!(
        renderer()?.render_cvars(&["x".repeat(64 * 1024 + 1)]),
        Err(RenderError::InputLimit)
    );
    assert_eq!(
        renderer()?.render_events(&vec![
            EventLiteral {
                name: "A".into(),
                payload: String::new()
            };
            4097
        ]),
        Err(RenderError::InputLimit)
    );
    assert_eq!(
        renderer()?.render_enums(&[enumeration("Large", vec![number("A", 1); 4097])], &[]),
        Err(RenderError::InputLimit)
    );
    assert_eq!(
        renderer()?.render_enums(
            &[],
            &[group(
                "Large",
                vec![member("A", LiteralValue::String("x".repeat(64 * 1024 + 1)))],
                MemberOrder::Name
            )]
        ),
        Err(RenderError::InputLimit)
    );
    Ok(())
}

#[test]
fn total_member_budget_is_checked_before_rendering_or_sorting() -> TestResult {
    let groups: Vec<_> = (0..17)
        .map(|i| {
            group(
                &format!("Group{i}"),
                vec![number("A", 0); 4096],
                MemberOrder::Name,
            )
        })
        .collect();
    assert_eq!(
        renderer()?.render_enums(&[], &groups),
        Err(RenderError::InputLimit)
    );
    Ok(())
}
