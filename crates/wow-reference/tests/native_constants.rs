#![allow(clippy::unwrap_used)]
use std::sync::atomic::AtomicBool;
use wow_reference::native::{DocumentationDocument, RawKind, ingest_document, source_digest};
use wow_reference::native_constants::{ResolvedScalar, ScalarCatalog, ScalarError, ScalarValue};
use wow_reference::native_model::{TableFact, normalize_document, object};

const REV: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
fn doc(path: &str, text: &str) -> DocumentationDocument {
    ingest_document(
        REV,
        path,
        text,
        &source_digest(text.as_bytes()),
        &AtomicBool::new(false),
    )
    .unwrap()
}
fn resolve(definitions: &[&str], expression: &str) -> Result<ResolvedScalar, ScalarError> {
    let mut docs = definitions
        .iter()
        .enumerate()
        .map(|(i, source)| doc(&format!("{i}.lua"), source))
        .collect::<Vec<_>>();
    docs.push(doc(
        "Query.lua",
        &format!("APIDocumentation:AddDocumentationTable({{Default={expression}}})"),
    ));
    let normalized = docs[..definitions.len()]
        .iter()
        .map(normalize_document)
        .collect::<Vec<_>>();
    let systems = normalized
        .iter()
        .flat_map(|d| d.systems.iter().map(|s| (d.source, s.as_ref().unwrap())))
        .collect::<Vec<_>>();
    let catalog = ScalarCatalog::new(REV, &systems)?;
    let query = docs.last().unwrap();
    let map = object(&query.registrations()[0].value).unwrap();
    catalog.resolve(query, map["Default"], None, &AtomicBool::new(false))
}
const ENUM: &str = r#"APIDocumentation:AddDocumentationTable({Tables={{Name="Mode",Type="Enumeration",Fields={{Name="First",EnumValue=5},{Name="Last",EnumValue=8}}}}})"#;
const CONSTANTS: &str = r#"APIDocumentation:AddDocumentationTable({Tables={{Name="Limits",Type="Constants",Values={
{Name="FIRST",Type="Mode",Value="First"},
{Name="LAST",Type="Mode",Value="Last"},
{Name="COUNT",Type="number",Value=Constants.Limits.LAST-Constants.Limits.FIRST+1},
{Name="ZERO",Type="number",Value=0},
{Name="FLAG",Type="bool",Value=false},
{Name="TEXT",Type="string",Value="Last"}
}}}})"#;

#[test]
fn constants_read_values_not_fields_and_preserve_types() {
    let d = doc("Values.lua", CONSTANTS);
    let n = normalize_document(&d);
    let system = n.systems[0].as_ref().unwrap();
    let TableFact::Constants { values, .. } = &system.tables[0] else {
        unreachable!()
    };
    assert_eq!(values.len(), 6);
    assert_eq!(values[0].type_name, Some("Mode"));
    assert_eq!(values[4].value.kind, RawKind::Boolean(false));
    let bad = doc("Bad.lua", &CONSTANTS.replace("Values=", "Fields="));
    assert!(normalize_document(&bad).systems[0].is_err());
}
#[test]
fn cross_document_enum_labels_and_additive_chains_are_exact() {
    let r = resolve(&[ENUM, CONSTANTS], "Constants.Limits.COUNT").unwrap();
    assert_eq!(r.value, ScalarValue::Number("4".into()));
    assert!(r.evidence.iter().any(|e| e.path == "0.lua"));
    assert!(r.evidence.iter().any(|e| e.path == "1.lua"));
    assert!(r.evidence.iter().any(|e| e.path == "Query.lua"));
    assert!(
        r.evidence
            .iter()
            .all(|e| e.sha256.starts_with("sha256:") && e.span.end > e.span.start)
    );
}
#[test]
fn typed_enum_labels_do_not_change_explicit_strings() {
    assert_eq!(
        resolve(&[ENUM, CONSTANTS], "Constants.Limits.FIRST")
            .unwrap()
            .value,
        ScalarValue::Number("5".into())
    );
    assert_eq!(
        resolve(&[ENUM, CONSTANTS], "Constants.Limits.TEXT")
            .unwrap()
            .value,
        ScalarValue::String("Last".into())
    );
    assert_eq!(
        resolve(&[ENUM, CONSTANTS], "Constants.Limits.FLAG")
            .unwrap()
            .value,
        ScalarValue::Boolean(false)
    );
    assert_eq!(
        resolve(&[ENUM, CONSTANTS], "Constants.Limits.ZERO")
            .unwrap()
            .value,
        ScalarValue::Number("0".into())
    );
}
#[test]
fn raw_expressions_and_unknown_names_are_not_replaced_by_values() {
    let source = "APIDocumentation:AddDocumentationTable({Value=(1 + 2)-1,Default=UNAVAILABLE_NATIVE_CONSTANT})";
    let d = doc("Raw.lua", source);
    let map = object(&d.registrations()[0].value).unwrap();
    assert!(matches!(
        map["Value"].kind,
        RawKind::BinaryExpression { .. }
    ));
    assert_eq!(
        &source[map["Value"].span.start..map["Value"].span.end],
        "(1 + 2)-1"
    );
    assert_eq!(
        map["Default"].kind,
        RawKind::UnresolvedName("UNAVAILABLE_NATIVE_CONSTANT".into())
    );
    let before = serde_json::to_string(&d).unwrap();
    let catalog = ScalarCatalog::new(REV, &[]).unwrap();
    assert_eq!(
        catalog.resolve(&d, map["Default"], None, &AtomicBool::new(false)),
        Err(ScalarError::UnresolvedReference)
    );
    assert_eq!(
        catalog
            .resolve(&d, map["Value"], None, &AtomicBool::new(false))
            .unwrap()
            .value,
        ScalarValue::Number("2".into())
    );
    assert_eq!(serde_json::to_string(&d).unwrap(), before);
}
#[test]
fn pet_style_reference_chains_resolve_without_load_order_dependence() {
    let source = r#"APIDocumentation:AddDocumentationTable({Tables={{Name="Pet",Type="Constants",Values={
{Name="TOTAL",Value=Constants.Pet.STABLE+Constants.Pet.LEARNED},
{Name="EXTRA",Value=Constants.Pet.MAX}, {Name="MAX",Value=Constants.Pet.LEARNED},
{Name="STABLE",Value=200},{Name="LEARNED",Value=5}
}}}})"#;
    assert_eq!(
        resolve(&[source], "Constants.Pet.TOTAL").unwrap().value,
        ScalarValue::Number("205".into())
    );
    assert_eq!(
        resolve(&[source], "Constants.Pet.EXTRA").unwrap().value,
        ScalarValue::Number("5".into())
    );
}
#[test]
fn duplicate_groups_and_members_block_resolution_even_when_equal() {
    assert_eq!(
        resolve(&[ENUM, ENUM], "Enum.Mode.First"),
        Err(ScalarError::Conflict)
    );
    let duplicate = ENUM.replace(
        "{Name=\"Last\",EnumValue=8}",
        "{Name=\"First\",EnumValue=5}",
    );
    assert_eq!(
        resolve(&[&duplicate], "Enum.Mode.First"),
        Err(ScalarError::Conflict)
    );
}
#[test]
fn self_and_transitive_cycles_are_explicit_errors() {
    for expr in ["Constants.C.A", "Constants.C.B"] {
        let source = format!(
            r#"APIDocumentation:AddDocumentationTable({{Tables={{{{Name="C",Type="Constants",Values={{{{Name="A",Value={expr}}},{{Name="B",Value=Constants.C.A}}}}}}}}}})"#
        );
        assert_eq!(
            resolve(&[&source], "Constants.C.A"),
            Err(ScalarError::Cycle)
        );
    }
}
#[test]
fn no_global_lookup_or_string_numeric_coercion() {
    for expression in [
        "NOT_DEFINED",
        "Enum.Missing.Value",
        "Constants.Missing.Value",
    ] {
        assert_eq!(
            resolve(&[], expression),
            Err(ScalarError::UnresolvedReference)
        );
    }
    for expression in ["\"1\"+2", "1.5+2", "true+1", "nil-1"] {
        assert!(matches!(
            resolve(&[], expression),
            Err(ScalarError::NonIntegralArithmetic | ScalarError::UnsupportedValue)
        ));
    }
}
#[test]
fn additive_arithmetic_is_bounded_to_exact_lua_integers() {
    assert_eq!(
        resolve(&[], "0x10 + 2 - 1").unwrap().value,
        ScalarValue::Number("17".into())
    );
    assert_eq!(
        resolve(&[], "1 - 2 - 3").unwrap().value,
        ScalarValue::Number("-4".into())
    );
    for expr in [
        "9007199254740991+1",
        "-9007199254740991-1",
        "9007199254740992-1",
    ] {
        assert_eq!(resolve(&[], expr), Err(ScalarError::OutOfRange));
    }
}
#[test]
fn cross_revision_and_cancellation_reject() {
    let d = doc(
        "A.lua",
        "APIDocumentation:AddDocumentationTable({Value=1+2})",
    );
    let value = object(&d.registrations()[0].value).unwrap()["Value"];
    let catalog = ScalarCatalog::new(REV, &[]).unwrap();
    assert_eq!(
        catalog.resolve(&d, value, None, &AtomicBool::new(true)),
        Err(ScalarError::Cancelled)
    );
    let wrong = ScalarCatalog::new("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", &[]).unwrap();
    assert_eq!(
        wrong.resolve(&d, value, None, &AtomicBool::new(false)),
        Err(ScalarError::InvalidSource)
    );
}
#[test]
fn reference_depth_budget_stops_long_chains() {
    let fields = (0..64)
        .map(|i| format!("{{Name=\"V{i}\",Value=Constants.C.V{}}}", i + 1))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(
        "APIDocumentation:AddDocumentationTable({{Tables={{{{Name=\"C\",Type=\"Constants\",Values={{{fields},{{Name=\"V64\",Value=1}}}}}}}}}})"
    );
    assert_eq!(
        resolve(&[&source], "Constants.C.V0"),
        Err(ScalarError::Limit)
    );
}

#[test]
fn unknown_named_constant_type_is_not_a_runtime_string() {
    let source = r#"APIDocumentation:AddDocumentationTable({Tables={{Name="C",Type="Constants",Values={{Name="A",Type="MissingEnum",Value="First"}}}}})"#;
    assert_eq!(
        resolve(&[source], "Constants.C.A"),
        Err(ScalarError::UnresolvedReference)
    );
}
#[test]
fn arithmetic_does_not_erase_negative_zero_semantics() {
    assert_eq!(
        resolve(&[], "-0 - 0"),
        Err(ScalarError::NonIntegralArithmetic)
    );
}
