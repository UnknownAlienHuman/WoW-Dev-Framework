#![forbid(unsafe_code)]

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let stderr = io::stderr();
    let code = wow_app::run(
        std::env::args_os().skip(1),
        stdin.lock(),
        stdout.lock(),
        stderr.lock(),
    );
    ExitCode::from(code)
}
