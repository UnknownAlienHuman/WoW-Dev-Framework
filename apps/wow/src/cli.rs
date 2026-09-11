use std::ffi::OsString;
use std::path::PathBuf;

use wow_service::{CheckScope, GenerationSelector, OperationId};

use crate::output::AppError;

pub(crate) const USAGE: &str = "Usage:\n  wow status --input <context.json> --operation-id <id>\n  wow check --input <context.json> --operation-id <id> (--exact <generation> | --current <project>) (--all | --file <path>...)\n  wow --help\n  wow --version";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Command {
    Status,
    Check {
        selector: GenerationSelector,
        scope: CheckScope,
    },
    Help,
    Version,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Arguments {
    pub command: Command,
    pub input: Option<PathBuf>,
    pub operation_id: Option<OperationId>,
}

impl Arguments {
    pub(crate) fn parse<I>(arguments: I) -> Result<Self, AppError>
    where
        I: IntoIterator<Item = OsString>,
    {
        let mut arguments = arguments.into_iter();
        let _program = arguments.next();
        let Some(command) = arguments.next() else {
            return Err(AppError::usage(USAGE));
        };
        let command = command
            .into_string()
            .map_err(|_| AppError::usage("command is not valid UTF-8"))?;
        if matches!(command.as_str(), "--help" | "-h" | "help") {
            ensure_empty(arguments)?;
            return Ok(Self {
                command: Command::Help,
                input: None,
                operation_id: None,
            });
        }
        if matches!(command.as_str(), "--version" | "-V" | "version") {
            ensure_empty(arguments)?;
            return Ok(Self {
                command: Command::Version,
                input: None,
                operation_id: None,
            });
        }
        if !matches!(command.as_str(), "status" | "check") {
            return Err(AppError::usage(format!(
                "unknown command {command:?}\n{USAGE}"
            )));
        }

        let mut input = None;
        let mut operation_id = None;
        let mut exact = None;
        let mut current = None;
        let mut all = false;
        let mut files = Vec::new();
        let mut remaining = arguments.peekable();
        while let Some(flag) = remaining.next() {
            let flag = flag
                .into_string()
                .map_err(|_| AppError::usage("argument is not valid UTF-8"))?;
            match flag.as_str() {
                "--input" => {
                    set_once(
                        &mut input,
                        PathBuf::from(required_value(&mut remaining, "--input")?),
                        "--input",
                    )?;
                }
                "--operation-id" => {
                    let value = required_value(&mut remaining, "--operation-id")?;
                    set_once(
                        &mut operation_id,
                        OperationId::new(value).map_err(AppError::from)?,
                        "--operation-id",
                    )?;
                }
                "--exact" => {
                    let value = required_value(&mut remaining, "--exact")?;
                    set_once(
                        &mut exact,
                        GenerationSelector::exact(value).map_err(AppError::from)?,
                        "--exact",
                    )?;
                }
                "--current" => {
                    let value = required_value(&mut remaining, "--current")?;
                    set_once(
                        &mut current,
                        GenerationSelector::current_published(value).map_err(AppError::from)?,
                        "--current",
                    )?;
                }
                "--all" => {
                    if all {
                        return Err(AppError::usage("duplicate --all"));
                    }
                    all = true;
                }
                "--file" => files.push(required_value(&mut remaining, "--file")?),
                _ => {
                    return Err(AppError::usage(format!(
                        "unknown argument {flag:?}\n{USAGE}"
                    )));
                }
            }
        }

        let input = input.ok_or_else(|| AppError::usage("missing --input"))?;
        let operation_id = operation_id.ok_or_else(|| AppError::usage("missing --operation-id"))?;
        let command = if command == "status" {
            if exact.is_some() || current.is_some() || all || !files.is_empty() {
                return Err(AppError::usage(
                    "status does not accept generation or scope arguments",
                ));
            }
            Command::Status
        } else {
            let selector = match (exact, current) {
                (Some(selector), None) | (None, Some(selector)) => selector,
                (None, None) => {
                    return Err(AppError::usage(
                        "check requires exactly one of --exact or --current",
                    ));
                }
                (Some(_), Some(_)) => {
                    return Err(AppError::usage(
                        "--exact and --current are mutually exclusive",
                    ));
                }
            };
            let scope = match (all, files.is_empty()) {
                (true, true) => CheckScope::WholeProject,
                (false, false) => CheckScope::files(files).map_err(AppError::from)?,
                (false, true) => {
                    return Err(AppError::usage(
                        "check requires exactly one of --all or --file",
                    ));
                }
                (true, false) => {
                    return Err(AppError::usage(
                        "--all and --file are mutually exclusive",
                    ));
                }
            };
            Command::Check { selector, scope }
        };
        Ok(Self {
            command,
            input: Some(input),
            operation_id: Some(operation_id),
        })
    }
}

fn required_value<I>(arguments: &mut std::iter::Peekable<I>, flag: &str) -> Result<String, AppError>
where
    I: Iterator<Item = OsString>,
{
    arguments
        .next()
        .ok_or_else(|| AppError::usage(format!("missing value for {flag}")))?
        .into_string()
        .map_err(|_| AppError::usage(format!("value for {flag} is not valid UTF-8")))
}

fn set_once<T>(target: &mut Option<T>, value: T, flag: &str) -> Result<(), AppError> {
    if target.replace(value).is_some() {
        return Err(AppError::usage(format!("duplicate {flag}")));
    }
    Ok(())
}

fn ensure_empty<I>(mut remaining: I) -> Result<(), AppError>
where
    I: Iterator<Item = OsString>,
{
    if remaining.next().is_some() {
        Err(AppError::usage(
            "help and version do not accept additional arguments",
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn parses_exact_whole_project_check() {
        let parsed = Arguments::parse(args(&[
            "wow",
            "check",
            "--input",
            "context.json",
            "--operation-id",
            "operation:1",
            "--exact",
            "generation:1",
            "--all",
        ]))
        .expect("valid arguments");
        assert!(matches!(
            parsed.command,
            Command::Check {
                selector: GenerationSelector::Exact(_),
                scope: CheckScope::WholeProject
            }
        ));
    }

    #[test]
    fn refuses_implicit_or_ambiguous_selection() {
        for values in [
            vec![
                "wow",
                "check",
                "--input",
                "context.json",
                "--operation-id",
                "operation:1",
                "--all",
            ],
            vec![
                "wow",
                "check",
                "--input",
                "context.json",
                "--operation-id",
                "operation:1",
                "--exact",
                "generation:1",
                "--current",
                "project:1",
                "--all",
            ],
            vec![
                "wow",
                "check",
                "--input",
                "context.json",
                "--operation-id",
                "operation:1",
                "--exact",
                "generation:1",
            ],
        ] {
            assert!(Arguments::parse(args(&values)).is_err());
        }
    }
}
