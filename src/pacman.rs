//! Pacman integration.

use std::collections::HashMap;
use std::process::Command;
use std::str;
use std::str::FromStr;

use regex::Regex;
use url::Url;

use crate::common::{BoxedError, Package, Source, SourceType};
use crate::{Logger, TargetRegistry};

#[derive(Debug)]
pub struct Pacman;

impl Source for Pacman {
    fn name(&self) -> &'static str {
        "pacman"
    }

    fn source_type(&self) -> SourceType {
        SourceType::Global
    }

    fn available(&self) -> bool {
        which::which("pacman").is_ok()
    }

    fn snapshot(
        &self,
        _logger: &Logger,
        _files: HashMap<&str, &[u8]>,
        targets: &TargetRegistry,
    ) -> Result<Vec<Package>, BoxedError> {
        let re = Regex::new(r#"Name +: (.+)[\s\S]*?URL +: (.+)"#).unwrap();

        let raw_output = Command::new("pacman").arg("-Qi").output()?.stdout;
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

    use super::Pacman;

    #[test]
    fn test_pacman() {
        test_source(&Pacman, HashMap::new(), |packages| {
            assert!(!packages.is_empty())
        });
    }
}
