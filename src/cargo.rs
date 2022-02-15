use std::collections::HashMap;

use serde::Deserialize;
use tap::TapFallible;
use url::Url;

use crate::common::{BoxedError, Package, Source, SourceType, HTTP};
use crate::{Logger, TargetRegistry};

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

        Ok(crates
            .into_iter()
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
            .collect())
    }
}

fn entry_to_name<'a>(key: &'a str, value: &'a CrateValue) -> &'a str {
    if let CrateValue::Map { name: Some(name) } = value {
        name.as_str()
    } else {
        key
    }
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
    use maplit::hashmap;

    use crate::tests::test_source;

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
}
