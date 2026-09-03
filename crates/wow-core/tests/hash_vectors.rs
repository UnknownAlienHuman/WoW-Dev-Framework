use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use wow_core::canonical_json_string;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HashVectors {
    canonicalization_version: String,
    digest_algorithm: String,
    notes: Vec<String>,
    vectors: Vec<HashVector>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HashVector {
    canonical_utf8: String,
    domain: String,
    sha256: String,
    #[serde(default)]
    typed_id: Option<String>,
    value: Value,
    vector_id: String,
}

#[test]
fn committed_hash_vectors_are_byte_exact() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = include_str!("../examples/HASH_VECTORS.json");
    let vectors: HashVectors = serde_json::from_str(fixture)?;
    assert_eq!(vectors.canonicalization_version, "wow-core-json/e0-1");
    assert_eq!(vectors.digest_algorithm, "sha256");
    assert!(!vectors.notes.is_empty());
    assert!(!vectors.vectors.is_empty());

    for vector in vectors.vectors {
        let material = serde_json::json!({
            "domain": vector.domain,
            "value": vector.value,
        });
        let actual_canonical = canonical_json_string(&material)?;
        assert_eq!(
            actual_canonical, vector.canonical_utf8,
            "canonical mismatch for {}",
            vector.vector_id
        );
        let actual_digest = encode_hex(&Sha256::digest(actual_canonical.as_bytes()));
        assert_eq!(
            actual_digest, vector.sha256,
            "digest mismatch for {}",
            vector.vector_id
        );
        if let Some(typed_id) = vector.typed_id {
            assert!(
                typed_id.ends_with(&actual_digest),
                "typed ID mismatch for {}",
                vector.vector_id
            );
        }
    }
    Ok(())
}

fn encode_hex(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(TABLE[usize::from(byte >> 4)]));
        output.push(char::from(TABLE[usize::from(byte & 0x0f)]));
    }
    output
}
