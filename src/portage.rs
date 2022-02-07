//! Portage integration.

use std::collections::HashMap;
use std::process::Command;
use std::str;
use std::str::FromStr;

use regex::Regex;
use url::Url;

use crate::common::{BoxedError, Package, Source, SourceType};
use crate::{Logger, TargetRegistry};

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
        _logger: &Logger,
        _files: HashMap<&str, &[u8]>,
        targets: &TargetRegistry,
    ) -> Result<Vec<Package>, BoxedError> {
        let re = Regex::new("Homepage: *(.+)").unwrap();

        let raw_output = Command::new("equery").arg("list").arg("*").output()?.stdout;
        let packages = str::from_utf8(&*raw_output)?;

        Ok(packages
            .lines()
            .filter_map(|package| {
                let (_, name) = package.split_once("/")?;
                let raw_output = Command::new("equery")
                    .arg("meta")
                    .arg("-U")
                    .arg(package)
                    .output()
                    .ok()?
                    .stdout;
                let metadata = str::from_utf8(&*raw_output).ok()?;
                re.captures(metadata).and_then(|cap| {
                    let homepage = &cap[1];
                    targets.try_parse(name.to_string(), &Url::from_str(homepage).ok()?)
                })
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::test_source;

    use super::Portage;

    #[test]
    fn test_pacman() {
        test_source(&Portage);
    }
}