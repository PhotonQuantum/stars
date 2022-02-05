//! Homebrew integration.

use std::collections::HashMap;
use std::process::Command;

use serde::Deserialize;
use url::Url;

use crate::common::{BoxedError, Package, Source, SourceType};
use crate::registry::TargetRegistry;
use crate::Logger;

#[derive(Debug)]
pub struct Homebrew;

impl Source for Homebrew {
    fn name(&self) -> &'static str {
        "homebrew"
    }

    fn source_type(&self) -> SourceType {
        SourceType::Global
    }

    fn available(&self) -> bool {
        which::which("brew").is_ok() // ensure `brew` command is present.
    }

    fn snapshot(
        &self,
        _logger: &Logger,
        _files: HashMap<&str, &[u8]>,
        targets: &TargetRegistry,
    ) -> Result<Vec<Package>, BoxedError> {
        // Call `brew info --json=v2 --installed` to get a report of all installed packages.
        let raw_output = Command::new("brew")
            .arg("info")
            .arg("--json=v2")
            .arg("--installed")
            .output()?
            .stdout;
        // Parse output.
        let output: Output = serde_json::from_slice(&raw_output)?;

        // Extract repositories from formulae.
        let formulae_iter = output.formulae.into_iter().filter_map(|formula| {
            targets
                // Try to extract the repository from homepage first.
                .try_parse(formula.name.clone(), &formula.homepage)
                .or_else(|| {
                    // If failed, extract the repository from the first parsable url.
                    formula
                        .urls
                        .into_iter()
                        .find_map(|(_, rel)| targets.try_parse(formula.name.clone(), &rel.url))
                })
        });
        // Extract repositories from casks.
        let casks_iter = output.casks.into_iter().filter_map(|cask| {
            targets
                // Try to extract the repository from homepage first.
                .try_parse(cask.token.clone(), &cask.homepage)
                // If failed, extract the repository from url.
                .or_else(|| targets.try_parse(cask.token, &cask.url))
        });

        // Chain the two iterators together to get a list of all repositories.
        Ok(formulae_iter.chain(casks_iter).collect())
    }
}

// --- homebrew json metadata definitions (simplified) ---

#[derive(Debug, Deserialize)]
struct Output {
    formulae: Vec<Formula>,
    casks: Vec<Cask>,
}

#[derive(Debug, Deserialize)]
struct Formula {
    name: String,
    homepage: Url,
    urls: HashMap<String, Release>,
}

#[derive(Debug, Deserialize)]
struct Release {
    url: Url,
}

#[derive(Debug, Deserialize)]
struct Cask {
    token: String,
    homepage: Url,
    url: Url,
}
