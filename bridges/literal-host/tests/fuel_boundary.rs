//! Recovered regression from the unpublished fuel repair, using the current API.
//! Measure the boundary at runtime: no compiler/VM instruction cost is pinned.
use std::error::Error;
use wow_literal_host::{
    BridgeError, Limits, LiteralInput, ModuleHandle, Request, SCHEMA, module_digest,
};

#[test]
fn cold_and_warm_calls_enforce_the_exact_fuel_boundary() -> Result<(), Box<dyn Error>> {
    let response = r#"{"schema":1,"result":{"Ok":"ok"}}"#;
    let encoded = response
        .bytes()
        .map(|byte| format!("\\{byte:02x}"))
        .collect::<String>();
    let bytes = wat::parse_str(format!(
        r#"(module
        (memory (export "memory") 1)
        (func (export "wow_abi_version") (result i32) i32.const 1)
        (func (export "wow_request_buffer") (param i32) (result i32) i32.const 4096)
        (func (export "wow_render") (result i32) i32.const 0)
        (func (export "wow_response_ptr") (result i32) i32.const 0)
        (func (export "wow_response_len") (result i32) i32.const {})
        (data (i32.const 0) "{encoded}"))"#,
        response.len()
    ))?;
    let request = Request {
        schema: SCHEMA,
        max_output_bytes: 1024,
        input: LiteralInput::CVars(vec![]),
    };
    let limits = Limits {
        fuel: 1000,
        memory_bytes: 65_536,
    };
    let digest = module_digest(&bytes);
    let handle = ModuleHandle::load(&bytes, &digest, limits)?;
    let measured = handle.render(&request)?;
    assert!(measured.fuel_consumed > 1);
    assert_eq!(handle.render(&request)?, measured);

    let exact = ModuleHandle::load(
        &bytes,
        &digest,
        Limits {
            fuel: measured.fuel_consumed,
            ..limits
        },
    )?;
    let short = ModuleHandle::load(
        &bytes,
        &digest,
        Limits {
            fuel: measured.fuel_consumed - 1,
            ..limits
        },
    )?;
    for _ in 0..2 {
        let receipt = exact.render(&request)?;
        assert_eq!(receipt.fuel_consumed, measured.fuel_consumed);
        assert_eq!(receipt.fuel_consumed, receipt.limits.fuel);
        assert_eq!(receipt.text, measured.text);
        assert_eq!(receipt.request_sha256, measured.request_sha256);
        assert_eq!(receipt.response_sha256, measured.response_sha256);
        assert_eq!(short.render(&request), Err(BridgeError::FuelExhausted));
    }
    // Exhaustion in another handle must not consume or poison this handle.
    assert_eq!(handle.render(&request)?, measured);
    Ok(())
}
