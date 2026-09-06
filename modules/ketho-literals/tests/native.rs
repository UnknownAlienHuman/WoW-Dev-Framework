use wow_render_contract::*;
#[test]
fn adapter_calls_real_ketho_renderer_and_preserves_domain_failure() -> Result<(), LiteralError> {
    let request = Request {
        schema: SCHEMA,
        max_output_bytes: 1024,
        input: LiteralInput::CVars(vec!["zeta".into(), "alpha".into()]),
    };
    let bytes = wow_ketho_literals_wasm::evaluate(&request.encode()?)?;
    let response = Response::decode(&bytes, 1024)?;
    assert_eq!(
        response.result?,
        "---@meta _\n---@alias CVar string\n---|\"alpha\"\n---|\"zeta\"\n"
    );
    let bad = Request {
        schema: SCHEMA,
        max_output_bytes: 1024,
        input: LiteralInput::CVars(vec!["same".into(), "same".into()]),
    };
    let bytes = wow_ketho_literals_wasm::evaluate(&bad.encode()?)?;
    assert_eq!(
        Response::decode(&bytes, 1024)?.result,
        Err(LiteralError::DuplicateName)
    );
    Ok(())
}
#[test]
fn invalid_wire_and_tiny_output_do_not_create_a_partial_success() -> Result<(), LiteralError> {
    assert!(wow_ketho_literals_wasm::evaluate(b"not-json").is_err());
    let request = Request {
        schema: SCHEMA,
        max_output_bytes: 1,
        input: LiteralInput::CVars(vec![]),
    };
    let bytes = wow_ketho_literals_wasm::evaluate(&request.encode()?)?;
    assert_eq!(
        Response::decode(&bytes, 1)?.result,
        Err(LiteralError::OutputLimit)
    );
    Ok(())
}
