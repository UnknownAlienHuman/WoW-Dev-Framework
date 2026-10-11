//! Closed E3 context-map and inspect projections. Acquisition and semantics are service-owned.
//! The native host document is a separate transport input, never an E3 semantic selector.
mod inspect;

use std::{
    ffi::{OsStr, OsString},
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use wow_service::{
    ServiceErrorCode,
    context::{
        CONTEXT_MAP_HOST_INPUT_MAX_BYTES, CONTEXT_MAP_ID_MAX_BYTES, CONTEXT_MAP_RESULT_MAX_BYTES,
        ContextInspectRequest, ContextMapCurrentGuard, ContextMapHostInput, ContextMapPublication,
        ContextMapReferenceGuard, ContextMapRequest, ContextMapRoot, ContextMapStatus,
        PRIMARY_NAVIGATION_MAP_PROFILE_ID, PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID,
        execute_context_inspect, execute_context_map,
    },
};

const HELP: &str = concat!(
    "wow context map --native-host-config <host.json> --project-store <registration-key> ",
    "--publication current|store-generation:<ProjectStoreGenerationId> ",
    "--map-target primary --map-profile-id <exact-reviewed-profile-id>\n",
    "  [--root project@<base64url-id>|graph-entity@<base64url-id>] [--platform omitted]\n",
    "  [--expect-current-record <id>] [--expect-current-digest <digest>]\n",
    "  [--expect-profile <id> --expect-reference-generation <id> --expect-reference-view <id>]\n",
    "  [--output envelope-json]\n",
    "The native host document is explicit transport configuration, separate from semantic ",
    "store IDs. No source discovery or execution. Only the reviewed primary direct-navigation ",
    "and project-navigation map profiles, omitted platform and canonical envelope JSON are supported.\n",
    "Use an explicit graph-entity@ root with wow-context/primary-direct-navigation/1, ",
    "or an explicit project@ root with wow-context/primary-project-navigation/1.\n",
    "Service validates the selected profile/root pair; the CLI never infers a root.\n",
    "wow context inspect --native-host-config <host.json> --project-store <registration-key> ",
    "--publication current|store-generation:<ProjectStoreGenerationId> ",
    "--root <exact-root-token> --level l0|l1|both\n",
    "  [--l0-profile-id wow-context/primary-l0/1] ",
    "[--l1-profile-id wow-context/primary-l1/1]\n",
    "  Same omitted-platform, complete reference/current guards and envelope-json transport. ",
    "L0 accepts project@ or an exact container graph-entity@; L1 accepts graph-entity@; ",
    "both requires an exact container graph-entity@. Service validates kind/existence ",
    "and keeps the two artifacts distinct.\n",
);

// Preserve the existing app argv/individual-path transport maxima.
const MAX_ARGUMENTS: usize = 4096;
const MAX_ARGUMENT_BYTES: usize = 1024 * 1024;
const MAX_VALUE_BYTES: usize = 32768;

#[derive(Clone, Copy)]
enum Command {
    Map,
    Inspect,
}

enum Request {
    Map(ContextMapRequest),
    Inspect(ContextInspectRequest),
}

struct Arguments {
    host_config: PathBuf,
    request: Request,
}

pub fn run(values: Vec<OsString>) -> u8 {
    if values.first().is_some_and(|value| value == "context")
        && (values.len() == 2
            || (values.len() == 3
                && values
                    .get(1)
                    .is_some_and(|value| value == "map" || value == "inspect")))
        && values
            .last()
            .is_some_and(|value| value == "--help" || value == "-h")
    {
        return if std::io::stdout().lock().write_all(HELP.as_bytes()).is_ok() {
            0
        } else {
            4
        };
    }
    let arguments = match parse(values) {
        Ok(arguments) => arguments,
        Err(message) => return crate::usage(message),
    };
    let Some(stop) = crate::cancellation_flag() else {
        return 64;
    };
    let host_bytes = match crate::graph::read_file(
        &arguments.host_config,
        CONTEXT_MAP_HOST_INPUT_MAX_BYTES,
        &stop,
    ) {
        Ok(bytes) => bytes,
        Err(_) => return input_failure("cannot read bounded native context host input", &stop),
    };
    // Pure strict transport decoding only; this must not acquire any owner view.
    let host = match ContextMapHostInput::from_json_slice(&host_bytes, &stop) {
        Ok(host) => host,
        Err(error) if error.code() == ServiceErrorCode::Cancelled => return 130,
        Err(_) => return input_failure("invalid native context host configuration", &stop),
    };
    if stop.load(Ordering::Acquire) {
        return 130;
    }

    // Exactly one semantic operation. Service pins Current once and closes resources.
    let operation = match &arguments.request {
        Request::Map(request) => execute_context_map(&host, request, &stop),
        Request::Inspect(request) => execute_context_inspect(&host, request, &stop),
    };
    let result = match operation {
        Ok(result) => result,
        Err(error) if error.code() == ServiceErrorCode::Cancelled => return 130,
        Err(_) => {
            crate::diagnostic("context operation could not finalize a result");
            return 4;
        }
    };
    let bytes = result.canonical_bytes();
    if bytes.is_empty() || bytes.len() > CONTEXT_MAP_RESULT_MAX_BYTES {
        crate::diagnostic("context result byte limit or encoding failure");
        return 4;
    }
    let exit = match result.status() {
        ContextMapStatus::Partial | ContextMapStatus::Truncated => 2,
        ContextMapStatus::Cancelled => 130,
        ContextMapStatus::Failed => match result.failure_code() {
            Some(
                ServiceErrorCode::InternalContractViolation
                | ServiceErrorCode::CanonicalizationFailed,
            ) => 4,
            _ => 3,
        },
    };
    // Emit the finalized service envelope unchanged, even when Failure/Cancelled.
    // Never replace a finalized result or retry after output has started.
    let mut output = std::io::stdout().lock();
    if output
        .write_all(bytes)
        .and_then(|()| output.write_all(b"\n"))
        .and_then(|()| output.flush())
        .is_err()
    {
        crate::diagnostic("context output failed");
        return if stop.load(Ordering::Acquire) { 130 } else { 4 };
    }
    exit
}

fn input_failure(message: &'static str, stop: &AtomicBool) -> u8 {
    if stop.load(Ordering::Acquire) {
        130
    } else {
        crate::usage(message)
    }
}

fn parse(values: Vec<OsString>) -> Result<Arguments, &'static str> {
    let bytes = values
        .iter()
        .try_fold(0usize, |total, value| total.checked_add(value.len()))
        .ok_or("context argument size overflow")?;
    if values.len() > MAX_ARGUMENTS || bytes > MAX_ARGUMENT_BYTES {
        return Err("context argument limit exceeded");
    }
    let mut values = values.into_iter();
    if values.next().as_deref() != Some(OsStr::new("context")) {
        return Err("expected context map or inspect");
    }
    let command = match values.next().as_deref().and_then(OsStr::to_str) {
        Some("map") => Command::Map,
        Some("inspect") => Command::Inspect,
        _ => return Err("expected context map or inspect"),
    };
    let mut host_config = None;
    let mut project_store = None;
    let mut publication = None;
    let mut current_record = None;
    let mut current_digest = None;
    let mut platform = None;
    let mut reference_profile = None;
    let mut reference_generation = None;
    let mut reference_view = None;
    let mut map_target = None;
    let mut root = None;
    let mut map_profile = None;
    let mut output = None;
    let mut inspect_options = inspect::Options::default();

    while let Some(option) = values.next() {
        let option = option.to_str().ok_or("invalid context option encoding")?;
        let common = matches!(
            option,
            "--native-host-config"
                | "--project-store"
                | "--publication"
                | "--expect-current-record"
                | "--expect-current-digest"
                | "--platform"
                | "--expect-profile"
                | "--expect-reference-generation"
                | "--expect-reference-view"
                | "--root"
                | "--output"
        );
        let selected = match command {
            Command::Map => matches!(option, "--map-target" | "--map-profile-id"),
            Command::Inspect => inspect::Options::accepts(option),
        };
        if !common && !selected {
            return Err(match command {
                Command::Map => "unknown context map option",
                Command::Inspect => "unknown context inspect option",
            });
        }
        let value = values.next().ok_or("missing context option value")?;
        if value.is_empty() || value.len() > MAX_VALUE_BYTES {
            return Err("invalid context option length");
        }
        if option == "--native-host-config" {
            set(&mut host_config, PathBuf::from(value), command)?;
            continue;
        }
        let value = value
            .into_string()
            .map_err(|_| "invalid context value encoding")?;
        if value.chars().any(char::is_control) || value.trim() != value {
            return Err("invalid context option value");
        }
        match option {
            "--project-store" => {
                id_syntax(&value)?;
                set(&mut project_store, value, command)?;
            }
            "--publication" => {
                let selected = if value == "current" {
                    ContextMapPublication::Current
                } else if let Some(id) = value.strip_prefix("store-generation:") {
                    id_syntax(id)?;
                    ContextMapPublication::store_generation(id)
                        .map_err(|_| "invalid exact context store generation")?
                } else {
                    return Err("unsupported context publication selector");
                };
                set(&mut publication, selected, command)?;
            }
            "--expect-current-record" => {
                id_syntax(&value)?;
                set(&mut current_record, value, command)?;
            }
            "--expect-current-digest" => {
                id_syntax(&value)?;
                set(&mut current_digest, value, command)?;
            }
            "--platform" => {
                if value != "omitted" {
                    return Err("context requires omitted platform");
                }
                set(&mut platform, (), command)?;
            }
            "--expect-profile" => {
                id_syntax(&value)?;
                set(&mut reference_profile, value, command)?;
            }
            "--expect-reference-generation" => {
                id_syntax(&value)?;
                set(&mut reference_generation, value, command)?;
            }
            "--expect-reference-view" => {
                id_syntax(&value)?;
                set(&mut reference_view, value, command)?;
            }
            "--map-target" => {
                if value != "primary" {
                    return Err("context map requires primary target");
                }
                set(&mut map_target, (), command)?;
            }
            "--root" => set(&mut root, parse_root(&value)?, command)?,
            "--map-profile-id" => {
                if value != PRIMARY_NAVIGATION_MAP_PROFILE_ID
                    && value != PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID
                {
                    return Err("unsupported exact context map profile");
                }
                set(&mut map_profile, value, command)?;
            }
            "--output" => {
                if value != "envelope-json" {
                    return Err("context supports envelope-json only");
                }
                set(&mut output, (), command)?;
            }
            "--level" | "--l0-profile-id" | "--l1-profile-id" => {
                inspect_options.option(option, &value)?;
            }
            _ => {
                return Err(match command {
                    Command::Map => "unknown context map option",
                    Command::Inspect => "unknown context inspect option",
                });
            }
        }
    }
    let host_config = host_config.ok_or("context requires explicit native host input")?;
    let project_store = project_store.ok_or("context requires --project-store")?;
    let publication = publication.ok_or("context requires --publication")?;
    let current_guard = if current_record.is_some() || current_digest.is_some() {
        if !matches!(&publication, ContextMapPublication::Current) {
            return Err("current guards require --publication current");
        }
        Some(
            ContextMapCurrentGuard::new(current_record.as_deref(), current_digest.as_deref())
                .map_err(|_| "invalid exact context current guard")?,
        )
    } else {
        None
    };
    let reference_guard = match (reference_profile, reference_generation, reference_view) {
        (None, None, None) => None,
        (Some(profile), Some(generation), Some(view)) => Some(
            ContextMapReferenceGuard::new(&profile, &generation, &view)
                .map_err(|_| "invalid exact context reference guards")?,
        ),
        _ => return Err("context reference guards require the complete three-option group"),
    };
    // These constructors validate syntax/canonical bytes only. They do not resolve
    // existence, aliases, Current, owner compatibility or filesystem identities.
    let request = match command {
        Command::Map => {
            map_target.ok_or("context map requires --map-target primary")?;
            let profile = map_profile.ok_or("context map requires --map-profile-id")?;
            Request::Map(
                ContextMapRequest::primary(
                    &project_store,
                    publication,
                    &profile,
                    root,
                    current_guard,
                    reference_guard,
                )
                .map_err(|_| "invalid context map request syntax")?,
            )
        }
        Command::Inspect => Request::Inspect(inspect_options.request(
            &project_store,
            publication,
            root,
            current_guard,
            reference_guard,
        )?),
    };
    Ok(Arguments {
        host_config,
        request,
    })
}

fn set<T>(slot: &mut Option<T>, value: T, command: Command) -> Result<(), &'static str> {
    if slot.is_some() {
        return Err(match command {
            Command::Map => "duplicate context map option",
            Command::Inspect => "duplicate context inspect option",
        });
    }
    *slot = Some(value);
    Ok(())
}

fn id_syntax(value: &str) -> Result<(), &'static str> {
    if value.is_empty()
        || value.len() > CONTEXT_MAP_ID_MAX_BYTES
        || value.trim() != value
        || value.chars().any(char::is_control)
    {
        return Err("invalid bounded context identifier");
    }
    Ok(())
}

fn parse_root(token: &str) -> Result<ContextMapRoot, &'static str> {
    let (kind, encoded) = token
        .split_once('@')
        .ok_or("invalid exact context root token")?;
    if !matches!(kind, "project" | "graph-entity") || encoded.contains('@') {
        return Err("unsupported kind or delimiter in exact context root");
    }
    let id = decode_root_id(encoded)?;
    match kind {
        "project" => ContextMapRoot::project(&id),
        "graph-entity" => ContextMapRoot::graph_entity(&id),
        _ => return Err("unsupported exact context root kind"),
    }
    .map_err(|_| "noncanonical exact context root identifier")
}

// Mechanical root transport codec only: no semantic parser, lookup or normalization.
// Reject nonzero unused bits so different spellings cannot decode to the same token.
fn decode_root_id(encoded: &str) -> Result<String, &'static str> {
    let remainder = match encoded.len() % 4 {
        0 => 0usize,
        2 => 1,
        3 => 2,
        _ => return Err("invalid unpadded context root encoding"),
    };
    let length = (encoded.len() / 4)
        .checked_mul(3)
        .and_then(|length| length.checked_add(remainder))
        .ok_or("context root size overflow")?;
    if length == 0 || length > CONTEXT_MAP_ID_MAX_BYTES {
        return Err("context root byte limit");
    }
    let mut decoded = Vec::with_capacity(length);
    let mut accumulator = 0u32;
    let mut bits = 0u32;
    for byte in encoded.bytes() {
        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'-' => 62,
            b'_' => 63,
            _ => return Err("invalid base64url context root alphabet"),
        };
        accumulator = (accumulator << 6) | u32::from(value);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            decoded.push(((accumulator >> bits) & 255) as u8);
            accumulator &= (1u32 << bits) - 1;
        }
    }
    if accumulator != 0 || decoded.len() != length {
        return Err("noncanonical base64url context root encoding");
    }
    let id = String::from_utf8(decoded).map_err(|_| "context root is not UTF-8")?;
    id_syntax(&id)?;
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROJECT_ROOT: &str = "project@Y29udGV4dC1jbGk";
    const GRAPH_ROOT: &str = "graph-entity@Z3JhcGgtbm9kZTpzaGEyNTY6YWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYQ";
    const GRAPH_ID: &str =
        "graph-node:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const CURRENT_RECORD: &str =
        "project-current:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const STORE_GENERATION: &str = "store-generation:project-store-generation:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const PROFILE: &str = "profile:fixture:context-cli";
    const REFERENCE_GENERATION: &str = "generation:reference:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    // Inert syntax fixtures only. No host file/store/reference/map is opened,
    // and syntactically valid IDs do not attest any owner publication.
    fn arguments() -> Vec<OsString> {
        [
            "context",
            "map",
            "--native-host-config",
            "private-native-host.json",
            "--project-store",
            "context-cli-store",
            "--publication",
            "current",
            "--map-target",
            "primary",
            "--map-profile-id",
            PRIMARY_NAVIGATION_MAP_PROFILE_ID,
        ]
        .into_iter()
        .map(OsString::from)
        .collect()
    }

    fn push(values: &mut Vec<OsString>, flag: &str, value: &str) {
        values.push(flag.into());
        values.push(value.into());
    }

    fn replace(values: &mut [OsString], flag: &str, value: &str) -> Result<(), &'static str> {
        let index = values
            .iter()
            .position(|argument| argument == flag)
            .ok_or("missing fixture option")?;
        let target = values.get_mut(index + 1).ok_or("missing fixture value")?;
        *target = value.into();
        Ok(())
    }

    fn request_json(values: Vec<OsString>) -> Result<serde_json::Value, &'static str> {
        let arguments = parse(values)?;
        match arguments.request {
            Request::Map(request) => serde_json::to_value(request),
            Request::Inspect(request) => serde_json::to_value(request),
        }
        .map_err(|_| "request serialization failed")
    }

    #[test]
    fn context_map_grammar_preserves_exact_selectors() -> Result<(), &'static str> {
        let no_root = request_json(arguments())?;
        assert_eq!(no_root["project_store_id"], "context-cli-store");
        assert_eq!(no_root["publication"]["kind"], "current");
        assert_eq!(no_root["map_profile_id"], PRIMARY_NAVIGATION_MAP_PROFILE_ID);
        assert_eq!(no_root["platform"], "omitted");
        assert!(no_root.get("root").is_none());
        assert!(no_root.get("project_store_root").is_none());
        assert!(no_root.get("native_host_config").is_none());

        let mut explicit = arguments();
        replace(
            &mut explicit,
            "--native-host-config",
            r"C:\private-transport\host.json",
        )?;
        push(&mut explicit, "--root", GRAPH_ROOT);
        push(&mut explicit, "--platform", "omitted");
        push(&mut explicit, "--output", "envelope-json");
        push(&mut explicit, "--expect-current-record", CURRENT_RECORD);
        push(&mut explicit, "--expect-current-digest", DIGEST);
        push(&mut explicit, "--expect-profile", PROFILE);
        push(
            &mut explicit,
            "--expect-reference-generation",
            REFERENCE_GENERATION,
        );
        push(&mut explicit, "--expect-reference-view", DIGEST);
        let exact = request_json(explicit)?;
        assert_eq!(exact["root"]["kind"], "graph_entity");
        assert_eq!(exact["root"]["id"], GRAPH_ID);
        assert_eq!(exact["current"]["record"], CURRENT_RECORD);
        assert_eq!(exact["current"]["digest"], DIGEST);
        assert_eq!(exact["reference"]["profile"], PROFILE);
        assert_eq!(exact["reference"]["generation"], REFERENCE_GENERATION);
        assert_eq!(exact["reference"]["view"], DIGEST);
        // Transport-path changes cannot change the serialized semantic request.
        let mut other_host = arguments();
        push(&mut other_host, "--root", GRAPH_ROOT);
        push(&mut other_host, "--platform", "omitted");
        push(&mut other_host, "--output", "envelope-json");
        push(&mut other_host, "--expect-current-record", CURRENT_RECORD);
        push(&mut other_host, "--expect-current-digest", DIGEST);
        push(&mut other_host, "--expect-profile", PROFILE);
        push(
            &mut other_host,
            "--expect-reference-generation",
            REFERENCE_GENERATION,
        );
        push(&mut other_host, "--expect-reference-view", DIGEST);
        assert_eq!(exact, request_json(other_host)?);

        let mut project = arguments();
        push(&mut project, "--root", PROJECT_ROOT);
        let project = request_json(project)?;
        assert_eq!(project["root"]["kind"], "project");
        assert_eq!(project["root"]["id"], "context-cli");
        // Project/None are admitted syntax; their current ComponentUnavailable
        // execution outcome belongs to service, not this grammar test.
        let mut generation = arguments();
        replace(&mut generation, "--publication", STORE_GENERATION)?;
        let generation = request_json(generation)?;
        assert_eq!(generation["publication"]["kind"], "store_generation");
        assert_eq!(
            generation["publication"]["id"],
            &STORE_GENERATION["store-generation:".len()..]
        );
        Ok(())
    }

    #[test]
    fn context_map_project_profile_preserves_exact_project_root() -> Result<(), &'static str> {
        assert_eq!(
            PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID,
            "wow-context/primary-project-navigation/1"
        );
        let mut values = arguments();
        replace(
            &mut values,
            "--map-profile-id",
            PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID,
        )?;
        push(&mut values, "--root", PROJECT_ROOT);
        let request = request_json(values)?;
        assert_eq!(
            request["map_profile_id"],
            PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID
        );
        assert_eq!(
            request["root"],
            serde_json::json!({"kind": "project", "id": "context-cli"})
        );
        assert_eq!(request["publication"]["kind"], "current");
        assert_eq!(request["platform"], "omitted");
        Ok(())
    }

    #[test]
    fn context_inspect_grammar_preserves_explicit_project_root() -> Result<(), &'static str> {
        let mut values = arguments();
        let command = values.get_mut(1).ok_or("missing fixture command")?;
        *command = "inspect".into();
        for flag in ["--map-target", "--map-profile-id"] {
            let index = values
                .iter()
                .position(|value| value == flag)
                .ok_or("missing fixture map option")?;
            values.drain(index..index + 2);
        }
        push(&mut values, "--root", PROJECT_ROOT);
        push(&mut values, "--level", "l0");
        push(
            &mut values,
            "--l0-profile-id",
            wow_service::context::PRIMARY_L0_PROFILE_ID,
        );
        push(&mut values, "--platform", "omitted");
        push(&mut values, "--output", "envelope-json");
        push(&mut values, "--expect-profile", PROFILE);
        push(
            &mut values,
            "--expect-reference-generation",
            REFERENCE_GENERATION,
        );
        push(&mut values, "--expect-reference-view", DIGEST);
        // A valid token/level pair reaches service even when that exact scope
        // cannot supply the selected level; semantic refusal keeps an envelope.
        let mut incompatible_scope = values.clone();
        replace(&mut incompatible_scope, "--level", "both")?;
        assert!(parse(incompatible_scope).is_ok());
        let arguments = parse(values)?;
        let Request::Inspect(request) = arguments.request else {
            return Err("inspect parser selected the wrong typed operation");
        };
        let request = serde_json::to_value(request).map_err(|_| "request serialization failed")?;
        assert_eq!(
            request["root"],
            serde_json::json!({"kind": "project", "id": "context-cli"})
        );
        assert_eq!(request["schema"], "wow-service/context-inspect-request/1");
        assert_eq!(request["project_store_id"], "context-cli-store");
        assert_eq!(request["publication"]["kind"], "current");
        assert_eq!(request["platform"], "omitted");
        assert_eq!(request["level"], "l0");
        assert_eq!(
            request["l0_profile_id"],
            wow_service::context::PRIMARY_L0_PROFILE_ID
        );
        assert!(request.get("l1_profile_id").is_none());
        assert_eq!(request["reference"]["profile"], PROFILE);
        assert_eq!(request["reference"]["generation"], REFERENCE_GENERATION);
        assert_eq!(request["reference"]["view"], DIGEST);
        assert!(request.get("project_store_root").is_none());
        assert!(request.get("native_host_config").is_none());
        Ok(())
    }

    #[test]
    fn context_map_grammar_refuses_unknown_duplicate_and_conflicting_options()
    -> Result<(), &'static str> {
        for (flag, value) in [
            ("--native-host-config", "other-native-host.json"),
            ("--project-store", "context-cli-store"),
            ("--publication", "current"),
            ("--map-target", "primary"),
            ("--map-profile-id", PRIMARY_NAVIGATION_MAP_PROFILE_ID),
            ("--platform", "omitted"),
            ("--output", "envelope-json"),
            ("--root", PROJECT_ROOT),
            ("--expect-current-record", CURRENT_RECORD),
            ("--expect-current-digest", DIGEST),
            ("--expect-profile", PROFILE),
            ("--expect-reference-generation", REFERENCE_GENERATION),
            ("--expect-reference-view", DIGEST),
        ] {
            let mut values = arguments();
            push(&mut values, flag, value);
            push(&mut values, flag, value);
            assert!(
                matches!(parse(values), Err("duplicate context map option")),
                "{flag}"
            );
        }
        for (flag, value) in [
            ("--store-root", r"C:\private-transport\store"),
            ("--config", "semantic-config.json"),
            ("--platform-store", "other-store"),
            ("--map-profile-alias", "primary"),
        ] {
            let mut values = arguments();
            push(&mut values, flag, value);
            assert!(
                matches!(parse(values), Err("unknown context map option")),
                "{flag}"
            );
        }
        for (flag, value) in [
            ("--publication", "publication-set:unimplemented"),
            ("--map-target", "platform"),
            ("--map-target", "combined"),
            ("--map-profile-id", "wow-context/unreviewed/1"),
            ("--project-store", r"C:\private-transport\store"),
            ("--project-store", "/private/store"),
        ] {
            let mut values = arguments();
            replace(&mut values, flag, value)?;
            assert!(parse(values).is_err(), "{flag}");
        }
        for (flag, value) in [
            ("--platform", "current"),
            ("--output", "text"),
            ("--output", "artifact"),
        ] {
            let mut values = arguments();
            push(&mut values, flag, value);
            assert!(parse(values).is_err(), "{flag}");
        }

        let mut partial_reference = arguments();
        push(&mut partial_reference, "--expect-profile", PROFILE);
        assert!(parse(partial_reference.clone()).is_err());
        push(
            &mut partial_reference,
            "--expect-reference-generation",
            REFERENCE_GENERATION,
        );
        assert!(matches!(
            parse(partial_reference),
            Err("context reference guards require the complete three-option group")
        ));
        for flag in ["--expect-current-record", "--expect-current-digest"] {
            let mut values = arguments();
            replace(&mut values, "--publication", STORE_GENERATION)?;
            push(
                &mut values,
                flag,
                if flag == "--expect-current-record" {
                    CURRENT_RECORD
                } else {
                    DIGEST
                },
            );
            assert!(matches!(
                parse(values),
                Err("current guards require --publication current")
            ));
        }
        let mut missing_value = arguments();
        missing_value.push("--root".into());
        assert!(matches!(
            parse(missing_value),
            Err("missing context option value")
        ));
        for flag in [
            "--native-host-config",
            "--project-store",
            "--publication",
            "--map-target",
            "--map-profile-id",
        ] {
            let mut values = arguments();
            let index = values
                .iter()
                .position(|value| value == flag)
                .ok_or("missing fixture required option")?;
            values.drain(index..index + 2);
            assert!(parse(values).is_err(), "{flag}");
        }
        Ok(())
    }

    #[test]
    fn context_map_root_token_bytes_are_canonical_and_bounded() -> Result<(), &'static str> {
        for (token, expected) in [
            ("Zg", "f"),
            ("Zm8", "fo"),
            ("Zm9v", "foo"),
            ("w6k", "é"),
            ("5L2g", "你"),
        ] {
            assert_eq!(decode_root_id(token)?, expected);
        }
        for token in [
            "", "A", "Zg=", "Zg==", "Zg+", "Zg/", "Zh", "Zm9", "_w", "AA", "Cg", "IA", "IGE", "YSA",
        ] {
            assert!(decode_root_id(token).is_err(), "{token}");
        }
        assert!(parse_root(PROJECT_ROOT).is_ok());
        assert!(parse_root(GRAPH_ROOT).is_ok());
        for token in [
            "project",
            "project@",
            "project@@Y29udGV4dC1jbGk",
            "Project@Y29udGV4dC1jbGk",
            "file@Y29udGV4dC1jbGk",
            "project@Q09OVEVYVC1DTEk",
            "graph-entity@Y29udGV4dC1jbGk",
        ] {
            assert!(parse_root(token).is_err(), "{token}");
        }
        fn repeated_a(bytes: usize) -> String {
            let mut encoded = "YWFh".repeat(bytes / 3);
            encoded.push_str(match bytes % 3 {
                1 => "YQ",
                2 => "YWE",
                _ => "",
            });
            encoded
        }
        assert_eq!(
            decode_root_id(&repeated_a(CONTEXT_MAP_ID_MAX_BYTES))?,
            "a".repeat(CONTEXT_MAP_ID_MAX_BYTES)
        );
        assert!(decode_root_id(&repeated_a(CONTEXT_MAP_ID_MAX_BYTES + 1)).is_err());
        Ok(())
    }
}
