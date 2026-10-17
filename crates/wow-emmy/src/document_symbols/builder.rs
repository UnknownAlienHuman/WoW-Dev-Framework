// Adapted from EmmyLuaLs/emmylua-analyzer-rust aaaca68425d9362876228649b0b8d92f07654daa.
// Copyright (c) 2024-2026 CppCXY. MIT: ../../DOCUMENT_SYMBOLS_DONOR_LICENSE.
use super::*;
use crate::LuaWorkspaceFile;
use emmylua_code_analysis::{
    DbIndex, FileId, LuaDecl, LuaDeclId, LuaDeclarationTree, LuaType, LuaTypeOwner,
};
use emmylua_parser::{LuaSyntaxId, LuaSyntaxNode, LuaSyntaxToken};
use rowan::TextRange;
use std::collections::{HashMap, HashSet};

pub(super) struct DocumentSymbolBuilder<'a> {
    db: &'a DbIndex,
    declarations: &'a LuaDeclarationTree,
    file_id: FileId,
    file: &'a LuaWorkspaceFile,
    symbols: HashMap<LuaSyntaxId, LuaSymbol>,
    budget: Budget<'a>,
}
impl<'a> DocumentSymbolBuilder<'a> {
    pub(super) fn new(
        db: &'a DbIndex,
        declarations: &'a LuaDeclarationTree,
        file_id: FileId,
        file: &'a LuaWorkspaceFile,
        budget: Budget<'a>,
    ) -> Self {
        Self {
            db,
            declarations,
            file_id,
            file,
            symbols: HashMap::new(),
            budget,
        }
    }
    pub(super) fn limits(&self) -> DocumentSymbolsLimits {
        self.budget.limits
    }
    pub(super) fn checkpoint(&mut self) -> Option<()> {
        match self.budget.node() {
            Ok(()) => Some(()),
            Err(error) => {
                self.budget.failure = Some(error.code());
                None
            }
        }
    }
    pub(super) fn limit<T>(&mut self) -> Option<T> {
        self.budget.failure = Some(DocumentSymbolsErrorCode::Limit);
        None
    }
    pub(super) fn get_file_id(&self) -> FileId {
        self.file_id
    }
    pub(super) fn get_decl(&self, id: &LuaDeclId) -> Option<&'a LuaDecl> {
        self.declarations.get_decl(id)
    }
    pub(super) fn get_type(&self, id: LuaTypeOwner) -> LuaType {
        self.db
            .get_type_index()
            .get_type_cache(&id)
            .map(|cache| cache.as_type())
            .unwrap_or(&LuaType::Unknown)
            .clone()
    }
    pub(super) fn add_node_symbol(
        &mut self,
        node: LuaSyntaxNode,
        symbol: LuaSymbol,
        parent: Option<LuaSyntaxId>,
    ) -> LuaSyntaxId {
        let id = LuaSyntaxId::new(node.kind(), node.text_range());
        self.insert(id, symbol);
        if let Some(parent) = parent {
            self.link(parent, id);
        } else {
            let mut current = node.parent();
            while let Some(node) = current {
                if self.checkpoint().is_none() {
                    break;
                }
                let ancestor = LuaSyntaxId::new(node.kind(), node.text_range());
                if self.symbols.contains_key(&ancestor) {
                    self.link(ancestor, id);
                    break;
                }
                current = node.parent();
            }
        }
        id
    }
    pub(super) fn add_token_symbol(
        &mut self,
        token: LuaSyntaxToken,
        symbol: LuaSymbol,
        parent: Option<LuaSyntaxId>,
    ) -> LuaSyntaxId {
        let id = LuaSyntaxId::new(token.kind(), token.text_range());
        self.insert(id, symbol);
        if let Some(parent) = parent {
            self.link(parent, id);
        } else {
            let mut current = token.parent();
            while let Some(node) = current {
                if self.checkpoint().is_none() {
                    break;
                }
                let ancestor = LuaSyntaxId::new(node.kind(), node.text_range());
                if self.symbols.contains_key(&ancestor) {
                    self.link(ancestor, id);
                    break;
                }
                current = node.parent();
            }
        }
        id
    }
    fn insert(&mut self, id: LuaSyntaxId, mut symbol: LuaSymbol) {
        if self.checkpoint().is_none() {
            return;
        }
        if self.symbols.contains_key(&id) {
            self.budget.failure = Some(DocumentSymbolsErrorCode::Analyzer);
            return;
        }
        if self.symbols.len() > self.budget.limits.max_symbols {
            self.budget.failure = Some(DocumentSymbolsErrorCode::Limit);
            return;
        }
        if symbol.name.len() > self.budget.limits.max_name_bytes
            || symbol
                .detail
                .as_ref()
                .is_some_and(|text| text.len() > self.budget.limits.max_detail_bytes)
        {
            self.budget.failure = Some(DocumentSymbolsErrorCode::Limit);
            return;
        }
        if symbol.kind != DocumentSymbolKind::File && symbol.name.trim().is_empty() {
            symbol.original_name = Some(symbol.name.clone());
            symbol.name = if symbol.name.is_empty() {
                "(empty)".into()
            } else {
                "(blank)".into()
            };
        }
        let bytes = symbol.name.len()
            + symbol.original_name.as_ref().map_or(0, String::len)
            + symbol.detail.as_ref().map_or(0, String::len)
            + 256;
        if self.budget.charge(bytes).is_err()
            || self
                .budget
                .retained
                .checked_add(bytes)
                .is_none_or(|n| n > self.budget.limits.max_report_bytes)
        {
            if self.budget.failure.is_none() {
                self.budget.failure = Some(DocumentSymbolsErrorCode::Limit);
            }
            return;
        }
        self.budget.retained += bytes;
        self.symbols.insert(id, symbol);
    }
    fn link(&mut self, parent: LuaSyntaxId, child: LuaSyntaxId) {
        if self.checkpoint().is_none() {
            return;
        }
        if parent == child {
            self.budget.failure = Some(DocumentSymbolsErrorCode::Analyzer);
            return;
        }
        let Some(child_range) = self.symbols.get(&child).map(|symbol| symbol.range) else {
            self.budget.failure = Some(DocumentSymbolsErrorCode::Analyzer);
            return;
        };
        let Some(symbol) = self.symbols.get_mut(&parent) else {
            self.budget.failure = Some(DocumentSymbolsErrorCode::Analyzer);
            return;
        };
        if !symbol.children.contains(&child) {
            symbol.children.push(child);
        }
        let expanded = TextRange::new(
            symbol.range.start().min(child_range.start()),
            symbol.range.end().max(child_range.end()),
        );
        if expanded != symbol.range {
            if symbol.selection_range.is_none() {
                symbol.selection_range = Some(symbol.range);
            }
            symbol.range = expanded;
        }
    }
    pub(super) fn contains_symbol(&self, id: &LuaSyntaxId) -> bool {
        self.symbols.contains_key(id)
    }
    pub(super) fn with_symbol_mut<F: FnOnce(&mut LuaSymbol)>(
        &mut self,
        id: &LuaSyntaxId,
        change: F,
    ) -> Option<()> {
        self.checkpoint()?;
        let symbol = self.symbols.get_mut(id)?;
        change(symbol);
        let bytes = symbol.detail.as_ref().map_or(0, String::len);
        if bytes > self.budget.limits.max_detail_bytes
            || self
                .budget
                .retained
                .checked_add(bytes)
                .is_none_or(|n| n > self.budget.limits.max_report_bytes)
        {
            return self.limit();
        }
        if let Err(error) = self.budget.charge(bytes) {
            self.budget.failure = Some(error.code());
            return None;
        }
        self.budget.retained += bytes;
        Some(())
    }
    pub(super) fn get_symbol_kind_and_detail(
        &mut self,
        typ: Option<&LuaType>,
    ) -> (DocumentSymbolKind, Option<String>) {
        let Some(typ) = typ else {
            return (DocumentSymbolKind::Variable, None);
        };
        if typ.is_def() {
            return (DocumentSymbolKind::Class, None);
        }
        if typ.is_string() {
            return (DocumentSymbolKind::String, None);
        }
        if typ.is_table() {
            return (DocumentSymbolKind::Object, None);
        }
        if typ.is_number() {
            return (
                DocumentSymbolKind::Number,
                match typ {
                    LuaType::IntegerConst(value) => Some(value.to_string()),
                    LuaType::FloatConst(value) => Some(value.to_string()),
                    _ => None,
                },
            );
        }
        if typ.is_boolean() {
            return (DocumentSymbolKind::Boolean, None);
        }
        if typ.is_function() {
            let mut text = String::from("(");
            let complete = match typ {
                LuaType::DocFunction(function) => {
                    let params = function.get_params();
                    if params.len() > self.budget.limits.max_symbols {
                        self.limit::<()>()
                    } else {
                        let mut complete = Some(());
                        for (index, param) in params.iter().enumerate() {
                            if self
                                .push_parameter(&mut text, param.0.as_ref(), index)
                                .is_none()
                            {
                                complete = None;
                                break;
                            }
                        }
                        complete
                    }
                }
                LuaType::Signature(id) => {
                    if let Some(signature) = self.db.get_signature_index().get(id) {
                        let params = signature.get_type_params();
                        if params.len() > self.budget.limits.max_symbols {
                            self.limit::<()>()
                        } else {
                            let mut complete = Some(());
                            for (index, param) in params.iter().enumerate() {
                                if self
                                    .push_parameter(&mut text, param.0.as_ref(), index)
                                    .is_none()
                                {
                                    complete = None;
                                    break;
                                }
                            }
                            complete
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            };
            let detail = complete.map(|()| {
                text.push(')');
                text
            });
            return (DocumentSymbolKind::Function, detail);
        }
        (DocumentSymbolKind::Variable, None)
    }
    fn push_parameter(&mut self, text: &mut String, name: &str, index: usize) -> Option<()> {
        self.checkpoint()?;
        let separator = if index == 0 { 0 } else { 2 };
        if name.len() > self.budget.limits.max_name_bytes
            || text
                .len()
                .checked_add(name.len() + separator + 1)
                .is_none_or(|n| n > self.budget.limits.max_detail_bytes)
        {
            return self.limit();
        }
        if index != 0 {
            text.push_str(", ");
        }
        text.push_str(name);
        Some(())
    }
    pub(super) fn finish(
        mut self,
        root: LuaSyntaxId,
        complete: bool,
    ) -> DocumentSymbolsResult<(Vec<DocumentSymbol>, usize)> {
        if let Some(code) = self.budget.failure {
            return Err(error(code));
        }
        if !complete {
            return Ok((Vec::new(), self.budget.work));
        }
        let root = self
            .symbols
            .remove(&root)
            .ok_or_else(|| error(DocumentSymbolsErrorCode::Analyzer))?;
        let mut pending = root
            .children
            .into_iter()
            .rev()
            .map(|id| (id, None, 1usize))
            .collect::<Vec<_>>();
        let mut result = Vec::<DocumentSymbol>::new();
        let mut seen = HashSet::new();
        while let Some((id, parent, depth)) = pending.pop() {
            self.budget.node()?;
            if depth > self.budget.limits.max_depth
                || result.len() >= self.budget.limits.max_symbols
            {
                return Err(error(DocumentSymbolsErrorCode::Limit));
            }
            if !seen.insert(id) {
                return Err(error(DocumentSymbolsErrorCode::Analyzer));
            }
            let symbol = self
                .symbols
                .remove(&id)
                .ok_or_else(|| error(DocumentSymbolsErrorCode::Analyzer))?;
            let range = crate::references::ast_span(self.file, symbol.range)
                .map_err(|_| error(DocumentSymbolsErrorCode::InvalidSpan))?;
            let selection = crate::references::ast_span(
                self.file,
                symbol.selection_range.unwrap_or(symbol.range),
            )
            .map_err(|_| error(DocumentSymbolsErrorCode::InvalidSpan))?;
            if selection.byte_start() < range.byte_start()
                || selection.byte_end() > range.byte_end()
            {
                return Err(error(DocumentSymbolsErrorCode::InvalidSpan));
            }
            let index = result.len();
            for child in symbol.children.into_iter().rev() {
                pending.push((child, Some(index), depth + 1));
            }
            result.push(DocumentSymbol {
                name: symbol.name.into(),
                original_name: symbol.original_name.map(Into::into),
                detail: symbol.detail.map(Into::into),
                kind: symbol.kind,
                range,
                selection_range: selection,
                parent_index: parent,
            });
        }
        Ok((result, self.budget.work))
    }
}

pub(super) struct LuaSymbol {
    name: String,
    original_name: Option<String>,
    detail: Option<String>,
    kind: DocumentSymbolKind,
    range: TextRange,
    selection_range: Option<TextRange>,
    children: Vec<LuaSyntaxId>,
}
impl LuaSymbol {
    pub(super) fn new(
        name: String,
        detail: Option<String>,
        kind: DocumentSymbolKind,
        range: TextRange,
    ) -> Self {
        Self {
            name,
            original_name: None,
            detail,
            kind,
            range,
            selection_range: None,
            children: Vec::new(),
        }
    }
    pub(super) fn with_selection_range(
        name: String,
        detail: Option<String>,
        kind: DocumentSymbolKind,
        range: TextRange,
        selection_range: TextRange,
    ) -> Self {
        Self {
            name,
            original_name: None,
            detail,
            kind,
            range,
            selection_range: Some(selection_range),
            children: Vec::new(),
        }
    }
    pub(super) fn set_kind(&mut self, kind: DocumentSymbolKind) {
        self.kind = kind;
    }
    pub(super) fn set_detail(&mut self, detail: Option<String>) {
        self.detail = detail;
    }
}
