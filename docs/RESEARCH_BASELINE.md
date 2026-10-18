# Donors used in the current review

| Donor | Reviewed revision / role |
| --- | --- |
| [EmmyLua](https://github.com/EmmyLuaLs/emmylua-analyzer-rust) | `aaaca68425d9362876228649b0b8d92f07654daa`: the compiled parser/analysis engine; standard language-server behavior. |
| [Ketho](https://github.com/Ketho/vscode-wow-api) | `d0b5b51fac4c52c493371b9b18e66ce604ea4326`: annotation loading, normalization, types and emission. |
| [Blizzard UI mirror](https://github.com/Gethe/wow-ui-source) | Select the requested flavor/revision when source-sensitive work starts. Current API facts come from those actual files. |
| [SQLite FTS5](https://www.sqlite.org/fts5.html) | Existing local search substrate; use the dependency selected by Cargo.lock. |

These are review inputs, not permanent current-version claims. Read only the
relevant donor path and retain attribution. Do not copy an upstream's entire
architecture or execute donor/source scripts during indexing. Native maps are
in [KETHO_RUST_PORT.md](KETHO_RUST_PORT.md) and
[EMMYLUA_UPSTREAM.md](EMMYLUA_UPSTREAM.md). [Architecture](ARCHITECTURE.md) defines
what this project builds; optional providers and old architecture research are
not active work.
