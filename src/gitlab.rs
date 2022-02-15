//! Gitlab integration.

use attohttpc::header::AUTHORIZATION;
use attohttpc::StatusCode;
use console::style;
use itertools::Itertools;
use url::Url;

use crate::common::{BoxedError, Package, Target, HTTP};
use crate::{Logger, Persist};

#[derive(Default)]
pub struct Gitlab {
    access_token: Option<String>,
}

impl Target for Gitlab {
    fn name(&self) -> &'static str {
        "gitlab"
    }

    fn init(&mut self, logger: &Logger, persist: &mut Persist) -> bool {
        #[allow(clippy::option_if_let_else)] // borrow ck fails
        // Check for saved token.
        let token = if let Some(token) =
            persist.get_state(|state| state.get("gitlab_token").cloned())
        {
            token.as_str().unwrap().to_string()
        } else {
            // No saved token. Ask for one.

            // Pause progressbar ticking for user input.
            logger.pause_progress_bar();

            // Ask for token.
            logger.info("Please enter your GitLab token.");
            logger
                .info("Acquire a personal access token at https://gitlab.com/-/profile/personal_access_tokens.");
            logger.info("`api` and `read_api` scopes are required.");
            logger.warn("Beware that star actions will be publicly visible.");
            logger.warn("To avoid polluting your timeline, consider using a dedicated account.");

            let token: String = dialoguer::Password::new()
                .with_prompt(format!(
                    "{} Please input your GitLab token",
                    style("?").cyan()
                ))
                .interact()
                .expect("write to term");

            // Save token.
            persist.with_state(|state| {
                state.insert(String::from("gitlab_token"), token.clone().into());
            });

            // Resume progressbar ticking.
            logger.resume_progress_bar();
            token
        };

        self.access_token = Some(token);
        true
    }

    fn try_handle(&self, url: &Url) -> Option<String> {
        let domain = url.domain()?;
        let segments = url.path_segments()?;
        if domain != "gitlab.com" && domain != "www.gitlab.com" {
            return None;
        }

        let (user, repo) = segments.take(2).filter(|s| !s.is_empty()).collect_tuple()?;
        let repo = repo.trim_end_matches(".git");
        let encoded = urlencoding::encode(&format!("{}/{}", user, repo)).to_string();
        Some(encoded)
    }

    fn star(&self, logger: &Logger, package: &Package) -> Result<(), BoxedError> {
        let resp = HTTP
            .post(
                format!(
                    "https://gitlab.com/api/v4/projects/{}/star",
                    package.identifier
                )
                .as_str(),
            )
            .header(
                AUTHORIZATION,
                format!("Bearer {}", self.access_token.clone().unwrap()).as_str(),
            )
            .send()?;

        if !resp.status().is_success() && resp.status() != StatusCode::NOT_MODIFIED {
            logger.warn(format!(
                "Non-2xx/304 response for {}: {} {}",
                package,
                resp.status(),
                resp.text().unwrap_or_default()
            ));
        }

        Ok(())
    }
}
