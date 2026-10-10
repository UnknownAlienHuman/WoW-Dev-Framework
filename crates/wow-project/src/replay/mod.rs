//! Exact captured inputs for an approved native project replay. Stored analyzer
//! identifiers are compared after real analysis; they never manufacture a session.
mod configuration;
mod load;
mod packages;
mod platform;
pub mod publication;

use crate::{
    ProjectError, ProjectErrorCode, ProjectInputBundle, ProjectInputFile, ProjectPhase,
    ProjectPublisher, ProjectResult, ProjectView,
};
use configuration::ReplayConfiguration;
use load::ReplayLoad;
use packages::ReplayPackages;
use platform::ReplayPlatform;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use wow_emmy::{
    LuaWorkspaceFileInput, LuaWorkspaceLimits, LuaWorkspaceSnapshot, LuaWorkspaceUniverse,
};

const REPLAY_SCHEMA: &str = "wow-project/native-project-replay/1";
const LOAD_REPLAY_SCHEMA: &str = "wow-project/native-project-replay/2";
const PACKAGE_REPLAY_SCHEMA: &str = "wow-project/native-project-replay/3";
const LIBRARY_BOUND_REPLAY_SCHEMA: &str = "wow-project/native-project-replay/4";
const PLATFORM_REPLAY_SCHEMA: &str = "wow-project/native-project-replay/5";
const MAX_FILES: usize = 8192;
const MAX_LIBRARIES: usize = 64;
const MAX_FILE_BYTES: usize = 16 * 1024 * 1024;
const MAX_SOURCE_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayFile {
    path: String,
    text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    fixture_ref: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReplayUniverse {
    Project,
    BlizzardUi,
    Fixture,
    BlizzardUiMain,
}
impl ReplayUniverse {
    fn capture(universe: LuaWorkspaceUniverse) -> Self {
        match universe {
            LuaWorkspaceUniverse::Project => Self::Project,
            LuaWorkspaceUniverse::BlizzardUi => Self::BlizzardUi,
            LuaWorkspaceUniverse::Fixture => Self::Fixture,
            LuaWorkspaceUniverse::BlizzardUiMain => Self::BlizzardUiMain,
        }
    }
    fn restore(self) -> LuaWorkspaceUniverse {
        match self {
            Self::Project => LuaWorkspaceUniverse::Project,
            Self::BlizzardUi => LuaWorkspaceUniverse::BlizzardUi,
            Self::Fixture => LuaWorkspaceUniverse::Fixture,
            Self::BlizzardUiMain => LuaWorkspaceUniverse::BlizzardUiMain,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayLibrary {
    snapshot_id: String,
    universe: ReplayUniverse,
    files: Vec<ReplayFile>,
}

/// Data-only archive, distinct from the executable owner view. New publications
/// use v4 with Library-bound generation or v5 for a genuine platform corpus;
/// v1/v2/v3 retain their original recipe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectReplay {
    schema: String,
    configuration: ReplayConfiguration,
    files: Vec<ReplayFile>,
    libraries: Vec<ReplayLibrary>,
    function_calls: bool,
    project_snapshot_id: String,
    analyzer_snapshot_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    generation_schema_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    load: Option<ReplayLoad>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    packages: Option<ReplayPackages>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    platform: Option<ReplayPlatform>,
}
impl ProjectReplay {
    /// Bound the raw envelope before strict typed decoding. This constructs only
    /// an archive DTO; genuine owner admission still happens in `hydrate`.
    pub fn from_json(bytes: &[u8], stop: &AtomicBool) -> ProjectResult<Self> {
        crate::analyzer::checkpoint(stop)?;
        if bytes.len() > wow_store::project::MAX_RECORD_BYTES {
            return Err(exhausted());
        }
        let replay: Self = serde_json::from_slice(bytes).map_err(|_| invalid())?;
        replay.validate_budget(stop)?;
        Ok(replay)
    }
    pub fn capture(publisher: &ProjectPublisher, stop: &AtomicBool) -> ProjectResult<Self> {
        crate::analyzer::checkpoint(stop)?;
        let snapshot = publisher.current_snapshot().ok_or_else(invalid)?;
        snapshot.validate()?;
        let (inputs, libraries, function_calls) = publisher.replay_inputs();
        let configuration = ReplayConfiguration::from_configuration(snapshot.configuration())?;
        if inputs.len() > MAX_FILES || libraries.len() > MAX_LIBRARIES {
            return Err(exhausted());
        }
        let platform_owner = snapshot.configuration().platform_packages();
        let package_plan = snapshot
            .configuration()
            .package_load_plan()
            .filter(|_| platform_owner.is_none());
        let mut count = if package_plan.is_some() || platform_owner.is_some() {
            0
        } else {
            inputs.len()
        };
        let mut bytes = 0usize;
        // Charge borrowed input bytes before copying an archive.
        for file in inputs
            .iter()
            .filter(|_| package_plan.is_none() && platform_owner.is_none())
        {
            crate::analyzer::checkpoint(stop)?;
            bytes = bytes
                .checked_add(file.retained_text().len())
                .ok_or_else(exhausted)?;
            if bytes > MAX_SOURCE_BYTES || file.retained_text().len() > MAX_FILE_BYTES {
                return Err(exhausted());
            }
        }
        for library in libraries {
            count = count
                .checked_add(library.files().len())
                .ok_or_else(exhausted)?;
            if count > MAX_FILES {
                return Err(exhausted());
            }
            for file in library.files() {
                crate::analyzer::checkpoint(stop)?;
                bytes = bytes.checked_add(file.text().len()).ok_or_else(exhausted)?;
                if bytes > MAX_SOURCE_BYTES || file.text().len() > MAX_FILE_BYTES {
                    return Err(exhausted());
                }
            }
        }
        if let Some(plan) = snapshot.configuration().load_plan() {
            for source in plan
                .sources()
                .iter()
                .filter(|source| !source.path.ends_with(".lua"))
            {
                crate::analyzer::checkpoint(stop)?;
                let text = plan.document_text(&source.path).ok_or_else(invalid)?;
                count = count.checked_add(1).ok_or_else(exhausted)?;
                bytes = bytes.checked_add(text.len()).ok_or_else(exhausted)?;
                if count > MAX_FILES || bytes > MAX_SOURCE_BYTES || text.len() > MAX_FILE_BYTES {
                    return Err(exhausted());
                }
            }
        }
        let load = snapshot
            .configuration()
            .load_plan()
            .map(|plan| ReplayLoad::capture(plan, inputs))
            .transpose()?;
        let packages = package_plan
            .map(|plan| {
                ReplayPackages::capture(
                    plan,
                    snapshot
                        .configuration()
                        .package_main_plan()
                        .ok_or_else(invalid)?,
                    inputs,
                    stop,
                    &mut count,
                    &mut bytes,
                )
            })
            .transpose()?;
        let mut files = inputs
            .iter()
            .filter(|_| packages.is_none() && platform_owner.is_none())
            .map(|file| ReplayFile {
                path: file.relative_path().as_str().into(),
                text: file.retained_text().into(),
                fixture_ref: file.source_fixture_ref().map(str::to_owned),
            })
            .collect::<Vec<_>>();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let mut retained_libraries = Vec::new();
        for library in libraries {
            crate::analyzer::checkpoint(stop)?;
            if library.universe() == LuaWorkspaceUniverse::BlizzardUiMain
                || library.backend() != snapshot.configuration().analyzer_binding().backend()
            {
                return Err(invalid());
            }
            retained_libraries.push(ReplayLibrary {
                snapshot_id: library.snapshot_id().into(),
                universe: ReplayUniverse::capture(library.universe()),
                files: library
                    .files()
                    .iter()
                    .map(|file| ReplayFile {
                        path: file.path().into(),
                        text: file.text().into(),
                        fixture_ref: None,
                    })
                    .collect(),
            });
        }
        retained_libraries.sort_by(|a, b| a.snapshot_id.cmp(&b.snapshot_id));
        let generation_schema_version = match snapshot
            .generation_candidate()
            .project_generation_schema_version()
        {
            1 => None,
            2 => Some(2),
            _ => return Err(invalid()),
        };
        let platform = if let Some(owner) = platform_owner {
            if generation_schema_version != Some(2) {
                return Err(invalid());
            }
            let envelope_size = encoded_size(
                &PlatformEnvelopeBudget {
                    schema: PLATFORM_REPLAY_SCHEMA,
                    configuration: &configuration,
                    files: &files,
                    libraries: &retained_libraries,
                    function_calls,
                    project_snapshot_id: snapshot.snapshot_id(),
                    analyzer_snapshot_id: snapshot.analyzer_binding().analyzer_snapshot_id(),
                    generation_schema_version: 2,
                    platform: (),
                },
                stop,
            )?
            .checked_sub(4)
            .ok_or_else(exhausted)?;
            let remaining = wow_store::project::MAX_RECORD_BYTES
                .checked_sub(envelope_size)
                .ok_or_else(exhausted)?;
            Some(ReplayPlatform::capture(
                owner, inputs, stop, &mut count, &mut bytes, remaining,
            )?)
        } else {
            None
        };
        let replay = Self {
            schema: if platform.is_some() {
                PLATFORM_REPLAY_SCHEMA
            } else if generation_schema_version.is_some() {
                LIBRARY_BOUND_REPLAY_SCHEMA
            } else if packages.is_some() {
                PACKAGE_REPLAY_SCHEMA
            } else if load.is_some() {
                LOAD_REPLAY_SCHEMA
            } else {
                REPLAY_SCHEMA
            }
            .into(),
            configuration,
            files,
            libraries: retained_libraries,
            function_calls,
            project_snapshot_id: snapshot.snapshot_id().into(),
            analyzer_snapshot_id: snapshot.analyzer_binding().analyzer_snapshot_id().into(),
            generation_schema_version,
            load,
            packages,
            platform,
        };
        replay.validate_budget(stop)?;
        Ok(replay)
    }
    fn validate_budget(&self, stop: &AtomicBool) -> ProjectResult<()> {
        let expected_schema = match self.generation_schema_version {
            Some(2) if self.platform.is_some() => PLATFORM_REPLAY_SCHEMA,
            Some(2) => LIBRARY_BOUND_REPLAY_SCHEMA,
            None if self.packages.is_some() => PACKAGE_REPLAY_SCHEMA,
            None if self.load.is_some() => LOAD_REPLAY_SCHEMA,
            None => REPLAY_SCHEMA,
            Some(_) => return Err(invalid()),
        };
        if (self.load.is_some() && self.packages.is_some())
            || (self.packages.is_some() && !self.files.is_empty())
            || self.schema != expected_schema
            || self.libraries.len() > MAX_LIBRARIES
            || self.files.len() > MAX_FILES
            || self.configuration.is_platform() != self.platform.is_some()
            || (self.platform.is_some()
                && (self.load.is_some()
                    || self.packages.is_some()
                    || !self.files.is_empty()
                    || self.generation_schema_version != Some(2)))
            || self.files.windows(2).any(|p| p[0].path >= p[1].path)
            || self
                .libraries
                .windows(2)
                .any(|p| p[0].snapshot_id >= p[1].snapshot_id)
        {
            return Err(invalid());
        }
        let mut count = 0usize;
        let mut bytes = 0usize;
        if let Some(platform) = &self.platform {
            encoded_size(self, stop)?;
            platform.validate_budget(&mut count, &mut bytes, stop)?;
        }
        let documents = self
            .load
            .as_ref()
            .map(ReplayLoad::documents)
            .transpose()?
            .unwrap_or(&[]);
        for file in self
            .files
            .iter()
            .chain(self.libraries.iter().flat_map(|lib| &lib.files))
            .chain(documents)
            .chain(
                self.packages
                    .as_ref()
                    .map(ReplayPackages::sources)
                    .transpose()?
                    .into_iter()
                    .flatten(),
            )
        {
            crate::analyzer::checkpoint(stop)?;
            count = count.checked_add(1).ok_or_else(exhausted)?;
            bytes = bytes.checked_add(file.text.len()).ok_or_else(exhausted)?;
            if count > MAX_FILES
                || bytes > MAX_SOURCE_BYTES
                || file.text.len() > MAX_FILE_BYTES
                || file.path.len() > 4096
            {
                return Err(exhausted());
            }
        }
        for library in &self.libraries {
            if library.universe == ReplayUniverse::BlizzardUiMain
                || library.files.windows(2).any(|p| p[0].path >= p[1].path)
                || library.files.iter().any(|file| file.fixture_ref.is_some())
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
    /// Executes the approved existing owner path over exact archived Main and
    /// Library bytes, then compares both original semantic identities.
    pub fn hydrate(&self, stop: &AtomicBool) -> ProjectResult<ProjectView> {
        self.hydrate_owner(stop)?.open_current()
    }

    /// Restore and validate the original native owner once for leased updates.
    pub(crate) fn hydrate_owner(&self, stop: &AtomicBool) -> ProjectResult<ProjectPublisher> {
        self.validate_budget(stop)?;
        let load_plan = self
            .load
            .as_ref()
            .map(|load| load.rebuild(&self.files, self.configuration.profile(), stop))
            .transpose()?;
        let package_main = self
            .packages
            .as_ref()
            .map(|packages| packages.rebuild(self.configuration.profile(), stop))
            .transpose()?;
        let package_parts = package_main.map(crate::load::ProjectPackageMainInput::into_parts);
        let platform_owner = self
            .platform
            .as_ref()
            .map(|platform| platform.rebuild(stop))
            .transpose()?;
        let config = self.configuration.rebuild(
            load_plan.as_ref(),
            package_parts.as_ref().map(|(_, load, main)| (load, main)),
            platform_owner.clone(),
        )?;
        let backend = config.analyzer_binding().backend().clone();
        let limits = LuaWorkspaceLimits::new(
            MAX_FILES as u64,
            4096,
            MAX_FILE_BYTES as u64,
            MAX_SOURCE_BYTES as u64,
        )
        .map_err(|_| exhausted())?;
        let mut libraries = Vec::new();
        for library in &self.libraries {
            crate::analyzer::checkpoint(stop)?;
            let snapshot = LuaWorkspaceSnapshot::build(
                backend.clone(),
                library.universe.restore(),
                library
                    .files
                    .iter()
                    .map(|file| LuaWorkspaceFileInput::new(file.path.clone(), file.text.clone()))
                    .collect(),
                limits,
            )
            .map_err(|_| invalid())?;
            if snapshot.snapshot_id() != library.snapshot_id {
                return Err(invalid());
            }
            libraries.push(snapshot);
        }
        let files = if let Some(owner) = platform_owner {
            self.platform
                .as_ref()
                .ok_or_else(invalid)?
                .main_files(&owner)?
        } else if let Some((files, _, _)) = package_parts {
            self.packages
                .as_ref()
                .ok_or_else(invalid)?
                .restore_fixture_refs(files)?
        } else {
            self.files
                .iter()
                .map(|file| {
                    ProjectInputFile::declared(
                        file.path.clone(),
                        file.text.clone(),
                        crate::ProjectLanguageKind::Lua,
                        crate::ProjectFileRole::FirstPartyMain,
                        file.fixture_ref.clone(),
                    )
                })
                .collect::<ProjectResult<Vec<_>>>()?
        };
        let bundle = ProjectInputBundle::closed(config, files, libraries)?;
        let mut publisher = if self.generation_schema_version.is_none() {
            ProjectPublisher::legacy_replay(self.function_calls)
        } else if self.function_calls {
            ProjectPublisher::with_function_call_facts()
        } else {
            ProjectPublisher::new()
        };
        let snapshot = publisher.publish_initial_cancellable(bundle, stop)?;
        if snapshot.snapshot_id() != self.project_snapshot_id
            || snapshot.analyzer_binding().analyzer_snapshot_id() != self.analyzer_snapshot_id
            || Self::capture(&publisher, stop)? != *self
        {
            return Err(invalid());
        }
        crate::analyzer::checkpoint(stop)?;
        Ok(publisher)
    }

    /// Whether this archive carries the modern Library-bound physical profile
    /// whose retained owner may accept a durable update. Legacy v1/v2/v3 and
    /// standalone or package corpora remain read-only compatibility records.
    pub(crate) fn supports_physical_update(&self) -> bool {
        self.generation_schema_version == Some(2)
            && self.load.is_none()
            && self.packages.is_none()
            && self.platform.is_none()
    }
    fn storage_schema(&self) -> &'static str {
        if self.platform.is_some() {
            "wow-project.live-replay.v5"
        } else if self.generation_schema_version == Some(2) {
            "wow-project.live-replay.v4"
        } else if self.packages.is_some() {
            "wow-project.live-replay.v3"
        } else if self.load.is_some() {
            "wow-project.live-replay.v2"
        } else {
            "wow-project.live-replay.v1"
        }
    }
}
/// Same envelope shape as a platform archive, with a four-byte null in place of
/// its corpus. Charge the remaining record budget before that corpus is copied.
#[derive(Serialize)]
struct PlatformEnvelopeBudget<'a> {
    schema: &'a str,
    configuration: &'a ReplayConfiguration,
    files: &'a [ReplayFile],
    libraries: &'a [ReplayLibrary],
    function_calls: bool,
    project_snapshot_id: &'a str,
    analyzer_snapshot_id: &'a str,
    generation_schema_version: u64,
    platform: (),
}

fn encoded_size(value: &impl Serialize, stop: &AtomicBool) -> ProjectResult<usize> {
    encoded_size_with_limit(value, wow_store::project::MAX_RECORD_BYTES, stop)
}

fn encoded_size_with_limit(
    value: &impl Serialize,
    limit: usize,
    stop: &AtomicBool,
) -> ProjectResult<usize> {
    use std::io::Write;
    struct Counter<'a> {
        used: usize,
        limit: usize,
        stop: &'a AtomicBool,
    }
    impl Write for Counter<'_> {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.stop.load(std::sync::atomic::Ordering::Relaxed) {
                return Err(std::io::Error::other("replay serialization cancelled"));
            }
            self.used = self
                .used
                .checked_add(bytes.len())
                .filter(|used| *used <= self.limit)
                .ok_or_else(|| std::io::Error::other("replay record budget exceeded"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter {
        used: 0,
        limit,
        stop,
    };
    let result = serde_json::to_writer(&mut counter, value);
    crate::analyzer::checkpoint(stop)?;
    result.map_err(|_| exhausted())?;
    Ok(counter.used)
}

fn invalid() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SnapshotInvalid,
        ProjectPhase::Publication,
        "native project replay is unavailable or inconsistent",
    )
}
fn exhausted() -> ProjectError {
    ProjectError::new(
        ProjectErrorCode::SourceBudgetExceeded,
        ProjectPhase::Publication,
        "native project replay exceeds the admitted archive profile",
    )
}
