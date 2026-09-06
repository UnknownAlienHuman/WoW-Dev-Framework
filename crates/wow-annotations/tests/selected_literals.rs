//! Source-to-bridge routing regressions. Synthetic bridges test failure policy;
//! actual Rust Wasm artifacts are exercised in the isolated host's real_source test.
use std::{
    cell::Cell,
    error::Error,
    sync::atomic::{AtomicBool, Ordering},
};
use wow_annotations::{
    ketho::RenderError,
    native::{project, project_with_literal_bridge},
};
use wow_reference::native::{DocumentationDocument, ingest_document, source_digest};
use wow_render_contract::{LiteralBridge, LiteralError, Request, SelectedModule};
type TestResult = Result<(), Box<dyn Error>>;
const SOURCE: &str = r#"APIDocumentation:AddDocumentationTable({Name="Synthetic",Type="System",Namespace="C_Synthetic",
Functions={{Name="Read",Returns={{Name="value",Type="bool"}}}},
Events={{Name="Changed",LiteralName="SYNTHETIC_CHANGED",Payload={{Name="value",Type="number"}}}},
Tables={{Name="Choice",Type="Enumeration",Fields={{Name="First",EnumValue=1}}},
{Name="Limits",Type="Constants",Values={{Name="Count",Value=3}}}}})"#;
fn doc(source: &str) -> Result<DocumentationDocument, Box<dyn Error>> {
    Ok(ingest_document(
        &"a".repeat(40),
        "API.lua",
        source,
        &source_digest(source.as_bytes()),
        &AtomicBool::new(false),
    )?)
}
#[derive(Default)]
struct Selected<'a> {
    calls: Cell<usize>,
    epoch: Cell<u64>,
    failure: Option<(usize, LiteralError)>,
    suffix: bool,
    move_epoch: bool,
    overflow: bool,
    invalid_identity: bool,
    cancel: Option<&'a AtomicBool>,
}
impl LiteralBridge for Selected<'_> {
    fn selected_module(&self) -> Option<SelectedModule> {
        Some(SelectedModule {
            sha256: if self.invalid_identity {
                "bad".into()
            } else {
                format!("sha256:{}", "b".repeat(64))
            },
            epoch: self.epoch.get(),
        })
    }
    fn render(&self, request: &Request) -> Result<String, LiteralError> {
        self.calls.set(self.calls.get() + 1);
        if self.move_epoch {
            self.epoch.set(self.epoch.get() + 1);
        }
        if let Some(cancelled) = self.cancel {
            cancelled.store(true, Ordering::Relaxed);
        }
        if let Some((call, error)) = self.failure
            && call == self.calls.get()
        {
            return Err(error);
        }
        if self.overflow {
            return Ok("x".repeat(request.max_output_bytes + 1));
        }
        let mut text =
            wow_ketho_literals::LiteralRenderer::new(request.max_output_bytes)?.render(request)?;
        if self.suffix {
            text.push_str("\n-- selected fixture\n");
        }
        Ok(text)
    }
}
#[test]
fn selected_module_preserves_native_bytes_and_binds_every_literal_artifact() -> TestResult {
    let docs = [doc(SOURCE)?];
    let cancelled = AtomicBool::new(false);
    let bridge = Selected::default();
    let native = project(&docs, "Mainline", &cancelled)?;
    let selected =
        project_with_literal_bridge(&docs, "Mainline", None, None, Some(&bridge), &cancelled)?;
    assert_eq!(selected.schema, "wow-native-annotation-library/6");
    let trace = selected.literal_execution.as_ref().ok_or("missing trace")?;
    assert_eq!(
        trace.module,
        bridge.selected_module().ok_or("missing identity")?
    );
    assert_eq!(trace.calls.len(), bridge.calls.get());
    assert_eq!(trace.artifacts.len(), 2);
    for artifact in &trace.artifacts {
        let file = selected
            .files
            .iter()
            .find(|f| f.path == artifact.path)
            .ok_or("missing file")?;
        assert_eq!(
            trace.calls[artifact.call_ordinal].result,
            Ok(file.sha256.clone())
        );
    }
    let mut wire = serde_json::to_value(&selected)?;
    wire.as_object_mut()
        .ok_or("not object")?
        .remove("literal_execution");
    wire["schema"] = serde_json::json!(native.schema);
    assert_eq!(wire, serde_json::to_value(&native)?);
    Ok(())
}
#[test]
fn selected_output_is_not_silently_replaced_by_native_bytes() -> TestResult {
    let docs = [doc(SOURCE)?];
    let bridge = Selected {
        suffix: true,
        ..Default::default()
    };
    let library = project_with_literal_bridge(
        &docs,
        "Mainline",
        None,
        None,
        Some(&bridge),
        &AtomicBool::new(false),
    )?;
    for file in &library.files {
        assert_eq!(file.sha256, source_digest(file.text.as_bytes()));
        if file.path.starts_with("values-") || file.path.starts_with("events-") {
            assert!(file.text.ends_with("-- selected fixture\n"));
            assert!(
                file.mappings
                    .iter()
                    .all(|m| m.generated.end == file.text.len())
            );
        } else {
            assert!(!file.text.contains("selected fixture"));
        }
    }
    Ok(())
}
#[test]
fn fatal_validation_errors_cannot_escape_as_partial_success() -> TestResult {
    let docs = [doc(SOURCE)?];
    for error in [
        LiteralError::BridgeFailure,
        LiteralError::InvalidWire,
        LiteralError::IncompatibleSchema,
    ] {
        let bridge = Selected {
            failure: Some((1, error)),
            ..Default::default()
        };
        let actual = project_with_literal_bridge(
            &docs,
            "Mainline",
            None,
            None,
            Some(&bridge),
            &AtomicBool::new(false),
        );
        assert!(matches!(actual, Err(RenderError::BridgeFailure)));
        assert_eq!(bridge.calls.get(), 1);
    }
    Ok(())
}
#[test]
fn failure_in_final_aggregation_is_not_a_native_fallback() -> TestResult {
    let docs = [doc(SOURCE)?];
    let bridge = Selected {
        failure: Some((5, LiteralError::BridgeFailure)),
        ..Default::default()
    };
    assert!(matches!(
        project_with_literal_bridge(
            &docs,
            "Mainline",
            None,
            None,
            Some(&bridge),
            &AtomicBool::new(false)
        ),
        Err(RenderError::BridgeFailure)
    ));
    assert_eq!(bridge.calls.get(), 5);
    Ok(())
}
#[test]
fn mutable_selection_is_rejected_inside_one_operation() -> TestResult {
    let docs = [doc(SOURCE)?];
    let bridge = Selected {
        move_epoch: true,
        ..Default::default()
    };
    assert!(matches!(
        project_with_literal_bridge(
            &docs,
            "Mainline",
            None,
            None,
            Some(&bridge),
            &AtomicBool::new(false)
        ),
        Err(RenderError::BridgeFailure)
    ));
    assert_eq!(bridge.calls.get(), 1);
    Ok(())
}
#[test]
fn identity_is_required_and_validated_before_any_dispatch() -> TestResult {
    let docs = [doc(SOURCE)?];
    let cancelled = AtomicBool::new(false);
    let native = wow_ketho_literals::LiteralRenderer::new(4096)?;
    assert!(matches!(
        project_with_literal_bridge(&docs, "Mainline", None, None, Some(&native), &cancelled),
        Err(RenderError::BridgeFailure)
    ));
    let invalid = Selected {
        invalid_identity: true,
        ..Default::default()
    };
    assert!(matches!(
        project_with_literal_bridge(&docs, "Mainline", None, None, Some(&invalid), &cancelled),
        Err(RenderError::BridgeFailure)
    ));
    assert_eq!(invalid.calls.get(), 0);
    Ok(())
}
#[test]
fn cancellation_during_dispatch_returns_no_artifact() -> TestResult {
    let docs = [doc(SOURCE)?];
    let cancelled = AtomicBool::new(false);
    let bridge = Selected {
        cancel: Some(&cancelled),
        ..Default::default()
    };
    assert!(matches!(
        project_with_literal_bridge(&docs, "Mainline", None, None, Some(&bridge), &cancelled),
        Err(RenderError::Cancelled)
    ));
    assert_eq!(bridge.calls.get(), 1);
    Ok(())
}
#[test]
fn excessive_selected_output_is_fatal_not_an_omission() -> TestResult {
    let docs = [doc(SOURCE)?];
    let bridge = Selected {
        overflow: true,
        ..Default::default()
    };
    assert!(matches!(
        project_with_literal_bridge(
            &docs,
            "Mainline",
            None,
            None,
            Some(&bridge),
            &AtomicBool::new(false)
        ),
        Err(RenderError::BridgeFailure)
    ));
    assert_eq!(bridge.calls.get(), 1);
    Ok(())
}
#[test]
fn domain_rejection_remains_partial_with_valid_siblings_and_no_fallback() -> TestResult {
    let docs = [doc(SOURCE)?];
    let bridge = Selected {
        failure: Some((1, LiteralError::UnsupportedLiteral)),
        ..Default::default()
    };
    let library = project_with_literal_bridge(
        &docs,
        "Mainline",
        None,
        None,
        Some(&bridge),
        &AtomicBool::new(false),
    )?;
    assert_eq!(library.projection, "partial");
    let text = library
        .files
        .iter()
        .map(|f| f.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("C_Synthetic.Read"));
    assert!(!text.contains("Enum.Choice"));
    assert!(text.contains("Count = 3"));
    let trace = library.literal_execution.as_ref().ok_or("missing trace")?;
    assert_eq!(trace.calls[0].result, Err(LiteralError::UnsupportedLiteral));
    assert!(!library.negative_authority);
    Ok(())
}
#[test]
fn no_literals_still_records_selection_without_inventing_calls() -> TestResult {
    let docs = [doc(
        r#"APIDocumentation:AddDocumentationTable({Name="OnlyAPI",Type="System",Functions={{Name="Read"}}})"#,
    )?];
    let bridge = Selected::default();
    let library = project_with_literal_bridge(
        &docs,
        "Mainline",
        None,
        None,
        Some(&bridge),
        &AtomicBool::new(false),
    )?;
    let trace = library.literal_execution.as_ref().ok_or("missing trace")?;
    assert!(trace.calls.is_empty());
    assert!(trace.artifacts.is_empty());
    assert_eq!(bridge.calls.get(), 0);
    Ok(())
}
