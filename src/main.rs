#[doc(hidden)]
mod args;
/// Parse and work with PNG chunks.
mod chunk;
/// Parse and work with the chunk type of PNG chunks.
mod chunk_type;
#[doc(hidden)]
mod commands;
/// Parse and work with a fully-built PNG, like adding/removing chunks.
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
