use std::process::ExitCode;

use console::style;

fn main() -> ExitCode {
    if let Err(err) = grov::run() {
        eprintln!("{} {err:#}", style("error:").red().bold());
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
