use super::*;
use serde_json::json;

fn table(start: usize, end: usize, fields: Vec<Value>) -> Value {
    json!({"kind":{"Table":fields},"span":{"start":start,"end":end}})
}

fn named(name: &str, value: Value) -> Value {
    json!({"key":{"Name":name},"value":value})
}

fn mapping(role: &str, generated: (usize, usize), source: (usize, usize)) -> Value {
    json!({"granularity":role,
        "generated":{"start":generated.0,"end":generated.1},
        "source":{"path":"API.lua","sha256":"same-source",
            "span":{"start":source.0,"end":source.1}}})
}

fn fixture() -> (Value, Value) {
    let argument = table(1, 9, vec![]);
    let result = table(91, 99, vec![]);
    let raw = table(
        40,
        120,
        vec![
            named(
                "Arguments",
                table(50, 60, vec![json!({"key":{"Index":1},"value":argument})]),
            ),
            named(
                "Returns",
                table(70, 80, vec![json!({"key":{"Index":1},"value":result})]),
            ),
        ],
    );
    (
        json!({"registrations":[{"value":raw}]}),
        json!({"text":"éabcdefghijkl","mappings":[
            mapping("declaration", (0, 14), (40, 120)),
            mapping("parameter", (2, 5), (1, 9)),
            mapping("return", (6, 9), (91, 99))
        ]}),
    )
}

#[test]
fn exact_local_table_members_do_not_require_parent_source_containment() -> Result<()> {
    let (source, file) = fixture();
    let sources = BTreeMap::from([("API.lua", &source)]);
    let tables = source_tables(&sources, true)?;
    let mut count = 0;
    verify(&file, true, &tables, &mut count)?;
    assert_eq!(count, 3);
    Ok(())
}

#[test]
fn incomplete_reordered_foreign_and_invalid_ranges_reject() -> Result<()> {
    let (source, file) = fixture();
    let sources = BTreeMap::from([("API.lua", &source)]);
    let tables = source_tables(&sources, true)?;
    for index in 0..3 {
        let mut changed = file.clone();
        changed["mappings"].as_array_mut().ok_or("maps")?.remove(index);
        assert!(verify(&changed, true, &tables, &mut 0).is_err());
    }
    for (pointer, replacement) in [
        ("/mappings/1/granularity", json!("field")),
        ("/mappings/1/source/path", json!("foreign.lua")),
        ("/mappings/1/source/sha256", json!("other-source")),
        ("/mappings/1/source/span/start", json!(2)),
        ("/mappings/1/generated/start", json!(1)),
        ("/mappings/1/generated/end", json!(20)),
        ("/mappings/2/generated/start", json!(4)),
        ("/mappings/2/generated/end", json!(6)),
    ] {
        let mut changed = file.clone();
        *changed.pointer_mut(pointer).ok_or("mutation target")? = replacement;
        assert!(verify(&changed, true, &tables, &mut 0).is_err(), "{pointer}");
    }
    let mut changed = file.clone();
    let maps = changed["mappings"].as_array_mut().ok_or("maps")?;
    maps.swap(1, 2);
    assert!(verify(&changed, true, &tables, &mut 0).is_err());
    let mut changed = file.clone();
    let maps = changed["mappings"].as_array_mut().ok_or("maps")?;
    maps.push(maps[2].clone());
    assert!(verify(&changed, true, &tables, &mut 0).is_err());
    let mut count = MAX_MAPPINGS;
    assert!(verify(&file, true, &tables, &mut count).is_err());
    Ok(())
}

#[test]
fn profile_and_whole_literal_maps_remain_distinct() -> Result<()> {
    assert!(profile(&json!({"source_map_profile":PROFILE}))?);
    assert!(!profile(&json!({}))?);
    for value in [Value::Null, json!(false), json!("unknown")] {
        assert!(profile(&json!({"source_map_profile":value})).is_err());
    }
    let file = json!({"text":"abc","mappings":[
        mapping("literal_file", (0, 3), (1, 9)),
        mapping("literal_file", (0, 3), (40, 120))
    ]});
    verify(&file, true, &Tables::new(), &mut 0)?;
    let mut changed = file.clone();
    changed["mappings"][0]["generated"]["end"] = json!(2);
    assert!(verify(&changed, true, &Tables::new(), &mut 0).is_err());
    let empty = json!({"text":"abc","mappings":[]});
    assert!(verify(&empty, true, &Tables::new(), &mut 0).is_err());
    verify(&empty, false, &Tables::new(), &mut 0)?;
    Ok(())
}
