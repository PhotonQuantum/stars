//! Argument parsing.
use argh::FromArgs;

#[derive(Debug, FromArgs)]
/// Star your upstream.
pub struct Args {
    /// do not actually star upstream repositories
    #[argh(switch, short = 'd')]
    pub dry_run: bool,
    /// suppress all output
    #[argh(switch, short = 'q')]
    pub quiet: bool,
    /// disable specific modules
    #[argh(option)]
    pub disable: Vec<String>,
}
