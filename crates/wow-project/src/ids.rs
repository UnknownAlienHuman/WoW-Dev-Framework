use std::fmt;
use std::str::FromStr;

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use wow_core::NormalizedSourcePath;

use crate::{ProjectError, ProjectErrorCode, ProjectPhase, ProjectResult};

const MAX_ID_BYTES: usize = 512;

macro_rules! project_text_id {
    ($name:ident, $validator:ident) => {
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(Box<str>);

        impl $name {
            pub fn new(candidate: impl Into<String>) -> ProjectResult<Self> {
                let candidate = candidate.into();
                $validator(&candidate)?;
                Ok(Self(candidate.into_boxed_str()))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_tuple(stringify!($name))
                    .field(&self.0)
                    .finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = ProjectError;

            fn from_str(input: &str) -> Result<Self, Self::Err> {
                Self::new(input)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(D::Error::custom)
            }
        }
    };
}

project_text_id!(ProjectId, validate_project_id);
project_text_id!(ProjectWorkspaceId, validate_workspace_id);
project_text_id!(ProjectSourceOriginId, validate_origin_id);

/// Stable logical file identity. Content is deliberately excluded.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectFileId(Box<str>);

impl ProjectFileId {
    pub fn from_path(path: &NormalizedSourcePath) -> ProjectResult<Self> {
        let value = format!("project-file:{}", path.as_str());
        if value.len() > MAX_ID_BYTES {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidFileId,
                ProjectPhase::Inventory,
                "project file ID exceeds the configured identity limit",
            )
            .with_relative_path(path.as_str()));
        }
        Ok(Self(value.into_boxed_str()))
    }

    pub fn parse(candidate: &str) -> ProjectResult<Self> {
        let path = candidate.strip_prefix("project-file:").ok_or_else(|| {
            ProjectError::new(
                ProjectErrorCode::InvalidFileId,
                ProjectPhase::Inventory,
                "project file ID has an invalid prefix",
            )
        })?;
        let parsed = NormalizedSourcePath::parse(path).map_err(|_| {
            ProjectError::new(
                ProjectErrorCode::InvalidFileId,
                ProjectPhase::Inventory,
                "project file ID contains a noncanonical path",
            )
            .with_relative_path(path)
        })?;
        if !parsed.was_canonical() || parsed.value().as_str() != path {
            return Err(ProjectError::new(
                ProjectErrorCode::InvalidFileId,
                ProjectPhase::Inventory,
                "project file ID contains a noncanonical path",
            )
            .with_relative_path(path));
        }
        Self::from_path(parsed.value())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ProjectFileId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ProjectFileId")
            .field(&self.0)
            .finish()
    }
}

impl fmt::Display for ProjectFileId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for ProjectFileId {
    type Err = ProjectError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input)
    }
}

impl Serialize for ProjectFileId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ProjectFileId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(D::Error::custom)
    }
}

fn validate_project_id(value: &str) -> ProjectResult<()> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
        && !value.ends_with('-')
        && !value.contains("--")
        && !matches!(value, "current" | "latest" | "default" | "auto");
    if valid {
        Ok(())
    } else {
        Err(ProjectError::new(
            ProjectErrorCode::InvalidConfiguration,
            ProjectPhase::Configuration,
            "project ID is not canonical",
        ))
    }
}

fn validate_workspace_id(value: &str) -> ProjectResult<()> {
    validate_prefixed_id(value, "workspace:", "workspace ID")
}

fn validate_origin_id(value: &str) -> ProjectResult<()> {
    validate_prefixed_id(value, "project-origin:", "project source-origin ID")
}

fn validate_prefixed_id(value: &str, prefix: &str, label: &str) -> ProjectResult<()> {
    let payload = value.strip_prefix(prefix).ok_or_else(|| {
        ProjectError::new(
            ProjectErrorCode::InvalidConfiguration,
            ProjectPhase::Configuration,
            format!("{label} has an invalid prefix"),
        )
    })?;
    let valid = !payload.is_empty()
        && value.len() <= MAX_ID_BYTES
        && payload.split(':').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
                && !matches!(segment, "current" | "latest" | "default" | "auto")
        });
    if valid {
        Ok(())
    } else {
        Err(ProjectError::new(
            ProjectErrorCode::InvalidConfiguration,
            ProjectPhase::Configuration,
            format!("{label} is not canonical"),
        ))
    }
}
