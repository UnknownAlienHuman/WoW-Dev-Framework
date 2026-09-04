from pathlib import Path


def replace_once(source: str, old: str, new: str, label: str) -> str:
    count = source.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return source.replace(old, new, 1)


path = Path("crates/wow-core/src/envelope.rs")
source = path.read_text(encoding="utf-8")
source = replace_once(
    source,
    '''        self.coverage_records
            .sort_by_cached_key(|record| record_id(record, "coverage_id").unwrap_or_default());''',
    '''        self.coverage_records
            .sort_by_cached_key(coverage_sort_key_lossless);''',
    "draft coverage ordering",
)
source = replace_once(
    source,
    '''    envelope
        .coverage_records
        .sort_by_key(crate::CoverageRecord::coverage_id);''',
    '''    envelope
        .coverage_records
        .sort_by_cached_key(coverage_sort_key_lossless);''',
    "envelope coverage ordering",
)
marker = '''fn summary_sort_key_lossless(record: &crate::CapabilitySummary) -> Vec<u8> {
    canonical_json_bytes(record).unwrap_or_default()
}
'''
helper = '''fn coverage_sort_key_lossless(
    record: &crate::CoverageRecord,
) -> (String, String, String, String, String) {
    (
        record.capability_id().as_str().to_owned(),
        record.partition_id().canonical(),
        record.producer_id().as_str().to_owned(),
        record.producer_version().to_string(),
        record.coverage_id().canonical(),
    )
}

'''
source = replace_once(
    source,
    marker,
    helper + marker,
    "coverage ordering helper",
)
path.write_text(source, encoding="utf-8")
