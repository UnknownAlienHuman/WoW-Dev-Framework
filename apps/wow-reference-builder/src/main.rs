use std::io;

fn main() {
    let exit = wow_reference_builder::run(
        std::env::args().skip(1),
        &mut io::stdout().lock(),
        &mut io::stderr().lock(),
    );
    std::process::exit(exit);
}
