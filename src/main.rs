mod args;
mod chunk;
mod chunk_type;
mod commands;
mod png;

use structopt::StructOpt;

/// Holds any kind of error.
pub type Error = Box<dyn std::error::Error>;
/// Holds a `Result` of any kind of error.
pub type Result<T> = std::result::Result<T, Error>;

#[doc(hidden)]
fn main() -> Result<()> {
    let cli = args::Cli::from_args();
    commands::run(cli.subcommand)
}
