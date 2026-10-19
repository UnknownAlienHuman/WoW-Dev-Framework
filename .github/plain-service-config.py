from pathlib import Path

path = Path('crates/wow-service/src/configuration.rs')
text = path.read_text()

replacements = {
    '/// Immutable, content-addressed E0 service configuration.\n':
        '/// Immutable service configuration with an identity over operational fields.\n',
    "    schema: &'static str,\n": '',
    "    service_version: &'static str,\n": '',
    "    result_schema: &'static str,\n": '',
    '            schema: SERVICE_SCHEMA,\n': '',
    '            service_version: SERVICE_VERSION,\n': '',
    '            result_schema: RESULT_SCHEMA,\n': '',
}

for old, new in replacements.items():
    count = text.count(old)
    expected = 2 if old != '/// Immutable, content-addressed E0 service configuration.\n' else 1
    if count != expected:
        raise SystemExit(f'unexpected count for {old!r}: {count}, expected {expected}')
    text = text.replace(old, new)

path.write_text(text)
