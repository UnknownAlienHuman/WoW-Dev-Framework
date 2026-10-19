from pathlib import Path


def replace(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if text.count(old) != 1:
        raise SystemExit(f'unexpected block in {path}')
    file.write_text(text.replace(old, new))


replace(
    'crates/wow-store/src/project/read.rs',
    '''        validate_membership(&connection, &manifest)?;
        // Every member is validated, not just the record eventually requested.
        for member in &manifest.members {
            checkpoint(stop)?;
            read_partition(&connection, member)?;
        }
        checkpoint(stop)?;
''',
    '''        validate_membership(&connection, &manifest)?;
        // Payloads are validated when record() reads them. Opening a snapshot must
        // not decode and hash every unrelated partition before the caller asks.
        checkpoint(stop)?;
''',
)

replace(
    'docs/IMPLEMENTATION.md',
    '''**#131 replaces the Project read boundary; #169 implements direct SQLite search;
#106 connects the user workflow.** `wow-project/src/replay/publication.rs` reads
through `AcquiredProjectPair::read` → `hydrate_owner`, rebuilding analysis and
recapturing the replay for equality. Even `wow-store/src/project/read.rs` opens
a read by loading every partition and decoding/re-encoding/hashing its JSON;
`record` reads the requested partition again. Bypassing Emmy alone is insufficient.

Start #131 with stored file paths/text and the selected source/flavor: open in a
later process and read a file without source/analyzer callbacks or a whole-index
payload validation pass. Use ordinary SQLite rows/transactions in `wow-store`;
do not serialize another snapshot framework. #169 adds name/span and FTS queries
to this same index. Existing in-memory Project updates can stay while their
readers move; updating source and reading facts have different work to do.
''',
    '''**#131 replaces the Project read boundary; #169 implements direct SQLite search;
#106 connects the user workflow.** `wow-project/src/replay/publication.rs` still
reads through `AcquiredProjectPair::read` → `hydrate_owner`, rebuilding analysis
and recapturing replay for equality. `ReadSnapshot::acquire` now validates the
selected manifest/membership without loading every partition; `record` validates
only the requested payload. Removing Project/Emmy reconstruction remains the
larger boundary.

Continue #131 with stored file paths/text and selected source/flavor: open in a
later process and read a file without source/analyzer callbacks. Use ordinary
SQLite rows/transactions in `wow-store`; do not serialize another snapshot
framework. #169 adds name/span and FTS queries to this same index. Existing
in-memory Project updates can stay while readers move; updating source and
reading facts have different work to do.
''',
)

replace(
    'docs/REFACTOR_QUEUE.md',
    '| 3 | [#131 — current index](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/131) | Reopen stored file/text facts without rehydration or all-partition decode/hash passes. Coordinate #194. |\n',
    '| 3 | [#131 — current index](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/131) | Eager all-partition payload reads are removed; store/read file text directly without Project/Emmy rehydration. |\n',
)

replace(
    'crates/wow-store/README.md',
    '''Project indexes are derived from source and configuration, so one current format
can be rebuilt. On unsupported/corrupt derived state, give a clear rebuild/error
path; never delete or rewrite user source/configuration.
''',
    '''Project indexes are derived from source and configuration, so one current format
can be rebuilt. Snapshot open validates manifest membership; requested records
validate their own payload on read instead of scanning every partition. On
unsupported/corrupt derived state, give a clear rebuild/error path; never delete
or rewrite user source/configuration.
''',
)
