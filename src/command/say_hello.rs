use clap::Args;

#[derive(Args)]
pub struct HelloArgs {
    /// The name of the person to say hello to, or "world" by default
    #[arg(short, long)]
    name: Option<String>,
}

/// Runs the "say-hello" command
pub(super) fn exec_say_hello(args: &HelloArgs) {
    match &args.name {
        Some(name) => println!("Hello, {name}!"),
        None => println!("Hello, world!"),
    }
}
