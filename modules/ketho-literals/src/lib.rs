//! Separately updateable core-Wasm adapter. No WASI, imports, IO or source code.
use wow_ketho_literals::LiteralRenderer;
use wow_render_contract::{LiteralBridge, LiteralError, Request, Response, SCHEMA};

pub fn evaluate(bytes: &[u8]) -> Result<Vec<u8>, LiteralError> {
    let request = Request::decode(bytes)?;
    let result = LiteralRenderer::new(request.max_output_bytes)?.render(&request);
    let response = Response {
        schema: SCHEMA,
        result,
    };
    let bytes = response.encode()?;
    if bytes.len() > wow_render_contract::MAX_RESPONSE_BYTES {
        return Err(LiteralError::OutputLimit);
    }
    Ok(bytes)
}
#[cfg(target_arch = "wasm32")]
mod exports;
