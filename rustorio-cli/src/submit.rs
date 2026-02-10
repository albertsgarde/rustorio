use anyhow::{Context, Result};
use clap::Args;
use reqwest::blocking::Client;
use rustorio_common::SubmitRunRequest;

use crate::ProjectInfo;

#[derive(Args)]
pub struct SubmitArgs {
    #[clap()]
    ticks: u64,
}

impl SubmitArgs {
    pub fn run(&self) -> Result<()> {
        let ProjectInfo {root_path, mut config} = ProjectInfo::get().context("Failed to get project project info")?
                .context("Can only run command in a Rustorio project. Please either navigate to a Rustorio project or run 'rustorio setup' first.")?;

        let username = if let Some(username) = &config.username {
            username
        } else {
            let username = dialoguer::Input::new()
                .with_prompt("What name would you like to submit under?")
                .interact_text()
                .context("Failed to get username from user")?;
            config.username = Some(username);
            if let Err(error) = config.save(root_path.as_path()) {
                eprintln!("Failed to save config due to error: {error:#}");
            } else {
                let username = config.username.as_ref().unwrap();
                println!(
                    "Username '{username}' saved to config at `rustorio.toml`. You can edit it there at any time."
                );
            }
            config.username.as_ref().unwrap()
        };

        let SubmitArgs { ticks } = self;
        let request = SubmitRunRequest {
            name: username.to_string(),
            tick_count: *ticks,
        };

        let url = format!(
            "{url}{}/runs",
            rustorio_common::BASE_API_PATH,
            url = config.rustorio_url,
        );
        let response = Client::new()
            .post(&url)
            .json(&request)
            .send()
            .with_context(|| format!("Failed to send request to url `{url}`."))?;

        response
            .error_for_status()
            .with_context(|| format!("Request to url `{url}` failed"))?;

        Ok(())
    }
}
