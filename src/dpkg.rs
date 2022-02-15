//! Dpkg integration.
use std::collections::HashMap;
use std::process::Command;
use std::str;
use std::str::FromStr;

use url::Url;

use crate::common::{BoxedError, Package, Source, SourceType};
use crate::{Logger, TargetRegistry};

#[derive(Debug)]
pub struct Dpkg;

impl Source for Dpkg {
    fn name(&self) -> &'static str {
        "dpkg"
    }

    fn source_type(&self) -> SourceType {
        SourceType::Global
    }

    fn available(&self) -> bool {
        which::which("dpkg-query").is_ok()
    }

    fn snapshot(
        &self,
        _logger: &Logger,
        _files: HashMap<&str, &[u8]>,
        targets: &TargetRegistry,
    ) -> Result<Vec<Package>, BoxedError> {
        let raw_output = Command::new("dpkg-query")
            .arg("-f")
            .arg("${source:Package}\t${Homepage}\n")
            .arg("-W")
            .output()?
            .stdout;
        let output = str::from_utf8(&*raw_output)?;

        Ok(output
            .lines()
            .filter_map(|line| {
                line.split_once('\t').and_then(|(name, homepage)| {
                    targets.try_parse(name.to_string(), &Url::from_str(homepage).ok()?)
                })
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::tests::test_source;

    use super::Dpkg;

    #[test]
    fn test_dpkg() {
        test_source(&Dpkg, HashMap::new(), |packages| {
            assert!(!packages.is_empty())
        });
    }
}
