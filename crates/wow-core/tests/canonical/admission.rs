use std::cell::Cell;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use serde::Serialize;
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct};
use serde_json::{Value, json};
use wow_core::{
    CoreErrorCode, CoreResult, E0CheckResultEnvelope, E0OperationErrorEnvelope, StableHandleId,
    canonical_json_bytes, canonical_json_string, domain_separated_digest,
};

type TestResult = Result<(), Box<dyn Error>>;

// A real SerializeMap emitter, not a Value in which repeats have already vanished.
struct Entries<'a>(&'a [(&'a str, u64)]);

impl Serialize for Entries<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (key, value) in self.0 {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

fn rejected<T>(result: CoreResult<T>, code: CoreErrorCode) -> TestResult {
    let Err(error) = result else {
        return Err("invalid material was accepted".into());
    };
    assert_eq!(error.code(), code);
    error.validate()?;
    Ok(())
}

#[test]
fn canonical_001_duplicate_map_keys_never_collapse_even_when_values_match() -> TestResult {
    for second in [1, 2] {
        let values = [("field", 1), ("field", second)];
        let duplicate = Entries(&values);
        rejected(
            canonical_json_bytes(&duplicate),
            CoreErrorCode::DuplicateField,
        )?;
        rejected(
            canonical_json_string(&duplicate),
            CoreErrorCode::DuplicateField,
        )?;
        rejected(
            domain_separated_digest("wow-core/test/e0-1", &duplicate),
            CoreErrorCode::DuplicateField,
        )?;
        rejected(
            StableHandleId::derive(&duplicate),
            CoreErrorCode::DuplicateField,
        )?;
    }
    Ok(())
}

#[test]
fn canonical_002_nested_duplicate_maps_reject_in_every_container() -> TestResult {
    #[derive(Serialize)]
    struct Wrapped<T> {
        inner: T,
    }
    #[derive(Serialize)]
    struct Newtype<T>(T);
    #[derive(Serialize)]
    enum Variant<T> {
        Single(T),
        Tuple(u64, T),
        Struct { inner: T },
    }
    let entries = [("same", 1), ("same", 2)];
    let duplicate = Entries(&entries);
    rejected(
        canonical_json_bytes(&[&duplicate]),
        CoreErrorCode::DuplicateField,
    )?;
    rejected(
        canonical_json_bytes(&(&duplicate, true)),
        CoreErrorCode::DuplicateField,
    )?;
    rejected(
        canonical_json_bytes(&Some(&duplicate)),
        CoreErrorCode::DuplicateField,
    )?;
    rejected(
        canonical_json_bytes(&Wrapped { inner: &duplicate }),
        CoreErrorCode::DuplicateField,
    )?;
    rejected(
        canonical_json_bytes(&Newtype(&duplicate)),
        CoreErrorCode::DuplicateField,
    )?;
    rejected(
        canonical_json_bytes(&Variant::Single(&duplicate)),
        CoreErrorCode::DuplicateField,
    )?;
    rejected(
        canonical_json_bytes(&Variant::Tuple(1, &duplicate)),
        CoreErrorCode::DuplicateField,
    )?;
    rejected(
        canonical_json_bytes(&Variant::Struct { inner: &duplicate }),
        CoreErrorCode::DuplicateField,
    )?;
    Ok(())
}

#[test]
fn canonical_003_flattened_fields_cannot_overwrite_declared_fields() -> TestResult {
    #[derive(Serialize)]
    struct Flattened {
        field: u64,
        #[serde(flatten)]
        extras: BTreeMap<String, u64>,
    }
    for second in [1, 2] {
        let value = Flattened {
            field: 1,
            extras: [("field".to_owned(), second)].into(),
        };
        rejected(canonical_json_bytes(&value), CoreErrorCode::DuplicateField)?;
    }
    let valid = Flattened {
        field: 1,
        extras: [("other".to_owned(), 2)].into(),
    };
    assert_eq!(canonical_json_bytes(&valid)?, br#"{"field":1,"other":2}"#);
    Ok(())
}

#[test]
fn canonical_004_json_key_spelling_collisions_reject_before_hashing() -> TestResult {
    struct KeyCollision(u8);
    impl Serialize for KeyCollision {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut map = serializer.serialize_map(Some(2))?;
            match self.0 {
                0 => {
                    map.serialize_entry(&1_u8, &1)?;
                    map.serialize_entry("1", &2)?;
                }
                1 => {
                    map.serialize_entry(&true, &1)?;
                    map.serialize_entry("true", &2)?;
                }
                _ => {
                    map.serialize_entry(&'x', &1)?;
                    map.serialize_entry("x", &2)?;
                }
            }
            map.end()
        }
    }
    for kind in 0..3 {
        rejected(
            canonical_json_bytes(&KeyCollision(kind)),
            CoreErrorCode::DuplicateField,
        )?;
    }
    Ok(())
}

#[test]
fn canonical_005_duplicate_struct_fields_reject_without_last_value_wins() -> TestResult {
    struct RepeatedField;
    impl Serialize for RepeatedField {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut record = serializer.serialize_struct("RepeatedField", 2)?;
            record.serialize_field("value", &1)?;
            record.serialize_field("value", &2)?;
            record.end()
        }
    }
    rejected(
        canonical_json_bytes(&RepeatedField),
        CoreErrorCode::DuplicateField,
    )
}

#[test]
fn canonical_006_custom_serializer_errors_do_not_echo_source_payloads() -> TestResult {
    const PAYLOAD: &str = "SYNTHETIC_SOURCE_PAYLOAD /host/private/addon.lua untrusted prose";
    struct Failure;
    impl Serialize for Failure {
        fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom(PAYLOAD))
        }
    }
    for result in [
        canonical_json_bytes(&Failure),
        canonical_json_bytes(&[Failure]),
    ] {
        let Err(error) = result else {
            return Err("serializer failure was accepted".into());
        };
        assert_eq!(error.code(), CoreErrorCode::CanonicalizationFailure);
        error.validate()?;
        assert!(!serde_json::to_string(&error)?.contains(PAYLOAD));
        assert!(!format!("{error:?}").contains(PAYLOAD));
        assert!(!error.to_string().contains(PAYLOAD));
    }
    Ok(())
}

#[test]
fn canonical_007_custom_error_prose_is_not_even_formatted() -> TestResult {
    struct Message<'a>(&'a Cell<usize>);
    impl fmt::Display for Message<'_> {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.set(self.0.get() + 1);
            formatter.write_str("untrusted diagnostic")
        }
    }
    struct Failure<'a>(&'a Cell<usize>);
    impl Serialize for Failure<'_> {
        fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom(Message(self.0)))
        }
    }
    let calls = Cell::new(0);
    rejected(
        canonical_json_bytes(&Failure(&calls)),
        CoreErrorCode::CanonicalizationFailure,
    )?;
    assert_eq!(calls.get(), 0);
    Ok(())
}

#[test]
fn canonical_008_scalars_keep_exact_escaping_unicode_and_unsigned_boundaries() -> TestResult {
    #[derive(Serialize)]
    struct Scalars<'a> {
        z: u64,
        a: &'a str,
        enabled: bool,
        character: char,
    }
    let value = Scalars {
        z: u64::MAX,
        a: "line\n\"\\\té",
        enabled: true,
        character: '字',
    };
    let expected = "{\"a\":\"line\\n\\\"\\\\\\té\",\"character\":\"字\",\"enabled\":true,\"z\":18446744073709551615}";
    assert_eq!(canonical_json_string(&value)?, expected);
    assert_eq!(canonical_json_bytes(&0_u64)?, b"0");
    assert_eq!(canonical_json_bytes(&i64::MAX)?, b"9223372036854775807");
    assert_eq!(
        canonical_json_bytes(&u128::from(u64::MAX))?,
        b"18446744073709551615"
    );
    assert_eq!(
        canonical_json_bytes(&i128::from(u64::MAX))?,
        b"18446744073709551615"
    );
    Ok(())
}

#[test]
fn canonical_009_null_negative_float_and_oversized_numbers_are_not_hashable() -> TestResult {
    for value in [json!(null), json!(-1), json!(0.0), json!(-0.0), json!(1.5)] {
        rejected(
            canonical_json_bytes(&value),
            CoreErrorCode::CanonicalizationFailure,
        )?;
        rejected(
            canonical_json_bytes(&[value]),
            CoreErrorCode::CanonicalizationFailure,
        )?;
    }
    rejected(
        canonical_json_bytes(&None::<u8>),
        CoreErrorCode::CanonicalizationFailure,
    )?;
    rejected(
        canonical_json_bytes(&()),
        CoreErrorCode::CanonicalizationFailure,
    )?;
    rejected(
        canonical_json_bytes(&u128::MAX),
        CoreErrorCode::CanonicalizationFailure,
    )?;
    rejected(
        canonical_json_bytes(&i128::MIN),
        CoreErrorCode::CanonicalizationFailure,
    )?;
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        rejected(
            canonical_json_bytes(&value),
            CoreErrorCode::CanonicalizationFailure,
        )?;
    }
    rejected(
        canonical_json_bytes(&1.0_f32),
        CoreErrorCode::CanonicalizationFailure,
    )?;
    assert_eq!(canonical_json_bytes(&Some(4_u8))?, b"4");
    Ok(())
}

#[test]
fn canonical_010_all_serde_container_forms_preserve_the_existing_wire_model() -> TestResult {
    #[derive(Serialize)]
    struct Newtype(u8);
    #[derive(Serialize)]
    struct Tuple(u8, bool);
    #[derive(Serialize)]
    struct Empty {}
    #[derive(Serialize)]
    enum Enum {
        Unit,
        Newtype(u8),
        Tuple(u8, bool),
        Struct { z: u8, a: bool },
    }
    #[derive(Serialize)]
    #[serde(tag = "kind", content = "data")]
    enum Tagged {
        Item(u8),
    }
    #[derive(Serialize)]
    #[serde(untagged)]
    enum Untagged {
        Number(u8),
    }
    assert_eq!(canonical_json_bytes(&Newtype(3))?, b"3");
    assert_eq!(canonical_json_bytes(&Tuple(3, false))?, b"[3,false]");
    assert_eq!(canonical_json_bytes(&(3, false))?, b"[3,false]");
    assert_eq!(canonical_json_bytes(&Empty {})?, b"{}");
    assert_eq!(canonical_json_bytes(&Vec::<u8>::new())?, b"[]");
    assert_eq!(canonical_json_bytes(&Enum::Unit)?, br#""Unit""#);
    assert_eq!(
        canonical_json_bytes(&Enum::Newtype(3))?,
        br#"{"Newtype":3}"#
    );
    assert_eq!(
        canonical_json_bytes(&Enum::Tuple(3, false))?,
        br#"{"Tuple":[3,false]}"#
    );
    assert_eq!(
        canonical_json_bytes(&Enum::Struct { z: 3, a: true })?,
        br#"{"Struct":{"a":true,"z":3}}"#
    );
    assert_eq!(
        canonical_json_bytes(&Tagged::Item(3))?,
        br#"{"data":3,"kind":"Item"}"#
    );
    assert_eq!(canonical_json_bytes(&Untagged::Number(3))?, b"3");
    Ok(())
}

#[test]
fn canonical_011_json_keys_keep_previous_spelling_and_bytewise_order() -> TestResult {
    let integers: BTreeMap<i64, u64> = [(-1, 1), (20, 2), (3, 3)].into();
    assert_eq!(
        canonical_json_bytes(&integers)?,
        br#"{"-1":1,"20":2,"3":3}"#
    );
    let booleans: BTreeMap<bool, u8> = [(true, 1), (false, 0)].into();
    assert_eq!(canonical_json_bytes(&booleans)?, br#"{"false":0,"true":1}"#);
    let utf8 = Entries(&[("字", 1), ("é", 2), ("a", 3)]);
    assert_eq!(canonical_json_string(&utf8)?, "{\"a\":3,\"é\":2,\"字\":1}");
    // Key spelling is exact; Unicode normalization is not semantic identity.
    let distinct = Entries(&[("é", 1), ("e\u{301}", 2)]);
    let result: Value = serde_json::from_slice(&canonical_json_bytes(&distinct)?)?;
    assert_eq!(result.as_object().ok_or("object")?.len(), 2);
    Ok(())
}

#[test]
fn canonical_012_invalid_keys_fail_without_echo_or_serializing_their_values() -> TestResult {
    for key in ["", "line\nkey", "nul\0key"] {
        rejected(
            canonical_json_bytes(&Entries(&[(key, 1)])),
            CoreErrorCode::CanonicalizationFailure,
        )?;
    }
    #[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
    struct CompositeKey {
        field: u8,
    }
    let map: BTreeMap<CompositeKey, u8> = [(CompositeKey { field: 1 }, 2)].into();
    rejected(
        canonical_json_bytes(&map),
        CoreErrorCode::CanonicalizationFailure,
    )?;
    struct Unread<'a>(&'a Cell<usize>);
    impl Serialize for Unread<'_> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.0.set(self.0.get() + 1);
            serializer.serialize_u8(2)
        }
    }
    struct InvalidKey<'a>(&'a Cell<usize>);
    impl Serialize for InvalidKey<'_> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut map = serializer.serialize_map(Some(1))?;
            map.serialize_entry("bad\nkey", &Unread(self.0))?;
            map.end()
        }
    }
    let calls = Cell::new(0);
    rejected(
        canonical_json_bytes(&InvalidKey(&calls)),
        CoreErrorCode::CanonicalizationFailure,
    )?;
    assert_eq!(calls.get(), 0);
    Ok(())
}

#[test]
fn canonical_013_malformed_map_protocol_returns_errors_instead_of_partial_objects() -> TestResult {
    struct Broken(u8);
    impl Serialize for Broken {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut map = serializer.serialize_map(None)?;
            match self.0 {
                0 => map.serialize_value(&1)?,
                1 => map.serialize_key("missing")?,
                _ => {
                    map.serialize_key("first")?;
                    map.serialize_key("second")?;
                }
            }
            map.end()
        }
    }
    for kind in 0..3 {
        rejected(
            canonical_json_bytes(&Broken(kind)),
            CoreErrorCode::CanonicalizationFailure,
        )?;
    }
    Ok(())
}

#[test]
fn canonical_014_values_are_serialized_once_and_length_hints_do_not_allocate() -> TestResult {
    struct Counted<'a>(&'a Cell<usize>);
    impl Serialize for Counted<'_> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            self.0.set(self.0.get() + 1);
            serializer.serialize_u8(1)
        }
    }
    struct Sequence<'a>(&'a Cell<usize>);
    impl Serialize for Sequence<'_> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut sequence = serializer.serialize_seq(Some(usize::MAX))?;
            sequence.serialize_element(&Counted(self.0))?;
            sequence.end()
        }
    }
    struct MapHint;
    impl Serialize for MapHint {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut map = serializer.serialize_map(Some(usize::MAX))?;
            map.serialize_entry("a", &1)?;
            map.end()
        }
    }
    let calls = Cell::new(0);
    assert_eq!(canonical_json_bytes(&Sequence(&calls))?, b"[1]");
    assert_eq!(calls.get(), 1);
    assert_eq!(canonical_json_bytes(&MapHint)?, br#"{"a":1}"#);
    Ok(())
}

#[test]
fn canonical_015_seeded_order_and_worker_count_do_not_change_bytes_or_digests() -> TestResult {
    let original = [("a", 1), ("b", 2), ("c", 3), ("d", 4)];
    let expected = br#"{"a":1,"b":2,"c":3,"d":4}"#;
    let expected_hash = domain_separated_digest("wow-core/test/e0-1", &Entries(&original))?;
    for seed in 0_u64..64 {
        let mut entries = original;
        let mut state = seed;
        for index in (1..entries.len()).rev() {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            let other = usize::try_from(state % u64::try_from(index + 1)?)?;
            entries.swap(index, other);
        }
        assert_eq!(
            canonical_json_bytes(&Entries(&entries))?,
            expected,
            "seed {seed}"
        );
        assert_eq!(
            domain_separated_digest("wow-core/test/e0-1", &Entries(&entries))?,
            expected_hash
        );
    }
    for workers in [1, 2, 4] {
        let outputs = std::thread::scope(|scope| {
            let handles = (0..workers)
                .map(|_| scope.spawn(|| canonical_json_bytes(&Entries(&original))))
                .collect::<Vec<_>>();
            handles
                .into_iter()
                .map(|handle| handle.join())
                .collect::<Vec<_>>()
        });
        for output in outputs {
            assert_eq!(output.map_err(|_| "worker panicked")??, expected);
        }
    }
    assert_ne!(
        canonical_json_bytes(&[1, 2])?,
        canonical_json_bytes(&[2, 1])?
    );
    Ok(())
}

#[test]
fn canonical_016_all_committed_envelopes_match_independent_raw_json_encoding() -> TestResult {
    for golden in [
        include_str!("../../examples/e0-clean-result.json"),
        include_str!("../../examples/e0-findings-result.json"),
        include_str!("../../examples/e0-not-evaluated-result.json"),
        include_str!("../../examples/e0-conflict-not-evaluated-result.json"),
    ] {
        let raw: Value = serde_json::from_str(golden)?;
        // Independent encoding: do not build this oracle with the function under test.
        let expected = serde_json::to_vec(&raw)?;
        let envelope: E0CheckResultEnvelope = serde_json::from_str(golden)?;
        assert_eq!(canonical_json_bytes(&raw)?, expected);
        assert_eq!(envelope.canonical_bytes()?, expected);
        let crlf: E0CheckResultEnvelope = serde_json::from_str(&golden.replace('\n', "\r\n"))?;
        assert_eq!(crlf.canonical_bytes()?, expected);
    }
    let golden = include_str!("../../examples/e0-generation-mismatch-error.json");
    let raw: Value = serde_json::from_str(golden)?;
    let envelope: E0OperationErrorEnvelope = serde_json::from_str(golden)?;
    assert_eq!(envelope.canonical_bytes()?, serde_json::to_vec(&raw)?);
    Ok(())
}

#[test]
fn canonical_017_feature_unified_number_values_keep_scalar_semantics() -> TestResult {
    // Execute both as a core-only target and in the workspace, whose reference
    // owner enables arbitrary_precision. Never parse large numbers through f64.
    for (text, expected) in [
        ("0", "0"),
        ("9007199254740993", "9007199254740993"),
        ("18446744073709551615", "18446744073709551615"),
    ] {
        let number: serde_json::Number = serde_json::from_str(text)?;
        assert_eq!(canonical_json_string(&number)?, expected);
        let nested: Value = serde_json::from_str(&format!("{{\"count\":{text}}}"))?;
        assert_eq!(
            canonical_json_string(&nested)?,
            format!("{{\"count\":{expected}}}")
        );
    }
    for text in ["-1", "1.0", "1e1", "18446744073709551616"] {
        if let Ok(value) = serde_json::from_str::<Value>(text) {
            rejected(
                canonical_json_bytes(&value),
                CoreErrorCode::CanonicalizationFailure,
            )?;
        }
    }
    // serde_json's arbitrary_precision parser converts the lexical -0 token
    // into an integer zero BEFORE this serialization API sees it. Without that
    // feature it preserves a floating negative zero. Assert the precise boundary,
    // rather than pretending a serializer can recover discarded lexical data.
    let parsed_zero: Value = serde_json::from_str("-0")?;
    if parsed_zero.as_u64() == Some(0) {
        assert_eq!(serde_json::to_vec(&parsed_zero)?, b"0");
        assert_eq!(canonical_json_bytes(&parsed_zero)?, b"0");
    } else {
        assert!(parsed_zero.is_f64());
        assert_eq!(serde_json::to_vec(&parsed_zero)?, b"-0.0");
        rejected(
            canonical_json_bytes(&parsed_zero),
            CoreErrorCode::CanonicalizationFailure,
        )?;
    }
    // When a noncanonical numeric token IS retained by the Number protocol,
    // it must fail before delegation could normalize it. This also tests the
    // marker with both feature profiles, without using an unchecked Number API.
    struct NumberText<'a>(&'a str);
    impl Serialize for NumberText<'_> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut number = serializer.serialize_struct("$serde_json::private::Number", 1)?;
            number.serialize_field("$serde_json::private::Number", self.0)?;
            number.end()
        }
    }
    for text in [
        "-1",
        "-0",
        "+1",
        "01",
        "1.0",
        "1e1",
        "18446744073709551616",
        "",
    ] {
        rejected(
            canonical_json_bytes(&NumberText(text)),
            CoreErrorCode::CanonicalizationFailure,
        )?;
    }
    // A user map key is not a serde_json scalar protocol marker.
    let map = json!({"$serde_json::private::Number": "42"});
    assert_eq!(
        canonical_json_bytes(&map)?,
        br#"{"$serde_json::private::Number":"42"}"#
    );
    Ok(())
}

#[test]
fn canonical_018_raw_json_protocol_cannot_bypass_duplicate_admission() -> TestResult {
    // RawValue's protocol is exercised without enabling a new project feature.
    struct Raw;
    impl Serialize for Raw {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut record = serializer.serialize_struct("$serde_json::private::RawValue", 1)?;
            record.serialize_field("$serde_json::private::RawValue", r#"{"a":1,"a":2}"#)?;
            record.end()
        }
    }
    rejected(
        canonical_json_bytes(&Raw),
        CoreErrorCode::CanonicalizationFailure,
    )
}

#[test]
fn canonical_019_bytes_display_and_skipped_optionals_keep_the_json_contract() -> TestResult {
    struct Bytes;
    impl Serialize for Bytes {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_bytes(&[0, 128, 255])
        }
    }
    struct DisplayValue;
    impl Serialize for DisplayValue {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_str(&format_args!("value-{}", 3))
        }
    }
    #[derive(Serialize)]
    struct Optional {
        present: u8,
        #[serde(skip_serializing_if = "Option::is_none")]
        absent: Option<u8>,
    }
    assert_eq!(canonical_json_bytes(&Bytes)?, b"[0,128,255]");
    assert_eq!(canonical_json_bytes(&DisplayValue)?, br#""value-3""#);
    assert_eq!(
        canonical_json_bytes(&Optional {
            present: 1,
            absent: None
        })?,
        br#"{"present":1}"#
    );
    Ok(())
}
