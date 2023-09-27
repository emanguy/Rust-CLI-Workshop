use crate::command::{exec_command, CLIArgs};
use clap::Parser;
use dotenvy::dotenv;

mod command;
mod config;
mod getoutline_connection;
mod logic;

fn main() {
    // Parse the CLI arguments via clap
    let args = CLIArgs::parse();

    // Read config data from environment variables
    let _ = dotenv();
    let config_result = config::parse_from_env();
    let config = match config_result {
        Ok(config) => config,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    // Match the entered command and route execution accordingly
    exec_command(&args, &config);
}
