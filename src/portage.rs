//! Portage integration.

use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use std::str::FromStr;

use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use tap::TapOptional;
use url::Url;

use crate::common::{BoxedError, Package, Source, SourceType};
use crate::{Logger, TargetRegistry};

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?m)^HOMEPAGE="(.+)"$"#).unwrap());

#[derive(Debug)]
pub struct Portage;

impl Source for Portage {
    fn name(&self) -> &'static str {
        "portage"
    }

    fn source_type(&self) -> SourceType {
        SourceType::Global
    }

    fn available(&self) -> bool {
        which::which("equery").is_ok()
    }

    fn snapshot(
        &self,
        logger: &Logger,
        _files: HashMap<&str, &[u8]>,
        targets: &TargetRegistry,
    ) -> Result<Vec<Package>, BoxedError> {
        let raw_output = Command::new("equery")
            .arg("list")
            .arg("*")
            .arg("-F")
            .arg("$repo;$category;$name;$fullversion")
            .output()?
            .stdout;
        let packages = str::from_utf8(&*raw_output)?;

        let mut repo_path_cache = HashMap::new();

        Ok(packages
            .lines()
            .filter_map(|package| {
                let (repo, category, name, full_version) = package.split(';').collect_tuple()?;
                let repo_path = repo_path_cache
                    .entry(repo.to_string())
                    .or_insert_with(|| {
                        repo_path(repo).tap_none(|| {
                            logger.warn(format!("repo {} not found", repo));
                        })
                    })
                    .as_ref()?;
                let homepages = portage_homepage(repo_path, category, name, full_version)
                    .unwrap_or_else(|e| {
                        logger.warn(format!("package {}/{}: {}", category, name, e));
                        None
                    })?;
                homepages
                    .into_iter()
                    .find_map(|homepage| targets.try_parse(name.to_string(), &homepage))
            })
            .collect())
    }
}

fn repo_path(repo: &str) -> Option<PathBuf> {
    PathBuf::from_str(
        str::from_utf8(
            &*Command::new("portageq")
                .arg("get_repo_path")
                .arg("/")
                .arg(repo)
                .output()
                .ok()?
                .stdout,
        )
        .ok()?
        .trim(),
    )
    .ok()
}

fn portage_homepage(
    repo_path: impl AsRef<Path>,
    category: &str,
    name: &str,
    full_version: &str,
) -> Result<Option<Vec<Url>>, BoxedError> {
    let ebuild_path = repo_path
        .as_ref()
        .join(category)
        .join(name)
        .join(format!("{}-{}.ebuild", name, full_version));
    match fs::read_to_string(ebuild_path) {
        Ok(ebuild) => Ok(RE
            .captures(ebuild.as_str())
            .map(|cap| {
                cap[1]
                    .split(' ')
                    .map(|homepage| Url::from_str(homepage).ok())
                    .collect::<Option<Vec<Url>>>()
            })
            .unwrap_or_default()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Box::new(e)),
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::test_source;

    use super::Portage;

    #[test]
    fn test_portage() {
        test_source(&Portage);
    }
}
