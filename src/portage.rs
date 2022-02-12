//! Portage integration.

use std::collections::HashMap;
use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use std::str::FromStr;
use std::{fs, io};

use once_cell::sync::Lazy;
use regex::Regex;
use tap::TapFallible;
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
        which::which("portageq").is_ok()
    }

    fn snapshot(
        &self,
        logger: &Logger,
        _files: HashMap<&str, &[u8]>,
        targets: &TargetRegistry,
    ) -> Result<Vec<Package>, BoxedError> {
        let vdb = vdb_path()?;

        Ok(iter_atoms(vdb, logger)?
            .into_iter()
            .filter_map(|atom| {
                let name =
                    extract_name_from_fullname(&*atom.fullname.to_string_lossy()).to_string();
                let homepages = homepages(atom.ebuild_path).unwrap_or_else(|e| {
                    logger.warn(format!("atom {:?}: {}", name, e));
                    None
                })?;
                homepages
                    .into_iter()
                    .find_map(|homepage| targets.try_parse(name.clone(), &homepage))
            })
            .collect())
    }
}

/// Get vdb path from portage (normally /var/db/pkg).
fn vdb_path() -> Result<PathBuf, BoxedError> {
    let raw_output = Command::new("portageq").arg("vdb_path").output()?;
    let vdb_path = str::from_utf8(&*raw_output.stdout)?;
    Ok(PathBuf::from(vdb_path.trim()))
}

/// Extract atom name (PN) from fullname (PF).
fn extract_name_from_fullname(pf: &str) -> &str {
    // Regex adopted from
    // https://github.com/gentoo/portage-utils/blob/297e49f62a6520fce353b8e82f3bba430f1ea613/libq/atom.c#L277
    static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"-\d[.\d]*[a-z]?[^_-]*"#).unwrap());

    RE.find_iter(pf)
        .last() // Get the last component matching version syntax ...
        .map_or(pf, |m| &pf[..m.start()]) // ... and truncate str to it. If there's no version component, take the whole str.
}

struct Atom {
    fullname: OsString,
    ebuild_path: OsString,
}

/// Iterate over all atoms in vdb.
fn iter_atoms(vdb: impl AsRef<Path>, logger: &Logger) -> Result<Vec<Atom>, BoxedError> {
    Ok(fs::read_dir(vdb)? // iterate through category dir
        .filter_map(Result::ok)
        .map(|e| e.path())
        .flat_map(|category| {
            // iterate through atom dir
            fs::read_dir(&category)
                .tap_err(|e| {
                    logger.warn(format!(
                        "unable to iterate through category {:?}: {}",
                        category, e
                    ));
                })
                .into_iter()
                .flatten()
                .filter_map(Result::ok)
                .map(|e| {
                    let fullname = e.file_name();
                    // Guess ebuild filename from path.
                    let ebuild_path = {
                        let mut s = e.path().join(&fullname).into_os_string();
                        s.push(".ebuild");
                        s
                    };

                    Atom {
                        fullname,
                        ebuild_path,
                    }
                })
        })
        .collect())
}

/// Extract homepages from ebuild.
fn homepages(ebuild_path: impl AsRef<Path>) -> Result<Option<Vec<Url>>, io::Error> {
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
        Err(e) => Err(e),
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
