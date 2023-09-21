# Rust CLI Workshop

This repository contains code that will comprise a CLI app which is able to both list and download documents from getOutline's API.

The workshop for this codebase is run in 4 steps:
1. Scaffolding out the app (pulling in CLI app dependencies and setting up a skeleton)
2. Setting up methods to list the getOutline documents from the API using Cargo Examples
3. Writing business logic and tests
4. Integrating the API with the business logic and commands, plus adding a download command

The repository has branches which contain snapshots of the code at each of the checkpoints listed above. These can be used as reference during the workshop to see if you're going in the right direction.

## Workshop 1 - Step 1

For the first part of the workshop, we're going to install [CLAP](https://docs.rs/clap/latest/clap/index.html) as a dependency and write a simple command that says hello to the caller.

### 1 - Adding CLAP as a dependency

To start, we need to add our command line library to our Rust app. We'll start by using the `cargo add` command to do so:

    cargo add clap

When the command finishes running, you should notice that `Cargo.toml` has been updated with a new entry in the `[dependencies]` section. This lists CLAP as a
dependency and specifies the version we're depending on. We now have CLAP available to us!

### 2 - Creating the argument parsing data structure

To the main file we'll want to add a `struct` to hold the information provided to us on the command line. We can later add things from CLAP to auto-fill the struct based on command line input.

Let's start with this (it will produce an error, but stick with me):

```rust
/// A CLI for listing and downloading markdown files from GetOutline
#[derive(Parser)]
struct CLIArgs {
    
}
```

Notice the `#[derive(Parser)]` line at the very beginning - this is what's called a **derive macro**. Rust itself and 3rd party libraries can provide these to auto-implement functionality based on
data structures in the language. In this case, we're saying that we want to auto-implement the `Parser` trait from CLAP. This is a bundle of functionality that will
handle parsing command line arguments and fill the struct based on said arguments. Many derive macros for various traits exist in Rust, such as the `Debug` trait which will allow printing the contents
of a data structure to the command line.

However, as stated previously you'll notice that `#[derive(Parser)]` returns an error:

```
error: cannot find derive macro `Parser` in this scope
 --> src/main.rs:3:10
  |
3 | #[derive(Parser)]
  |          ^^^^^^
```

Let's fix that.

### 3 - Adding dependency features

If you take a look at [clap's documentation](https://docs.rs/clap/latest/clap/_derive/_tutorial/chapter_0/index.html), you'll see that its
derive macro features require the "derive" feature flag.

Rust crates can expose optional, opt-in functionality using feature flags. In this case, because we want the derive macro features, we should make sure
to perform a `cargo add` specifying this feature flag, and we'll also tack on the `wrap_help` feature to wrap helptext if it gets too long. To do this, we can
just run `cargo add` again, this time adding the feature flags we want enabled:

    cargo add clap --features derive wrap_help

Now we should just need to import `clap::Parser` at the top of the file and the code should compile:

```rust
use clap::Parser;
```

### 4 - Running CLAP and displaying helptext

Next up, let's actually use the "parser" functionality and render some helptext. Let's update our main function to just perform the parse, using
the `CLIArgs::parse()` function that gets generated from the `Parser` derive macro:

```rust
fn main() {
    // Parse the CLI arguments via clap
    let args = CLIArgs::parse();
}
```

Now we can run the command like so, providing the `--help` flag to render our helptext:

    cargo run -- --help

This should output the following:

```
A CLI for listing and downloading markdown files from GetOutline

Usage: get-outline

Options:
  -h, --help  Print help
```

But wait, how did that description get generated? Well, when we applied the `Parser` derive macro, it was able to look up both the name of the project and the
doc comment on the struct it was generating code for. Using that data it was automatically able to generate the helptext you see when using the `--help` flag. As
we continue building the CLI app, the helptext will stay up-to-date and pull in any doc comments we put on the data structure we're deriving `Parser` for.

Speaking of, let's add a simple command to get this command line app to do something

### 5 - Adding subcommands to our CLI app

For this step in the workshop, we'll build a simple subcommand which will either print "Hello world" or greet the caller of the command
via a `--name` flag. Let's first update our imports to the following:

```rust
use clap::{Args, Parser, Subcommand};
```

This will import several submodules from the `clap` crate, giving us the `Args` and `Subcommand` derive macros. Below our main `CLIArgs` struct, let's now
add 2 more structs to handle the `say-hello` subcommand and its arguments:

```rust
#[derive(Subcommand)]
enum Subcommands {
    /// Say hello either to the world or to someone
    SayHello(HelloArgs),
}

#[derive(Args)]
struct HelloArgs {
    /// The name of the person to say hello to, or "world" by default
    #[arg(short, long)]
    name: Option<String>
}
```

The `Subcommands` enum specifies the list of subcommands available off of the main command. It defines the `SayHello` variant for the `say-hello` command
and it holds `HelloArgs`, which can then be broken out of the command structure and passed to a function to execute the `say-hello` command. `HelloArgs` then
defines the arguments available for the `say-hello` subcommand. It specifies one argument, an optional name, which can be filled when the command is invoked to
specify the name of the person to greet via the `say-hello` function. Using an attribute available from CLAP, we're able to specify that we want the argument to have
both a "short" and "long" name (so `-n` or `--name`). We'll define the implementation later, but let's first attach these subcommands to the main
CLAP data structure, which will now look like this:

```rust
/// A CLI for listing and downloading markdown files from GetOutline
#[derive(Parser)]
struct CLIArgs {
    #[command(subcommand)]
    subcommand: Subcommands,
}
```

Now the parser is aware of the subcommands we defined in `Subcommands`, and we use the `#[command(subcommand)]` attribute to tell the
`Parser` derive macro that the `subcommand` property should be treated as a set of subcommands. Let's now render the helptext again:

```
A CLI for listing and downloading markdown files from GetOutline

Usage: get-outline <COMMAND>

Commands:
  say-hello  Say hello either to the world or to someone
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

We can even see the `--name` option for the `say-hello` subcommand if we run `--help` on it:

    cargo run -- say-hello --help

Which should render:

```
Say hello either to the world or to someone

Usage: get-outline say-hello [OPTIONS]

Options:
  -n, --name <NAME>  The name of the person to say hello to, or "world" by default
  -h, --help         Print help
```

Okay, let's run the command!

    cargo run -- say-hello

...which does nothing. We'll need to actually provide an implementation of the `say-hello` command next.

### 6 - Adding an implementation for the subcommand

OK! Using the arguments of the `say-hello` subcommand, let's write a function that will output what we want:

```rust
/// Runs the "say-hello" command
fn exec_say_hello(args: &HelloArgs) {
    match &args.name {
        Some(name) => println!("Hello, {name}!"),
        None => println!("Hello, world!"),
    }
}
```

Let's break down what this function is doing:
1. It's accepting the arguments via a **reference** to `HelloArgs`. This is due to Rust's ownership and borrowing system.
2. It's performing a match against the `name` argument to determine how to handle the case where the name is present and where it isn't.

#### On accepting a reference

Rust has an "ownership and borrowing" system to keep track of data ownership within a program. Effectively, one piece of code owns a piece of data
and if the data is no longer used past the end of the owner's function (or *scope*) the memory used to store the data is freed. This is how rust gets
away with not using garbage collection.

If we were to take `args` as just a `HelloArgs` and not a `&HelloArgs`, we would then steal ownership of `HelloArgs` away from whatever code was providing
the arguments to us. There's no need to do that, as we just need to read the data stored with the arguments to do our thing. Because of that, we accept a reference
to just "borrow" the data from the caller rather than taking full ownership of it, and then the caller can continue doing stuff with the argument data as much as it wants
before the data falls out of scope.

#### On the match statement

In Rust, there is no concept of "null". Instead, to represent data that may not be present, the `Option` enum is used. this enum has two variants: `Some(T)` meaning
the data is actually present and `None` which means the data is not present. Rust does not allow you access to the enclosed data until you determine which variant the
enum currently is. To do this, we use Rust's `match` statement to match against the possible `name` variants and provide behavior for the case when the data is or isn't present.

In the first `match` arm, we provide the functionality for if the data is actually present. In that case, we want to greet the name passed to the `say-hello` argument. Otherwise, in the
second `match` arm we provide functionality for if the data is missing. In that case, we just want to print "Hello world".

You'll notice in the first match arm we're actually able to extract the data from the `Some(T)` variant of the optional. This is a technique called **destructuring**, where
by specifying a variable name in a pattern match we're able to pull data out of the data structure and reference it by the provided name. It should be noted that `match` statements
are **exhaustive**, so you need to provide a match arm for every possible input, or just use the catch-all pattern `_`. Try removing the `None` arm and see for yourself!

#### Continuing on...

Now we just need to determine what command was run and make sure to execute our `exec_say_hello()` function when the user specifies the `say-hello` subcommand!

We can just inspect the parsed data structure to do this with a `match` statement. Add the following to `main()` after the call to `CLIArgs::parse()`:

```rust
fn main() {
    // ...argument parsing...
    
    // Match the entered command and route execution accordingly
    match args.subcommand {
        Subcommands::SayHello(args) => {
            exec_say_hello(&args);
        }
    }
}
```

The great part about the `match` statement being exhaustive is the fact that if you add more subcommands later, Rust will give you a compile error reminding
you to provide functionality for the other subcommands you haven't provided match arms for!

Now, finally, let's run our command:

    cargo run -- say-hello

This should have printed "Hello, world!"! Now let's try it with the `name` argument:

    cargo run -- say-hello --name John

Now, it should print "Hello, John!". Let's make this a standalone executable now!

### 7 - Building the code

To build an optimized, production executable just run the following command:

    cargo build --release

Once the command finishes, you should have an executable ready in the `target/release` directory. Switch to that directory and try out the `say-hello` subcommand!

    ./get-outline say-hello
    ./get-outline say-hello --name John

### Conclusion

You should now have a fully working, basic CLI app using CLAP! In the next section, we'll break up the code into modules to make everything a little more maintainable down the line.
