use std::io;
use std::sync::atomic::AtomicBool;

static STOP: AtomicBool = AtomicBool::new(false);

fn main() {
    if ctrlc::set_handler(|| {
        STOP.store(true, std::sync::atomic::Ordering::Release);
    })
    .is_err()
    {
        eprintln!("signal_handler_unavailable: cancellation handler could not be installed");
        std::process::exit(8);
    }
    let exit = wow_reference_builder::run_with_stop(
        std::env::args().skip(1),
        &mut io::stdout().lock(),
        &mut io::stderr().lock(),
        &STOP,
    );
    std::process::exit(exit);
}
