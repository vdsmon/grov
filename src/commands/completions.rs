use clap::CommandFactory;
use clap_complete::Shell;

use crate::cli::Cli;

pub fn execute(shell: Shell) {
    clap_complete::generate(shell, &mut Cli::command(), "grov", &mut std::io::stdout());
}
