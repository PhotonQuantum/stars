use std::collections::HashMap;
use std::process::Command;
use std::str;

use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use tap::TapFallible;
use url::Url;

use crate::common::{BoxedError, Package, Source, SourceType, HTTP};
use crate::{Logger, TargetRegistry};

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^[a-zA-Z][a-zA-Z0-9-_]*").unwrap());

pub struct CargoGlobal;

impl Source for CargoGlobal {
    fn name(&self) -> &'static str {
        "cargo(global)"
    }

    fn source_type(&self) -> SourceType {
        SourceType::Global
    }

    fn available(&self) -> bool {
        which::which("cargo").is_ok()
    }

    fn snapshot(
        &self,
        logger: &Logger,
        _files: HashMap<&str, &[u8]>,
        targets: &TargetRegistry,
    ) -> Result<Vec<Package>, BoxedError> {
        let raw_output = Command::new("cargo")
            .arg("install")
            .arg("--list")
            .output()?
            .stdout;
        let output = str::from_utf8(&*raw_output)?;

        let crates: Vec<_> = RE
            .find_iter(output)
            .map(|m| m.as_str().to_string())
            .collect();

        logger.set_progress_bar_determinate(crates.len() as u64);

        Ok(fetch_crates_meta(logger, targets, &crates))
    }
}

pub struct Cargo;

impl Source for Cargo {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn source_type(&self) -> SourceType {
        SourceType::Local(&["Cargo.toml"])
    }

    fn available(&self) -> bool {
        true
    }

    fn snapshot(
        &self,
        logger: &Logger,
        files: HashMap<&str, &[u8]>,
        targets: &TargetRegistry,
    ) -> Result<Vec<Package>, BoxedError> {
        let raw_cargo_toml = files.get("Cargo.toml").unwrap();
        let cargo_toml: CargoToml = toml::from_slice(raw_cargo_toml)?;
        let crates: Vec<_> = cargo_toml
            .all_dependencies()
            .map(|(key, value)| entry_to_name(&*key, value).to_string())
            .collect();

        logger.set_progress_bar_determinate(crates.len() as u64);

        Ok(fetch_crates_meta(logger, targets, &crates))
    }
}

fn entry_to_name<'a>(key: &'a str, value: &'a CrateValue) -> &'a str {
    if let CrateValue::Map { name: Some(name) } = value {
        name.as_str()
    } else {
        key
    }
}

fn fetch_crates_meta(logger: &Logger, targets: &TargetRegistry, crates: &[String]) -> Vec<Package> {
    crates
        .iter()
        .filter_map(|name| {
            logger.set_message(name.as_str());
            let crate_ = query_crate(name.as_str())
                .tap_err(|e| {
                    logger.error(&format!(
                        "Failed to query metadata for crate {}: {}",
                        name, e
                    ));
                })
                .ok()?;
            logger.with_progress_bar(|pb| pb.inc(1));
            Some((name, crate_))
        })
        .filter_map(|(name, crate_)| {
            crate_
                .homepage
                .into_iter()
                .chain(crate_.repository)
                .find_map(|url| targets.try_parse(name.clone(), &url))
        })
        .collect()
}

fn query_crate(name: &str) -> Result<Crate, BoxedError> {
    let url = format!("https://crates.io/api/v1/crates/{}", name);
    let resp: Resp = HTTP.get(&url).send()?.json()?;
    Ok(resp.crate_data)
}

#[derive(Deserialize)]
struct Resp {
    #[serde(rename = "crate")]
    crate_data: Crate,
}

#[derive(Deserialize)]
struct Crate {
    homepage: Option<Url>,
    repository: Option<Url>,
}

#[derive(Debug, Deserialize)]
struct CargoToml {
    #[serde(default)]
    dependencies: HashMap<String, CrateValue>,
    #[serde(default)]
    dev_dependencies: HashMap<String, CrateValue>,
    #[serde(default)]
    build_dependencies: HashMap<String, CrateValue>,
}

impl CargoToml {
    fn all_dependencies(&self) -> impl Iterator<Item = (&str, &CrateValue)> {
        self.dependencies
            .iter()
            .chain(self.dev_dependencies.iter())
            .chain(self.build_dependencies.iter())
            .map(|(k, v)| (k.as_str(), v))
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CrateValue {
    Ver(String),
    Map { name: Option<String> },
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use maplit::hashmap;

    use crate::tests::test_source;
    use crate::CargoGlobal;

    use super::Cargo;

    #[test]
    fn test_cargo() {
        test_source(
            &Cargo,
            hashmap! {
                "Cargo.toml" => &include_bytes!("../Cargo.toml")[..],
            },
            |packages| assert!(!packages.is_empty()),
        );
    }

    #[test]
    fn test_cargo_global() {
        test_source(&CargoGlobal, HashMap::new(), |packages| {
            assert!(!packages.is_empty());
        });
    }
}
