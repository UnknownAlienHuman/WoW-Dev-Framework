use std::ffi::OsString;
use std::path::PathBuf;
use wow_service::LocalCommand;

pub const HELP: &str = "wow status --config <project.json> [--project <ProjectId>] [--detail summary|capabilities] [--format json|text]\nwow check --config <project.json> --project <ProjectId> [--generation current|<ProjectGenerationId>] [--file <ProjectFileId> ...] [--rule <RuleId>@1 ...] [--format json|text]\nwow graph build --config <project.json> --project <ProjectId> [--generation current|<ProjectGenerationId>] [--format json|snapshot|text]\nwow graph entity|neighbors|subgraph|axis|explain|path --snapshot <partition-snapshot.json> --request <query.json> [--format json|text]\n\nInput is an explicit file manifest or materialized project, Library and reference view. No discovery, source execution, source writes or persistent current pointer.\n";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Text,
}

pub struct Arguments {
    pub config: PathBuf,
    pub command: LocalCommand,
    pub format: Format,
    pub capabilities: bool,
}

pub fn parse(values: Vec<OsString>) -> Result<Arguments, &'static str> {
    let total = values
        .iter()
        .try_fold(0usize, |total, value| total.checked_add(value.len()))
        .ok_or("argument size overflow")?;
    if values.len() > 4096 || total > 1024 * 1024 {
        return Err("argument limit exceeded");
    }
    let mut values = values.into_iter();
    let command = values.next().ok_or("missing command")?;
    let command = command.to_str().ok_or("invalid command encoding")?;
    if !matches!(command, "status" | "check") {
        return Err("unknown command");
    }
    let mut config = None;
    let mut project = None;
    let mut generation = None;
    let mut format = None;
    let mut detail = None;
    let mut files = Vec::new();
    let mut rules = Vec::new();
    while let Some(option) = values.next() {
        let option = option.to_str().ok_or("invalid option encoding")?;
        if !matches!(
            option,
            "--config"
                | "--project"
                | "--generation"
                | "--format"
                | "--detail"
                | "--file"
                | "--rule"
        ) {
            return Err("unknown option");
        }
        let value = values.next().ok_or("missing option value")?;
        if option == "--config" {
            if config.replace(PathBuf::from(value)).is_some() {
                return Err("duplicate --config");
            }
            continue;
        }
        let value = value
            .into_string()
            .map_err(|_| "invalid option value encoding")?;
        if value.len() > 4096 || value.chars().any(char::is_control) {
            return Err("invalid option value");
        }
        match option {
            "--project" => set(&mut project, value)?,
            "--generation" if command == "check" => set(&mut generation, value)?,
            "--format" => set(&mut format, value)?,
            "--detail" if command == "status" => set(&mut detail, value)?,
            "--file" if command == "check" => files.push(value.into_boxed_str()),
            "--rule" if command == "check" => rules.push(value.into_boxed_str()),
            _ => return Err("option is not supported by this command"),
        }
    }
    let format = match format.as_deref().unwrap_or("json") {
        "json" => Format::Json,
        "text" => Format::Text,
        _ => return Err("unknown output format"),
    };
    let capabilities = match detail.as_deref().unwrap_or("summary") {
        "summary" => false,
        "capabilities" => true,
        _ => return Err("unknown status detail"),
    };
    let command = if command == "status" {
        LocalCommand::status(project)
    } else {
        LocalCommand::check(
            project.ok_or("check requires --project")?,
            generation.unwrap_or_else(|| "current".into()),
            files,
            rules,
        )
    }
    .map_err(|_| "invalid project, generation, file or rule selector")?;
    Ok(Arguments {
        config: config.ok_or("--config is required; implicit discovery is disabled")?,
        command,
        format,
        capabilities,
    })
}

fn set(slot: &mut Option<String>, value: String) -> Result<(), &'static str> {
    if slot.replace(value).is_some() {
        Err("duplicate option")
    } else {
        Ok(())
    }
}
