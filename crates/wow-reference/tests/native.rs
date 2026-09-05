#![allow(clippy::unwrap_used)]
use std::sync::atomic::AtomicBool;
use wow_reference::native::{
    DocumentationDocument, NativeErrorCode, RawKey, RawKind, ingest_document, source_digest,
};
use wow_reference::native_model::{SystemOwner, TableFact, array, normalize_document, object};

const REV: &str = "1111111111111111111111111111111111111111";
fn read(text: &str) -> DocumentationDocument {
    ingest_document(
        REV,
        "Interface/API.lua",
        text,
        &source_digest(text.as_bytes()),
        &AtomicBool::new(false),
    )
    .unwrap()
}
const SOURCE: &str = r#"-- inert source comments
local api = {
 Name = "Example", Type = "System", Namespace = "C_Example", Environment = "All",
 Functions = {{Name="Read", Type="Function", SecretReturns=true,
  Arguments={{Name="flag",Type="bool",Nilable=false,Default=false}},
  Returns={{Name="value",Type="number",Nilable=true}}}},
 Events = {{Name="Changed",LiteralName="EXAMPLE_CHANGED",Payload={{Name="value",Type="number"}}}},
 Tables={{Name="Mode", Type="Enumeration",Fields={{Name="First",EnumValue=0},{Name="Last",EnumValue=0x10}}}},
 UnknownFuture = {flag=nil, predicate=Enum.Example.Condition, huge=9223372036854775808, tiny=1e-9999}
};
APIDocumentation:AddDocumentationTable(api);
"#;
#[test]
fn raw_metadata_and_numbers_survive() {
    let doc = read(SOURCE);
    let raw = &doc.registrations()[0].value;
    let fields = object(raw).unwrap();
    let future = object(fields["UnknownFuture"]).unwrap();
    assert_eq!(
        future["huge"].kind,
        RawKind::Number("9223372036854775808".into())
    );
    assert_eq!(future["tiny"].kind, RawKind::Number("1e-9999".into()));
    assert!(matches!(future["flag"].kind, RawKind::Nil));
    assert_eq!(
        future["predicate"].kind,
        RawKind::Reference(vec!["Enum".into(), "Example".into(), "Condition".into()])
    );
    assert_eq!(
        &SOURCE[future["huge"].span.start..future["huge"].span.end],
        "9223372036854775808"
    );
}
#[test]
fn enums_are_classified_by_type_not_collection() {
    let doc = read(SOURCE);
    let model = normalize_document(&doc);
    let system = model.systems[0].as_ref().unwrap();
    assert_eq!(system.owner, SystemOwner::Namespace("C_Example"));
    assert!(matches!(
        system.tables[0],
        TableFact::Enumeration { name: "Mode", .. }
    ));
    assert_eq!(
        system.functions[0].arguments[0].default.unwrap().kind,
        RawKind::Boolean(false)
    );
}
#[test]
fn script_objects_do_not_become_globals() {
    let doc = read(
        r#"local api={Name="SimpleFrameAPI",Type="ScriptObject",Functions={{Name="Show"}}} APIDocumentation:AddDocumentationTable(api)"#,
    );
    assert_eq!(
        normalize_document(&doc).systems[0].as_ref().unwrap().owner,
        SystemOwner::ScriptObject("SimpleFrameAPI")
    );
}
#[test]
fn unnamed_tables_have_no_invented_namespace() {
    let doc = read(
        r#"local not_a_namespace={Tables={{Name="E",Type="Enumeration",Fields={}}}} APIDocumentation:AddDocumentationTable(not_a_namespace)"#,
    );
    let model = normalize_document(&doc);
    let sys = model.systems[0].as_ref().unwrap();
    assert_eq!(sys.owner, SystemOwner::Global);
    assert!(sys.name.is_none());
}
#[test]
fn duplicates_remain_raw_but_fail_normalization() {
    let doc = read(r#"APIDocumentation:AddDocumentationTable({Name="A",Name="B",Type="System"})"#);
    assert_eq!(
        doc.registrations()[0]
            .value
            .fields()
            .unwrap()
            .iter()
            .filter(|v| v.key == RawKey::Name("Name".into()))
            .count(),
        2
    );
    assert!(normalize_document(&doc).systems[0].is_err());
}
#[test]
fn explicit_array_indices_are_ordered_and_holes_rejected() {
    let doc = read(r#"APIDocumentation:AddDocumentationTable({[2]="b",[1]="a"})"#);
    let seq = array(&doc.registrations()[0].value).unwrap();
    assert_eq!(seq[0].kind, RawKind::String("a".into()));
    for text in [
        "{[0]=1}",
        "{[2]=1}",
        "{nil,1}",
        "{[1]=1,[1]=2}",
        "{value=1}",
    ] {
        let doc = read(&format!("APIDocumentation:AddDocumentationTable({text})"));
        assert!(array(&doc.registrations()[0].value).is_err());
    }
}
#[test]
fn callback_returns_are_retained_for_projection_decisions() {
    let doc = read(
        r#"APIDocumentation:AddDocumentationTable({Tables={{Name="CB",Type="CallbackType",Returns={{Name="ok",Type="bool"}}}}})"#,
    );
    let model = normalize_document(&doc);
    assert!(
        matches!(&model.systems[0].as_ref().unwrap().tables[0],TableFact::Callback{returns,..} if returns.len()==1)
    );
}
#[test]
fn utf8_bom_and_long_strings_have_exact_byte_spans() {
    let source = "\u{feff}-- Пример\nlocal a={Name=\"Café\",Type=\"System\",Documentation={[=[\nline\ntext]=]}} APIDocumentation:AddDocumentationTable(a)";
    let doc = read(source);
    let map = object(&doc.registrations()[0].value).unwrap();
    let value = map["Name"];
    assert_eq!(&source[value.span.start..value.span.end], "\"Café\"");
    assert_eq!(
        array(map["Documentation"]).unwrap()[0].kind,
        RawKind::String("line\ntext".into())
    );
}
#[test]
fn exact_revision_path_digest_and_cancellation_checked() {
    let c = AtomicBool::new(false);
    let hash = source_digest(SOURCE.as_bytes());
    for (rev, path) in [
        ("main", "API.lua"),
        (REV, "../API.lua"),
        (REV, "C:/API.lua"),
        (REV, "/API.lua"),
        (REV, "a\\b.lua"),
    ] {
        assert_eq!(
            ingest_document(rev, path, SOURCE, &hash, &c)
                .unwrap_err()
                .code,
            NativeErrorCode::InvalidIdentity
        );
    }
    assert_eq!(
        ingest_document(REV, "API.lua", SOURCE, "sha256:bad", &c)
            .unwrap_err()
            .code,
        NativeErrorCode::DigestMismatch
    );
    c.store(true, std::sync::atomic::Ordering::Relaxed);
    assert_eq!(
        ingest_document(REV, "API.lua", SOURCE, &hash, &c)
            .unwrap_err()
            .code,
        NativeErrorCode::Cancelled
    );
    assert!(
        ingest_document(
            &"2".repeat(64),
            "API.lua",
            SOURCE,
            &hash,
            &AtomicBool::new(false)
        )
        .is_ok()
    );
}
#[test]
fn source_execution_and_unreviewed_semantics_are_rejected() {
    for text in [
        "os.execute('echo unsafe')",
        "local a=require('unsafe')",
        "local a=loadfile('unsafe')()",
        "local a={} a.Name='bad' APIDocumentation:AddDocumentationTable(a)",
        "APIDocumentation.AddDocumentationTable({})",
        "other:AddDocumentationTable({})",
        "APIDocumentation:AddDocumentationTable({}, {})",
        "APIDocumentation:AddDocumentationTable(missing)",
        "local Enum={} APIDocumentation:AddDocumentationTable(Enum)",
        "local a=function() end",
        "while true do end",
        "APIDocumentation:AddDocumentationTable({value=1+2})",
        "APIDocumentation:AddDocumentationTable({value=_G.X})",
        "APIDocumentation:AddDocumentationTable({value=Enum:Call()})",
        "APIDocumentation:AddDocumentationTable({[nil]=1})",
        "APIDocumentation:AddDocumentationTable({Name='incomplete'}) garbage",
    ] {
        assert!(
            ingest_document(
                REV,
                "API.lua",
                text,
                &source_digest(text.as_bytes()),
                &AtomicBool::new(false)
            )
            .is_err(),
            "accepted {text}"
        );
    }
}
#[test]
fn byte_escapes_never_silently_change_string_content() {
    for text in [
        r#"APIDocumentation:AddDocumentationTable({value="\255"})"#,
        r#"APIDocumentation:AddDocumentationTable({value="Я\255"})"#,
        r#"APIDocumentation:AddDocumentationTable({["\255"]=1})"#,
    ] {
        assert!(
            ingest_document(
                REV,
                "API.lua",
                text,
                &source_digest(text.as_bytes()),
                &AtomicBool::new(false)
            )
            .is_err()
        );
    }
    let doc = read(r#"APIDocumentation:AddDocumentationTable({value="\\255",quote="\""})"#);
    let map = object(&doc.registrations()[0].value).unwrap();
    assert_eq!(map["value"].kind, RawKind::String("\\255".into()));
}
#[test]
fn nesting_chain_and_input_size_are_bounded_before_parse() {
    for text in [
        format!(
            "APIDocumentation:AddDocumentationTable({}0{})",
            "{".repeat(60),
            "}".repeat(60)
        ),
        format!(
            "APIDocumentation:AddDocumentationTable({{x=Enum{}}})",
            ".x".repeat(60)
        ),
        " ".repeat(1024 * 1024 + 1),
    ] {
        assert_eq!(
            ingest_document(
                REV,
                "API.lua",
                &text,
                &source_digest(text.as_bytes()),
                &AtomicBool::new(false)
            )
            .unwrap_err()
            .code,
            NativeErrorCode::Limit
        );
    }
}
#[test]
fn multiple_immutable_bindings_and_registrations_preserve_order() {
    let doc = read(
        "local a={Tables={}} local b=a APIDocumentation:AddDocumentationTable(a) APIDocumentation:AddDocumentationTable(b)",
    );
    assert_eq!(doc.registrations().len(), 2);
    assert_eq!(doc.registrations()[1].ordinal, 1);
    assert_eq!(doc.registrations()[0].value, doc.registrations()[1].value);
}
#[test]
fn unknown_table_type_remains_explicitly_unsupported() {
    let doc = read(
        r#"APIDocumentation:AddDocumentationTable({Tables={{Name="Future",Type="FutureType",NewFlag=true}}})"#,
    );
    assert!(matches!(
        normalize_document(&doc).systems[0].as_ref().unwrap().tables[0],
        TableFact::Unsupported {
            type_name: "FutureType",
            ..
        }
    ));
}
