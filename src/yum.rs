//! Yum integration (dnf compatible).

use std::collections::HashMap;
use std::process::Command;
use std::str;
use std::str::FromStr;

use regex::Regex;
use url::Url;

use crate::common::{BoxedError, Package, Source, SourceType};
use crate::{Logger, TargetRegistry};

#[derive(Debug)]
pub struct Yum;

impl Source for Yum {
    fn name(&self) -> &'static str {
        "yum"
    }

    fn source_type(&self) -> SourceType {
        SourceType::Global
    }

    fn available(&self) -> bool {
        which::which("yum").is_ok()
    }

    fn snapshot(
        &self,
        _logger: &Logger,
        _files: HashMap<&str, &[u8]>,
        targets: &TargetRegistry,
    ) -> Result<Vec<Package>, BoxedError> {
        let re = Regex::new(r#"Name +: (.+)[\s\S]*?URL +: (.+)"#).unwrap();

        let raw_output = Command::new("yum")
            .arg("info")
            .arg("installed")
            .output()?
            .stdout;
        let output = str::from_utf8(&*raw_output)?;

        Ok(re
            .captures_iter(output)
            .filter_map(|cap| targets.try_parse(cap[1].to_string(), &Url::from_str(&cap[2]).ok()?))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::tests::test_source;

    use super::Yum;

    #[test]
    fn test_yum() {
        test_source(&Yum, HashMap::new(), |packages| {
            assert!(!packages.is_empty());
        });
    }
}
