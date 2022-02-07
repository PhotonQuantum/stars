#![allow(clippy::module_name_repetitions, clippy::default_trait_access)]
extern crate core;

use indicatif::{ProgressBar, ProgressStyle};

use crate::args::Args;
use crate::dpkg::Dpkg;
use crate::github::Github;
use crate::homebrew::Homebrew;
use crate::logger::Logger;
use crate::pacman::Pacman;
use crate::persist::Persist;
use crate::portage::Portage;
use crate::registry::{SourceRegistry, TargetRegistry};
use crate::yum::Yum;

mod args;
mod common;
mod dpkg;
mod github;
mod homebrew;
mod logger;
mod pacman;
mod persist;
mod portage;
mod registry;
mod yum;

#[cfg(test)]
mod tests;

fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(512)
        .build_global()
        .unwrap();

    let args: Args = argh::from_env();
    let logger = Logger::new(args.quiet);

    let mut persist = Persist::new(&logger);

    // !! When you implement a new source, you need to add it to the SourceRegistry.
    let mut sources = SourceRegistry::new(&logger);
    sources.register(Homebrew);
    sources.register(Pacman);
    sources.register(Dpkg);
    sources.register(Yum);
    sources.register(Portage);

    // !! When you implement a new target, you need to add it to the TargetRegistry.
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
    logger.progress_bar(pb);
    let packages = sources.aggregate(&targets);
    logger.plain();

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
    logger.progress_bar(pb.clone());
    for package in &packages {
        pb.set_message(package.to_string());
        if args.dry_run {
            logger.debug(format!("Dry-run: star {}, ignored", package));
        } else {
            targets.star(package);
        }
        pb.inc(1);
    }
    logger.plain();
}
