use chrono::{DateTime, Utc};
use reqwest::Url;
use serde::Deserialize;
use tracing::trace;

use crate::utils;

pub async fn lookup_account(base_url: &Url, name: &str) -> anyhow::Result<Account> {
    trace!("fetching account info for {name} on {base_url}");
    // https://docs.joinmastodon.org/methods/accounts/#lookup
    reqwest::Client::new()
        .get(base_url.join("api/v1/accounts/lookup")?)
        .query(&[("acct", name)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
        .map_err(Into::into)
}

pub async fn fetch_original_media_posts(
    base_url: &Url,
    Id(account_id): Id,
    Id(min_id): Id,
    limit: u16,
) -> anyhow::Result<Vec<Post>> {
    trace!("fetching posts of user {account_id} on {base_url}");
    // https://docs.joinmastodon.org/methods/accounts/#statuses
    reqwest::Client::new()
        .get(base_url.join(&format!("api/v1/accounts/{account_id}/statuses"))?)
        .query(&[("min_id", min_id)])
        .query(&[("limit", limit)])
        .query(&[("only_media", true)])
        .query(&[("exclude_replies", true)])
        .query(&[("exclude_reblogs", true)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
        .map_err(Into::into)
}

#[derive(Deserialize, Debug)]
pub struct Post {
    pub id: Id,
    pub url: Url,
    pub created_at: DateTime<Utc>,
    pub content: String,
    pub media_attachments: Vec<MediaAttachment>,
    pub account: Account,
}

#[derive(Deserialize, Debug)]
pub struct Account {
    pub id: Id,
    pub username: String,
    pub url: Url,
}

#[derive(Deserialize, Debug)]
pub struct MediaAttachment {
    pub r#type: AttachmentType,
    pub url: Url,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AttachmentType {
    Image,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct Id(#[serde(with = "utils::serde::via_string")] pub u64);
