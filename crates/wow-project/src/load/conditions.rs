//! Explicit, generation-bound TOC selection. No client/flavor table or host defaults.
use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, de};

use super::{LoadIssueKind as Issue, Record, budget, invalid};
use crate::ProjectResult;

/// Caller-supplied facts for one selected profile, not a runtime attestation.
/// An omitted selector is unknown; it is never inferred from a build number.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TocLoadContext {
    /// Known game-type aliases and whether each matches this target. Omitted
    /// aliases remain unknown, rather than being treated as a negative match.
    #[serde(default, deserialize_with = "game_types")]
    pub game_types: BTreeMap<String, bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_locale: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<TocLoadLocation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<TocLuaEnvironment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TocLoadLocation {
    Game,
    Glue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TocLuaEnvironment {
    Global,
    Secure,
}

impl TocLoadContext {
    /// Validate both direct Rust construction and deserialized input before use.
    pub fn validate(&self) -> ProjectResult<()> {
        if self.game_types.len() > 64
            || self
                .game_types
                .keys()
                .any(|key| !atom(key) || key.bytes().any(|byte| byte.is_ascii_uppercase()))
            || self.family.as_deref().is_some_and(|value| !atom(value))
            || self.game.as_deref().is_some_and(|value| !atom(value))
            || self
                .text_locale
                .as_deref()
                .is_some_and(|value| !locale(value))
        {
            return Err(invalid("invalid explicit TOC load context"));
        }
        Ok(())
    }
}

// BTreeMap's ordinary decoder overwrites duplicate aliases. Reject contradictions
// before a selected file can be included/excluded using the overwritten value.
fn game_types<'de, D: Deserializer<'de>>(decoder: D) -> Result<BTreeMap<String, bool>, D::Error> {
    struct Visitor;
    impl<'de> de::Visitor<'de> for Visitor {
        type Value = BTreeMap<String, bool>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("bounded unique game-type matches")
        }
        fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut values = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, bool>()? {
                if values.len() >= 64 || !atom(&key) || values.insert(key, value).is_some() {
                    return Err(de::Error::custom("invalid or duplicate game-type match"));
                }
            }
            Ok(values)
        }
    }
    decoder.deserialize_map(Visitor)
}

fn atom(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphabetic)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}
fn locale(value: &str) -> bool {
    value.len() == 4 && value.bytes().all(|byte| byte.is_ascii_alphabetic())
}

/// Static selection under an explicit context, never successful client execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadSelection {
    Included,
    Excluded,
    Unresolved,
}

impl LoadSelection {
    pub(super) fn and(self, other: Self) -> Self {
        // Preserve unsupported syntax instead of claiming it is irrelevant because
        // another predicate happens not to match. No unknown syntax grants access.
        match (self, other) {
            (Self::Unresolved, _) | (_, Self::Unresolved) => Self::Unresolved,
            (Self::Excluded, _) | (_, Self::Excluded) => Self::Excluded,
            _ => Self::Included,
        }
    }
    fn from_match(matches: bool) -> Self {
        if matches {
            Self::Included
        } else {
            Self::Excluded
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TocConditionKind {
    AllowLoadGameType,
    ExcludeLoadGameType,
    AllowLoadTextLocale,
    AllowLoad,
    AllowLoadEnvironment,
}

impl TocConditionKind {
    pub(super) fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "allowloadgametype" => Some(Self::AllowLoadGameType),
            "excludeloadgametype" => Some(Self::ExcludeLoadGameType),
            "allowloadtextlocale" => Some(Self::AllowLoadTextLocale),
            "allowload" => Some(Self::AllowLoad),
            "allowloadenvironment" => Some(Self::AllowLoadEnvironment),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TocCondition {
    pub kind: TocConditionKind,
    pub values: Vec<String>,
    pub selection: LoadSelection,
}

fn evaluate(
    kind: TocConditionKind,
    values: &[String],
    context: Option<&TocLoadContext>,
) -> LoadSelection {
    let mut matched = false;
    for value in values {
        let known = match kind {
            TocConditionKind::AllowLoadGameType | TocConditionKind::ExcludeLoadGameType => context
                .and_then(|context| context.game_types.get(&value.to_ascii_lowercase()).copied()),
            TocConditionKind::AllowLoadTextLocale => {
                if !locale(value) {
                    None
                } else {
                    context
                        .and_then(|context| context.text_locale.as_ref())
                        .map(|target| target.eq_ignore_ascii_case(value))
                }
            }
            TocConditionKind::AllowLoad => match value.to_ascii_lowercase().as_str() {
                "both" => Some(true),
                "game" => context
                    .and_then(|context| context.location)
                    .map(|location| location == TocLoadLocation::Game),
                "glue" => context
                    .and_then(|context| context.location)
                    .map(|location| location == TocLoadLocation::Glue),
                _ => None,
            },
            TocConditionKind::AllowLoadEnvironment => match value.to_ascii_lowercase().as_str() {
                "global" => context
                    .and_then(|context| context.environment)
                    .map(|env| env == TocLuaEnvironment::Global),
                "secure" => context
                    .and_then(|context| context.environment)
                    .map(|env| env == TocLuaEnvironment::Secure),
                _ => None,
            },
        };
        let Some(known) = known else {
            return LoadSelection::Unresolved;
        };
        matched |= known;
    }
    LoadSelection::from_match(if kind == TocConditionKind::ExcludeLoadGameType {
        !matched
    } else {
        matched
    })
}

/// Shared bounded value admission for line predicates and package metadata.
/// An unsupported or malformed value cannot be interpreted as an absent gate.
pub(super) fn condition(
    kind: TocConditionKind,
    tail: &str,
    context: Option<&TocLoadContext>,
) -> ProjectResult<TocCondition> {
    let mut values = Vec::new();
    for value in tail.split(|ch: char| ch.is_ascii_whitespace() || ch == ',') {
        if value.is_empty() {
            continue;
        }
        if values.len() >= 64 {
            return Err(budget());
        }
        values.push(value.to_owned());
    }
    let well_formed = !values.is_empty()
        && values.iter().all(|value| atom(value))
        && (!tail.contains(',') || tail.split(',').all(|part| !part.trim().is_empty()));
    let selection = if well_formed {
        evaluate(kind, &values, context)
    } else {
        LoadSelection::Unresolved
    };
    Ok(TocCondition {
        kind,
        values: if well_formed { values } else { Vec::new() },
        selection,
    })
}

/// Strip recognized conditions and expand only the three supported variables.
/// Unknown clauses, absent context and environment-changing tags never become paths.
pub(super) fn project(
    content: &str,
    record: &mut Record,
    context: Option<&TocLoadContext>,
    metadata: bool,
) -> ProjectResult<String> {
    let mut path = String::new();
    let mut rest = content;
    let mut variables = Vec::new();
    let mut clauses = 0;
    while let Some(start) = rest.find('[') {
        if rest[..start].contains(']') {
            record.selection = LoadSelection::Unresolved;
            record.issues.push(Issue::UnknownTocSyntax);
        }
        path.push_str(&rest[..start]);
        rest = &rest[start..];
        let Some(end) = rest.find(']') else {
            record.selection = LoadSelection::Unresolved;
            record.issues.push(Issue::UnknownTocSyntax);
            return Ok(String::new());
        };
        clauses += 1;
        if clauses > 16 {
            return Err(budget());
        }
        let clause = rest[1..end].trim();
        let mut words = clause.split_ascii_whitespace();
        let head = words.next().unwrap_or("");
        if !metadata && matches!(clause, "Family" | "Game" | "TextLocale") {
            variables.push(clause);
            path.push_str(&rest[..=end]);
        } else if !metadata && clause == "Bootstrap" && !record.bootstrap {
            record.bootstrap = true;
        } else if let Some(kind) = TocConditionKind::parse(head) {
            let tail = clause[head.len()..].trim();
            let condition = condition(kind, tail, context)?;
            let selection = condition.selection;
            record.selection = record.selection.and(selection);
            record.conditions.push(condition);
            if selection == LoadSelection::Unresolved {
                record.issues.push(Issue::LoadConditionUnresolved);
            }
        } else {
            record.selection = LoadSelection::Unresolved;
            record.issues.push(Issue::UnknownTocSyntax);
        }
        rest = &rest[end + 1..];
    }
    path.push_str(rest);
    if rest.contains(']') {
        record.selection = LoadSelection::Unresolved;
        record.issues.push(Issue::UnknownTocSyntax);
    }
    let path = path.trim().to_owned();
    if !metadata {
        if path.len() > 4096 {
            return Err(budget());
        }
        record.declared_target = Some(path.clone());
    }
    if record.selection != LoadSelection::Included {
        return Ok(path);
    }
    let mut expanded = path;
    for variable in variables {
        let value = context.and_then(|context| match variable {
            "Family" => context.family.as_deref(),
            "Game" => context.game.as_deref(),
            "TextLocale" => context.text_locale.as_deref(),
            _ => None,
        });
        if let Some(value) = value {
            expanded = expanded.replace(&format!("[{variable}]"), value);
        } else {
            record.selection = LoadSelection::Unresolved;
            record.issues.push(Issue::LoadContextRequired);
        }
    }
    Ok(expanded)
}
