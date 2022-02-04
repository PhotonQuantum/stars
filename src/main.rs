#![allow(clippy::module_name_repetitions)]
extern crate core;

use indicatif::{ProgressBar, ProgressStyle};

use crate::args::Args;
use crate::github::Github;
use crate::homebrew::Homebrew;
use crate::logger::{LogTarget, Logger};
use crate::registry::{SourceRegistry, TargetRegistry};

mod args;
mod common;
mod github;
mod homebrew;
mod logger;
mod registry;

fn main() {
    let logger = Logger::default();
    let args: Args = argh::from_env();

    let mut sources = SourceRegistry::new(&logger);
    sources.register(Homebrew);
    for disabled in args.disable_source {
        sources.deregister(disabled.as_str());
    }

    let mut targets = TargetRegistry::new(&logger);
    targets.register(Github::default());
    for disabled in args.disable_target {
        targets.deregister(disabled.as_str());
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}"));
    pb.set_message("Aggregating packages...");
    pb.enable_steady_tick(100);
    logger.set_target(LogTarget::Progress(pb));
    let packages = sources.aggregate(&targets);
    logger.set_target(LogTarget::Plain);

    let pb = ProgressBar::new(packages.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(100);
    logger.set_target(LogTarget::Progress(pb.clone()));
    for package in &packages {
        logger.info(format!("Starring {}", package));
        targets.star(package);
        pb.inc(1);
    }
    drop(pb);
    logger.set_target(LogTarget::Plain);
}
