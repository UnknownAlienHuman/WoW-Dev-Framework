//! Strict transport for live project publication, acquisition and physical recovery.
use crate::args::Format;
use std::{
    collections::BTreeMap,
    ffi::{OsStr, OsString},
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
use wow_service::{
    ServiceErrorCode,
    graph::GraphBuildRequest,
    live_project::{
        CurrentDomainObservation, CurrentState, LiveProjectLibraryMode, LiveProjectPublishRequest,
        LiveProjectUpdateRequest, RecoveryReport, ScopeState, publish_local_project,
        read_live_project, reconcile_live_project, recover_current_live_project,
        recover_live_project, update_local_project,
    },
};

const HELP: &str = concat!(
    "wow project publish --config <project.json> --project <ProjectId> [--generation current|<ProjectGenerationId>] --store-root <private-directory> --operation-id <id> --expected-current absent|<record-id> --allow-partial [--initialize] [--format json|text]\n",
    "wow project update --config <final-project.json> --project <ProjectId> [--generation current|<ProjectGenerationId>] --store-root <directory> --operation-id <id> --expected-current <record-id> --library keep|replace|clear --allow-partial [--format json|text]\n",
    "wow project read --store-root <directory> --store-generation current|<generation-id> [--format json|text]\n",
    "wow project reconcile --store-root <directory> --operation-id <id> [--format json|text]\n",
    "wow project recover --store-root <directory> [--domain-current] [--format json|text]\n\n",
    "Publish retains exact Main/Library inputs and the full native graph chain. Update applies explicit final physical Lua inputs against the exact retained base. Read acquires one leased project/graph pair; TOC/XML/package replay is supported for reads/publication. Reconcile observes the original operation. Partial coverage exits 2. See apps/wow/LIVE_PROJECT.md.\n",
    "Recover observes physical store state without initialization, repair, activation or domain approval. Validated/not-applicable coverage exits 0; incomplete/unverified exits 2; invalid/corrupt exits 4; cancellation exits 130.\n",
    "--domain-current also replays the exact observed Current through native Project/Graph owners and retains physical evidence on failure. Other generations receive no domain verdict.\n",
);

enum Command {
    Publish {
        config: PathBuf,
        graph: GraphBuildRequest,
        request: LiveProjectPublishRequest,
    },
    Update {
        config: PathBuf,
        graph: GraphBuildRequest,
        request: LiveProjectUpdateRequest,
    },
    Read {
        generation: String,
    },
    Reconcile {
        operation: String,
    },
    Recover {
        domain_current: bool,
    },
}
struct Arguments {
    root: PathBuf,
    command: Command,
    format: Format,
}

pub fn run(values: Vec<OsString>) -> u8 {
    if (values.len() == 2
        || (values.len() == 3
            && values.get(1).is_some_and(|v| {
                v == "publish" || v == "update" || v == "read" || v == "reconcile" || v == "recover"
            })))
        && values.last().is_some_and(|v| v == "--help" || v == "-h")
    {
        return if std::io::stdout().lock().write_all(HELP.as_bytes()).is_ok() {
            0
        } else {
            4
        };
    }
    let args = match parse(values) {
        Ok(a) => a,
        Err(e) => return super::usage(e),
    };
    let Some(stop) = super::cancellation_flag() else {
        return 64;
    };
    let result = match args.command {
        Command::Publish {
            config,
            graph,
            request,
        } => publish_local_project(&config, &graph, &args.root, &request, &stop),
        Command::Update {
            config,
            graph,
            request,
        } => update_local_project(&config, &graph, &args.root, &request, &stop),
        Command::Read { generation } => read_live_project(&args.root, &generation, &stop),
        Command::Reconcile { operation } => reconcile_live_project(&args.root, &operation, &stop),
        Command::Recover { domain_current } => {
            return run_recovery(&args.root, domain_current, args.format, &stop);
        }
    };
    let result = match result {
        Ok(r) => r,
        Err(e) => {
            super::diagnostic(&format!("{:?}: {}", e.code(), e.message()));
            return if e.code() == ServiceErrorCode::Cancelled {
                130
            } else {
                4
            };
        }
    };
    // A committed receipt remains committed after late Ctrl-C or output loss.
    let exit = result.exit_code();
    let mut bytes = match result.canonical_bytes() {
        Ok(b) => b,
        Err(_) => return 4,
    };
    if args.format == Format::Text {
        bytes.splice(
            0..0,
            b"Native project pair (partial source coverage)\n"
                .iter()
                .copied(),
        );
    }
    bytes.push(b'\n');
    let mut out = std::io::stdout().lock();
    if out.write_all(&bytes).and_then(|()| out.flush()).is_err() {
        super::diagnostic(
            "project output failed; reconcile the original operation ID before retrying publication",
        );
        return 4;
    }
    exit
}

fn run_recovery(root: &Path, domain_current: bool, format: Format, stop: &AtomicBool) -> u8 {
    if domain_current {
        return run_current_recovery(root, format, stop);
    }
    let report = match recover_live_project(root, stop) {
        Ok(report) => report,
        Err(error) => {
            super::diagnostic(&format!("{:?}: {}", error.code(), error.message()));
            return if error.code() == ServiceErrorCode::Cancelled {
                130
            } else {
                4
            };
        }
    };
    let exit = physical_recovery_exit(&report);
    let mut bytes = match report.canonical_bytes() {
        Ok(bytes) => bytes,
        Err(_) => {
            super::diagnostic("physical recovery report encoding failed");
            return 4;
        }
    };
    if format == Format::Text {
        bytes.splice(
            0..0,
            b"Physical project-store recovery observation\n"
                .iter()
                .copied(),
        );
    }
    if stop.load(Ordering::Acquire) {
        return 130;
    }
    write_recovery(bytes, exit)
}

fn physical_recovery_exit(report: &RecoveryReport) -> u8 {
    if report.current_state() == CurrentState::Corrupt
        || report
            .coverage()
            .iter()
            .any(|coverage| coverage.state() == ScopeState::Invalid)
    {
        4
    } else if report.current_state() == CurrentState::Unverified
        || report
            .coverage()
            .iter()
            .any(|coverage| coverage.state() == ScopeState::Incomplete)
    {
        2
    } else {
        0
    }
}

fn run_current_recovery(root: &Path, format: Format, stop: &AtomicBool) -> u8 {
    let report = match recover_current_live_project(root, stop) {
        Ok(report) => report,
        Err(error) => {
            super::diagnostic(&format!("{:?}: {}", error.code(), error.message()));
            return if error.code() == ServiceErrorCode::Cancelled {
                130
            } else {
                4
            };
        }
    };
    let physical_exit = physical_recovery_exit(report.physical());
    let exit = match report.current_domain() {
        CurrentDomainObservation::Cancelled(_) => 130,
        CurrentDomainObservation::Failed(_) => 4,
        _ if physical_exit == 4 => 4,
        CurrentDomainObservation::Unverified | CurrentDomainObservation::Incomplete(_) => 2,
        _ => physical_exit,
    };
    let mut bytes = match report.canonical_bytes() {
        Ok(bytes) => bytes,
        Err(_) => {
            super::diagnostic("current domain recovery report encoding failed");
            return 4;
        }
    };
    if format == Format::Text {
        bytes.splice(
            0..0,
            b"Exact Current Project/Graph recovery observation\n"
                .iter()
                .copied(),
        );
    }
    // A completed physical observation remains useful after replay cancellation.
    let exit = if stop.load(Ordering::Acquire) {
        130
    } else {
        exit
    };
    write_recovery(bytes, exit)
}

fn write_recovery(mut bytes: Vec<u8>, exit: u8) -> u8 {
    bytes.push(b'\n');
    let mut out = std::io::stdout().lock();
    if out.write_all(&bytes).and_then(|()| out.flush()).is_err() {
        super::diagnostic("recovery report output failed");
        return 4;
    }
    exit
}

fn parse(values: Vec<OsString>) -> Result<Arguments, &'static str> {
    if values.len() > 64 || values.iter().map(|v| v.len()).sum::<usize>() > 128 * 1024 {
        return Err("project argument limit exceeded");
    }
    let mut values = values.into_iter();
    if values.next().as_deref() != Some(OsStr::new("project")) {
        return Err("missing project command");
    }
    let action = values.next().ok_or("missing project operation")?;
    let action = action.to_str().ok_or("invalid project operation")?;
    if !matches!(
        action,
        "publish" | "update" | "read" | "reconcile" | "recover"
    ) {
        return Err("unknown project operation");
    }
    let mut options = BTreeMap::new();
    let (mut initialize, mut allow_partial, mut domain_current) = (false, false, false);
    while let Some(option) = values.next() {
        let option = option.into_string().map_err(|_| "invalid project option")?;
        match option.as_str() {
            "--initialize" if action == "publish" && !initialize => {
                initialize = true;
                continue;
            }
            "--allow-partial" if matches!(action, "publish" | "update") && !allow_partial => {
                allow_partial = true;
                continue;
            }
            "--domain-current" if action == "recover" && !domain_current => {
                domain_current = true;
                continue;
            }
            "--store-root" | "--format" => {}
            "--config" | "--project" | "--generation" | "--expected-current"
                if matches!(action, "publish" | "update") => {}
            "--library" if action == "update" => {}
            "--operation-id" if matches!(action, "publish" | "update" | "reconcile") => {}
            "--store-generation" if action == "read" => {}
            _ => return Err("unknown or duplicate project option"),
        }
        let value = values.next().ok_or("missing project option value")?;
        if value.is_empty() || value.len() > 32768 {
            return Err("invalid project option length");
        }
        if options.insert(option, value).is_some() {
            return Err("duplicate project option");
        }
    }
    let root = PathBuf::from(
        options
            .remove("--store-root")
            .ok_or("project requires --store-root")?,
    );
    let format = match text(&mut options, "--format")?.as_deref().unwrap_or("json") {
        "json" => Format::Json,
        "text" => Format::Text,
        _ => return Err("invalid project format"),
    };
    let command = match action {
        "publish" => {
            let config = PathBuf::from(
                options
                    .remove("--config")
                    .ok_or("publish requires --config")?,
            );
            let graph = GraphBuildRequest::new(
                required_text(&mut options, "--project")?,
                text(&mut options, "--generation")?.unwrap_or_else(|| "current".into()),
            )
            .map_err(|_| "invalid project or generation")?;
            let request = LiveProjectPublishRequest::new(
                &required_text(&mut options, "--operation-id")?,
                &required_text(&mut options, "--expected-current")?,
                initialize,
                allow_partial,
            )
            .map_err(|_| "publish requires valid exact guards and --allow-partial")?;
            Command::Publish {
                config,
                graph,
                request,
            }
        }
        "update" => {
            let config = PathBuf::from(
                options
                    .remove("--config")
                    .ok_or("update requires --config")?,
            );
            let graph = GraphBuildRequest::new(
                required_text(&mut options, "--project")?,
                text(&mut options, "--generation")?.unwrap_or_else(|| "current".into()),
            )
            .map_err(|_| "invalid project or generation")?;
            let libraries = match required_text(&mut options, "--library")?.as_str() {
                "keep" => LiveProjectLibraryMode::Keep,
                "replace" => LiveProjectLibraryMode::Replace,
                "clear" => LiveProjectLibraryMode::Clear,
                _ => return Err("update requires --library keep|replace|clear"),
            };
            let request = LiveProjectUpdateRequest::new(
                &required_text(&mut options, "--operation-id")?,
                &required_text(&mut options, "--expected-current")?,
                libraries,
                allow_partial,
            )
            .map_err(|_| "update requires valid exact guards and --allow-partial")?;
            Command::Update {
                config,
                graph,
                request,
            }
        }
        "read" => Command::Read {
            generation: required_text(&mut options, "--store-generation")?,
        },
        "reconcile" => Command::Reconcile {
            operation: required_text(&mut options, "--operation-id")?,
        },
        "recover" => Command::Recover { domain_current },
        _ => return Err("unknown project operation"),
    };
    Ok(Arguments {
        root,
        command,
        format,
    })
}
fn text(
    options: &mut BTreeMap<String, OsString>,
    key: &str,
) -> Result<Option<String>, &'static str> {
    options
        .remove(key)
        .map(|v| {
            let v = v
                .into_string()
                .map_err(|_| "invalid project text encoding")?;
            if v.len() > 4096 || v.chars().any(char::is_control) {
                return Err("invalid project text value");
            }
            Ok(v)
        })
        .transpose()
}
fn required_text(
    options: &mut BTreeMap<String, OsString>,
    key: &str,
) -> Result<String, &'static str> {
    text(options, key)?.ok_or("missing required project option")
}
