# Security policy

The tool parses addon, Blizzard and annotation source as data. It never executes
Lua, repository hooks or source-provided commands. XML entities/schema URLs do not
cause filesystem or network access. Reads stay inside configured roots; file
counts, input sizes, nesting, protocol messages and output are bounded.

Index/library cleanup affects only identified tool-owned output. It must never
rewrite addon source, user configuration or arbitrary paths remembered in an old
database. Use ordinary SQLite transactions and parameterized queries. Keep private
credentials out of generated output and protocol messages; useful local diagnostic
paths are not a license to disclose unrelated files.

WoW Secret Values and protected/tainted execution are game semantics, separate
from the tool's own access boundaries. Static checks do not certify game-runtime
safety; see [WoW behavior](docs/WOW_LINTER.md).

Report a suspected vulnerability privately through GitHub private vulnerability
reporting/security advisories when available. Include the affected commit,
minimal reproduction, required access and impact. No production release is
claimed by this documentation rewrite.
