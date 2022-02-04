//! Github integration.

use std::thread::sleep;
use std::time::Duration;

use ureq::Agent;
use url::Url;

use crate::common::{BoxedError, Target};
use crate::{Logger, Persist};

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

    fn init(&mut self, logger: &Logger, persist: &mut Persist) -> bool {
        let token =
            if let Some(token) = persist.get_state(|state| state.get("github_token").cloned()) {
                token.as_str().unwrap().to_string()
            } else {
                logger.pause_tick();

                let token = dialoguer::Password::new()
                    .with_prompt("Please input your github token")
                    .interact()
                    .expect("write to term");

                persist.with_state(|state| {
                    state.insert(String::from("github_token"), token.clone().into());
                });

                logger.resume_tick();
                token
            };

        self.token = Some(token);
        true
    }

    fn can_handle(&self, url: &Url) -> bool {
        url.domain().map_or(false, |domain| {
            domain == "github.com" || domain == "www.github.com"
        })
    }

    fn star(
        &self,
        _logger: &Logger,
        _persist: &mut Persist,
        _package: &Url,
    ) -> Result<(), BoxedError> {
        // TODO
        sleep(Duration::from_millis(100));
        Ok(())
    }
}
