use clap::Parser;
use crate::command::{CLIArgs, exec_command};

mod command;

fn main() {
    // Parse the CLI arguments via clap
    let args = CLIArgs::parse();

    // Match the entered command and route execution accordingly
    exec_command(&args);
}
