//! Compatibility bridge to the extracted Ketho algorithm. No duplicate emitter.
//! This facade preserves existing owner APIs and error classifications.
use crate::ketho::RenderError;
pub use wow_ketho_literals::{
    ConstantGroup, EnumDeclaration, EventLiteral, IntegerFormat, LiteralMember, LiteralValue,
    MemberOrder,
};
use wow_ketho_literals::{LiteralError, LiteralRenderer as NativeRenderer};
#[derive(Clone, Debug)]
pub struct LiteralRenderer(NativeRenderer);
impl LiteralRenderer {
    pub fn new(limit: usize) -> Result<Self, RenderError> {
        NativeRenderer::new(limit).map(Self).map_err(error)
    }
    pub fn render_events(&self, values: &[EventLiteral]) -> Result<String, RenderError> {
        self.0.render_events(values).map_err(error)
    }
    pub fn render_cvars(&self, values: &[String]) -> Result<String, RenderError> {
        self.0.render_cvars(values).map_err(error)
    }
    pub fn render_enums(
        &self,
        enums: &[EnumDeclaration],
        constants: &[ConstantGroup],
    ) -> Result<String, RenderError> {
        self.0.render_enums(enums, constants).map_err(error)
    }
}
fn error(error: LiteralError) -> RenderError {
    match error {
        LiteralError::InvalidIdentifier => RenderError::InvalidIdentifier,
        LiteralError::UnsafeDocumentation => RenderError::UnsafeDocumentation,
        LiteralError::DuplicateName => RenderError::DuplicateName,
        LiteralError::InputLimit => RenderError::InputLimit,
        LiteralError::OutputLimit => RenderError::OutputLimit,
        LiteralError::UnsupportedLiteral => RenderError::UnsupportedLiteral,
        LiteralError::InvalidWire
        | LiteralError::IncompatibleSchema
        | LiteralError::BridgeFailure => RenderError::InvalidSource,
    }
}
