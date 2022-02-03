use std::collections::HashMap;
use std::process::Command;

use serde::Deserialize;
use url::Url;

use crate::common::{BoxedError, Package, Source, SourceType};
use crate::registry::StargazerRegistry;
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
        which::which("brew").is_ok()
    }

    fn snapshot(
        &self,
        logger: &Logger,
        _files: HashMap<&str, &[u8]>,
        stargazers: &StargazerRegistry,
    ) -> Result<Vec<Package>, BoxedError> {
        logger.debug("Fetching homebrew metadata...");

        let raw_output = Command::new("brew")
            .arg("info")
            .arg("--json=v2")
            .arg("--installed")
            .output()?
            .stdout;
        let output: Output = serde_json::from_slice(&raw_output)?;

        let formulae_iter = output.formulae.into_iter().filter_map(|formula| {
            stargazers
                .pack(formula.name.clone(), formula.homepage)
                .or_else(|| {
                    formula
                        .urls
                        .into_iter()
                        .find_map(|(_, rel)| stargazers.pack(formula.name.clone(), rel.url))
                })
        });
        let casks_iter = output.casks.into_iter().filter_map(|cask| {
            stargazers
                .pack(cask.token.clone(), cask.homepage)
                .or_else(|| stargazers.pack(cask.token, cask.url))
        });

        Ok(formulae_iter.chain(casks_iter).collect())
    }
}

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
