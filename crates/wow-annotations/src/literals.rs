//! Thin native facade over the Ketho literal algorithm. No duplicate emitter.
//! This preserves the annotation owner's errors and public types.
use crate::ketho::RenderError;
pub use wow_ketho_literals::{
    ConstantGroup, EnumDeclaration, EventLiteral, IntegerFormat, LiteralMember, LiteralValue,
    MemberOrder, RenderedEnumDeclaration, RenderedEnums,
};
use wow_ketho_literals::{LiteralError, LiteralRenderer as NativeRenderer};
#[derive(Clone, Debug)]
pub struct LiteralRenderer(NativeRenderer);
impl LiteralRenderer {
    pub fn new(limit: usize) -> Result<Self, RenderError> {
        NativeRenderer::new(limit).map(Self).map_err(error)
    }
    pub fn validate_event(value: &EventLiteral) -> Result<(), RenderError> {
        NativeRenderer::validate_event(value).map_err(error)
    }
    pub fn validate_enum(value: &EnumDeclaration) -> Result<(), RenderError> {
        NativeRenderer::validate_enum(value).map_err(error)
    }
    pub fn validate_constant_group(value: &ConstantGroup) -> Result<(), RenderError> {
        NativeRenderer::validate_constant_group(value).map_err(error)
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

    pub fn render_enums_mapped(
        &self,
        enums: &[EnumDeclaration],
        constants: &[ConstantGroup],
    ) -> Result<RenderedEnums, RenderError> {
        self.0.render_enums_mapped(enums, constants).map_err(error)
    }
}
pub(crate) fn error(error: LiteralError) -> RenderError {
    match error {
        LiteralError::InvalidIdentifier => RenderError::InvalidIdentifier,
        LiteralError::UnsafeDocumentation => RenderError::UnsafeDocumentation,
        LiteralError::DuplicateName => RenderError::DuplicateName,
        LiteralError::InputLimit => RenderError::InputLimit,
        LiteralError::OutputLimit => RenderError::OutputLimit,
        LiteralError::UnsupportedLiteral => RenderError::UnsupportedLiteral,
    }
}
