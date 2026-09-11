#![forbid(unsafe_code)]

use std::{
    collections::BTreeMap,
    fs,
    io::{self, Write},
    path::PathBuf,
};

use serde::Serialize;
use wow_service::reference_admin::{
    PublishReferenceRequest, ReferenceAdminConfiguration, ReferenceAdminError,
    ReferenceAdminErrorCode, ReferenceAdminExpectation, ReferenceAdminObjectId,
    ReferenceAdminOperationId, ReferenceAdminPublicationKey, ReferenceAdminService,
    ReferenceAdminStoreLimits,
};

const MAX_INPUT_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug)]
enum CliError {
    Usage(Box<str>),
    Io(Box<str>),
    Service(ReferenceAdminError),
}

impl From<ReferenceAdminError> for CliError {
    fn from(source: ReferenceAdminError) -> Self {
        Self::Service(source)
    }
}

#[derive(Debug, Default)]
struct Options {
    values: BTreeMap<Box<str>, Box<str>>,
    flags: BTreeMap<Box<str>, bool>,
}

impl Options {
    fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self, CliError> {
        let mut arguments = arguments.into_iter();
        let mut options = Self::default();
        while let Some(argument) = arguments.next() {
            if argument == "--expect-absent" {
                if options.flags.insert(argument.into(), true).is_some() {
                    return Err(CliError::Usage("duplicate --expect-absent".into()));
                }
                continue;
            }
            if !argument.starts_with("--") {
                return Err(CliError::Usage("unexpected positional argument".into()));
            }
            let value = arguments
                .next()
                .ok_or_else(|| CliError::Usage("option is missing its value".into()))?;
            if value.starts_with("--") {
                return Err(CliError::Usage("option is missing its value".into()));
            }
            if options
                .values
                .insert(argument.into(), value.into())
                .is_some()
            {
                return Err(CliError::Usage("duplicate option".into()));
            }
        }
        Ok(options)
    }

    fn required(&self, name: &str) -> Result<&str, CliError> {
        self.values
            .get(name)
            .map(AsRef::as_ref)
            .ok_or_else(|| CliError::Usage(format!("missing {name}").into()))
    }

    fn optional(&self, name: &str) -> Option<&str> {
        self.values.get(name).map(AsRef::as_ref)
    }

    fn flag(&self, name: &str) -> bool {
        self.flags.get(name).copied().unwrap_or(false)
    }

    fn reject_unknown(&self, allowed_values: &[&str], allowed_flags: &[&str]) -> Result<(), CliError> {
        if self
            .values
            .keys()
            .any(|name| !allowed_values.contains(&name.as_ref()))
            || self
                .flags
                .keys()
                .any(|name| !allowed_flags.contains(&name.as_ref()))
        {
            return Err(CliError::Usage("unknown option".into()));
        }
        Ok(())
    }
}

pub fn run(
    arguments: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    match execute(arguments, stdout) {
        Ok(()) => 0,
        Err(CliError::Usage(message)) => {
            let _ = writeln!(stderr, "usage_error: {message}");
            64
        }
        Err(CliError::Io(message)) => {
            let _ = writeln!(stderr, "io_error: {message}");
            65
        }
        Err(CliError::Service(error)) => {
            let _ = writeln!(stderr, "service_error:{:?}: {}", error.code(), error);
            match error.code() {
                ReferenceAdminErrorCode::PublicationConflict
                | ReferenceAdminErrorCode::OperationConflict
                | ReferenceAdminErrorCode::OperationIncomplete
                | ReferenceAdminErrorCode::OutcomeUnknown => 3,
                _ => 4,
            }
        }
    }
}

fn execute(
    arguments: impl IntoIterator<Item = String>,
    stdout: &mut dyn Write,
) -> Result<(), CliError> {
    let mut arguments = arguments.into_iter();
    let command = arguments
        .next()
        .ok_or_else(|| CliError::Usage("missing command".into()))?;
    let options = Options::parse(arguments)?;
    match command.as_str() {
        "status" => status(&options, stdout),
        "validate" => validate(&options, stdout),
        "publish" => publish(&options, stdout),
        "read" => read(&options, stdout),
        _ => Err(CliError::Usage("unknown command".into())),
    }
}

fn configuration(options: &Options) -> Result<ReferenceAdminConfiguration, CliError> {
    Ok(ReferenceAdminConfiguration::new(
        options.required("--store-profile")?,
        ReferenceAdminStoreLimits::default(),
        MAX_INPUT_BYTES,
    )?)
}

fn status(options: &Options, stdout: &mut dyn Write) -> Result<(), CliError> {
    options.reject_unknown(
        &["--db", "--store-profile", "--profile", "--channel"],
        &[],
    )?;
    let key = ReferenceAdminPublicationKey::new(
        options.required("--profile")?,
        options.required("--channel")?,
    )?;
    let mut service = ReferenceAdminService::open(
        PathBuf::from(options.required("--db")?),
        configuration(options)?,
    )?;
    write_json(stdout, &service.status(&key, 100_000)?)
}

fn validate(options: &Options, stdout: &mut dyn Write) -> Result<(), CliError> {
    options.reject_unknown(&["--store-profile", "--input"], &[])?;
    let bytes = read_bounded(options.required("--input")?)?;
    let service = ReferenceAdminService::open_in_memory(configuration(options)?)?;
    #[derive(Serialize)]
    struct ResultRecord<'a> {
        schema: &'static str,
        status: &'static str,
        reference_view_sha256: &'a str,
    }
    let digest = service.validate_view_bytes(&bytes)?;
    write_json(
        stdout,
        &ResultRecord {
            schema: "wow-reference-builder/validate/1",
            status: "valid",
            reference_view_sha256: &digest,
        },
    )
}

fn publish(options: &Options, stdout: &mut dyn Write) -> Result<(), CliError> {
    options.reject_unknown(
        &[
            "--db",
            "--store-profile",
            "--profile",
            "--channel",
            "--operation",
            "--input",
            "--expect-object",
        ],
        &["--expect-absent"],
    )?;
    let absent = options.flag("--expect-absent");
    let exact = options.optional("--expect-object");
    let expectation = match (absent, exact) {
        (true, None) => ReferenceAdminExpectation::Absent,
        (false, Some(value)) => {
            ReferenceAdminExpectation::Exact(ReferenceAdminObjectId::new(value)?)
        }
        _ => {
            return Err(CliError::Usage(
                "select exactly one publication expectation".into(),
            ));
        }
    };
    let bytes = read_bounded(options.required("--input")?)?;
    let request = PublishReferenceRequest::new(
        ReferenceAdminOperationId::new(options.required("--operation")?)?,
        ReferenceAdminPublicationKey::new(
            options.required("--profile")?,
            options.required("--channel")?,
        )?,
        expectation,
        bytes,
    );
    let mut service = ReferenceAdminService::open(
        PathBuf::from(options.required("--db")?),
        configuration(options)?,
    )?;
    write_json(stdout, &service.publish(request)?)
}

fn read(options: &Options, stdout: &mut dyn Write) -> Result<(), CliError> {
    options.reject_unknown(
        &["--db", "--store-profile", "--object"],
        &[],
    )?;
    let object_id = ReferenceAdminObjectId::new(options.required("--object")?)?;
    let mut service = ReferenceAdminService::open(
        PathBuf::from(options.required("--db")?),
        configuration(options)?,
    )?;
    let view = service
        .read_exact(&object_id)?
        .ok_or_else(|| CliError::Service(ReferenceAdminError::not_found()))?;
    write_json(stdout, &view)
}

fn read_bounded(path: &str) -> Result<Vec<u8>, CliError> {
    let metadata = fs::metadata(path)
        .map_err(|_| CliError::Io("input metadata is unavailable".into()))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_INPUT_BYTES {
        return Err(CliError::Io(
            "input is not a bounded nonempty regular file".into(),
        ));
    }
    let bytes = fs::read(path).map_err(|_| CliError::Io("input read failed".into()))?;
    if bytes.len() as u64 != metadata.len() {
        return Err(CliError::Io("input changed while being read".into()));
    }
    Ok(bytes)
}

fn write_json(writer: &mut dyn Write, value: &impl Serialize) -> Result<(), CliError> {
    let mut bytes = serde_json::to_vec(value)
        .map_err(|_| CliError::Io("result serialization failed".into()))?;
    bytes.push(b'\n');
    match writer.write_all(&bytes) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(_) => Err(CliError::Io("result write failed".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_unknown_and_ambiguous_options_fail_before_service() {
        for arguments in [
            vec!["status", "--db", "a", "--db", "b"],
            vec!["validate", "--unknown", "x"],
            vec![
                "publish",
                "--expect-absent",
                "--expect-object",
                "store-object:sha256:0000000000000000000000000000000000000000000000000000000000000000",
            ],
        ] {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let exit = run(
                arguments.into_iter().map(str::to_owned),
                &mut stdout,
                &mut stderr,
            );
            assert_ne!(exit, 0);
            assert!(stdout.is_empty());
        }
    }

    #[test]
    fn broken_pipe_never_retries_or_changes_semantics() {
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
                Err(io::Error::from(io::ErrorKind::BrokenPipe))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        #[derive(Serialize)]
        struct Value {
            value: u8,
        }
        assert!(write_json(&mut Broken, &Value { value: 1 }).is_ok());
    }
}
