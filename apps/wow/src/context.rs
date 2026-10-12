//! Closed E3 context-map, inspect and build projections. Semantics are service-owned.
//! The native host document is a separate transport input, never an E3 semantic selector.
mod build;
mod combined;
mod inspect;
mod platform;

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
        ContextBuildRequest, ContextInspectRequest, ContextMapCurrentGuard, ContextMapHostInput,
        ContextMapPublication, ContextMapReferenceGuard, ContextMapRequest, ContextMapResult,
        ContextMapRoot, ContextMapStatus, PRIMARY_NAVIGATION_MAP_PROFILE_ID,
        PRIMARY_PROJECT_NAVIGATION_MAP_PROFILE_ID, execute_context_build, execute_context_inspect,
        execute_context_map,
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
    "and project-navigation map profiles use omitted platform and canonical envelope JSON.\n",
    "Use an explicit graph-entity@ root with wow-context/primary-direct-navigation/1, ",
    "or an explicit project@ root with wow-context/primary-project-navigation/1.\n",
    "Service validates the selected profile/root pair; the CLI never infers a root.\n",
    "wow context inspect --native-host-config <host.json> --project-store <registration-key> ",
    "--publication current|store-generation:<ProjectStoreGenerationId> ",
    "--root <exact-root-token> --level l0|l1|both\n",
    "  [--l0-profile-id wow-context/primary-l0/1|wow-context/primary-state-root-l0/1] ",
    "[--l1-profile-id wow-context/primary-l1/1|wow-context/primary-state-detail-l1/1]\n",
    "  State-detail L1 requires l1 plus an exact graph-entity@ root, ",
    "and cannot be combined with frontier expansion.\n",
    "  [--expansion-profile-id wow-context/primary-native-frontier/1 ",
    "--frontier graph-entity@<base64url-id> ...]\n",
    "  Expansion requires l1, an exact graph-entity@ root and 1..32 distinct frontiers; ",
    "service validates initial-map membership.\n",
    "  Progressive inspect/build: repeat --frontier-round ",
    "'graph-entity@<base64url-id>,<...>' for 1..4 ordered batches, ",
    "1..32 globally distinct exact graph IDs per batch, excluding the initial root.\n",
    "  Select the exact progressive expansion profile listed below; requires l1 and ",
    "default/explicit entity L1. Legacy --frontier and progressive rounds cannot be mixed.\n",
    "  Same omitted-platform, complete reference/current guards and envelope-json transport. ",
    "Default L0 accepts project@ or an exact container graph-entity@. Select ",
    "wow-context/primary-state-root-l0/1 for an exact SavedVariables state-root graph-entity@; ",
    "this profile does not attest runtime values or exhaustive fields. L1 accepts graph-entity@. ",
    "Service validates the selected profile/root for l0 or both and keeps artifacts distinct.\n",
    "wow context build --native-host-config <host.json> --project-store <registration-key> ",
    "--publication current|store-generation:<ProjectStoreGenerationId> ",
    "--root <exact-root-token> --level l0|l1|both\n",
    "  --context-profile-id wow-context/primary-structural-pack/1|wow-context/primary-frontier-pack/1\n",
    "  [--l0-profile-id wow-context/primary-l0/1|wow-context/primary-state-root-l0/1] ",
    "[--l1-profile-id wow-context/primary-l1/1|wow-context/primary-state-detail-l1/1]\n",
    "  [--renderer-profile-id wow-context/primary-pack-json/1|wow-context/primary-frontier-pack-json/1]\n",
    "  [--renderer-profile-id wow-context/primary-pack-markdown/1|wow-context/primary-frontier-pack-markdown/1]\n",
    "  [--expansion-profile-id wow-context/primary-native-frontier/1 ",
    "--frontier graph-entity@<base64url-id> ...]\n",
    "  Frontier pack requires expansion, l1 and an exact graph-entity@ root. ",
    "Renderer IDs must match the selected pack family; frontiers follow the inspect bounds.\n",
    "  State-detail L1 requires l1, an exact graph-entity@ root and the structural pack; ",
    "use its existing renderers without frontier expansion.\n",
    "  Select zero or one of each renderer; a duplicate renderer refuses syntax. ",
    "Both renderers are ordered JSON then Markdown independent of argv order.\n",
    "  Same explicit host/root, omitted platform, optional current guards, complete reference ",
    "guard group and envelope-json output. Build supports the same explicit state-root L0 profile; ",
    "service owns profile/root/level compatibility.\n",
    "wow context map|inspect --platform native --native-host-config <host.json> ",
    "--platform-request-input <request.json> [--output envelope-json|text]\n",
    "  Native map/inspect use a bounded explicit service host and request document. ",
    "The request operation must match the command. Primary options cannot be mixed; ",
    "native build and artifact output are unavailable.\n",
    "wow context map --platform combined --native-host-config <combined-host.json> ",
    "--combined-request-input <request.json> [--output envelope-json]\n",
    "  Combined uses explicit bounded service host/request documents. Native platform ",
    "request flags and Primary selectors cannot be mixed. Inspect, build, text and artifact ",
    "are unavailable for this route.\n",
);

fn progressive_help() -> String {
    use wow_service::context::{
        PRIMARY_PROGRESSIVE_CONTEXT_JSON_RENDERER_PROFILE_ID,
        PRIMARY_PROGRESSIVE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
        PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE_ID, PRIMARY_PROGRESSIVE_EXPANSION_PROFILE_ID,
    };

    format!(
        "Progressive inspect/build: --expansion-profile-id {PRIMARY_PROGRESSIVE_EXPANSION_PROFILE_ID}\n\
         Progressive build: --context-profile-id {PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE_ID}\n\
           [--renderer-profile-id {PRIMARY_PROGRESSIVE_CONTEXT_JSON_RENDERER_PROFILE_ID}]\n\
           [--renderer-profile-id {PRIMARY_PROGRESSIVE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID}]\n\
         Progressive pack/renderers require the matching progressive rounds family.\n"
    )
}

// Preserve the existing app argv/individual-path transport maxima.
const MAX_ARGUMENTS: usize = 4096;
const MAX_ARGUMENT_BYTES: usize = 1024 * 1024;
const MAX_VALUE_BYTES: usize = 32768;

#[derive(Clone, Copy)]
enum Command {
    Map,
    Inspect,
    Build,
}

enum Request {
    Map(ContextMapRequest),
    Inspect(ContextInspectRequest),
    Build(ContextBuildRequest),
    Platform(platform::Request),
    Combined(combined::Request),
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
                    .is_some_and(|value| value == "map" || value == "inspect" || value == "build")))
        && values
            .last()
            .is_some_and(|value| value == "--help" || value == "-h")
    {
        let mut output = std::io::stdout().lock();
        return if output
            .write_all(HELP.as_bytes())
            .and_then(|()| output.write_all(progressive_help().as_bytes()))
            .is_ok()
        {
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
    if let Request::Platform(request) = &arguments.request {
        return platform::run(&arguments.host_config, request, &stop);
    }
    if let Request::Combined(request) = &arguments.request {
        return combined::run(&arguments.host_config, request, &stop);
    }
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
        Request::Build(request) => execute_context_build(&host, request, &stop),
        Request::Platform(_) => {
            crate::diagnostic("native context dispatch invariant failed");
            return 4;
        }
        Request::Combined(_) => {
            crate::diagnostic("combined context dispatch invariant failed");
            return 4;
        }
    };
    let result = match operation {
        Ok(result) => result,
        Err(error) if error.code() == ServiceErrorCode::Cancelled => return 130,
        Err(_) => {
            crate::diagnostic("context operation could not finalize a result");
            return 4;
        }
    };
    emit_json_result(&result, &stop)
}

fn emit_json_result(result: &ContextMapResult, stop: &AtomicBool) -> u8 {
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
    if let Some(arguments) = combined::parse(&values)? {
        return Ok(arguments);
    }
    if let Some(arguments) = platform::parse(&values)? {
        return Ok(arguments);
    }
    let mut values = values.into_iter();
    if values.next().as_deref() != Some(OsStr::new("context")) {
        return Err("expected context map, inspect or build");
    }
    let command = match values.next().as_deref().and_then(OsStr::to_str) {
        Some("map") => Command::Map,
        Some("inspect") => Command::Inspect,
        Some("build") => Command::Build,
        _ => return Err("expected context map, inspect or build"),
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
    let mut build_options = build::Options::default();

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
            Command::Inspect => {
                inspect::Options::accepts(option) || inspect::Options::accepts_expansion(option)
            }
            Command::Build => {
                inspect::Options::accepts(option)
                    || inspect::Options::accepts_expansion(option)
                    || build::Options::accepts(option)
            }
        };
        if !common && !selected {
            return Err(match command {
                Command::Map => "unknown context map option",
                Command::Inspect => "unknown context inspect option",
                Command::Build => "unknown context build option",
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
            "--level"
            | "--l0-profile-id"
            | "--l1-profile-id"
            | "--expansion-profile-id"
            | "--frontier"
            | "--frontier-round" => {
                inspect_options.option(option, &value, command)?;
            }
            "--context-profile-id" | "--renderer-profile-id" => {
                build_options.option(option, &value)?;
            }
            _ => {
                return Err(match command {
                    Command::Map => "unknown context map option",
                    Command::Inspect => "unknown context inspect option",
                    Command::Build => "unknown context build option",
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
        Command::Build => Request::Build(build_options.request(
            &project_store,
            publication,
            inspect_options.selection(root, Command::Build)?,
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
            Command::Build => "duplicate context build option",
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
            Request::Build(request) => serde_json::to_value(request),
            Request::Platform(_) | Request::Combined(_) => {
                return Err("native request is explicit file transport");
            }
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
        use wow_service::context::{
            ContextL1Profile, PRIMARY_FRONTIER_EXPANSION_PROFILE_ID, PRIMARY_L0_PROFILE_ID,
            PRIMARY_L1_PROFILE_ID, PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
            PRIMARY_STATE_ROOT_L0_PROFILE_ID,
        };

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

        let mut omitted_profile = values.clone();
        let index = omitted_profile
            .iter()
            .position(|value| value == "--l0-profile-id")
            .ok_or("missing fixture L0 profile option")?;
        omitted_profile.drain(index..index + 2);
        let Request::Inspect(omitted) = parse(omitted_profile)?.request else {
            return Err("inspect parser selected the wrong typed operation");
        };
        let Request::Inspect(explicit) = parse(values.clone())?.request else {
            return Err("inspect parser selected the wrong typed operation");
        };
        assert_eq!(
            serde_json::to_vec(&omitted).map_err(|_| "request serialization failed")?,
            serde_json::to_vec(&explicit).map_err(|_| "request serialization failed")?
        );

        let mut state_root = values.clone();
        replace(&mut state_root, "--root", GRAPH_ROOT)?;
        replace(
            &mut state_root,
            "--l0-profile-id",
            PRIMARY_STATE_ROOT_L0_PROFILE_ID,
        )?;
        let selected = request_json(state_root.clone())?;
        assert_eq!(selected["l0_profile_id"], PRIMARY_STATE_ROOT_L0_PROFILE_ID);
        assert_eq!(
            selected["root"],
            serde_json::json!({"kind": "graph_entity", "id": GRAPH_ID})
        );
        replace(&mut state_root, "--level", "both")?;
        assert_eq!(
            request_json(state_root.clone())?["l0_profile_id"],
            PRIMARY_STATE_ROOT_L0_PROFILE_ID
        );
        let mut duplicate = state_root.clone();
        push(&mut duplicate, "--l0-profile-id", PRIMARY_L0_PROFILE_ID);
        assert!(matches!(
            parse(duplicate),
            Err("duplicate context inspect option")
        ));
        let mut conflict = state_root.clone();
        replace(&mut conflict, "--level", "l1")?;
        assert!(matches!(
            parse(conflict),
            Err("L0 profile conflicts with context inspect level l1")
        ));
        replace(
            &mut state_root,
            "--l0-profile-id",
            "wow-context/unreviewed-l0/1",
        )?;
        assert!(matches!(
            parse(state_root),
            Err("unsupported exact context L0 profile")
        ));

        let mut expansion_base = values.clone();
        let index = expansion_base
            .iter()
            .position(|value| value == "--l0-profile-id")
            .ok_or("missing fixture L0 profile option")?;
        expansion_base.drain(index..index + 2);
        replace(&mut expansion_base, "--root", GRAPH_ROOT)?;
        replace(&mut expansion_base, "--level", "l1")?;
        let Request::Inspect(unexpanded) = parse(expansion_base.clone())?.request else {
            return Err("inspect parser selected the wrong typed operation");
        };
        // Inert exact tokens only; native initial-map membership is not asserted here.
        let frontier_b = format!(
            "graph-entity@Z3JhcGgtbm9kZTpzaGEyNTY6{}Yg",
            "YmJi".repeat(21)
        );
        let frontier_c = format!(
            "graph-entity@Z3JhcGgtbm9kZTpzaGEyNTY6{}Yw",
            "Y2Nj".repeat(21)
        );
        let expected = unexpanded
            .clone()
            .with_frontier_expansion(vec![parse_root(&frontier_b)?, parse_root(&frontier_c)?])
            .map_err(|_| "expected inspect expansion request refused")?;
        let mut expansion = expansion_base.clone();
        push(
            &mut expansion,
            "--expansion-profile-id",
            PRIMARY_FRONTIER_EXPANSION_PROFILE_ID,
        );
        push(&mut expansion, "--frontier", &frontier_c);
        push(&mut expansion, "--frontier", &frontier_b);
        let Request::Inspect(expanded) = parse(expansion.clone())?.request else {
            return Err("inspect parser selected the wrong typed operation");
        };
        // Reverse argv order yields the same request as sorted exact frontier IDs.
        assert_eq!(
            serde_json::to_vec(&expanded).map_err(|_| "request serialization failed")?,
            serde_json::to_vec(&expected).map_err(|_| "request serialization failed")?
        );
        let expected_unexpanded = ContextInspectRequest::primary(
            "context-cli-store",
            ContextMapPublication::Current,
            wow_service::context::ContextInspectLevel::L1,
            parse_root(GRAPH_ROOT)?,
            None,
            Some(
                ContextMapReferenceGuard::new(PROFILE, REFERENCE_GENERATION, DIGEST)
                    .map_err(|_| "expected inspect reference guard refused")?,
            ),
        )
        .map_err(|_| "expected unexpanded inspect request refused")?;
        assert_eq!(
            serde_json::to_vec(&unexpanded).map_err(|_| "request serialization failed")?,
            serde_json::to_vec(&expected_unexpanded).map_err(|_| "request serialization failed")?
        );
        let mut duplicate_frontier = expansion.clone();
        push(&mut duplicate_frontier, "--frontier", &frontier_b);
        assert!(matches!(
            parse(duplicate_frontier),
            Err("duplicate context inspect frontier")
        ));
        let mut duplicate_profile = expansion.clone();
        push(
            &mut duplicate_profile,
            "--expansion-profile-id",
            PRIMARY_FRONTIER_EXPANSION_PROFILE_ID,
        );
        assert!(matches!(
            parse(duplicate_profile),
            Err("duplicate context inspect option")
        ));
        let mut unknown_profile = expansion.clone();
        replace(
            &mut unknown_profile,
            "--expansion-profile-id",
            "wow-context/unreviewed-expansion/1",
        )?;
        assert!(matches!(
            parse(unknown_profile),
            Err("unsupported exact context expansion profile")
        ));
        let mut profile_only = expansion_base.clone();
        push(
            &mut profile_only,
            "--expansion-profile-id",
            PRIMARY_FRONTIER_EXPANSION_PROFILE_ID,
        );
        assert!(matches!(
            parse(profile_only),
            Err("context inspect expansion requires profile and frontier together")
        ));
        let mut frontier_only = expansion_base.clone();
        push(&mut frontier_only, "--frontier", &frontier_b);
        assert!(matches!(
            parse(frontier_only),
            Err("context inspect expansion requires profile and frontier together")
        ));
        let mut project_frontier = expansion.clone();
        replace(&mut project_frontier, "--frontier", PROJECT_ROOT)?;
        assert!(matches!(
            parse(project_frontier),
            Err("context inspect frontier requires graph-entity token")
        ));
        let mut own_root = expansion.clone();
        replace(&mut own_root, "--frontier", GRAPH_ROOT)?;
        assert!(matches!(
            parse(own_root),
            Err("context inspect frontier must exclude its initial root")
        ));
        let mut project_scope = expansion.clone();
        replace(&mut project_scope, "--root", PROJECT_ROOT)?;
        assert!(matches!(
            parse(project_scope),
            Err("context inspect expansion requires l1 and graph-entity root")
        ));
        for level in ["l0", "both"] {
            let mut conflict = expansion.clone();
            replace(&mut conflict, "--level", level)?;
            assert!(matches!(
                parse(conflict),
                Err("context inspect expansion requires l1 and graph-entity root")
            ));
        }
        for (flag, value) in [
            (
                "--expansion-profile-id",
                PRIMARY_FRONTIER_EXPANSION_PROFILE_ID,
            ),
            ("--frontier", frontier_b.as_str()),
        ] {
            let mut build = expansion_base.clone();
            *build.get_mut(1).ok_or("missing fixture command")? = "build".into();
            push(
                &mut build,
                "--context-profile-id",
                wow_service::context::PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE_ID,
            );
            push(&mut build, flag, value);
            assert!(matches!(
                parse(build),
                Err("context build expansion requires profile and frontier together")
            ));
            let mut map = arguments();
            push(&mut map, flag, value);
            assert!(matches!(parse(map), Err("unknown context map option")));
        }

        let mut explicit_entity = expansion_base.clone();
        push(
            &mut explicit_entity,
            "--l1-profile-id",
            PRIMARY_L1_PROFILE_ID,
        );
        let Request::Inspect(entity) = parse(explicit_entity)?.request else {
            return Err("inspect parser selected the wrong typed operation");
        };
        assert_eq!(
            serde_json::to_vec(&entity).map_err(|_| "request serialization failed")?,
            serde_json::to_vec(&unexpanded).map_err(|_| "request serialization failed")?
        );

        // Exact profile syntax does not assert this inert graph ID has state custody.
        let mut state_detail = expansion_base.clone();
        push(
            &mut state_detail,
            "--l1-profile-id",
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
        );
        let expected_detail = unexpanded
            .clone()
            .with_l1_profile(ContextL1Profile::StateDetailV1)
            .map_err(|_| "expected state-detail inspect request refused")?;
        let selected_detail = request_json(state_detail.clone())?;
        assert_eq!(
            selected_detail,
            serde_json::to_value(expected_detail).map_err(|_| "request serialization failed")?
        );
        assert_eq!(
            selected_detail["l1_profile_id"],
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID
        );
        assert_eq!(selected_detail["level"], "l1");
        assert_eq!(
            selected_detail["root"],
            serde_json::json!({"kind": "graph_entity", "id": GRAPH_ID})
        );
        assert!(selected_detail.get("expansion").is_none());
        let mut duplicate_l1 = state_detail.clone();
        push(&mut duplicate_l1, "--l1-profile-id", PRIMARY_L1_PROFILE_ID);
        assert!(matches!(
            parse(duplicate_l1),
            Err("duplicate context inspect option")
        ));
        for (level, message) in [
            ("l0", "L1 profile conflicts with context inspect level l0"),
            (
                "both",
                "state-detail L1 requires context inspect l1 and graph-entity root",
            ),
        ] {
            let mut wrong_level = state_detail.clone();
            replace(&mut wrong_level, "--level", level)?;
            assert!(matches!(parse(wrong_level), Err(actual) if actual == message));
        }
        let mut project_detail = state_detail.clone();
        replace(&mut project_detail, "--root", PROJECT_ROOT)?;
        assert!(matches!(
            parse(project_detail),
            Err("state-detail L1 requires context inspect l1 and graph-entity root")
        ));
        let mut detail_first = state_detail.clone();
        push(
            &mut detail_first,
            "--expansion-profile-id",
            PRIMARY_FRONTIER_EXPANSION_PROFILE_ID,
        );
        push(&mut detail_first, "--frontier", &frontier_b);
        assert!(matches!(
            parse(detail_first),
            Err("state-detail L1 conflicts with context frontier expansion")
        ));
        let mut expansion_first = expansion.clone();
        push(
            &mut expansion_first,
            "--l1-profile-id",
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
        );
        assert!(matches!(
            parse(expansion_first),
            Err("state-detail L1 conflicts with context frontier expansion")
        ));
        let mut build_detail = state_detail;
        *build_detail.get_mut(1).ok_or("missing fixture command")? = "build".into();
        push(
            &mut build_detail,
            "--context-profile-id",
            wow_service::context::PRIMARY_CONTEXT_PACK_PROFILE_ID,
        );
        assert_eq!(
            request_json(build_detail)?["l1_profile_id"],
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID
        );
        let mut default_build = expansion_base;
        *default_build.get_mut(1).ok_or("missing fixture command")? = "build".into();
        push(
            &mut default_build,
            "--context-profile-id",
            wow_service::context::PRIMARY_CONTEXT_PACK_PROFILE_ID,
        );
        let default_build_request = request_json(default_build.clone())?;
        push(&mut default_build, "--l1-profile-id", PRIMARY_L1_PROFILE_ID);
        assert_eq!(default_build_request, request_json(default_build)?);
        let mut map_detail = arguments();
        push(
            &mut map_detail,
            "--l1-profile-id",
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
        );
        assert!(matches!(
            parse(map_detail),
            Err("unknown context map option")
        ));

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

        // Combined retains explicit file transport; inspection level stays in its request.
        let combined_values = |command: &str| {
            [
                "context",
                command,
                "--platform",
                "combined",
                "--native-host-config",
                "combined-host.json",
                "--combined-request-input",
                "combined-request.json",
            ]
            .into_iter()
            .map(OsString::from)
            .collect::<Vec<_>>()
        };
        for (command, operation, other, duplicate) in [
            (
                "map",
                wow_service::context::ContextPlatformOperation::Map,
                wow_service::context::ContextPlatformOperation::Inspect,
                "duplicate context map option",
            ),
            (
                "inspect",
                wow_service::context::ContextPlatformOperation::Inspect,
                wow_service::context::ContextPlatformOperation::Map,
                "duplicate context inspect option",
            ),
        ] {
            let values = combined_values(command);
            let parsed = parse(values.clone())?;
            assert_eq!(parsed.host_config, PathBuf::from("combined-host.json"));
            let Request::Combined(request) = parsed.request else {
                return Err("expected combined explicit file transport");
            };
            assert!(request.operation_matches(operation));
            assert!(!request.operation_matches(other));
            let mut explicit_output = values.clone();
            push(&mut explicit_output, "--output", "envelope-json");
            assert!(parse(explicit_output).is_ok());
            for (flag, value) in [
                ("--native-host-config", "other-host.json"),
                ("--combined-request-input", "other-request.json"),
                ("--platform", "combined"),
                ("--output", "envelope-json"),
            ] {
                let mut repeated = values.clone();
                if flag == "--output" {
                    push(&mut repeated, flag, value);
                }
                push(&mut repeated, flag, value);
                assert!(matches!(parse(repeated), Err(error) if error == duplicate));
            }
            for (flag, value) in [
                ("--level", "l1"),
                ("--root", GRAPH_ROOT),
                ("--platform-request-input", "native-request.json"),
                ("--unknown", "value"),
                ("--output", "text"),
            ] {
                let mut rejected = values.clone();
                push(&mut rejected, flag, value);
                assert!(parse(rejected).is_err(), "{command} {flag}");
            }
            let mut mixed = values.clone();
            replace(&mut mixed, "--platform", "native")?;
            assert!(parse(mixed).is_err());
            for flag in [
                "--native-host-config",
                "--combined-request-input",
                "--platform",
            ] {
                let mut missing = values.clone();
                let index = missing
                    .iter()
                    .position(|value| value == flag)
                    .ok_or("missing combined fixture option")?;
                missing.drain(index..index + 2);
                assert!(parse(missing).is_err(), "{command} {flag}");
            }
            let mut missing_value = values;
            missing_value.push("--output".into());
            assert!(matches!(
                parse(missing_value),
                Err("missing combined context option value")
            ));
        }
        assert!(matches!(
            parse(combined_values("build")),
            Err("combined platform supports context map or inspect only")
        ));

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

    #[test]
    fn context_progressive_grammar_preserves_rounds_and_refuses_mixed_families()
    -> Result<(), &'static str> {
        use wow_service::context::{
            ContextInspectLevel, ContextRenderSelection, PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
            PRIMARY_CONTEXT_PACK_PROFILE_ID, PRIMARY_FRONTIER_EXPANSION_PROFILE_ID,
            PRIMARY_L1_PROFILE_ID, PRIMARY_PROGRESSIVE_CONTEXT_JSON_RENDERER_PROFILE_ID,
            PRIMARY_PROGRESSIVE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
            PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE_ID, PRIMARY_PROGRESSIVE_EXPANSION_PROFILE_ID,
            PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID, PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
        };

        fn base(command: &str) -> Result<Vec<OsString>, &'static str> {
            let mut values = arguments();
            *values.get_mut(1).ok_or("missing fixture command")? = command.into();
            for flag in ["--map-target", "--map-profile-id"] {
                let index = values
                    .iter()
                    .position(|value| value == flag)
                    .ok_or("missing fixture map option")?;
                values.drain(index..index + 2);
            }
            push(&mut values, "--root", GRAPH_ROOT);
            push(&mut values, "--level", "l1");
            Ok(values)
        }

        fn token(index: usize) -> String {
            let mut token = String::from("graph-entity@Z3JhcGgtbm9kZTpzaGEyNTY6");
            for bit in 0..8 {
                token.push_str(if index & (1 << bit) == 0 {
                    "YmJi"
                } else {
                    "Y2Nj"
                });
            }
            token.push_str(&"YmJi".repeat(13));
            token.push_str("Yg");
            token
        }

        let first = format!("{},{}", token(1), token(0));
        let second = token(2);
        let mut inspect = base("inspect")?;
        // Profile follows the rounds; root/profile order cannot weaken validation.
        push(&mut inspect, "--frontier-round", &first);
        push(&mut inspect, "--frontier-round", &second);
        push(
            &mut inspect,
            "--expansion-profile-id",
            PRIMARY_PROGRESSIVE_EXPANSION_PROFILE_ID,
        );
        let rounds = vec![
            vec![parse_root(&token(1))?, parse_root(&token(0))?],
            vec![parse_root(&second)?],
        ];
        let expected = ContextInspectRequest::primary(
            "context-cli-store",
            ContextMapPublication::Current,
            ContextInspectLevel::L1,
            parse_root(GRAPH_ROOT)?,
            None,
            None,
        )
        .map_err(|_| "inspect constructor failed")?
        .with_progressive_expansion(rounds.clone())
        .map_err(|_| "progressive inspect constructor failed")?;
        let exact = request_json(inspect.clone())?;
        assert_eq!(
            exact,
            serde_json::to_value(expected).map_err(|_| "serialization failed")?
        );
        assert!(exact.get("expansion").is_none());
        let mut asserted = inspect.clone();
        push(&mut asserted, "--l1-profile-id", PRIMARY_L1_PROFILE_ID);
        assert_eq!(request_json(asserted)?, exact);
        let mut reversed = base("inspect")?;
        push(
            &mut reversed,
            "--expansion-profile-id",
            PRIMARY_PROGRESSIVE_EXPANSION_PROFILE_ID,
        );
        push(&mut reversed, "--frontier-round", &second);
        push(&mut reversed, "--frontier-round", &first);
        assert_ne!(request_json(reversed)?, exact);

        let mut build = inspect.clone();
        *build.get_mut(1).ok_or("missing fixture command")? = "build".into();
        push(
            &mut build,
            "--context-profile-id",
            PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE_ID,
        );
        assert!(parse(build.clone()).is_ok());
        let unrendered_build = build.clone();
        push(
            &mut build,
            "--renderer-profile-id",
            PRIMARY_PROGRESSIVE_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
        );
        push(
            &mut build,
            "--renderer-profile-id",
            PRIMARY_PROGRESSIVE_CONTEXT_JSON_RENDERER_PROFILE_ID,
        );
        let expected = ContextBuildRequest::primary(
            "context-cli-store",
            ContextMapPublication::Current,
            ContextInspectLevel::L1,
            parse_root(GRAPH_ROOT)?,
            ContextRenderSelection::Both,
            None,
            None,
        )
        .map_err(|_| "build constructor failed")?
        .with_progressive_expansion(rounds)
        .map_err(|_| "progressive build constructor failed")?;
        assert_eq!(
            request_json(build.clone())?,
            serde_json::to_value(expected).map_err(|_| "serialization failed")?
        );

        for command in ["inspect", "build"] {
            let mut selected = base(command)?;
            if command == "build" {
                push(
                    &mut selected,
                    "--context-profile-id",
                    PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE_ID,
                );
            }
            push(
                &mut selected,
                "--expansion-profile-id",
                PRIMARY_PROGRESSIVE_EXPANSION_PROFILE_ID,
            );
            assert!(parse(selected.clone()).is_err()); // no rounds
            for invalid in [
                "".to_owned(),
                ",".to_owned(),
                format!("{second},"),
                format!(",{second}"),
                format!("{second},{second}"),
                GRAPH_ROOT.to_owned(),
                PROJECT_ROOT.to_owned(),
                format!("{second}="),
                format!("{second}, {second}"),
            ] {
                let mut values = selected.clone();
                push(&mut values, "--frontier-round", &invalid);
                assert!(parse(values).is_err());
            }
            push(&mut selected, "--frontier-round", &second);
            assert!(parse(selected.clone()).is_ok());
            let mut across = selected.clone();
            push(&mut across, "--frontier-round", &second);
            assert!(parse(across).is_err());
            for level in ["l0", "both"] {
                let mut values = selected.clone();
                replace(&mut values, "--level", level)?;
                assert!(parse(values).is_err());
            }
            for profile in [
                PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
                PRIMARY_REFERENCE_FACTS_L1_PROFILE_ID,
            ] {
                let mut values = selected.clone();
                push(&mut values, "--l1-profile-id", profile);
                assert!(parse(values).is_err());
            }
            let mut late_root = selected.clone();
            let index = late_root
                .iter()
                .position(|value| value == "--root")
                .ok_or("missing fixture root")?;
            late_root.drain(index..index + 2);
            push(&mut late_root, "--root", &second);
            assert!(parse(late_root).is_err());
            let mut project = selected.clone();
            replace(&mut project, "--root", PROJECT_ROOT)?;
            assert!(parse(project).is_err());
            let mut legacy_profile = selected.clone();
            replace(
                &mut legacy_profile,
                "--expansion-profile-id",
                PRIMARY_FRONTIER_EXPANSION_PROFILE_ID,
            )?;
            assert!(parse(legacy_profile).is_err());
            let mut duplicate_profile = selected.clone();
            push(
                &mut duplicate_profile,
                "--expansion-profile-id",
                PRIMARY_PROGRESSIVE_EXPANSION_PROFILE_ID,
            );
            assert!(parse(duplicate_profile).is_err());
            let mut unknown_profile = selected.clone();
            replace(&mut unknown_profile, "--expansion-profile-id", "unknown")?;
            assert!(parse(unknown_profile).is_err());
            let mut unknown_flag = selected.clone();
            push(&mut unknown_flag, "--frontier-rounds", &second);
            assert!(parse(unknown_flag).is_err());
            selected.pop(); // remove first round and its flag
            selected.pop();
            for legacy_first in [false, true] {
                let mut values = selected.clone();
                let flags = if legacy_first {
                    ["--frontier", "--frontier-round"]
                } else {
                    ["--frontier-round", "--frontier"]
                };
                for flag in flags {
                    push(&mut values, flag, &second);
                }
                assert!(parse(values).is_err());
            }
            let mut no_profile = base(command)?;
            push(&mut no_profile, "--frontier-round", &second);
            assert!(parse(no_profile).is_err());
            let mut no_value = selected.clone();
            no_value.push("--frontier-round".into());
            assert!(parse(no_value).is_err());

            // Four ordered batches of 32 distinct exact IDs, then each finite overflow.
            let mut bounded = selected;
            for batch in 0..4 {
                let ids = (batch * 32..(batch + 1) * 32)
                    .map(token)
                    .collect::<Vec<_>>()
                    .join(",");
                push(&mut bounded, "--frontier-round", &ids);
            }
            assert!(parse(bounded.clone()).is_ok());
            let mut fifth = bounded.clone();
            push(&mut fifth, "--frontier-round", &token(128));
            assert!(parse(fifth).is_err());
            let too_many = (0..33).map(token).collect::<Vec<_>>().join(",");
            replace(&mut bounded, "--frontier-round", &too_many)?;
            assert!(parse(bounded).is_err());
        }
        for (flag, wrong) in [
            ("--context-profile-id", PRIMARY_CONTEXT_PACK_PROFILE_ID),
            (
                "--renderer-profile-id",
                PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
            ),
        ] {
            let mut values = unrendered_build.clone();
            if flag == "--renderer-profile-id" {
                push(&mut values, flag, wrong);
            } else {
                replace(&mut values, flag, wrong)?;
            }
            assert!(parse(values).is_err());
        }
        let mut duplicate_renderer = build;
        push(
            &mut duplicate_renderer,
            "--renderer-profile-id",
            PRIMARY_PROGRESSIVE_CONTEXT_JSON_RENDERER_PROFILE_ID,
        );
        assert!(parse(duplicate_renderer).is_err());
        let mut legacy = base("build")?;
        push(
            &mut legacy,
            "--context-profile-id",
            PRIMARY_PROGRESSIVE_CONTEXT_PACK_PROFILE_ID,
        );
        assert!(parse(legacy).is_err());
        let mut map = arguments();
        push(&mut map, "--frontier-round", &second);
        assert!(parse(map).is_err());
        Ok(())
    }

    #[test]
    fn context_build_grammar_preserves_profiles_render_order_and_service_fences()
    -> Result<(), &'static str> {
        use wow_service::context::{
            ContextBuildRequest, ContextInspectLevel, ContextL1Profile, ContextRenderSelection,
            PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID, PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
            PRIMARY_CONTEXT_PACK_PROFILE_ID, PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE_ID,
            PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
            PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE_ID, PRIMARY_FRONTIER_EXPANSION_PROFILE_ID,
            PRIMARY_L0_PROFILE_ID, PRIMARY_L1_PROFILE_ID, PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
            PRIMARY_STATE_ROOT_L0_PROFILE_ID,
        };

        let mut values = arguments();
        let command = values.get_mut(1).ok_or("missing fixture command")?;
        *command = "build".into();
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
            "--context-profile-id",
            PRIMARY_CONTEXT_PACK_PROFILE_ID,
        );

        let expected = |render| -> Result<serde_json::Value, &'static str> {
            let request = ContextBuildRequest::primary(
                "context-cli-store",
                ContextMapPublication::Current,
                ContextInspectLevel::L0,
                parse_root(PROJECT_ROOT)?,
                render,
                None,
                None,
            )
            .map_err(|_| "expected build fixture request refused")?;
            serde_json::to_value(request).map_err(|_| "request serialization failed")
        };
        // No renderer is a genuine typed None selection, not an app-selected default.
        let no_renderer = request_json(values.clone())?;
        assert_eq!(no_renderer, expected(ContextRenderSelection::None)?);
        assert_eq!(
            no_renderer["context_profile_id"],
            PRIMARY_CONTEXT_PACK_PROFILE_ID
        );

        let mut explicit_profile = values.clone();
        push(
            &mut explicit_profile,
            "--l0-profile-id",
            PRIMARY_L0_PROFILE_ID,
        );
        let Request::Build(omitted) = parse(values.clone())?.request else {
            return Err("build parser selected the wrong typed operation");
        };
        let Request::Build(explicit) = parse(explicit_profile)?.request else {
            return Err("build parser selected the wrong typed operation");
        };
        assert_eq!(
            serde_json::to_vec(&omitted).map_err(|_| "request serialization failed")?,
            serde_json::to_vec(&explicit).map_err(|_| "request serialization failed")?
        );

        let mut state_root = values.clone();
        replace(&mut state_root, "--root", GRAPH_ROOT)?;
        push(
            &mut state_root,
            "--l0-profile-id",
            PRIMARY_STATE_ROOT_L0_PROFILE_ID,
        );
        let selected = request_json(state_root.clone())?;
        assert_eq!(selected["l0_profile_id"], PRIMARY_STATE_ROOT_L0_PROFILE_ID);
        assert_eq!(selected["render"], "none");
        assert_eq!(
            selected["root"],
            serde_json::json!({"kind": "graph_entity", "id": GRAPH_ID})
        );
        replace(&mut state_root, "--level", "both")?;
        assert_eq!(
            request_json(state_root.clone())?["l0_profile_id"],
            PRIMARY_STATE_ROOT_L0_PROFILE_ID
        );
        let mut duplicate = state_root.clone();
        push(&mut duplicate, "--l0-profile-id", PRIMARY_L0_PROFILE_ID);
        assert!(matches!(
            parse(duplicate),
            Err("duplicate context build option")
        ));
        let mut conflict = state_root.clone();
        replace(&mut conflict, "--level", "l1")?;
        assert!(matches!(
            parse(conflict),
            Err("L0 profile conflicts with context build level l1")
        ));
        replace(
            &mut state_root,
            "--l0-profile-id",
            "wow-context/unreviewed-l0/1",
        )?;
        assert!(matches!(
            parse(state_root),
            Err("unsupported exact context L0 profile")
        ));

        let mut reverse = values.clone();
        push(
            &mut reverse,
            "--renderer-profile-id",
            PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
        );
        push(
            &mut reverse,
            "--renderer-profile-id",
            PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
        );
        let reverse = request_json(reverse)?;
        assert_eq!(reverse, expected(ContextRenderSelection::Both)?);
        assert_eq!(
            reverse["renderer_profile_ids"],
            serde_json::json!([
                PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
                PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
            ])
        );
        let mut forward = values.clone();
        push(
            &mut forward,
            "--renderer-profile-id",
            PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
        );
        push(
            &mut forward,
            "--renderer-profile-id",
            PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
        );
        assert_eq!(reverse, request_json(forward.clone())?);

        push(
            &mut forward,
            "--renderer-profile-id",
            PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
        );
        assert!(matches!(
            parse(forward),
            Err("duplicate context build option")
        ));
        let mut unknown_profile = values.clone();
        replace(
            &mut unknown_profile,
            "--context-profile-id",
            "wow-context/unreviewed-pack/1",
        )?;
        assert!(matches!(
            parse(unknown_profile),
            Err("unsupported exact context pack profile")
        ));
        let mut unknown_renderer = values.clone();
        push(
            &mut unknown_renderer,
            "--renderer-profile-id",
            "wow-context/unreviewed-renderer/1",
        );
        assert!(matches!(
            parse(unknown_renderer),
            Err("unsupported exact context renderer profile")
        ));
        let mut missing_profile = values.clone();
        let index = missing_profile
            .iter()
            .position(|value| value == "--context-profile-id")
            .ok_or("missing fixture context profile option")?;
        missing_profile.drain(index..index + 2);
        assert!(matches!(
            parse(missing_profile),
            Err("context build requires --context-profile-id")
        ));

        // Syntax data only; acquired-map membership remains service-owned.
        let frontier = format!(
            "graph-entity@Z3JhcGgtbm9kZTpzaGEyNTY6{}Yg",
            "YmJi".repeat(21)
        );
        // Exact profile transport only; the service checks native state custody.
        let mut detail_base = values.clone();
        replace(&mut detail_base, "--root", GRAPH_ROOT)?;
        replace(&mut detail_base, "--level", "l1")?;
        let Request::Build(default_l1) = parse(detail_base.clone())?.request else {
            return Err("build parser selected the wrong typed operation");
        };
        let mut explicit_entity = detail_base.clone();
        push(
            &mut explicit_entity,
            "--l1-profile-id",
            PRIMARY_L1_PROFILE_ID,
        );
        let Request::Build(entity_l1) = parse(explicit_entity)?.request else {
            return Err("build parser selected the wrong typed operation");
        };
        assert_eq!(
            serde_json::to_vec(&default_l1).map_err(|_| "request serialization failed")?,
            serde_json::to_vec(&entity_l1).map_err(|_| "request serialization failed")?
        );
        let expected_detail = |render| -> Result<serde_json::Value, &'static str> {
            let request = ContextBuildRequest::primary(
                "context-cli-store",
                ContextMapPublication::Current,
                ContextInspectLevel::L1,
                parse_root(GRAPH_ROOT)?,
                render,
                None,
                None,
            )
            .map_err(|_| "expected state-detail build request refused")?
            .with_l1_profile(ContextL1Profile::StateDetailV1)
            .map_err(|_| "expected state-detail build profile refused")?;
            serde_json::to_value(request).map_err(|_| "request serialization failed")
        };
        let mut state_detail = detail_base.clone();
        push(
            &mut state_detail,
            "--l1-profile-id",
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
        );
        let selected = request_json(state_detail.clone())?;
        assert_eq!(selected, expected_detail(ContextRenderSelection::None)?);
        assert_eq!(
            selected["l1_profile_id"],
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID
        );
        assert_eq!(
            selected["context_profile_id"],
            PRIMARY_CONTEXT_PACK_PROFILE_ID
        );
        assert_eq!(selected["renderer_profile_ids"], serde_json::json!([]));
        assert!(selected.get("expansion").is_none());
        let mut rendered_detail = state_detail.clone();
        push(
            &mut rendered_detail,
            "--renderer-profile-id",
            PRIMARY_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
        );
        push(
            &mut rendered_detail,
            "--renderer-profile-id",
            PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
        );
        assert_eq!(
            request_json(rendered_detail)?,
            expected_detail(ContextRenderSelection::Both)?
        );
        for (level, message) in [
            ("l0", "L1 profile conflicts with context build level l0"),
            (
                "both",
                "state-detail L1 requires context build l1 and graph-entity root",
            ),
        ] {
            let mut wrong_level = state_detail.clone();
            replace(&mut wrong_level, "--level", level)?;
            assert!(matches!(parse(wrong_level), Err(actual) if actual == message));
        }
        let mut project_detail = state_detail.clone();
        replace(&mut project_detail, "--root", PROJECT_ROOT)?;
        assert!(matches!(
            parse(project_detail),
            Err("state-detail L1 requires context build l1 and graph-entity root")
        ));
        let mut duplicate_detail = state_detail.clone();
        push(
            &mut duplicate_detail,
            "--l1-profile-id",
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
        );
        assert!(matches!(
            parse(duplicate_detail),
            Err("duplicate context build option")
        ));
        for profiles in [
            [PRIMARY_L1_PROFILE_ID, PRIMARY_STATE_DETAIL_L1_PROFILE_ID],
            [PRIMARY_STATE_DETAIL_L1_PROFILE_ID, PRIMARY_L1_PROFILE_ID],
        ] {
            let mut duplicate = detail_base.clone();
            for profile in profiles {
                push(&mut duplicate, "--l1-profile-id", profile);
            }
            assert!(matches!(
                parse(duplicate),
                Err("duplicate context build option")
            ));
        }
        let mut wrong_pack = state_detail.clone();
        replace(
            &mut wrong_pack,
            "--context-profile-id",
            PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE_ID,
        )?;
        assert!(matches!(
            parse(wrong_pack),
            Err("context build pack profile conflicts with expansion selection")
        ));
        let mut wrong_renderer = state_detail.clone();
        push(
            &mut wrong_renderer,
            "--renderer-profile-id",
            PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE_ID,
        );
        assert!(matches!(
            parse(wrong_renderer),
            Err("context build renderer profile conflicts with context pack profile")
        ));
        let mut detail_first = state_detail;
        push(
            &mut detail_first,
            "--expansion-profile-id",
            PRIMARY_FRONTIER_EXPANSION_PROFILE_ID,
        );
        push(&mut detail_first, "--frontier", &frontier);
        assert!(matches!(
            parse(detail_first),
            Err("state-detail L1 conflicts with context build frontier expansion")
        ));
        let mut expansion_first = detail_base;
        push(
            &mut expansion_first,
            "--expansion-profile-id",
            PRIMARY_FRONTIER_EXPANSION_PROFILE_ID,
        );
        push(&mut expansion_first, "--frontier", &frontier);
        push(
            &mut expansion_first,
            "--l1-profile-id",
            PRIMARY_STATE_DETAIL_L1_PROFILE_ID,
        );
        assert!(matches!(
            parse(expansion_first),
            Err("state-detail L1 conflicts with context build frontier expansion")
        ));

        let mut frontier_base = values.clone();
        replace(&mut frontier_base, "--root", GRAPH_ROOT)?;
        replace(&mut frontier_base, "--level", "l1")?;
        replace(
            &mut frontier_base,
            "--context-profile-id",
            PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE_ID,
        )?;
        let mut frontier_pack = frontier_base.clone();
        push(
            &mut frontier_pack,
            "--expansion-profile-id",
            PRIMARY_FRONTIER_EXPANSION_PROFILE_ID,
        );
        push(&mut frontier_pack, "--frontier", &frontier);
        let expected_frontier = |render| -> Result<serde_json::Value, &'static str> {
            let request = ContextBuildRequest::primary(
                "context-cli-store",
                ContextMapPublication::Current,
                ContextInspectLevel::L1,
                parse_root(GRAPH_ROOT)?,
                render,
                None,
                None,
            )
            .map_err(|_| "expected frontier build fixture request refused")?
            .with_frontier_expansion(vec![parse_root(&frontier)?])
            .map_err(|_| "expected frontier build fixture request refused")?;
            serde_json::to_value(request).map_err(|_| "request serialization failed")
        };
        let no_frontier_renderer = request_json(frontier_pack.clone())?;
        assert_eq!(
            no_frontier_renderer,
            expected_frontier(ContextRenderSelection::None)?
        );
        assert_eq!(
            no_frontier_renderer["context_profile_id"],
            PRIMARY_FRONTIER_CONTEXT_PACK_PROFILE_ID
        );
        assert_eq!(
            no_frontier_renderer["renderer_profile_ids"],
            serde_json::json!([])
        );
        let mut rendered_frontier = frontier_pack.clone();
        push(
            &mut rendered_frontier,
            "--renderer-profile-id",
            PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
        );
        push(
            &mut rendered_frontier,
            "--renderer-profile-id",
            PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE_ID,
        );
        let rendered = request_json(rendered_frontier.clone())?;
        assert_eq!(rendered, expected_frontier(ContextRenderSelection::Both)?);
        assert_eq!(
            rendered["renderer_profile_ids"],
            serde_json::json!([
                PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE_ID,
                PRIMARY_FRONTIER_CONTEXT_MARKDOWN_RENDERER_PROFILE_ID,
            ])
        );

        let mut mixed_renderer = frontier_pack.clone();
        push(
            &mut mixed_renderer,
            "--renderer-profile-id",
            PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
        );
        assert!(matches!(
            parse(mixed_renderer),
            Err("context build renderer profile conflicts with context pack profile")
        ));
        let mut wrong_family = frontier_pack.clone();
        replace(
            &mut wrong_family,
            "--context-profile-id",
            PRIMARY_CONTEXT_PACK_PROFILE_ID,
        )?;
        assert!(matches!(
            parse(wrong_family),
            Err("context build pack profile conflicts with expansion selection")
        ));
        assert!(matches!(
            parse(frontier_base.clone()),
            Err("context build pack profile conflicts with expansion selection")
        ));
        let mut structural_with_frontier_renderer = values.clone();
        push(
            &mut structural_with_frontier_renderer,
            "--renderer-profile-id",
            PRIMARY_FRONTIER_CONTEXT_JSON_RENDERER_PROFILE_ID,
        );
        assert!(matches!(
            parse(structural_with_frontier_renderer),
            Err("context build renderer profile conflicts with context pack profile")
        ));
        push(
            &mut rendered_frontier,
            "--renderer-profile-id",
            PRIMARY_CONTEXT_JSON_RENDERER_PROFILE_ID,
        );
        assert!(matches!(
            parse(rendered_frontier),
            Err("duplicate context build option")
        ));
        let mut duplicate_frontier = frontier_pack.clone();
        push(&mut duplicate_frontier, "--frontier", &frontier);
        assert!(matches!(
            parse(duplicate_frontier),
            Err("duplicate context build frontier")
        ));
        let mut own_root = frontier_pack.clone();
        replace(&mut own_root, "--frontier", GRAPH_ROOT)?;
        assert!(matches!(
            parse(own_root),
            Err("context build frontier must exclude its initial root")
        ));
        let mut invalid_frontier = frontier_pack.clone();
        replace(&mut invalid_frontier, "--frontier", PROJECT_ROOT)?;
        assert!(matches!(
            parse(invalid_frontier),
            Err("context build frontier requires graph-entity token")
        ));
        let mut empty_frontier = frontier_pack.clone();
        replace(&mut empty_frontier, "--frontier", "")?;
        assert!(matches!(
            parse(empty_frontier),
            Err("invalid context option length")
        ));
        let mut project_scope = frontier_pack.clone();
        replace(&mut project_scope, "--root", PROJECT_ROOT)?;
        assert!(matches!(
            parse(project_scope),
            Err("context build expansion requires l1 and graph-entity root")
        ));
        for level in ["l0", "both"] {
            let mut wrong_level = frontier_pack.clone();
            replace(&mut wrong_level, "--level", level)?;
            assert!(matches!(
                parse(wrong_level),
                Err("context build expansion requires l1 and graph-entity root")
            ));
        }

        // One exact boundary: 32 distinct canonical IDs, then a 33rd refusal.
        let mut bounded_frontier = frontier_base;
        push(
            &mut bounded_frontier,
            "--expansion-profile-id",
            PRIMARY_FRONTIER_EXPANSION_PROFILE_ID,
        );
        for index in 0..33 {
            let mut token = String::from("graph-entity@Z3JhcGgtbm9kZTpzaGEyNTY6");
            for bit in 0..6 {
                token.push_str(if index & (1 << bit) == 0 {
                    "YmJi"
                } else {
                    "Y2Nj"
                });
            }
            token.push_str(&"YmJi".repeat(15));
            token.push_str("Yg");
            push(&mut bounded_frontier, "--frontier", &token);
            if index == 31 {
                assert!(parse(bounded_frontier.clone()).is_ok());
            }
        }
        assert!(matches!(
            parse(bounded_frontier),
            Err("context build frontier limit exceeded")
        ));

        // Individually valid Project+Both syntax reaches the service refusal boundary.
        replace(&mut values, "--level", "both")?;
        let arguments = parse(values)?;
        let Request::Build(request) = arguments.request else {
            return Err("build parser selected the wrong typed operation");
        };
        let request = serde_json::to_value(request).map_err(|_| "request serialization failed")?;
        assert_eq!(request["level"], "both");
        assert_eq!(
            request["root"],
            serde_json::json!({"kind": "project", "id": "context-cli"})
        );
        Ok(())
    }
}
