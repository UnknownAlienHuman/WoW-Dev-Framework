//! Every fixture id referenced by a recognizer pack must exist in a frozen E2-B
//! fixture file. A fabricated id would silently pass pack validation while
//! claiming evidence that no fixture provides, so this test fails closed.

use std::collections::BTreeSet;
use std::error::Error;

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

fn fixture_ids(path: &str) -> TestResult<BTreeSet<String>> {
    let bytes = std::fs::read(format!(
        "{}/e2/examples/{}",
        env!("CARGO_MANIFEST_DIR"),
        path
    ))?;
    let mut ids = BTreeSet::new();
    for value in bytes
        .split(|byte| !byte.is_ascii_alphanumeric() && *byte != b'-' && *byte != b'_')
        .filter(|part| !part.is_empty())
    {
        if value.starts_with(b"RECOG-") && value.len() > "RECOG-X-0".len() {
            ids.insert(String::from_utf8_lossy(value).to_string());
        }
    }
    Ok(ids)
}

fn rule_fixture_ids() -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for file in [
        "source_bridge.rs",
        "source_construction.rs",
        "source_mixins.rs",
        "source_scripts.rs",
        "source_signals.rs",
        "source_state.rs",
    ] {
        let path = format!("{}/src/{}", env!("CARGO_MANIFEST_DIR"), file);
        let source = match std::fs::read_to_string(&path) {
            Ok(source) => source,
            Err(_) => continue,
        };
        for token in source.split(['"', ',', ' ', '\n', '\t', '[', ']', '(']) {
            if token.starts_with("RECOG-") && token.len() == "RECOG-EVENT-001".len() {
                ids.insert(token.to_owned());
            }
        }
    }
    ids
}

#[test]
fn every_pack_fixture_id_exists_in_a_frozen_fixture_file() -> TestResult {
    let mut declared = BTreeSet::new();
    for name in [
        "match-cases.json",
        "ambiguity-cases.json",
        "mutation-cases.json",
        "graph-output.json",
        "core-pack.json",
        "fact-bundle.json",
    ] {
        declared.extend(fixture_ids(name)?);
    }

    let referenced = rule_fixture_ids();
    assert!(
        !referenced.is_empty(),
        "no fixture ids were collected from the recognizer sources"
    );

    let mut missing = referenced
        .difference(&declared)
        .cloned()
        .collect::<Vec<_>>();
    missing.sort();
    assert!(
        missing.is_empty(),
        "recognizer packs reference fixture ids that no fixture provides: {}",
        missing.join(", ")
    );

    // A pack array must not repeat one fixture id, which would inflate apparent
    // coverage without adding evidence.
    let mut seen = BTreeSet::new();
    for id in &referenced {
        assert!(
            seen.insert(id.clone()),
            "duplicate fixture id in packs: {id}"
        );
    }
    Ok(())
}

#[test]
fn frozen_match_cases_carry_a_non_empty_rule_id() -> TestResult {
    let bytes = std::fs::read(format!(
        "{}/e2/examples/match-cases.json",
        env!("CARGO_MANIFEST_DIR")
    ))?;
    let text = String::from_utf8(bytes)?;
    let cases = text.matches("\"case_id\"").count();
    let rules = text.matches("\"rule_id\"").count();
    assert_eq!(
        cases, rules,
        "each frozen match case must name exactly one rule_id"
    );
    Ok(())
}
