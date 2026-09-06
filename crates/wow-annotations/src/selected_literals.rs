//! One retained literal implementation per source operation. No VM dependency.
use crate::ketho::RenderError;
use crate::literals::{ConstantGroup, EnumDeclaration, EventLiteral, LiteralRenderer};
use serde::Serialize;
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_reference::native::source_digest;
use wow_render_contract::{
    LiteralBridge, LiteralError, LiteralInput, Request, SCHEMA, SelectedModule,
};

const MAX_CALLS: usize = 8192;
#[derive(Clone, Debug, Serialize)]
pub struct LiteralCall {
    pub ordinal: usize,
    pub operation: &'static str,
    pub request_sha256: String,
    /// Digest of returned UTF-8 text, not of the Wasm wire response envelope.
    pub result: Result<String, LiteralError>,
}
#[derive(Clone, Debug, Serialize)]
pub struct LiteralArtifact {
    pub path: String,
    pub call_ordinal: usize,
}
#[derive(Clone, Debug, Serialize)]
pub struct LiteralExecution {
    pub schema: &'static str,
    pub module: SelectedModule,
    pub calls: Vec<LiteralCall>,
    pub artifacts: Vec<LiteralArtifact>,
}
pub(crate) struct SelectedLiterals<'a> {
    native: LiteralRenderer,
    bridge: Option<&'a dyn LiteralBridge>,
    selection: Option<SelectedModule>,
    limit: usize,
    cancelled: &'a AtomicBool,
    failure: Cell<Option<RenderError>>,
    calls: RefCell<Vec<LiteralCall>>,
}
impl<'a> SelectedLiterals<'a> {
    pub(crate) fn new(
        bridge: Option<&'a dyn LiteralBridge>,
        limit: usize,
        cancelled: &'a AtomicBool,
    ) -> Result<Self, RenderError> {
        let selection = bridge
            .map(|bridge| {
                let selected = bridge.selected_module().ok_or(RenderError::BridgeFailure)?;
                selected
                    .validate()
                    .map_err(|_| RenderError::BridgeFailure)?;
                Ok::<_, RenderError>(selected)
            })
            .transpose()?;
        Ok(Self {
            native: LiteralRenderer::new(limit)?,
            bridge,
            selection,
            limit,
            cancelled,
            failure: Cell::new(None),
            calls: RefCell::new(Vec::new()),
        })
    }
    pub(crate) fn check(&self) -> Result<(), RenderError> {
        if let Some(error) = self.failure.get() {
            return Err(error);
        }
        if self.cancelled.load(Ordering::Relaxed) {
            return self.fail(RenderError::Cancelled);
        }
        if self
            .bridge
            .is_some_and(|b| b.selected_module() != self.selection)
        {
            return self.fail(RenderError::BridgeFailure);
        }
        Ok(())
    }
    fn fail<T>(&self, error: RenderError) -> Result<T, RenderError> {
        self.failure.set(Some(error));
        Err(error)
    }
    fn invoke(&self, input: LiteralInput) -> Result<String, RenderError> {
        self.check()?;
        if self.calls.borrow().len() >= MAX_CALLS {
            return self.fail(RenderError::InputLimit);
        }
        let operation = match &input {
            LiteralInput::Events(_) => "events",
            LiteralInput::CVars(_) => "cvars",
            LiteralInput::Enums { .. } => "enums",
        };
        let request = Request {
            schema: SCHEMA,
            max_output_bytes: self.limit,
            input,
        };
        // Bound the actual wire, not just item count, before dispatch.
        let bytes = match request.encode() {
            Ok(bytes) => bytes,
            Err(error) => return self.fail(crate::literals::error(error)),
        };
        let result = self
            .bridge
            .ok_or(RenderError::BridgeFailure)?
            .render(&request);
        self.check()?;
        if matches!(
            result,
            Err(LiteralError::BridgeFailure
                | LiteralError::InvalidWire
                | LiteralError::IncompatibleSchema)
        ) || result.as_ref().is_ok_and(|text| text.len() > self.limit)
        {
            return self.fail(RenderError::BridgeFailure);
        }
        let mut calls = self.calls.borrow_mut();
        let ordinal = calls.len();
        calls.push(LiteralCall {
            ordinal,
            operation,
            request_sha256: source_digest(&bytes),
            result: result
                .as_ref()
                .map(|text| source_digest(text.as_bytes()))
                .map_err(|error| *error),
        });
        result.map_err(crate::literals::error)
    }
    pub(crate) fn render_events(&self, values: &[EventLiteral]) -> Result<String, RenderError> {
        self.check()?;
        if self.bridge.is_some() {
            self.invoke(LiteralInput::Events(values.to_vec()))
        } else {
            self.native.render_events(values)
        }
    }
    pub(crate) fn render_enums(
        &self,
        enums: &[EnumDeclaration],
        constants: &[ConstantGroup],
    ) -> Result<String, RenderError> {
        self.check()?;
        if self.bridge.is_some() {
            self.invoke(LiteralInput::Enums {
                enums: enums.to_vec(),
                constants: constants.to_vec(),
            })
        } else {
            self.native.render_enums(enums, constants)
        }
    }
    pub(crate) fn finish(
        self,
        files: &[crate::native::AnnotationFile],
    ) -> Result<Option<LiteralExecution>, RenderError> {
        self.check()?;
        let Some(module) = self.selection else {
            return Ok(None);
        };
        let calls = self.calls.into_inner();
        let mut artifacts = Vec::new();
        for file in files {
            let operation = if file.path.starts_with("values-") {
                "enums"
            } else if file.path.starts_with("events-") {
                "events"
            } else {
                continue;
            };
            let call = calls
                .iter()
                .rev()
                .find(|call| call.operation == operation)
                .ok_or(RenderError::BridgeFailure)?;
            if call.result.as_ref().ok() != Some(&file.sha256) {
                return Err(RenderError::BridgeFailure);
            }
            artifacts.push(LiteralArtifact {
                path: file.path.clone(),
                call_ordinal: call.ordinal,
            });
        }
        Ok(Some(LiteralExecution {
            schema: "wow-literal-execution/1",
            module,
            calls,
            artifacts,
        }))
    }
}
