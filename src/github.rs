//! Github integration.

use std::thread::sleep;
use std::time::Duration;

use url::Url;

use crate::common::{BoxedError, Stargazer};
use crate::Logger;

pub struct Github;

impl Stargazer for Github {
    fn name(&self) -> &'static str {
        "github"
    }

    fn can_handle(&self, url: &Url) -> bool {
        url.domain().map_or(false, |domain| {
            domain == "github.com" || domain == "www.github.com"
        })
    }

    fn star(&self, _logger: &Logger, _package: &Url) -> Result<(), BoxedError> {
        // TODO
        sleep(Duration::from_millis(100));
        Ok(())
    }
}
