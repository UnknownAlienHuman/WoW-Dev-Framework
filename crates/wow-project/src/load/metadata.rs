//! One normalized TOC metadata directive retained by the single source parser.
//!
//! [TocMetadata] is a lexical projection of a directive normalized by the load
//! owner's parser. The containing record retains conditional predicates and
//! selection, including unknown or excluded conditions. Downstream owners read
//! the retained key/value pair without re-splitting raw source spans.
//!
//! Semantic classification is derived from the normalized key through one table
//! shared by TOC admission and package projection. The kind is deliberately not
//! stored as another identity field: raw key/value remain the source-backed
//! representation, while [TocMetadataKind] is the narrow current interpretation.
use serde::Serialize;

use super::TocConditionKind;

/// Current semantic class for one normalized TOC metadata key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TocMetadataKind {
    Interface,
    RequiredDependencies,
    OptionalDependencies,
    LoadOnDemand,
    SavedVariables,
    SavedVariablesPerCharacter,
    SavedVariablesMachine,
    DefaultState,
    LoadSavedVariablesFirst,
    EscalateErrorDuringLoad,
    AllowAddOnTableAccess,
    UseSecureEnvironment,
    SuppressLocalTableRef,
    OnlyBetaAndPtr,
    LoadWith,
    LoadFirst,
    Secure,
    AllowLoadGameType,
    ExcludeLoadGameType,
    AllowLoadTextLocale,
    AllowLoad,
    AllowLoadEnvironment,
    Display,
    Custom,
    Unknown,
}

impl TocMetadataKind {
    /// Classify one already lowercased, trimmed metadata key.
    #[must_use]
    pub fn classify(key: &str) -> Self {
        match key {
            "interface" => Self::Interface,
            "dep" | "dependencies" | "dependson" | "requireddep" | "requireddeps" => {
                Self::RequiredDependencies
            }
            "optionaldep" | "optionaldeps" => Self::OptionalDependencies,
            "loadondemand" => Self::LoadOnDemand,
            "savedvariables" => Self::SavedVariables,
            "savedvariablespercharacter" => Self::SavedVariablesPerCharacter,
            "savedvariablesmachine" => Self::SavedVariablesMachine,
            "defaultstate" => Self::DefaultState,
            "loadsavedvariablesfirst" => Self::LoadSavedVariablesFirst,
            "escalateerrorduringload" => Self::EscalateErrorDuringLoad,
            "allowaddontableaccess" => Self::AllowAddOnTableAccess,
            "usesecureenvironment" => Self::UseSecureEnvironment,
            "suppresslocaltableref" => Self::SuppressLocalTableRef,
            "onlybetaandptr" => Self::OnlyBetaAndPtr,
            "loadwith" => Self::LoadWith,
            "loadfirst" => Self::LoadFirst,
            "secure" => Self::Secure,
            "allowloadgametype" => Self::AllowLoadGameType,
            "excludeloadgametype" => Self::ExcludeLoadGameType,
            "allowloadtextlocale" => Self::AllowLoadTextLocale,
            "allowload" => Self::AllowLoad,
            "allowloadenvironment" => Self::AllowLoadEnvironment,
            "title" | "notes" | "author" | "version" | "icontexture" | "iconatlas"
            | "category" | "group" => Self::Display,
            _ if localized_display_key(key) => Self::Display,
            _ if key.starts_with("x-") => Self::Custom,
            _ => Self::Unknown,
        }
    }

    /// Package-wide filter kind, when this metadata key is a recognized gate.
    #[must_use]
    pub const fn condition_kind(self) -> Option<TocConditionKind> {
        match self {
            Self::AllowLoadGameType => Some(TocConditionKind::AllowLoadGameType),
            Self::ExcludeLoadGameType => Some(TocConditionKind::ExcludeLoadGameType),
            Self::AllowLoadTextLocale => Some(TocConditionKind::AllowLoadTextLocale),
            Self::AllowLoad => Some(TocConditionKind::AllowLoad),
            Self::AllowLoadEnvironment => Some(TocConditionKind::AllowLoadEnvironment),
            _ => None,
        }
    }

    /// Whether this key declares SavedVariables under a known persistence scope.
    #[must_use]
    pub const fn saved_variables(self) -> bool {
        matches!(
            self,
            Self::SavedVariables
                | Self::SavedVariablesPerCharacter
                | Self::SavedVariablesMachine
        )
    }

    /// Metadata whose semantics can change selected reachability, order,
    /// environment or chunk context and are not yet implemented by Project.
    #[must_use]
    pub const fn unsupported_load_context(self) -> bool {
        matches!(
            self,
            Self::UseSecureEnvironment
                | Self::SuppressLocalTableRef
                | Self::OnlyBetaAndPtr
                | Self::LoadWith
                | Self::LoadFirst
                | Self::Secure
        )
    }
}

fn localized_display_key(key: &str) -> bool {
    key.starts_with("title-")
        || key.starts_with("notes-")
        || key.starts_with("category-")
        || key.starts_with("group-")
}

/// One normalized TOC metadata directive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TocMetadata {
    /// Lowercased key, trimmed, with conditional clauses already stripped.
    pub key: String,
    /// Trimmed value, with conditional clauses already stripped.
    pub value: String,
}

impl TocMetadata {
    /// Semantic class derived from the retained normalized key.
    #[must_use]
    pub fn kind(&self) -> TocMetadataKind {
        TocMetadataKind::classify(&self.key)
    }
}
