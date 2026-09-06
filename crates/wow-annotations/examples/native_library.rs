//! Native development entrypoint; acquisition/publication lives in shared support.
#[path = "support/mod.rs"]
mod driver;
fn main() -> std::process::ExitCode {
    match driver::run(std::env::args_os().skip(1).collect(), None) {
        Ok(partial) => std::process::ExitCode::from(if partial { 3 } else { 0 }),
        Err(error) => {
            eprintln!("native_library: {error}");
            std::process::ExitCode::from(2)
        }
    }
}
