use std::ffi::OsString;
use std::io::{self, Write};

use wow_service::frontend::{FrontendService, MCP_MAX_OUTPUT_BYTES};

pub fn run(arguments: Vec<OsString>) -> u8 {
    if arguments.len() != 2 || arguments[0] != "transport" || arguments[1] != "capabilities" {
        diagnostic("wow: usage: wow transport capabilities");
        return 64;
    }
    let service = match FrontendService::new() {
        Ok(service) => service,
        Err(_) => {
            diagnostic("wow: frontend registry initialization failed");
            return 64;
        }
    };
    let bytes = match serde_json::to_vec(service.frontend_capabilities()) {
        Ok(bytes) if bytes.len() <= MCP_MAX_OUTPUT_BYTES => bytes,
        _ => {
            diagnostic("wow: frontend capability encoding failed");
            return 4;
        }
    };
    let mut stdout = io::stdout().lock();
    if stdout
        .write_all(&bytes)
        .and_then(|()| stdout.write_all(b"\n"))
        .is_err()
    {
        diagnostic("wow: capability output failed");
        4
    } else {
        0
    }
}

fn diagnostic(message: &str) {
    let _ = writeln!(io::stderr().lock(), "{message}");
}
