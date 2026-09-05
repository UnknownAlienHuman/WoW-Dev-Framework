use std::collections::BTreeSet;

use wow_annotations::ketho::{Field, Function, Owner, RenderError, Renderer, System, Table};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn field(name: &str, type_name: &str) -> Field {
    Field {
        name: name.into(),
        type_name: type_name.into(),
        inner_type: None,
        nilable: false,
        default_text: None,
        variadic: false,
    }
}

fn renderer() -> Result<Renderer, RenderError> {
    Renderer::new(BTreeSet::from(["AccountData".into()]), 1024 * 1024)
}

fn sample(owner: Owner) -> System {
    let mut enabled = field("enabled", "bool");
    enabled.default_text = Some("false".into());
    let mut count = field("count", "luaIndex");
    count.default_text = Some("0".into());
    let mut items = field("items", "table");
    items.inner_type = Some("AccountData".into());
    items.nilable = true;
    let mut values = field("values", "cstring");
    values.variadic = true;
    let mut result = field("result", "cstring");
    result.nilable = true;
    System {
        owner,
        functions: vec![Function {
            name: "Inspect".into(),
            documentation: Some(vec!["Inspect values.".into(), "No runtime body.".into()]),
            arguments: vec![enabled, count.clone(), items.clone(), values],
            returns: vec![result, field("kind", "AccountData")],
        }],
        tables: vec![
            Table::Structure {
                name: "Record".into(),
                fields: vec![items],
            },
            Table::Callback {
                name: "OnResult".into(),
                arguments: vec![count],
            },
        ],
    }
}

#[test]
fn donor_namespace_golden() -> TestResult {
    assert_eq!(
        renderer()?.render(&sample(Owner::Namespace("C_Test".into())))?,
        include_str!("golden/namespace.lua")
    );
    Ok(())
}

#[test]
fn donor_script_object_golden() -> TestResult {
    let owner = Owner::ScriptObject {
        system_name: "SimpleFrameAPI".into(),
        annotation_name: Some("Frame".into()),
    };
    assert_eq!(
        renderer()?.render(&sample(owner))?,
        include_str!("golden/widget.lua")
    );
    Ok(())
}

#[test]
fn global_and_unaliased_object_names_are_not_namespaces() -> TestResult {
    let renderer = renderer()?;
    let global = renderer.render(&sample(Owner::Global))?;
    assert!(global.contains("function Inspect(enabled, count, items, ...) end"));
    assert!(!global.contains(" = {}"));
    let object = renderer.render(&sample(Owner::ScriptObject {
        system_name: "FutureObject".into(),
        annotation_name: None,
    }))?;
    assert!(object.contains("function FutureObject:Inspect("));
    assert!(object.contains("API_FutureObject_Inspect)"));
    assert!(!object.contains("function FutureObject.Inspect("));
    Ok(())
}

#[test]
fn donor_type_lowering_preserves_unknown_names_without_widening() -> TestResult {
    let renderer = renderer()?;
    for (source, target) in [
        ("bool", "boolean"),
        ("cstring", "string"),
        ("luaIndex", "number"),
        ("AccountData", "Enum.AccountData"),
        ("Enum.AccountData", "Enum.AccountData"),
        ("FutureType", "FutureType"),
        ("function", "function"),
        ("nil", "nil"),
    ] {
        assert_eq!(renderer.lower_type(source)?, target);
    }
    let without_enums = Renderer::new(BTreeSet::new(), 1000)?;
    assert_eq!(without_enums.lower_type("AccountData")?, "AccountData");
    assert_eq!(
        renderer.lower_type("FutureType|nil"),
        Ok("FutureType|nil".into())
    );
    Ok(())
}

#[test]
fn optionality_depends_on_default_presence_not_truthiness() -> TestResult {
    let output = renderer()?.render(&sample(Owner::Global))?;
    assert!(output.contains("---@param enabled? boolean Default = false"));
    assert!(output.contains("---@param count? number Default = 0"));
    assert!(output.contains("---@param items? Enum.AccountData[]"));
    assert!(output.contains("---@return string? result\n---@return Enum.AccountData kind"));
    assert!(output.contains("---@field items Enum.AccountData[]?"));
    assert!(output.contains("---@alias OnResult FunctionContainer|fun(count?: number)"));
    Ok(())
}

#[test]
fn parameter_and_return_order_are_semantic() -> TestResult {
    let renderer = renderer()?;
    let mut system = sample(Owner::Global);
    let before = renderer.render(&system)?;
    system.functions[0].arguments.swap(0, 1);
    system.functions[0].returns.swap(0, 1);
    let after = renderer.render(&system)?;
    assert_ne!(before, after);
    assert!(after.contains("function Inspect(count, enabled, items, ...) end"));
    assert!(after.contains("---@return Enum.AccountData kind\n---@return string? result"));
    Ok(())
}

#[test]
fn absent_and_empty_documentation_remain_distinct() -> TestResult {
    let renderer = renderer()?;
    let mut system = sample(Owner::Global);
    system.functions[0].documentation = None;
    let absent = renderer.render(&system)?;
    system.functions[0].documentation = Some(vec![]);
    let empty = renderer.render(&system)?;
    assert!(absent.starts_with("---@meta _\n---[Documentation]"));
    assert!(empty.starts_with("---@meta _\n---\n---\n---[Documentation]"));
    Ok(())
}

#[test]
fn duplicate_and_nonterminal_varargs_are_rejected() -> TestResult {
    let renderer = renderer()?;
    let mut system = sample(Owner::Global);
    system.functions[0].arguments[0].variadic = true;
    assert_eq!(renderer.render(&system), Err(RenderError::InvalidVariadic));
    system.functions[0].arguments[0].variadic = false;
    system.functions[0].arguments[1].name = "enabled".into();
    assert_eq!(renderer.render(&system), Err(RenderError::DuplicateName));
    Ok(())
}

#[test]
fn fields_and_callbacks_cannot_silently_drop_unsupported_shapes() -> TestResult {
    let renderer = renderer()?;
    let mut system = sample(Owner::Global);
    let mut unsupported = field("items", "table");
    unsupported.inner_type = Some("number".into());
    system.tables = vec![Table::Callback {
        name: "Callback".into(),
        arguments: vec![unsupported],
    }];
    assert_eq!(renderer.render(&system), Err(RenderError::UnsupportedType));
    let mut variadic = field("values", "string");
    variadic.variadic = true;
    system.tables = vec![Table::Structure {
        name: "Record".into(),
        fields: vec![variadic],
    }];
    assert_eq!(renderer.render(&system), Err(RenderError::InvalidVariadic));
    Ok(())
}

#[test]
fn names_and_types_cannot_inject_code() -> TestResult {
    let renderer = renderer()?;
    for name in ["end", "a() end; Inject() --", "a\nb", "../file", "", "а"] {
        let mut system = sample(Owner::Global);
        system.functions[0].name = name.into();
        assert_eq!(
            renderer.render(&system),
            Err(RenderError::InvalidIdentifier)
        );
    }
    for name in ["a\n---@class Injected", "Foo;Run()", "Foo..Bar", "Foo."] {
        assert_eq!(renderer.lower_type(name), Err(RenderError::UnsupportedType));
    }
    Ok(())
}

#[test]
fn documentation_and_defaults_cannot_inject_directives() -> TestResult {
    let renderer = renderer()?;
    for text in [
        "@diagnostic disable",
        "\n---@class Injected",
        "first\rRun()",
        "first\u{2028}Run()",
    ] {
        let mut system = sample(Owner::Global);
        system.functions[0].documentation = Some(vec![text.into()]);
        assert_eq!(
            renderer.render(&system),
            Err(RenderError::UnsafeDocumentation)
        );
    }
    let mut system = sample(Owner::Global);
    system.functions[0].arguments[0].default_text = Some("false\nRun()".into());
    assert_eq!(
        renderer.render(&system),
        Err(RenderError::UnsafeDocumentation)
    );
    Ok(())
}

#[test]
fn bounded_output_never_returns_partial_success() -> TestResult {
    let system = sample(Owner::Global);
    let expected = renderer()?.render(&system)?;
    let enums = BTreeSet::from(["AccountData".into()]);
    assert_eq!(
        Renderer::new(enums.clone(), expected.len())?.render(&system)?,
        expected
    );
    assert_eq!(
        Renderer::new(enums, expected.len() - 1)?.render(&system),
        Err(RenderError::OutputLimit)
    );
    assert_eq!(
        Renderer::new(BTreeSet::new(), 0)?.render(&system),
        Err(RenderError::OutputLimit)
    );
    Ok(())
}

#[test]
fn rendering_is_repeatable_without_external_state() -> TestResult {
    let renderer = renderer()?;
    let system = sample(Owner::Namespace("C_Test".into()));
    assert_eq!(renderer.render(&system)?, renderer.render(&system)?);
    let empty = System {
        owner: Owner::Global,
        functions: vec![],
        tables: vec![],
    };
    assert_eq!(renderer.render(&empty)?, "---@meta _\n");
    Ok(())
}
