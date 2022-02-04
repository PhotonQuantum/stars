//! Github integration.

use std::thread::sleep;
use std::time::Duration;

use ureq::Agent;
use url::Url;

use crate::common::{BoxedError, Target};
use crate::Logger;

pub struct Github {
    client: Agent,
    token: Option<String>,
}

impl Default for Github {
    fn default() -> Self {
        Self {
            client: Agent::new(),
            token: None,
        }
    }
}

impl Target for Github {
    fn name(&self) -> &'static str {
        "github"
    }

    fn init(&mut self, logger: &Logger) -> bool {
        logger.pause_tick();
        let token = dialoguer::Password::new()
            .with_prompt("Please input your github token")
            .interact()
            .expect("write to term");
        self.token = Some(token);
        logger.resume_tick();
        true
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
