use anyhow::{Context, Result};
use clap::Args;
use reqwest::blocking::Client;
use rustorio_common::SubmitRunRequest;

use crate::project::ProjectInfo;

#[derive(Args)]
pub struct SubmitArgs {
    /// Name of the save game to submit.
    /// This should be the name of a folder in `src/bin` that contains a Rustorio save game.
    /// For example, if you have a save game in `src/bin/tutorial`, you can submit it with `rustorio submit tutorial`.
    #[clap()]
    save_name: String,
}

impl SubmitArgs {
    pub fn run(&self) -> Result<()> {
        let mut project_info = ProjectInfo::get().context("Failed to get project project info")?
                .context("Can only run command in a Rustorio project. Please either navigate to a Rustorio project or run 'rustorio setup' first.")?;

        let SubmitArgs { save_name } = self;

        let play_output = project_info.play(save_name).with_context(|| {
            format!("Failed to get play output for save with name '{save_name}'")
        })?;

        let ProjectInfo { root_path, config } = &mut project_info;

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
            config
                .username
                .as_ref()
                .expect("config username was just filled")
        };

        let request = SubmitRunRequest {
            name: username.to_string(),
            gamemode: play_output.gamemode,
            tick_count: play_output.ticks,
        };

        let url = format!(
            "{url}{}/runs",
            rustorio_common::BASE_API_PATH,
            url = project_info.config.rustorio_url,
        );

        let response = Client::new()
            .post(&url)
            .json(&request)
            .send()
            .with_context(|| format!("Failed to send request to url `{url}`."))?;

        response
            .error_for_status()
            .with_context(|| format!("Request to url `{url}` failed"))?;

        println!("Run submitted to leaderboard.");

        Ok(())
    }
}
