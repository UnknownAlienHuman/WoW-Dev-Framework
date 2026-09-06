//! This test is mandatory in the Wasm CI lane. Build both Rust guests first.
type TestResult<T> = std::result::Result<T, Box<dyn Error>>;
use std::{error::Error, fs};
use wow_literal_host::*;
use wow_render_contract::{
    EnumDeclaration, EventLiteral, IntegerFormat, LiteralMember, LiteralValue,
};
#[test]
#[ignore = "requires two independently compiled Rust Wasm artifacts; run the explicit CI probe"]
fn two_rust_guests_one_unchanged_host() -> TestResult<()> {
    let a = fs::read(std::env::var("WDF_WASM_A")?)?;
    let b = fs::read(std::env::var("WDF_WASM_B")?)?;
    assert_ne!(
        module_digest(&a),
        module_digest(&b),
        "probe requires distinct Wasm artifacts"
    );
    let first = ModuleHandle::load(&a, &module_digest(&a), Limits::default())?;
    let second = ModuleHandle::load(&b, &module_digest(&b), Limits::default())?;
    let slot = ModuleSlot::new(first.clone());
    let retained = slot.snapshot()?;
    let inputs = [
        LiteralInput::CVars(vec!["zeta".into(), "alpha".into()]),
        LiteralInput::Events(vec![EventLiteral {
            name: "EVENT_TEST".into(),
            payload: "value".into(),
        }]),
        LiteralInput::Enums {
            enums: vec![EnumDeclaration {
                name: "Example".into(),
                integer_format: IntegerFormat::Decimal,
                values: vec![LiteralMember {
                    name: "First".into(),
                    value: LiteralValue::Integer(1),
                }],
            }],
            constants: vec![],
        },
    ];
    let before = inputs
        .iter()
        .cloned()
        .map(|input| {
            retained.render(&Request {
                schema: SCHEMA,
                max_output_bytes: 4096,
                input,
            })
        })
        .collect::<wow_literal_host::Result<Vec<_>>>()?;
    let generation = slot.replace(retained.selection(), second.clone())?;
    let second_snapshot = slot.snapshot()?;
    for (input, original) in inputs.iter().cloned().zip(&before) {
        let request = Request {
            schema: SCHEMA,
            max_output_bytes: 4096,
            input,
        };
        let expected = wow_ketho_literals::LiteralRenderer::new(4096)?.render(&request)?;
        let old = retained.render(&request)?;
        let current = slot.snapshot()?.render(&request)?;
        assert_eq!(&old, original);
        assert_eq!(old.text, expected);
        assert_eq!(current.text, expected);
        assert_eq!(old.module_sha256, first.digest());
        assert_eq!(current.module_sha256, second.digest());
        assert_eq!(old.request_sha256, current.request_sha256);
        println!(
            "verified render: old={} current={} input={} output={}",
            old.module_sha256,
            current.module_sha256,
            current.request_sha256,
            current.response_sha256
        );
    }
    let rollback = slot.replace(&generation, first.clone())?;
    assert_eq!(rollback.epoch(), generation.epoch() + 1);
    for (input, original) in inputs.into_iter().zip(&before) {
        let request = Request {
            schema: SCHEMA,
            max_output_bytes: 4096,
            input,
        };
        assert_eq!(&slot.snapshot()?.render(&request)?, original);
        let retained_second = second_snapshot.render(&request)?;
        assert_eq!(retained_second.text, original.text);
        assert_eq!(retained_second.module_sha256, second.digest());
    }
    println!(
        "verified rollback and both retained generations: epoch={}",
        rollback.epoch()
    );
    assert_eq!(slot.snapshot()?.selection().module_sha256(), first.digest());
    assert_eq!(
        slot.replace(retained.selection(), second),
        Err(BridgeError::StaleSelection)
    );
    let duplicate = Request {
        schema: SCHEMA,
        max_output_bytes: 1024,
        input: LiteralInput::CVars(vec!["x".into(), "x".into()]),
    };
    assert_eq!(
        slot.snapshot()?.render(&duplicate),
        Err(BridgeError::Render(LiteralError::DuplicateName))
    );
    Ok(())
}
