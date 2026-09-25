//! Explicit persistence transport; no SQL or graph admission in the CLI.
use crate::args::Format;
use std::{
    ffi::{OsStr, OsString},
    io::Write,
    path::PathBuf,
    sync::atomic::Ordering,
};
use wow_service::{
    ServiceErrorCode,
    graph::{
        GRAPH_BUNDLE_MAX_BYTES, GraphStorePublishRequest, publish_graph_bundle,
        reconcile_graph_publication,
    },
};

struct Arguments {
    root: PathBuf,
    operation: String,
    bundle: Option<PathBuf>,
    expected: Option<String>,
    initialize: bool,
    allow_partial: bool,
    format: Format,
}
pub fn run(values: Vec<OsString>) -> u8 {
    let args = match parse(values) {
        Ok(a) => a,
        Err(e) => return super::usage(e),
    };
    let Some(stop) = super::cancellation_flag() else {
        return 64;
    };
    let result = if let Some(path) = &args.bundle {
        let request = match GraphStorePublishRequest::new(
            &args.operation,
            args.expected.as_deref().unwrap_or(""),
            args.initialize,
            args.allow_partial,
        ) {
            Ok(r) => r,
            Err(e) => {
                super::diagnostic(e.message());
                return 64;
            }
        };
        let bytes = match super::graph::read_file(path, GRAPH_BUNDLE_MAX_BYTES, &stop) {
            Ok(b) => b,
            Err(e) => {
                super::diagnostic(e);
                return if stop.load(Ordering::Acquire) {
                    130
                } else {
                    64
                };
            }
        };
        publish_graph_bundle(&bytes, &args.root, &request, &stop)
    } else {
        if stop.load(Ordering::Acquire) {
            return 130;
        }
        reconcile_graph_publication(&args.root, &args.operation)
    };
    let result = match result {
        Ok(r) => r,
        Err(e) => {
            super::diagnostic(e.message());
            super::diagnostic(
                "After an uncertain publication, reconcile the same operation ID; do not create a replacement operation.",
            );
            return if e.code() == ServiceErrorCode::Cancelled {
                130
            } else {
                4
            };
        }
    };
    // A completed effect is never relabeled cancelled because Ctrl-C arrived
    // after COMMIT. Output failure never retries publication.
    let exit = result.exit_code();
    let mut bytes = match result.canonical_bytes() {
        Ok(b) => b,
        Err(_) => return 4,
    };
    if args.format == Format::Text {
        let mut text =
            b"Graph store publication receipt (source metadata remains Partial)\n".to_vec();
        text.extend_from_slice(&bytes);
        bytes = text;
    }
    bytes.push(b'\n');
    let mut out = std::io::stdout().lock();
    if out.write_all(&bytes).and_then(|()| out.flush()).is_err() {
        super::diagnostic(
            "store receipt output failed; reconcile the same operation ID before retrying",
        );
        return 4;
    }
    exit
}
fn parse(values: Vec<OsString>) -> Result<Arguments, &'static str> {
    if values.len() > 64 || values.iter().map(|v| v.len()).sum::<usize>() > 128 * 1024 {
        return Err("store argument limit exceeded");
    }
    let mut values = values.into_iter();
    if values.next().as_deref() != Some(OsStr::new("graph")) {
        return Err("missing graph command");
    }
    let publish = match values.next().as_deref() {
        Some(v) if v == "publish" => true,
        Some(v) if v == "reconcile" => false,
        _ => return Err("expected publish or reconcile"),
    };
    let (mut root, mut operation, mut bundle, mut expected, mut format) =
        (None, None, None, None, None);
    let (mut initialize, mut allow_partial) = (false, false);
    while let Some(option) = values.next() {
        let option = option.to_str().ok_or("invalid store option encoding")?;
        match option {
            "--initialize" if publish && !initialize => {
                initialize = true;
                continue;
            }
            "--allow-partial" if publish && !allow_partial => {
                allow_partial = true;
                continue;
            }
            "--store-root" | "--operation-id" | "--format" => {}
            "--bundle" | "--expected-current" if publish => {}
            _ => return Err("unknown or duplicate store option"),
        }
        let value = values.next().ok_or("missing store option value")?;
        if value.is_empty() || value.len() > 32768 {
            return Err("invalid store option length");
        }
        let duplicate = match option {
            "--store-root" => root.replace(PathBuf::from(value)).is_some(),
            "--bundle" => bundle.replace(PathBuf::from(value)).is_some(),
            "--operation-id" => operation
                .replace(
                    value
                        .into_string()
                        .map_err(|_| "invalid operation ID encoding")?,
                )
                .is_some(),
            "--expected-current" => expected
                .replace(
                    value
                        .into_string()
                        .map_err(|_| "invalid current ID encoding")?,
                )
                .is_some(),
            "--format" => format
                .replace(match value.to_str() {
                    Some("json") => Format::Json,
                    Some("text") => Format::Text,
                    _ => return Err("invalid store format"),
                })
                .is_some(),
            _ => return Err("unknown store option"),
        };
        if duplicate {
            return Err("duplicate store option");
        }
    }
    if publish && (bundle.is_none() || expected.is_none() || !allow_partial) {
        return Err("publish requires --bundle, --expected-current and --allow-partial");
    }
    Ok(Arguments {
        root: root.ok_or("store command requires --store-root")?,
        operation: operation.ok_or("store command requires --operation-id")?,
        bundle,
        expected,
        initialize,
        allow_partial,
        format: format.unwrap_or(Format::Json),
    })
}
