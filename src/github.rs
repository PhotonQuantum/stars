//! Github integration.

use attohttpc::header::AUTHORIZATION;
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
            logger.pause_tick();

            // Ask for credentials.
            let username: String = dialoguer::Input::new()
                .with_prompt("Please input your github username")
                .interact()
                .expect("write to term");
            let token: String = dialoguer::Password::new()
                .with_prompt("Please input your github token")
                .interact()
                .expect("write to term");
            let cred = format!("{}:{}", username, token);

            // Save credentials.
            persist.with_state(|state| {
                state.insert(String::from("github_credential"), cred.clone().into());
            });

            // Resume progressbar ticking.
            logger.resume_tick();
            cred
        };

        self.credential = Some(credential);
        true
    }

    fn try_handle(&self, url: &Url) -> Option<String> {
        let domain = url.domain()?;
        let segments = url.path_segments()?;
        if domain != "github.com" {
            return None;
        }

        let (user, repo) = segments.take(2).filter(|s| !s.is_empty()).collect_tuple()?;
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
