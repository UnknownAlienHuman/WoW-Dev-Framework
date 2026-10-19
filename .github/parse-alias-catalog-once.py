from pathlib import Path


def replace(path: str, old: str, new: str) -> None:
    file = Path(path)
    text = file.read_text()
    if text.count(old) != 1:
        raise SystemExit(f"unexpected block count in {path}: {text.count(old)}")
    file.write_text(text.replace(old, new))


replace(
    "crates/wow-reference/src/native_aliases.rs",
    """use emmylua_parser::{
    LexerConfig, LuaAstNode, LuaDocTag, LuaDocType, LuaLanguageLevel, LuaLexer, LuaParser,
    LuaTokenKind, LuaTypeBinaryOperator, ParserConfig, Reader,
};
""",
    """use emmylua_parser::{
    LexerConfig, LuaAstNode, LuaChunk, LuaDocTag, LuaDocType, LuaLanguageLevel, LuaLexer,
    LuaParser, LuaTokenKind, LuaTypeBinaryOperator, ParserConfig, Reader,
};
""",
)
replace(
    "crates/wow-reference/src/native_aliases.rs",
    """        if !string_enums {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        }
        match namespaces::read(input, cancelled) {
""",
    """        if !string_enums {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        }
        let tree = LuaParser::parse(input, ParserConfig::with_level(LuaLanguageLevel::Lua51));
        if !tree.get_errors().is_empty() {
            return Err(error(NativeErrorCode::Syntax));
        }
        let chunk: LuaChunk = tree.get_chunk_node();
        match namespaces::read(&chunk, cancelled) {
""",
)
replace(
    "crates/wow-reference/src/native_aliases.rs",
    "match function_containers::read(input, cancelled) {",
    "match function_containers::read(&chunk, cancelled) {",
)
replace(
    "crates/wow-reference/src/native_aliases.rs",
    "let global_colors = global_colors::read(input, cancelled)?;",
    "let global_colors = global_colors::read(&chunk, cancelled)?;",
)

replace(
    "crates/wow-reference/src/native_aliases/namespaces.rs",
    """use emmylua_parser::{
    LuaAstNode, LuaAstToken, LuaDocTag, LuaExpr, LuaLanguageLevel, LuaParser, LuaStat, LuaVarExpr,
    ParserConfig,
};
""",
    """use emmylua_parser::{
    LuaAstNode, LuaAstToken, LuaChunk, LuaDocTag, LuaExpr, LuaStat, LuaVarExpr,
};
""",
)
replace(
    "crates/wow-reference/src/native_aliases/namespaces.rs",
    """pub(super) fn read(input: &str, cancelled: &AtomicBool) -> Result<Vec<NamespaceFact>> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(error(NativeErrorCode::Cancelled));
    }
    let tree = LuaParser::parse(input, ParserConfig::with_level(LuaLanguageLevel::Lua51));
    if !tree.get_errors().is_empty() {
        return Err(error(NativeErrorCode::Syntax));
    }
    let chunk = tree.get_chunk_node();
""",
    """pub(super) fn read(
    chunk: &LuaChunk,
    cancelled: &AtomicBool,
) -> Result<Vec<NamespaceFact>> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(error(NativeErrorCode::Cancelled));
    }
""",
)

replace(
    "crates/wow-reference/src/native_aliases/function_containers.rs",
    """use emmylua_parser::{
    LuaAstNode, LuaCommentOwner, LuaDocTag, LuaExpr, LuaIndexKey, LuaLanguageLevel, LuaParser,
    LuaStat, LuaVarExpr, ParserConfig,
};
""",
    """use emmylua_parser::{
    LuaAstNode, LuaChunk, LuaCommentOwner, LuaDocTag, LuaExpr, LuaIndexKey, LuaStat, LuaVarExpr,
};
""",
)
replace(
    "crates/wow-reference/src/native_aliases/function_containers.rs",
    """pub(super) fn read(input: &str, cancelled: &AtomicBool) -> Result<Vec<FunctionContainerFact>> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(error(NativeErrorCode::Cancelled));
    }
    let tree = LuaParser::parse(input, ParserConfig::with_level(LuaLanguageLevel::Lua51));
    if !tree.get_errors().is_empty() {
        return Err(error(NativeErrorCode::Syntax));
    }
    let chunk = tree.get_chunk_node();
""",
    """pub(super) fn read(
    chunk: &LuaChunk,
    cancelled: &AtomicBool,
) -> Result<Vec<FunctionContainerFact>> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(error(NativeErrorCode::Cancelled));
    }
""",
)

replace(
    "crates/wow-reference/src/native_aliases/global_colors.rs",
    """use emmylua_parser::{
    LuaAstNode, LuaAstToken, LuaDocTag, LuaExpr, LuaLanguageLevel, LuaLiteralToken, LuaParser,
    LuaStat, LuaVarExpr, ParserConfig,
};
""",
    """use emmylua_parser::{
    LuaAstNode, LuaAstToken, LuaChunk, LuaDocTag, LuaExpr, LuaLiteralToken, LuaStat, LuaVarExpr,
};
""",
)
replace(
    "crates/wow-reference/src/native_aliases/global_colors.rs",
    """pub(super) fn read(input: &str, cancelled: &AtomicBool) -> Result<Vec<GlobalColorFact>> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(error(NativeErrorCode::Cancelled));
    }
    let tree = LuaParser::parse(input, ParserConfig::with_level(LuaLanguageLevel::Lua51));
    if !tree.get_errors().is_empty() {
        return Err(error(NativeErrorCode::Syntax));
    }
    let chunk = tree.get_chunk_node();
""",
    """pub(super) fn read(
    chunk: &LuaChunk,
    cancelled: &AtomicBool,
) -> Result<Vec<GlobalColorFact>> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(error(NativeErrorCode::Cancelled));
    }
""",
)

replace(
    "docs/IMPLEMENTATION.md",
    """Do not duplicate rendering rules in a new validator. #202 shares a parsed
catalog tree between existing validators.
""",
    """Do not duplicate rendering rules in a new validator. Statement catalogs now
parse once and share one Emmy `LuaChunk` across the existing validators.
""",
)
replace(
    "docs/REFACTOR_QUEUE.md",
    "| Independent | [#202 — catalog parsing](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/202) | Share one Emmy tree across existing validators; keep comment-group isolation. |\n",
    "| Completed | [#202 — catalog parsing](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/202) | One Emmy Lua 5.1 parse now feeds namespace, FunctionContainer and GlobalColors validators. |\n",
)
