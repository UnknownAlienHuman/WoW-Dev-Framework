//! Syntax-only virtual Lua units. This is the pinned parser, not a second Lua
//! grammar or a semantic Main/Library session. No URI is opened and no wrapper,
//! implicit callback parameter, global binding, or executable code is generated.
use std::collections::BTreeSet;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

use emmylua_parser::{LuaLanguageLevel, LuaParseErrorKind, LuaParser, ParserConfig};
use serde::Serialize;
use sha2::{Digest, Sha256};
use wow_core::{
    CanonicalResult, ContentDigest, ProjectGenerationId, SourceContent, domain_separated_digest,
};

use crate::{EMMYLUA_CODE_ANALYSIS_VERSION, EMMYLUA_REVISION, EMMYLUA_TREE, EmmyBackendIdentity};

pub const VIRTUAL_SYNTAX_PROFILE: &str = "wow-emmy/virtual-lua51-syntax/1";
const MAX_UNITS: usize = 4_096;
const MAX_UNIT_BYTES: usize = 1024 * 1024;
const MAX_TOTAL_BYTES: usize = 16 * 1024 * 1024;
const MAX_UNIT_DIAGNOSTICS: usize = 4_096;
const MAX_DIAGNOSTICS: usize = 65_536;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualSyntaxError {
    InvalidInput,
    IncompatibleBackend,
    BudgetExceeded,
    InvalidRange,
    Cancelled,
    CanonicalizationFailed,
}
impl fmt::Display for VirtualSyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidInput => "invalid virtual Lua input",
            Self::IncompatibleBackend => "virtual Lua backend differs from compiled pin",
            Self::BudgetExceeded => "virtual Lua analysis budget exceeded",
            Self::InvalidRange => "virtual Lua parser returned an invalid range",
            Self::Cancelled => "virtual Lua analysis cancelled",
            Self::CanonicalizationFailed => "virtual Lua report identity failed",
        })
    }
}
impl std::error::Error for VirtualSyntaxError {}

/// Borrowed immutable bytes from an explicit owner. IDs are logical only.
pub struct VirtualLuaInput<'a> {
    unit_id: &'a str,
    text: &'a str,
    content_digest: ContentDigest<SourceContent>,
}
impl<'a> VirtualLuaInput<'a> {
    pub fn new(
        unit_id: &'a str,
        text: &'a str,
        content_digest: ContentDigest<SourceContent>,
    ) -> Self {
        Self {
            unit_id,
            text,
            content_digest,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VirtualDiagnosticKind {
    LuaSyntax,
    DocumentationSyntax,
}
impl VirtualDiagnosticKind {
    #[must_use]
    pub const fn upstream_code(self) -> &'static str {
        match self {
            Self::LuaSyntax => "syntax-error",
            Self::DocumentationSyntax => "doc-syntax-error",
        }
    }
}

/// UTF-8 byte offsets in the exact unwrapped virtual source. Empty ranges are
/// retained: the source owner decides whether a caret has one or many mappings.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct VirtualSyntaxDiagnostic {
    pub byte_start: u64,
    pub byte_end: u64,
    pub kind: VirtualDiagnosticKind,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VirtualSyntaxUnit {
    pub unit_id: String,
    pub content_digest: ContentDigest<SourceContent>,
    pub byte_length: u64,
    pub diagnostics: Vec<VirtualSyntaxDiagnostic>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VirtualSyntaxReport {
    profile: &'static str,
    upstream_revision: &'static str,
    upstream_tree: &'static str,
    project_generation: ProjectGenerationId,
    units: Vec<VirtualSyntaxUnit>,
    analysis_id: String,
}
impl VirtualSyntaxReport {
    #[must_use]
    pub fn analysis_id(&self) -> &str {
        &self.analysis_id
    }
    #[must_use]
    pub fn units(&self) -> &[VirtualSyntaxUnit] {
        &self.units
    }
}

/// Parse each distinct unit once with the exact compiled EmmyLua Lua 5.1 grammar.
/// The complete inventory is admitted before parsing; source/error prose is not
/// retained. Cancellation is checked between parser calls, not within upstream.
pub fn analyze_virtual_syntax(
    backend: &EmmyBackendIdentity,
    project_generation: ProjectGenerationId,
    inputs: &[VirtualLuaInput<'_>],
    stop: &AtomicBool,
) -> Result<VirtualSyntaxReport, VirtualSyntaxError> {
    checkpoint(stop)?;
    if backend.crate_name() != "emmylua_code_analysis"
        || backend.crate_version() != Some(EMMYLUA_CODE_ANALYSIS_VERSION)
        || backend.revision() != EMMYLUA_REVISION
        || backend.tree() != EMMYLUA_TREE
    {
        return Err(VirtualSyntaxError::IncompatibleBackend);
    }
    if inputs.len() > MAX_UNITS {
        return Err(VirtualSyntaxError::BudgetExceeded);
    }
    let mut ids = BTreeSet::new();
    let mut total = 0_usize;
    for input in inputs {
        checkpoint(stop)?;
        total = total
            .checked_add(input.text.len())
            .ok_or(VirtualSyntaxError::BudgetExceeded)?;
        if input.text.len() > MAX_UNIT_BYTES || total > MAX_TOTAL_BYTES {
            return Err(VirtualSyntaxError::BudgetExceeded);
        }
        if input.unit_id.is_empty()
            || input.unit_id.len() > 256
            || !input
                .unit_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._:-".contains(&b))
            || !ids.insert(input.unit_id)
        {
            return Err(VirtualSyntaxError::InvalidInput);
        }
        let digest: [u8; 32] = Sha256::digest(input.text.as_bytes()).into();
        if ContentDigest::<SourceContent>::from_bytes(digest) != input.content_digest {
            return Err(VirtualSyntaxError::InvalidInput);
        }
    }
    let mut ordered = inputs.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|input| input.unit_id);
    let mut units = Vec::with_capacity(inputs.len());
    let mut diagnostic_count = 0_usize;
    for input in ordered {
        checkpoint(stop)?;
        let tree = LuaParser::parse(
            input.text,
            ParserConfig::with_level(LuaLanguageLevel::Lua51),
        );
        checkpoint(stop)?;
        let errors = tree.get_errors();
        diagnostic_count = diagnostic_count
            .checked_add(errors.len())
            .ok_or(VirtualSyntaxError::BudgetExceeded)?;
        if errors.len() > MAX_UNIT_DIAGNOSTICS || diagnostic_count > MAX_DIAGNOSTICS {
            return Err(VirtualSyntaxError::BudgetExceeded);
        }
        let mut diagnostics = Vec::with_capacity(errors.len());
        for error in errors {
            let start = u32::from(error.range.start()) as usize;
            let end = u32::from(error.range.end()) as usize;
            if start > end
                || end > input.text.len()
                || !input.text.is_char_boundary(start)
                || !input.text.is_char_boundary(end)
            {
                return Err(VirtualSyntaxError::InvalidRange);
            }
            let kind = match &error.kind {
                LuaParseErrorKind::SyntaxError => VirtualDiagnosticKind::LuaSyntax,
                LuaParseErrorKind::DocError => VirtualDiagnosticKind::DocumentationSyntax,
            };
            diagnostics.push(VirtualSyntaxDiagnostic {
                byte_start: start as u64,
                byte_end: end as u64,
                kind,
            });
        }
        diagnostics.sort();
        units.push(VirtualSyntaxUnit {
            unit_id: input.unit_id.to_owned(),
            content_digest: input.content_digest,
            byte_length: input.text.len() as u64,
            diagnostics,
        });
    }
    let digest = domain_separated_digest(
        VIRTUAL_SYNTAX_PROFILE,
        &(EMMYLUA_REVISION, EMMYLUA_TREE, project_generation, &units),
    )
    .map_err(|_| VirtualSyntaxError::CanonicalizationFailed)?;
    let hex = ContentDigest::<CanonicalResult>::from_bytes(digest).to_string();
    checkpoint(stop)?;
    Ok(VirtualSyntaxReport {
        profile: VIRTUAL_SYNTAX_PROFILE,
        upstream_revision: EMMYLUA_REVISION,
        upstream_tree: EMMYLUA_TREE,
        project_generation,
        units,
        analysis_id: format!("emmy-virtual-syntax:{hex}"),
    })
}
fn checkpoint(stop: &AtomicBool) -> Result<(), VirtualSyntaxError> {
    if stop.load(Ordering::Relaxed) {
        Err(VirtualSyntaxError::Cancelled)
    } else {
        Ok(())
    }
}
