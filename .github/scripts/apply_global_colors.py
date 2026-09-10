from pathlib import Path
import re, sys

ROOT = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else Path.cwd()


def load(path: str) -> str:
    return (ROOT / path).read_text(encoding='utf-8')


def save(path: str, text: str) -> None:
    p = ROOT / path
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding='utf-8')


def replace_once(path: str, old: str, new: str) -> None:
    text = load(path)
    n = text.count(old)
    if n != 1:
        raise RuntimeError(f'{path}: expected one literal match, got {n}: {old[:120]!r}')
    save(path, text.replace(old, new, 1))


def sub_once(path: str, pattern: str, repl: str) -> None:
    text = load(path)
    text2, n = re.subn(pattern, repl, text, count=1, flags=re.M | re.S)
    if n != 1:
        raise RuntimeError(f'{path}: expected one regex match, got {n}: {pattern[:120]!r}')
    save(path, text2)

REFERENCE_GLOBALS = r'''//! Exact Ketho GlobalColors declarations parsed as inert source data.
use super::{Result, error};
use crate::native::{NativeErrorCode, Span};
use emmylua_parser::{LuaLanguageLevel, LuaParser, ParserConfig};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};

/// One observed `NAME = CreateColor(...)` declaration. Components remain source
/// lexemes; they are never executed, normalized to binary floats or promoted to
/// Blizzard/runtime truth.
#[derive(Clone, Debug, Serialize)]
pub struct GlobalColorFact {
    pub name: String,
    pub components: Vec<String>,
    pub span: Span,
}

pub(super) fn read(input: &str, cancelled: &AtomicBool) -> Result<Vec<GlobalColorFact>> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(error(NativeErrorCode::Cancelled));
    }
    let tree = LuaParser::parse(input, ParserConfig::with_level(LuaLanguageLevel::Lua51));
    if !tree.get_errors().is_empty() {
        return Err(error(NativeErrorCode::Syntax));
    }
    let mut facts = Vec::new();
    let mut offset = 0usize;
    for chunk in input.split_inclusive('\n') {
        if cancelled.load(Ordering::Relaxed) {
            return Err(error(NativeErrorCode::Cancelled));
        }
        let line = chunk.strip_suffix('\n').unwrap_or(chunk);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let leading = line.len() - line.trim_start_matches([' ', '\t']).len();
        let mut end = line.trim_end_matches([' ', '\t']).len();
        let mut code = &line[leading..end];
        if code.is_empty() || code.starts_with("--") {
            offset += chunk.len();
            continue;
        }
        if let Some((before, _)) = code.split_once("--") {
            end = leading + before.trim_end_matches([' ', '\t']).len();
            code = &line[leading..end];
        }
        let code = code.strip_suffix(';').unwrap_or(code).trim_end_matches([' ', '\t']);
        if code.is_empty() {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        }
        let (name, call) = code
            .split_once('=')
            .ok_or_else(|| error(NativeErrorCode::UnsupportedStatement))?;
        if call.contains('=') {
            return Err(error(NativeErrorCode::UnsupportedStatement));
        }
        let name = name.trim();
        if !identifier(name) {
            return Err(error(NativeErrorCode::InvalidRegistration));
        }
        let call = call.trim();
        let arguments = call
            .strip_prefix("CreateColor(")
            .and_then(|value| value.strip_suffix(')'))
            .ok_or_else(|| error(NativeErrorCode::UnsupportedExpression))?;
        let components = arguments
            .split(',')
            .map(str::trim)
            .map(|value| {
                if numeric(value) {
                    Ok(value.to_owned())
                } else {
                    Err(error(NativeErrorCode::UnsupportedExpression))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        if !matches!(components.len(), 3 | 4) {
            return Err(error(NativeErrorCode::UnsupportedExpression));
        }
        if facts.len() >= super::MAX_ALIASES {
            return Err(error(NativeErrorCode::Limit));
        }
        facts.push(GlobalColorFact {
            name: name.to_owned(),
            components,
            span: Span {
                start: offset + leading,
                end: offset + leading + code.len(),
            },
        });
        offset += chunk.len();
    }
    if facts.is_empty() {
        return Err(error(NativeErrorCode::InvalidRegistration));
    }
    Ok(facts)
}

fn identifier(name: &str) -> bool {
    if name.is_empty() || name.len() > 1024 {
        return false;
    }
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

fn numeric(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'+' | b'-' | b'.' | b'e' | b'E'))
        && value.parse::<f64>().is_ok_and(f64::is_finite)
}
'''

ANNOTATION_GLOBALS = r'''//! Inert global color declarations from one explicitly selected Ketho resource.
use super::{AliasDocument, AliasOutcome, reserved_name, source};
use crate::ketho::{RenderError, identifier};
use crate::native::{ProjectionIssue, SourceLink, SourceMapping};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, Ordering};
use wow_reference::native::Span;
use wow_reference::native_aliases::GlobalColorFact;

pub(super) struct GlobalColors<'a> {
    entries: Vec<(&'a AliasDocument, &'a GlobalColorFact)>,
    emitted: Vec<bool>,
    pub outcomes: Vec<AliasOutcome>,
    pub issues: Vec<ProjectionIssue>,
}

impl<'a> GlobalColors<'a> {
    pub fn prepare(
        resources: &[&'a AliasDocument],
        counts: &BTreeMap<&str, usize>,
        defined: &BTreeSet<String>,
        reserved: &BTreeSet<String>,
        cancelled: &AtomicBool,
    ) -> Result<Self, RenderError> {
        let entries = resources
            .iter()
            .flat_map(|document| {
                document
                    .global_colors()
                    .iter()
                    .map(move |fact| (*document, fact))
            })
            .collect::<Vec<_>>();
        let mut result = Self {
            emitted: Vec::with_capacity(entries.len()),
            outcomes: Vec::with_capacity(entries.len()),
            issues: Vec::new(),
            entries,
        };
        for (ordinal, (document, fact)) in result.entries.iter().enumerate() {
            if cancelled.load(Ordering::Relaxed) {
                return Err(RenderError::Cancelled);
            }
            let status = if identifier(&fact.name).is_err() {
                "invalid_global_color_name"
            } else if counts.get(fact.name.as_str()) != Some(&1) {
                "duplicate_global_color"
            } else if reserved_name(&fact.name, reserved) || defined.contains(&fact.name) {
                "source_name_conflict"
            } else {
                "emitted"
            };
            let emitted = status == "emitted";
            result.emitted.push(emitted);
            result.outcomes.push(AliasOutcome {
                ordinal,
                name: fact.name.clone(),
                status,
            });
            if !emitted {
                result.issues.push(ProjectionIssue {
                    code: status.into(),
                    source: source(document, fact.span),
                });
            }
        }
        Ok(result)
    }

    pub fn has_output(&self) -> bool {
        self.emitted.iter().any(|value| *value)
    }

    pub fn unresolved_types(
        &mut self,
        known: &BTreeSet<String>,
        cancelled: &AtomicBool,
    ) -> Result<Vec<SourceLink>, RenderError> {
        if known.contains("colorRGBA") {
            return Ok(Vec::new());
        }
        let mut unresolved = Vec::new();
        for ((document, fact), emitted) in self.entries.iter().zip(&self.emitted) {
            if cancelled.load(Ordering::Relaxed) {
                return Err(RenderError::Cancelled);
            }
            if *emitted {
                let link = source(document, fact.span);
                self.issues.push(ProjectionIssue {
                    code: "unresolved_global_color_type".into(),
                    source: link.clone(),
                });
                unresolved.push(link);
            }
        }
        Ok(unresolved)
    }

    pub fn append(
        &self,
        text: &mut String,
        mappings: &mut Vec<SourceMapping>,
        cancelled: &AtomicBool,
    ) -> Result<(), RenderError> {
        let mut order = (0..self.entries.len()).collect::<Vec<_>>();
        order.sort_by_key(|&index| &self.entries[index].1.name);
        for index in order {
            if cancelled.load(Ordering::Relaxed) {
                return Err(RenderError::Cancelled);
            }
            if !self.emitted[index] {
                continue;
            }
            let (document, fact) = self.entries[index];
            let fragment = format!("---@type colorRGBA\n{} = nil", fact.name);
            if text
                .len()
                .saturating_add(fragment.len())
                .saturating_add(2)
                > crate::ketho::MAX_OUTPUT_BYTES
            {
                return Err(RenderError::OutputLimit);
            }
            if !text.is_empty() && !text.ends_with("\n\n") {
                text.push('\n');
            }
            let start = text.len();
            text.push_str(&fragment);
            let end = text.len();
            text.push('\n');
            mappings.push(SourceMapping {
                granularity: "declaration",
                generated: Span { start, end },
                source: source(document, fact.span),
            });
        }
        Ok(())
    }
}
'''

VERIFY_GLOBALS = r'''//! Exact inert GlobalColors artifact validation; source calls are never evaluated.
use super::{Result, list, span_key, text};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

type Key<'a> = (&'a str, (usize, usize));

pub(super) fn verify<'a>(
    library: &'a Value,
    sources: &BTreeMap<&'a str, &'a Value>,
    mapped: &mut BTreeMap<Key<'a>, &'a str>,
    emitted: &mut BTreeSet<&'a str>,
) -> Result<bool> {
    let report = &library["aliases"];
    if report["schema"] != "wow-native-alias-projection/9" {
        if report.get("global_color_outcomes").is_some()
            || report.get("unresolved_global_color_types").is_some()
        {
            return Err("unexpected global color projection field".into());
        }
        return Ok(false);
    }
    let mut entries = Vec::new();
    for (&path, &resource) in sources {
        if resource.get("global_colors").is_some() {
            for color in list(resource, "global_colors")? {
                entries.push((path, resource, color));
            }
        }
    }
    let outcomes = list(report, "global_color_outcomes")?;
    if entries.is_empty() || entries.len() != outcomes.len() {
        return Err("missing external global color outcomes".into());
    }
    let mut blocked = false;
    let mut observed = BTreeSet::new();
    let mut emitted_links = BTreeSet::new();
    for (ordinal, ((path, resource, color), outcome)) in
        entries.into_iter().zip(outcomes).enumerate()
    {
        let name = text(color, "name")?;
        let span = span_key(&color["span"])?;
        let raw = text(resource, "text")?;
        let components = list(color, "components")?;
        if span.0 >= span.1
            || raw.get(span.0..span.1).is_none()
            || !observed.insert((path, span))
            || !matches!(components.len(), 3 | 4)
            || components.iter().any(|component| {
                component
                    .as_str()
                    .is_none_or(|value| value.len() > 64 || value.parse::<f64>().is_err())
            })
            || outcome["ordinal"] != json!(ordinal)
            || outcome["name"] != name
        {
            return Err("global color source/outcome mismatch".into());
        }
        let status = text(outcome, "status")?;
        if status == "emitted" {
            if !valid_identifier(name) || !emitted.insert(name) {
                return Err("invalid emitted global color".into());
            }
            let expected = format!("---@type colorRGBA\n{name} = nil");
            if mapped.remove(&(path, span)) != Some(expected.as_str()) {
                return Err("missing or changed global color declaration".into());
            }
            emitted_links.insert((path, span));
        } else {
            if !matches!(
                status,
                "invalid_global_color_name" | "duplicate_global_color" | "source_name_conflict"
            ) || mapped.contains_key(&(path, span))
            {
                return Err("invalid blocked global color".into());
            }
            require_issue(library, status, resource, &color["span"])?;
            blocked = true;
        }
    }
    let mut unresolved = BTreeSet::new();
    if report.get("unresolved_global_color_types").is_some() {
        let links = list(report, "unresolved_global_color_types")?;
        if links.is_empty() || links.len() > 4096 {
            return Err("invalid unresolved global color inventory".into());
        }
        for link in links {
            let path = text(link, "path")?;
            let key = (path, span_key(&link["span"])?);
            let resource = sources.get(path).ok_or("unknown global color resource")?;
            if link["scope"] != "annotation_alias_catalog"
                || link["sha256"] != resource["sha256"]
                || !emitted_links.contains(&key)
                || !unresolved.insert(key)
            {
                return Err("invalid unresolved global color link".into());
            }
            require_issue(
                library,
                "unresolved_global_color_type",
                resource,
                &link["span"],
            )?;
        }
        blocked = true;
    }
    for issue in list(library, "issues")? {
        if issue["code"] == "unresolved_global_color_type" {
            let source = &issue["source"];
            let path = text(source, "path")?;
            let resource = sources
                .get(path)
                .ok_or("unknown global color issue resource")?;
            if source["scope"] != "annotation_alias_catalog"
                || source["sha256"] != resource["sha256"]
                || !unresolved.contains(&(path, span_key(&source["span"])?))
            {
                return Err("unreported unresolved global color type".into());
            }
        }
    }
    Ok(blocked)
}

fn valid_identifier(name: &str) -> bool {
    if name.is_empty() || name.len() > 1024 || keyword(name) {
        return false;
    }
    let mut bytes = name.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte == b'_' || byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
}

fn keyword(name: &str) -> bool {
    matches!(
        name,
        "and" | "break" | "do" | "else" | "elseif" | "end" | "false" | "for"
            | "function" | "goto" | "if" | "in" | "local" | "nil" | "not" | "or"
            | "repeat" | "return" | "then" | "true" | "until" | "while"
    )
}

fn require_issue(library: &Value, code: &str, resource: &Value, span: &Value) -> Result<()> {
    if !list(library, "issues")?.iter().any(|issue| {
        issue["code"] == code
            && issue["source"]["scope"] == "annotation_alias_catalog"
            && issue["source"]["path"] == resource["path"]
            && issue["source"]["sha256"] == resource["sha256"]
            && issue["source"]["span"] == *span
    }) {
        return Err("missing external global color issue".into());
    }
    Ok(())
}
'''

REFERENCE_TEST = r'''//! Static GlobalColors resources retain numeric lexemes without executing calls.
use std::sync::atomic::AtomicBool;
use wow_reference::native::{NativeErrorCode, source_digest};
use wow_reference::native_aliases::ingest_alias_catalog;

const REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn exact_create_color_assignments_are_retained_with_source_spans() {
    let raw = "---@meta _\n---@type colorRGBA\nACCOUNT_WIDE_FONT_COLOR = CreateColor(0.53, 0.67, 1)\nDISABLED_FONT_COLOR = CreateColor(0.5, 0.5, 0.5, 0.8)\n";
    let document = ingest_alias_catalog(
        REVISION,
        "GlobalColors.lua",
        raw,
        &source_digest(raw.as_bytes()),
        &AtomicBool::new(false),
    )
    .expect("global colors");
    assert_eq!(document.global_colors().len(), 2);
    assert_eq!(document.global_colors()[0].name, "ACCOUNT_WIDE_FONT_COLOR");
    assert_eq!(document.global_colors()[0].components, ["0.53", "0.67", "1"]);
    let fact = &document.global_colors()[1];
    assert_eq!(
        raw.get(fact.span.start..fact.span.end),
        Some("DISABLED_FONT_COLOR = CreateColor(0.5, 0.5, 0.5, 0.8)")
    );
    assert_eq!(
        serde_json::to_value(document).expect("serialize")["schema"],
        "wow-native-alias-resource/7"
    );
}

#[test]
fn executable_or_non_numeric_color_forms_fail_closed() {
    for raw in [
        "COLOR = Other(1, 1, 1)\n",
        "COLOR = CreateColor(GetValue(), 1, 1)\n",
        "COLOR = CreateColor(1, 1)\n",
        "COLOR = CreateColor(1, 1, 1, 1, 1)\n",
        "local COLOR = CreateColor(1, 1, 1)\n",
        "COLOR = CreateColor(1, 1, 1)\nreturn COLOR\n",
    ] {
        let error = ingest_alias_catalog(
            REVISION,
            "GlobalColors.lua",
            raw,
            &source_digest(raw.as_bytes()),
            &AtomicBool::new(false),
        )
        .expect_err(raw);
        assert_ne!(error.code, NativeErrorCode::Cancelled);
    }
}
'''

ANNOTATION_TEST = r'''//! GlobalColors project to inert globals with exact external navigation.
use std::sync::atomic::AtomicBool;
use wow_annotations::native::project_with_alias_catalog;
use wow_annotations::navigation::{GeneratedLookup, NavigationIndex, SourceFile, SourceLookup};
use wow_reference::native::{ingest_document, source_digest};
use wow_reference::native_aliases::ingest_alias_catalog;

const SOURCE_REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DONOR_REVISION: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn project(
    source: &'static str,
    raw: &'static str,
) -> Result<wow_annotations::native::NativeLibrary<'static>, Box<dyn std::error::Error>> {
    let documents = Box::leak(Box::new([ingest_document(
        SOURCE_REVISION,
        "API.lua",
        source,
        &source_digest(source.as_bytes()),
        &AtomicBool::new(false),
    )?]));
    let catalog = Box::leak(Box::new(ingest_alias_catalog(
        DONOR_REVISION,
        "GlobalColors.lua",
        raw,
        &source_digest(raw.as_bytes()),
        &AtomicBool::new(false),
    )?));
    Ok(project_with_alias_catalog(
        documents,
        "Mainline",
        None,
        Some(catalog),
        &AtomicBool::new(false),
    )?)
}

#[test]
fn color_globals_are_inert_and_navigate_to_the_exact_donor_assignment()
-> Result<(), Box<dyn std::error::Error>> {
    let source = r#"APIDocumentation:AddDocumentationTable({Name="colorRGBA",Type="Structure",Fields={{Name="r",Type="number"}}})"#;
    let raw = "---@type colorRGBA\nACCOUNT_WIDE_FONT_COLOR = CreateColor(0.53, 0.67, 1)\n";
    let library = project(source, raw)?;
    assert_eq!(library.projection, "projected_with_sidecars");
    let report = library.aliases.as_ref().ok_or("alias report")?;
    assert_eq!(report.schema, "wow-native-alias-projection/9");
    assert_eq!(report.global_color_outcomes[0].status, "emitted");
    assert!(report.unresolved_global_color_types.is_empty());
    let file = library.files.last().ok_or("external file")?;
    assert!(file.text.contains("---@type colorRGBA\nACCOUNT_WIDE_FONT_COLOR = nil"));
    let mapping = file.mappings.last().ok_or("global color map")?;
    assert_eq!(mapping.source.scope, Some("annotation_alias_catalog"));
    let index = NavigationIndex::new(&library, &AtomicBool::new(false))?;
    let SourceLookup::Mapped { candidates, .. } = index.source_at(
        &file.path,
        &file.sha256,
        mapping.generated.start,
        &AtomicBool::new(false),
    )? else {
        return Err("generated global did not navigate".into());
    };
    assert_eq!(candidates[0].revision, DONOR_REVISION);
    let source = index.bind_source(
        SourceFile {
            scope: Some("annotation_alias_catalog"),
            revision: DONOR_REVISION,
            path: report.source.path(),
            sha256: report.source.sha256(),
        },
        raw,
        &AtomicBool::new(false),
    )?;
    assert!(matches!(
        source.generated_at(mapping.source.span.start, &AtomicBool::new(false))?,
        GeneratedLookup::Mapped { .. }
    ));
    Ok(())
}

#[test]
fn missing_color_type_keeps_the_global_but_marks_projection_partial()
-> Result<(), Box<dyn std::error::Error>> {
    let source = r#"APIDocumentation:AddDocumentationTable({Name="Probe",Type="System",Namespace="C_Probe",Functions={{Name="Read"}}})"#;
    let raw = "COLOR = CreateColor(1, 0.5, 0)\n";
    let library = project(source, raw)?;
    assert_eq!(library.projection, "partial");
    let report = library.aliases.as_ref().ok_or("alias report")?;
    assert_eq!(report.unresolved_global_color_types.len(), 1);
    assert!(library.issues.iter().any(|issue| {
        issue.code == "unresolved_global_color_type"
            && issue.source.scope == Some("annotation_alias_catalog")
    }));
    assert!(library.files.last().ok_or("output")?.text.contains("COLOR = nil"));
    Ok(())
}
'''

save('crates/wow-reference/src/native_aliases/global_colors.rs', REFERENCE_GLOBALS)
save('crates/wow-annotations/src/aliases/global_colors.rs', ANNOTATION_GLOBALS)
save('tools/xtask/src/library/aliases/global_colors.rs', VERIFY_GLOBALS)
save('crates/wow-reference/tests/global_colors.rs', REFERENCE_TEST)
save('crates/wow-annotations/tests/global_colors.rs', ANNOTATION_TEST)

replace_once('crates/wow-reference/src/native_aliases.rs', 'mod function_containers;\nmod namespaces;', 'mod function_containers;\nmod global_colors;\nmod namespaces;')
replace_once('crates/wow-reference/src/native_aliases.rs', '};\npub use namespaces::NamespaceFact;', '};\npub use global_colors::GlobalColorFact;\npub use namespaces::NamespaceFact;')
replace_once('crates/wow-reference/src/native_aliases.rs', '    function_containers: Vec<FunctionContainerFact>,\n}', '    function_containers: Vec<FunctionContainerFact>,\n    #[serde(skip_serializing_if = "Vec::is_empty")]\n    global_colors: Vec<GlobalColorFact>,\n}')
replace_once('crates/wow-reference/src/native_aliases.rs', '    pub fn function_containers(&self) -> &[FunctionContainerFact] {\n        &self.function_containers\n    }\n}', '    pub fn function_containers(&self) -> &[FunctionContainerFact] {\n        &self.function_containers\n    }\n    pub fn global_colors(&self) -> &[GlobalColorFact] {\n        &self.global_colors\n    }\n}')
text = load('crates/wow-reference/src/native_aliases.rs')
text = text.replace('            function_containers: Vec::new(),\n        });', '            function_containers: Vec::new(),\n            global_colors: Vec::new(),\n        });')
text = text.replace('                    function_containers: Vec::new(),\n                });', '                    function_containers: Vec::new(),\n                    global_colors: Vec::new(),\n                });')
text = text.replace('            function_containers,\n        });', '            function_containers,\n            global_colors: Vec::new(),\n        });')
text = text.replace('        function_containers: Vec::new(),\n    })', '        function_containers: Vec::new(),\n        global_colors: Vec::new(),\n    })')
save('crates/wow-reference/src/native_aliases.rs', text)
sub_once(
    'crates/wow-reference/src/native_aliases.rs',
    r'''        let function_containers = function_containers::read\(input, cancelled\)\?;\n        return Ok\(AliasDocument \{\n            schema: "wow-native-alias-resource/6",\n            revision: revision\.into\(\),\n            path: path\.into\(\),\n            sha256: digest,\n            source_bytes: text\.len\(\),\n            text: text\.into\(\),\n            aliases: Vec::new\(\),\n            structures: Vec::new\(\),\n            namespaces: Vec::new\(\),\n            function_containers,\n            global_colors: Vec::new\(\),\n        \}\);''',
    '''        match function_containers::read(input, cancelled) {
            Ok(function_containers) => {
                return Ok(AliasDocument {
                    schema: "wow-native-alias-resource/6",
                    revision: revision.into(),
                    path: path.into(),
                    sha256: digest,
                    source_bytes: text.len(),
                    text: text.into(),
                    aliases: Vec::new(),
                    structures: Vec::new(),
                    namespaces: Vec::new(),
                    function_containers,
                    global_colors: Vec::new(),
                });
            }
            Err(failure) if failure.code == NativeErrorCode::Cancelled => return Err(failure),
            Err(_) => {}
        }
        let global_colors = global_colors::read(input, cancelled)?;
        return Ok(AliasDocument {
            schema: "wow-native-alias-resource/7",
            revision: revision.into(),
            path: path.into(),
            sha256: digest,
            source_bytes: text.len(),
            text: text.into(),
            aliases: Vec::new(),
            structures: Vec::new(),
            namespaces: Vec::new(),
            function_containers: Vec::new(),
            global_colors,
        });'''
)

replace_once('crates/wow-annotations/src/aliases.rs', 'mod function_containers;\nmod namespaces;', 'mod function_containers;\nmod global_colors;\nmod namespaces;')
replace_once('crates/wow-annotations/src/aliases.rs', '    pub unresolved_function_container_returns: Vec<SourceLink>,\n    pub limitations:', '    pub unresolved_function_container_returns: Vec<SourceLink>,\n    #[serde(skip_serializing_if = "Vec::is_empty")]\n    pub global_color_outcomes: Vec<AliasOutcome>,\n    #[serde(skip_serializing_if = "Vec::is_empty")]\n    pub unresolved_global_color_types: Vec<SourceLink>,\n    pub limitations:')
replace_once('crates/wow-annotations/src/aliases.rs', '                        .sum::<usize>()\n            })', '                        .sum::<usize>()\n                    + s.global_colors().len()\n            })')
replace_once('crates/wow-annotations/src/aliases.rs', '        for fact in resource.function_containers() {\n            *counts.entry(&fact.name).or_default() += 1;\n        }\n    }', '        for fact in resource.function_containers() {\n            *counts.entry(&fact.name).or_default() += 1;\n        }\n        for fact in resource.global_colors() {\n            *counts.entry(&fact.name).or_default() += 1;\n        }\n    }')
replace_once('crates/wow-annotations/src/aliases.rs', '    let mut function_containers = function_containers::FunctionContainers::prepare(\n        &sources, &renderer, &counts, defined, reserved, cancelled,\n    )?;\n    let mut known = defined.clone();', '    let mut function_containers = function_containers::FunctionContainers::prepare(\n        &sources, &renderer, &counts, defined, reserved, cancelled,\n    )?;\n    let mut global_colors =\n        global_colors::GlobalColors::prepare(&sources, &counts, defined, reserved, cancelled)?;\n    let mut known = defined.clone();')
replace_once('crates/wow-annotations/src/aliases.rs', '    let unresolved_function_container_returns =\n        function_containers.unresolved_returns(&renderer, &known, cancelled)?;', '    let unresolved_function_container_returns =\n        function_containers.unresolved_returns(&renderer, &known, cancelled)?;\n    let unresolved_global_color_types = global_colors.unresolved_types(&known, cancelled)?;')
replace_once('crates/wow-annotations/src/aliases.rs', '        || function_containers.has_output()\n    {', '        || function_containers.has_output()\n        || global_colors.has_output()\n    {')
replace_once('crates/wow-annotations/src/aliases.rs', '    function_containers.append(&mut text, &mut mappings, cancelled)?;\n    structures.append', '    function_containers.append(&mut text, &mut mappings, cancelled)?;\n    global_colors.append(&mut text, &mut mappings, cancelled)?;\n    structures.append')
replace_once('crates/wow-annotations/src/aliases.rs', '    issues.append(&mut function_containers.issues);\n    let has_structures', '    issues.append(&mut function_containers.issues);\n    issues.append(&mut global_colors.issues);\n    let has_structures')
replace_once('crates/wow-annotations/src/aliases.rs', '    let has_function_containers = sources\n        .iter()\n        .any(|source| !source.function_containers().is_empty());', '    let has_function_containers = sources\n        .iter()\n        .any(|source| !source.function_containers().is_empty());\n    let has_global_colors = sources\n        .iter()\n        .any(|source| !source.global_colors().is_empty());')
replace_once('crates/wow-annotations/src/aliases.rs', '    if has_function_containers {\n        limitations.push("external function containers retain exact class and method maps; empty bodies are syntax-only declarations and do not imply runtime behavior");\n    }', '    if has_function_containers {\n        limitations.push("external function containers retain exact class and method maps; empty bodies are syntax-only declarations and do not imply runtime behavior");\n    }\n    if has_global_colors {\n        limitations.push("external CreateColor components are retained as source lexemes; output declares inert colorRGBA globals and does not execute calls or claim exact runtime values");\n    }')
replace_once('crates/wow-annotations/src/aliases.rs', '            schema: if has_function_containers {\n                "wow-native-alias-projection/8"', '            schema: if has_global_colors {\n                "wow-native-alias-projection/9"\n            } else if has_function_containers {\n                "wow-native-alias-projection/8"')
replace_once('crates/wow-annotations/src/aliases.rs', '            unresolved_function_container_returns,\n            limitations,', '            unresolved_function_container_returns,\n            global_color_outcomes: global_colors.outcomes,\n            unresolved_global_color_types,\n            limitations,')

replace_once('tools/xtask/src/library/aliases.rs', 'mod function_containers;\nmod namespaces;', 'mod function_containers;\nmod global_colors;\nmod namespaces;')
replace_once('tools/xtask/src/library/aliases.rs', '    blocked |= function_containers::verify(library, &source_map, &mut mapped, &mut emitted)?;\n    if !mapped.is_empty()', '    blocked |= function_containers::verify(library, &source_map, &mut mapped, &mut emitted)?;\n    blocked |= global_colors::verify(library, &source_map, &mut mapped, &mut emitted)?;\n    if !mapped.is_empty()')

path = 'tools/xtask/src/library/aliases/resources.rs'
text = load(path)
text = text.replace('    let function_profile = schema == Some("wow-native-alias-projection/8");', '    let global_profile = schema == Some("wow-native-alias-projection/9");\n    let function_capable = matches!(\n        schema,\n        Some("wow-native-alias-projection/8" | "wow-native-alias-projection/9")\n    );\n    let function_required = schema == Some("wow-native-alias-projection/8");')
text = text.replace('Some("wow-native-alias-projection/7" | "wow-native-alias-projection/8")', 'Some(\n            "wow-native-alias-projection/7"\n                | "wow-native-alias-projection/8"\n                | "wow-native-alias-projection/9"\n        )')
text = text.replace('                | "wow-native-alias-projection/8"\n        )', '                | "wow-native-alias-projection/8"\n                | "wow-native-alias-projection/9"\n        )')
text = text.replace('    let extended_profile = structure_capable || namespace_capable || function_profile;', '    let extended_profile =\n        structure_capable || namespace_capable || function_capable || global_profile;')
text = text.replace('    let mut has_function_containers = false;', '    let mut has_function_containers = false;\n    let mut has_global_colors = false;')
text = text.replace('            Some("wow-native-alias-resource/6") if function_profile => 6,', '            Some("wow-native-alias-resource/6") if function_capable => 6,\n            Some("wow-native-alias-resource/7") if global_profile => 7,')
global_block = '''        let global_items = if profile == 7 {
            let colors = list(resource, "global_colors")?;
            if colors.is_empty()
                || !aliases.is_empty()
                || structures != 0
                || namespaces != 0
                || function_items != 0
            {
                return Err("invalid external global color resource".into());
            }
            for color in colors {
                if !matches!(list(color, "components")?.len(), 3 | 4) {
                    return Err("invalid external global color component count".into());
                }
            }
            has_global_colors = true;
            colors.len()
        } else {
            if resource.get("global_colors").is_some() {
                return Err("unexpected external global color field".into());
            }
            0
        };
'''
marker = "        if (aliases.is_empty()"
if text.count(marker) != 1:
    raise RuntimeError(f'resources empty marker count {text.count(marker)}')
text = text.replace(marker, global_block + marker, 1)
if text.count("&& function_items == 0)") != 1:
    raise RuntimeError("resources function item condition mismatch")
text = text.replace("&& function_items == 0)", "&& function_items == 0\n            && global_items == 0)", 1)
text = text.replace('            .checked_add(aliases.len() + structures + namespaces + function_items)', '            .checked_add(\n                aliases.len() + structures + namespaces + function_items + global_items,\n            )')
text = text.replace('    if function_profile != has_function_containers\n        || (!namespace_capable && has_namespaces)', '    if global_profile != has_global_colors\n        || (function_required && !has_function_containers)\n        || (!function_capable && has_function_containers)\n        || (!namespace_capable && has_namespaces)')
for token in ['global_profile', 'has_global_colors', 'wow-native-alias-resource/7']:
    if token not in text:
        raise RuntimeError(f'resources missing {token}')
save(path, text)

replace_once('crates/wow-annotations/src/navigation/sources.rs', '                    | "wow-native-alias-projection/8"', '                    | "wow-native-alias-projection/8"\n                    | "wow-native-alias-projection/9"')
replace_once('crates/wow-annotations/examples/support/catalogs.rs', '                .sum::<usize>();', '                .sum::<usize>()\n            + document.global_colors().len();')

path = '.github/workflows/current-source-bundle.yml'
text = load(path)
needle = '''            Annotations/Core/Type/FunctionContainer.lua \\
            --alias-catalog "$RUNNER_TEMP/ketho-source" "$(cat "$RUNNER_TEMP/alias-revision")" \\
            Annotations/Core/Type/StringEnum.lua || result=$?'''
replacement = '''            Annotations/Core/Type/FunctionContainer.lua \\
            --alias-catalog "$RUNNER_TEMP/ketho-source" "$(cat "$RUNNER_TEMP/alias-revision")" \\
            Annotations/Core/Type/GlobalColors.lua \\
            --alias-catalog "$RUNNER_TEMP/ketho-source" "$(cat "$RUNNER_TEMP/alias-revision")" \\
            Annotations/Core/Type/StringEnum.lua || result=$?'''
if text.count(needle) != 1:
    raise RuntimeError(f'workflow composition marker count {text.count(needle)}')
save(path, text.replace(needle, replacement, 1))

assert 'wow-native-alias-projection/9' in load('crates/wow-annotations/src/aliases.rs')
assert 'wow-native-alias-resource/7' in load('crates/wow-reference/src/native_aliases.rs')
assert 'GlobalColors.lua' in load('.github/workflows/current-source-bundle.yml')
print('GlobalColors patch applied')
