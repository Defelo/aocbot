use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use matrix_sdk::{
    ruma::events::room::message::{
        FormattedBody, ImageMessageEventContent, MessageType, RoomMessageEventContent,
    },
    RoomState,
};
use mime_guess::MimeGuess;
use regex::Regex;
use tracing::{error, trace, warn};

use crate::{
    mastodon::{self, AttachmentType},
    utils::datetime::now,
    Context,
};

const LAST_ID_STORE_KEY: &[u8] = b"garygrady_last_id";

static CONTENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"((?i)advent\s*of\s*code)|AoC").unwrap());

pub async fn start(context: Arc<Context>) -> ! {
    let mut last_id = mastodon::Id(
        context
            .store
            .get::<u64>(LAST_ID_STORE_KEY)
            .await
            .ok()
            .flatten()
            .unwrap_or(0),
    );

    loop {
        if let Err(err) = trigger(&context, &mut last_id).await {
            error!("Failed to check for member join/leave events: {err}");
        }
        tokio::time::sleep(Duration::from_secs(context.config.garygrady.interval)).await;
    }
}

async fn trigger(context: &Context, last_id: &mut mastodon::Id) -> anyhow::Result<()> {
    let room = &context.room;
    if room.state() != RoomState::Joined {
        warn!("not a member of target room {}", room.room_id());
        room.join().await?;
    }

    trace!("checking for new garygrady posts");

    let now = now();
    let max_age = Duration::from_secs(context.config.garygrady.max_age);
    let not_before = now - max_age;

    loop {
        let posts = mastodon::fetch_original_media_posts(
            &context.garygrady.server,
            context.garygrady.user_id,
            *last_id,
            20,
        )
        .await?;
        if posts.is_empty() {
            break;
        }

        for post in posts.into_iter().rev() {
            *last_id = post.id.max(*last_id);

            if !CONTENT_REGEX.is_match(&post.content) || post.created_at < not_before {
                continue;
            }

            for media in post.media_attachments {
                let AttachmentType::Image = media.r#type else {
                    continue;
                };

                let Some(mime) = MimeGuess::from_path(media.url.path()).first() else {
                    warn!(
                        "Failed to determine mime type of media attachment at {}",
                        media.url
                    );
                    continue;
                };

                let filename = media.url.path().rsplit('/').next().unwrap().to_owned();

                let image = reqwest::Client::new()
                    .get(media.url)
                    .send()
                    .await?
                    .error_for_status()?
                    .bytes()
                    .await?;

                let response = room
                    .client()
                    .media()
                    .upload(&mime, image.to_vec(), None)
                    .await?;

                let caption = format!(
                    r#"<a href="{}">{}</a> (created by <a href="{}">@{}</a>)"#,
                    post.url,
                    remove_html_tags(&post.content),
                    post.account.url,
                    post.account.username
                );

                let mut image_message =
                    ImageMessageEventContent::plain(caption.clone(), response.content_uri);
                image_message.filename = Some(filename);
                image_message.formatted = Some(FormattedBody::html(caption));

                room.send(RoomMessageEventContent::new(MessageType::Image(
                    image_message,
                )))
                .await?;
            }

            context
                .store
                .set::<u64>(LAST_ID_STORE_KEY, &last_id.0)
                .await?;
        }
    }

    Ok(())
}

fn remove_html_tags(html: &str) -> String {
    static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"<.*?>"#).unwrap());
    REGEX.replace_all(html, "").into()
}
