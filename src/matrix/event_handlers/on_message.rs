use std::sync::Arc;

use matrix_sdk::{
    event_handler::Ctx,
    ruma::{
        api::client::receipt::create_receipt::v3::ReceiptType,
        events::{
            receipt::ReceiptThread,
            room::message::{MessageType, OriginalSyncRoomMessageEvent},
        },
    },
    Client, Room, RoomState,
};

use crate::{context::Context, matrix::commands};

pub async fn handle(
    event: OriginalSyncRoomMessageEvent,
    client: Client,
    room: Room,
    context: Ctx<Arc<Context>>,
) -> anyhow::Result<()> {
    let event = event.into_full_event(room.room_id().to_owned());
    if room.state() != RoomState::Joined || event.sender == client.user_id().unwrap() {
        return Ok(());
    }

    tokio::spawn({
        let room = room.clone();
        let event_id = event.event_id.clone();
        async move {
            room.send_single_receipt(ReceiptType::Read, ReceiptThread::Unthreaded, event_id)
                .await
        }
    });

    let MessageType::Text(content) = &event.content.msgtype else {
        return Ok(());
    };

    if content
        .body
        .trim()
        .starts_with(client.user_id().unwrap().as_str())
        || content.formatted.as_ref().is_some_and(|f| {
            f.body.trim().starts_with(&format!(
                r#"<a href="{}">"#,
                client.user_id().unwrap().matrix_to_uri(),
            ))
        })
    {
        commands::help(&event, room, &context.config).await?;
        return Ok(());
    }

    let Some(command) = content
        .body
        .strip_prefix(&context.config.matrix.command_prefix)
    else {
        return Ok(());
    };

    let mut parts = command.split_whitespace();
    let Some(cmd) = parts.next() else {
        return Ok(());
    };

    commands::handle(&event, room, context.0, cmd, parts).await
}
