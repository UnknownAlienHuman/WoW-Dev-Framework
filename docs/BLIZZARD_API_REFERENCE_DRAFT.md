# Generated API reference draft compatibility

`wow-reference::generated_api` retains the v1 wire importer for existing artifacts.
The old producer/verification entrypoints are retired. Current native generation
uses the source model and Ketho renderer described in
[Blizzard API input](BLIZZARD_API_REFERENCE.md); it emits a different native report
and is not a renamed implementation of the v1 JSON producer.

Source wire JSON permits null, signed and decimal values. The dedicated
`wow-reference::wire_json` profile retains numeric lexemes during digest checks;
it does not relax the stricter core-ID canonicalization contract. Rust unit and
CLI fixtures exercise this compatibility boundary with no external interpreter.
See [implementation status](IMPLEMENTATION_STATUS.md) for the remaining scope.
