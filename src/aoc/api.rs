use std::sync::LazyLock;

use anyhow::anyhow;
use regex::Regex;
use reqwest::Client;

use super::models::{AocWhoami, PrivateLeaderboard};

static INVITE_CODE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<code>((\d+)-[\da-z]+)</code>").unwrap());

pub struct AocApiClient {
    http: Client,
}

impl AocApiClient {
    pub fn new(session: &str) -> anyhow::Result<Self> {
        Ok(Self {
            http: Client::builder()
                .default_headers(
                    [(
                        "Cookie".try_into()?,
                        format!("session={session}").try_into()?,
                    )]
                    .into_iter()
                    .collect(),
                )
                .build()?,
        })
    }

    pub async fn whoami(&self) -> anyhow::Result<AocWhoami> {
        let response = self
            .http
            .get("https://adventofcode.com/leaderboard/private")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;

        let cap = INVITE_CODE_REGEX
            .captures(&response)
            .ok_or_else(|| anyhow!("Failed to find invite code in response"))?;
        let invite_code = cap[1].to_owned();
        let user_id = cap[2].parse()?;

        Ok(AocWhoami {
            invite_code,
            user_id,
        })
    }

    pub async fn get_private_leaderboard(
        &self,
        year: i32,
        user_id: u64,
    ) -> reqwest::Result<PrivateLeaderboard> {
        self.http
            .get(format!(
                "https://adventofcode.com/{year}/leaderboard/private/view/{user_id}.json"
            ))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invite_code_regex() {
        let c = INVITE_CODE_REGEX
            .captures(
                "Others can join it using the code <code>123456-42ff1337</code>. Up to \
                 <code>200</code> users can join, including yourself.",
            )
            .unwrap();
        assert_eq!(&c[1], "123456-42ff1337");
        assert_eq!(&c[2], "123456");
    }
}
