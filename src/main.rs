#![allow(
    clippy::module_name_repetitions,
    clippy::default_trait_access,
    clippy::similar_names
)]
extern crate core;

use crate::args::Args;
use crate::cargo::{Cargo, CargoGlobal};
use crate::dpkg::Dpkg;
use crate::github::Github;
use crate::gitlab::Gitlab;
use crate::homebrew::Homebrew;
use crate::logger::Logger;
use crate::pacman::Pacman;
use crate::persist::Persist;
use crate::portage::Portage;
use crate::registry::{SourceRegistry, TargetRegistry};
use crate::yum::Yum;
use crate::zypper::Zypper;

mod args;
mod cargo;
mod common;
mod dpkg;
mod github;
mod gitlab;
mod homebrew;
mod logger;
mod pacman;
mod persist;
mod portage;
mod registry;
mod yum;
mod zypper;

#[cfg(test)]
mod tests;

fn main() {
    let args: Args = argh::from_env();
    let logger = Logger::new(args.quiet);

    let mut persist = Persist::new(&logger, args.ignore_saved);

    // !! When you implement a new source, you need to add it to the SourceRegistry.
    let mut sources = SourceRegistry::new(&logger);
    sources.register(Homebrew);
    sources.register(Pacman);
    sources.register(Dpkg);
    sources.register(Yum);
    sources.register(Portage);
    sources.register(Cargo);
    sources.register(CargoGlobal);
    sources.register(Zypper);

    // !! When you implement a new target, you need to add it to the TargetRegistry.
    let mut targets = TargetRegistry::new(&logger, &mut persist);
    targets.register(Github::default());
    targets.register(Gitlab::default());

    for disabled in args.disable {
        sources.deregister(disabled.as_str());
        targets.deregister(disabled.as_str());
    }

    logger.set_prefix("Aggregating packages...");
    let packages = sources.aggregate(&targets);

    logger.set_prefix("Starring packages...");
    logger.set_progress_bar_determinate(packages.len() as u64);
    for package in &packages {
        logger.set_message(package);
        if args.dry_run {
            logger.debug(format!("Dry-run: star {}, ignored", package));
        } else {
            targets.star(package);
        }
        logger.with_progress_bar(|pb| pb.inc(1));
    }
    logger.set_plain();

    logger.info(format!("Done! {} packages starred.", packages.len()));
}
