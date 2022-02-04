#![allow(clippy::module_name_repetitions, clippy::default_trait_access)]
extern crate core;

use indicatif::{ProgressBar, ProgressStyle};

use crate::args::Args;
use crate::github::Github;
use crate::homebrew::Homebrew;
use crate::logger::{LogTarget, Logger};
use crate::persist::Persist;
use crate::registry::{SourceRegistry, TargetRegistry};

mod args;
mod common;
mod github;
mod homebrew;
mod logger;
mod persist;
mod registry;

fn main() {
    let args: Args = argh::from_env();
    let logger = Logger::new(args.quiet);

    let mut persist = Persist::new();

    let mut sources = SourceRegistry::new(&logger);
    sources.register(Homebrew);
    let mut targets = TargetRegistry::new(&logger, &mut persist);
    targets.register(Github::default());
    for disabled in args.disable {
        sources.deregister(disabled.as_str());
        targets.deregister(disabled.as_str());
    }

    let pb = if args.quiet {
        ProgressBar::hidden()
    } else {
        ProgressBar::new_spinner()
    };
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}"));
    pb.set_message("Aggregating packages...");
    pb.enable_steady_tick(100);
    logger.set_target(LogTarget::Progress(pb));
    let packages = sources.aggregate(&targets);
    logger.set_target(LogTarget::Plain);

    let pb = if args.quiet {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(packages.len() as u64)
    };
    let max_name_len = packages
        .iter()
        .map(|package| package.name.len())
        .max()
        .unwrap_or_default();
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                format!(
                    "{{spinner:.green}} [{{wide_bar:.cyan/blue}}] {{pos}}/{{len}} ({{eta}}) {{msg:{}}}",
                    max_name_len
                )
                    .as_str(),
            )
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(100);
    logger.set_target(LogTarget::Progress(pb.clone()));
    for package in &packages {
        pb.set_message(package.to_string());
        targets.star(package);
        pb.inc(1);
    }
    drop(pb);
    logger.set_target(LogTarget::Plain);
}
