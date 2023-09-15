use clap::{Args, Parser, Subcommand};

/// A CLI for listing and downloading markdown files from GetOutline
#[derive(Parser)]
struct CLIArgs {
    #[command(subcommand)]
    subcommand: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    /// Say hello either to the world or to someone
    SayHello(HelloArgs),
}

#[derive(Args)]
struct HelloArgs {
    /// The name of the person to say hello to, or "world" by default
    #[arg(short, long)]
    name: Option<String>,
}

/// Runs the "say-hello" command
fn exec_say_hello(args: &HelloArgs) {
    match &args.name {
        Some(name) => println!("Hello, {name}!"),
        None => println!("Hello, world!"),
    }
}

/// An alternative way to write the "say-hello" function with a single println
#[allow(dead_code)]
fn alternative_exec_say_hello(args: &HelloArgs) {
    println!(
        "Hello, {}!",
        // .as_ref() converts &Option<T> to Option<&T>. .map_or will return the first parameter if the value isn't present
        //   and transform the contained value via the provided function if it is present and return that.
        args.name.as_ref().map_or("world", |name| name.as_str())
    );
}

fn main() {
    // Parse the CLI arguments via clap
    let args = CLIArgs::parse();

    // Match the entered command and route execution accordingly
    match args.subcommand {
        Subcommands::SayHello(args) => {
            exec_say_hello(&args);
            // alternative_say_hello(&args)
        }
    }
}
