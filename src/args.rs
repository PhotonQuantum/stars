/// Argument parsing.
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Args {
    /// Do not actually star upstream repositories.
    #[clap(short, long)]
    pub dry_run: bool,
    /// Suppress all output.
    #[clap(short, long)]
    pub quiet: bool,
    /// Disable specific sources.
    #[clap(long)]
    pub disable_source: Vec<String>,
    /// Disable specific targets.
    #[clap(long)]
    pub disable_target: Vec<String>,
}
