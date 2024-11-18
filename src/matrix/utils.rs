use std::future::Future;

use matrix_sdk::{
    ruma::{
        api::client::message::send_message_event,
        events::room::message::{OriginalRoomMessageEvent, Relation, RoomMessageEventContent},
    },
    Room,
};

pub trait RoomExt {
    fn reply_to(
        &self,
        event: &OriginalRoomMessageEvent,
        content: RoomMessageEventContent,
    ) -> impl Future<Output = Result<send_message_event::v3::Response, matrix_sdk::Error>> + Send;
}

impl RoomExt for Room {
    async fn reply_to(
        &self,
        event: &OriginalRoomMessageEvent,
        mut content: RoomMessageEventContent,
    ) -> Result<send_message_event::v3::Response, matrix_sdk::Error> {
        content.relates_to = event
            .content
            .relates_to
            .as_ref()
            .filter(|rl| matches!(rl, Relation::Thread(_)))
            .cloned();
        self.send(content).await
    }
}

pub fn message(text: impl AsRef<str> + Into<String>) -> RoomMessageEventContent {
    RoomMessageEventContent::text_markdown(text)
}

pub fn notice(text: impl AsRef<str> + Into<String>) -> RoomMessageEventContent {
    RoomMessageEventContent::notice_markdown(text)
}

pub fn html_message(html: impl Into<String> + Clone) -> RoomMessageEventContent {
    RoomMessageEventContent::text_html(html.clone(), html)
}

pub fn error_message(text: impl AsRef<str>) -> RoomMessageEventContent {
    message(format!("❌ Error: {}", text.as_ref()))
}
