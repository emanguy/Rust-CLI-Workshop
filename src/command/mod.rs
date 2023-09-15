use clap::{Parser, Subcommand};

pub mod say_hello;

/// A CLI for listing and downloading markdown files from GetOutline
#[derive(Parser)]
pub struct CLIArgs {
    #[command(subcommand)]
    subcommand: Subcommands,
}

#[derive(Subcommand)]
pub enum Subcommands {
    /// Say hello either to the world or to someone
    SayHello(say_hello::HelloArgs),
}

/// Run the appropriate command based on the parameters
pub fn exec_command(args: &CLIArgs) {
    match &args.subcommand {
        Subcommands::SayHello(args) => {
            say_hello::exec_say_hello(args);
        }
    }
}
