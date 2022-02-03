#![allow(clippy::module_name_repetitions)]
extern crate core;

use indicatif::{ProgressBar, ProgressStyle};

use crate::args::Args;
use crate::github::Github;
use crate::homebrew::Homebrew;
use crate::logger::{LogTarget, Logger};
use crate::registry::{SourceRegistry, StargazerRegistry};
use crate::utils::Spinner;

mod args;
mod common;
mod github;
mod homebrew;
mod logger;
mod registry;
mod utils;

fn main() {
    let logger = Logger::default();
    let args: Args = argh::from_env();

    let mut sources = SourceRegistry::new(&logger);
    sources.register(Homebrew);
    for disabled in args.disable_source {
        sources.deregister(disabled.as_str());
    }

    let mut stargazers = StargazerRegistry::new(&logger);
    stargazers.register(Github);
    for disabled in args.disable_target {
        stargazers.deregister(disabled.as_str());
    }

    let pb = ProgressBar::new_spinner();
    pb.set_message("Aggregating packages...");
    logger.set_target(LogTarget::Progress(pb.clone()));
    let packages = {
        let _spinner = Spinner::new(pb);
        sources.aggregate(&stargazers)
    };
    logger.set_target(LogTarget::Plain);

    let pb = ProgressBar::new(packages.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
            .progress_chars("#>-"),
    );
    logger.set_target(LogTarget::Progress(pb.clone()));
    for package in &packages {
        logger.info(format!("Starring {}", package));
        if let Err(e) = stargazers.star(package) {
            logger.error(format!("error when starring {} - {}", package, e));
        }
        pb.inc(1);
    }
    logger.set_target(LogTarget::Plain);
}
