use crate::command::{exec_command, CLIArgs};
use clap::Parser;

mod command;
mod getoutline_connection;

fn main() {
    // Parse the CLI arguments via clap
    let args = CLIArgs::parse();

    // Match the entered command and route execution accordingly
    exec_command(&args);
}
