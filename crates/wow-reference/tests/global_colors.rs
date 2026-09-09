//! Static GlobalColors resources retain evidence without executing `CreateColor`.
use std::sync::atomic::AtomicBool;
use wow_reference::native::{NativeErrorCode, source_digest};
use wow_reference::native_aliases::ingest_alias_catalog;

const REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn ingest(text: &str) -> wow_reference::native_aliases::AliasDocument {
    ingest_alias_catalog(
        REVISION,
        "GlobalColors.lua",
        text,
        &source_digest(text.as_bytes()),
        &AtomicBool::new(false),
    )
    .expect("valid global colors")
}

#[test]
fn exact_assignments_keep_number_lexemes_ranges_and_duplicates() {
    let raw = "\u{feff}---@meta _\r\n-- generated evidence\r\nDUPLICATE_COLOR = CreateColor(0.000, 0.500, 1.000, 1.000)\r\nDUPLICATE_COLOR = CreateColor(1, 0, 0, 1)\r\nUNIQUE_COLOR=CreateColor(1e0,0x0,0.25,1)\r\n";
    let document = ingest(raw);
    assert!(document.aliases().is_empty());
    assert!(document.structures().is_empty());
    assert!(document.namespaces().is_empty());
    assert!(document.function_containers().is_empty());
    let colors = document.global_colors();
    assert_eq!(colors.len(), 3);
    assert_eq!(colors[0].name, "DUPLICATE_COLOR");
    assert_eq!(colors[0].components, ["0.000", "0.500", "1.000", "1.000"]);
    assert_eq!(colors[1].components, ["1", "0", "0", "1"]);
    assert_eq!(colors[2].components, ["1e0", "0x0", "0.25", "1"]);
    assert_eq!(
        raw.get(colors[0].span.start..colors[0].span.end),
        Some("DUPLICATE_COLOR = CreateColor(0.000, 0.500, 1.000, 1.000)")
    );
    assert_eq!(
        raw.get(colors[2].span.start..colors[2].span.end),
        Some("UNIQUE_COLOR=CreateColor(1e0,0x0,0.25,1)")
    );
    let value = serde_json::to_value(document).expect("serialize resource");
    assert_eq!(value["schema"], "wow-native-alias-resource/7");
}

#[test]
fn executable_or_noncanonical_color_shapes_fail_closed() {
    for raw in [
        "local COLOR = CreateColor(0, 0, 0, 1)\n",
        "COLOR.r = CreateColor(0, 0, 0, 1)\n",
        "COLOR = Other(0, 0, 0, 1)\n",
        "COLOR = CreateColor(0, 0, 0)\n",
        "COLOR = CreateColor(0, 0, 0, 1, 1)\n",
        "COLOR = CreateColor(-1, 0, 0, 1)\n",
        "COLOR = CreateColor(0, '0', 0, 1)\n",
        "COLOR = CreateColor(0, 0, 0, 1); os.execute('never')\n",
        "---@type colorRGBA\nCOLOR = CreateColor(0, 0, 0, 1)\n",
        "CreateColor(0, 0, 0, 1)\n",
    ] {
        assert!(
            ingest_alias_catalog(
                REVISION,
                "GlobalColors.lua",
                raw,
                &source_digest(raw.as_bytes()),
                &AtomicBool::new(false),
            )
            .is_err(),
            "{raw}"
        );
    }
}

#[test]
fn cancellation_is_preserved_across_statement_profile_dispatch() {
    let raw = "COLOR = CreateColor(0, 0, 0, 1)\n";
    let error = ingest_alias_catalog(
        REVISION,
        "GlobalColors.lua",
        raw,
        &source_digest(raw.as_bytes()),
        &AtomicBool::new(true),
    )
    .expect_err("cancelled input");
    assert_eq!(error.code, NativeErrorCode::Cancelled);
}
