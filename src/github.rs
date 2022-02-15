//! Github integration.

use attohttpc::header::AUTHORIZATION;
use console::style;
use itertools::Itertools;
use url::Url;

use crate::common::{BoxedError, Package, Target, HTTP};
use crate::{Logger, Persist};

#[derive(Default)]
pub struct Github {
    credential: Option<String>,
}

impl Target for Github {
    fn name(&self) -> &'static str {
        "github"
    }

    fn init(&mut self, logger: &Logger, persist: &mut Persist) -> bool {
        #[allow(clippy::option_if_let_else)] // borrow ck fails
        // Check for saved credentials.
        let credential = if let Some(token) =
            persist.get_state(|state| state.get("github_credential").cloned())
        {
            token.as_str().unwrap().to_string()
        } else {
            // No saved credentials. Ask for one.

            // Pause progressbar ticking for user input.
            logger.pause_progress_bar();

            // Ask for credentials.
            logger.info("Please enter your GitHub credentials.");
            logger
                .info("Acquire a personal access token at https://github.com/settings/tokens/new.");
            logger.info("`public_repo` scope is required.");
            logger.warn("Beware that star actions will be publicly visible.");
            logger.warn("To avoid polluting your timeline, consider using a dedicated account.");

            let username: String = dialoguer::Input::new()
                .with_prompt(format!(
                    "{} Please input your GitHub username",
                    style("?").cyan()
                ))
                .interact()
                .expect("write to term");
            let token: String = dialoguer::Password::new()
                .with_prompt(format!(
                    "{} Please input your GitHub token",
                    style("?").cyan()
                ))
                .interact()
                .expect("write to term");
            let cred = format!("{}:{}", username, token);

            // Save credentials.
            persist.with_state(|state| {
                state.insert(String::from("github_credential"), cred.clone().into());
            });

            // Resume progressbar ticking.
            logger.resume_progress_bar();
            cred
        };

        self.credential = Some(credential);
        true
    }

    fn try_handle(&self, url: &Url) -> Option<String> {
        let domain = url.domain()?;
        let segments = url.path_segments()?;
        if domain != "github.com" && domain != "www.github.com" {
            return None;
        }

        let (user, repo) = segments.take(2).filter(|s| !s.is_empty()).collect_tuple()?;
        let repo = repo.trim_end_matches(".git");
        Some(format!("{}/{}", user, repo))
    }

    fn star(&self, logger: &Logger, package: &Package) -> Result<(), BoxedError> {
        let resp = HTTP
            .put(format!("https://api.github.com/user/starred/{}", package.identifier).as_str())
            .header(
                AUTHORIZATION,
                format!("Basic {}", base64::encode(self.credential.clone().unwrap())).as_str(),
            )
            .send()?;

        if !resp.status().is_success() {
            logger.warn(format!(
                "Non-2xx response for {}: {} {}",
                package,
                resp.status(),
                resp.text().unwrap_or_default()
            ));
        }

        Ok(())
    }
}
