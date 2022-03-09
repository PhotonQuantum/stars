//! Zypper integration.

use std::collections::HashMap;
use std::process::Command;
use std::str;
use std::str::FromStr;

use regex::Regex;
use url::Url;

use crate::common::{BoxedError, Package, Source, SourceType};
use crate::{Logger, TargetRegistry};

#[derive(Debug)]
pub struct Zypper;

impl Source for Zypper {
    fn name(&self) -> &'static str {
        "zypper"
    }

    fn source_type(&self) -> SourceType {
        SourceType::Global
    }

    fn available(&self) -> bool {
        which::which("zypper").is_ok()
    }

    fn snapshot(
        &self,
        _logger: &Logger,
        _files: HashMap<&str, &[u8]>,
        targets: &TargetRegistry,
    ) -> Result<Vec<Package>, BoxedError> {
        let re_installed = Regex::new(r#"<solvable status="installed" name="([\w-]+)""#).unwrap();
        let re_detail = Regex::new(r#"Name +: (.+)[\s\S]*?URL +: (.+)"#).unwrap();

        let raw_output = Command::new("zypper")
            .arg("-x")
            .arg("search")
            .arg("-i")
            .output()?
            .stdout;
        let output = str::from_utf8(&*raw_output)?;
        let installed: Vec<_> = re_installed
            .captures_iter(output)
            .map(|cap| cap[1].to_string())
            .collect();

        let raw_output = Command::new("zypper")
            .arg("info")
            .args(installed)
            .output()?
            .stdout;
        let output = str::from_utf8(&*raw_output)?;

        Ok(re_detail
            .captures_iter(output)
            .filter_map(|cap| targets.try_parse(cap[1].to_string(), &Url::from_str(&cap[2]).ok()?))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::tests::test_source;

    use super::Zypper;

    #[test]
    fn test_zypper() {
        test_source(&Zypper, HashMap::new(), |packages| {
            assert!(!packages.is_empty());
        });
    }
}
