//! Github integration.

use itertools::Itertools;
use url::Url;

use crate::common::{BoxedError, Target, HTTP};
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
        let credential = if let Some(token) =
            persist.get_state(|state| state.get("github_credential").cloned())
        {
            token.as_str().unwrap().to_string()
        } else {
            logger.pause_tick();

            let username: String = dialoguer::Input::new()
                .with_prompt("Please input your github username")
                .interact()
                .expect("write to term");
            let token: String = dialoguer::Password::new()
                .with_prompt("Please input your github token")
                .interact()
                .expect("write to term");
            let cred = format!("{}:{}", username, token);

            persist.with_state(|state| {
                state.insert(String::from("github_credential"), cred.clone().into());
            });

            logger.resume_tick();
            cred
        };

        self.credential = Some(credential);
        true
    }

    fn can_handle(&self, url: &Url) -> bool {
        url.domain().map_or(false, |domain| {
            domain == "github.com" || domain == "www.github.com"
        })
    }

    fn star(&self, logger: &Logger, _persist: &mut Persist, url: &Url) -> Result<(), BoxedError> {
        let (user, repo): (&str, &str) = url
            .path_segments()
            .ok_or("not valid github repo")?
            .take(2)
            .collect_tuple()
            .ok_or("not valid github repo")?;

        let resp = HTTP
            .put(format!("https://api.github.com/user/starred/{}/{}", user, repo).as_str())
            .set(
                "Authorization",
                format!("Basic {}", base64::encode(self.credential.clone().unwrap())).as_str(),
            )
            .call()?;

        if resp.status() < 200 || resp.status() >= 300 {
            #[allow(clippy::to_string_in_format_args)] // borrow ck fails
            logger.warn(format!(
                "Non-2xx response for {}: {} {}",
                url,
                resp.status_text().to_string(),
                resp.into_string().unwrap_or_default()
            ));
        }

        Ok(())
    }
}
