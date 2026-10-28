use clap::Parser;

#[derive(Debug, Parser)]
#[command(
    name = "envsentry",
    version,
    about = "Detect configuration drift without exposing secrets.",
    arg_required_else_help = true
)]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
}
