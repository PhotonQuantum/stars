/// Argument parsing.
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Args {
    /// Do not actually star upstream repositories.
    #[clap(short, long)]
    dry_run: bool,
    /// Suppress all output.
    #[clap(short, long)]
    quiet: bool
}