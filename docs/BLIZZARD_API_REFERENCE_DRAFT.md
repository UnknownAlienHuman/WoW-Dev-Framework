# Generated API reference draft

The executable producer and wire schema are owned by
[Blizzard API reference](BLIZZARD_API_REFERENCE.md),
`scripts/wow_api_reference.py` and `wow-reference::generated_api`.

The build and verify entrypoints delegate to that single module. They consume
an explicit local Git repository and verified source manifest. Historical
GitHub-only CLI flags and the older `draft_digest` shape are not supported.
Use each entrypoint's `--help` for its actual command surface.

Source JSON permits null, signed and decimal values. Rust uses the dedicated
`wow-reference::wire_json` profile to preserve Python-produced number lexemes;
this does not relax the stricter `wow-core` identity canonicalization contract.
Full implementation scope and remaining work are in
[the status ledger](IMPLEMENTATION_STATUS.md).
