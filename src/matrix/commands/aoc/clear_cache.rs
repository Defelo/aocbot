use matrix_sdk::{
    ruma::events::{
        reaction::ReactionEventContent, relation::Annotation,
        room::message::OriginalRoomMessageEvent,
    },
    Room,
};

use crate::{context::Context, matrix::commands::send_error};

pub async fn invoke(
    event: &OriginalRoomMessageEvent,
    room: Room,
    context: &Context,
) -> anyhow::Result<()> {
    if !context.config.matrix.admin_ids.contains(&event.sender) {
        return send_error(&room, event, "Permission denied").await;
    }

    context.aoc_client.clear_leaderboard_cache().await?;

    room.send(ReactionEventContent::new(Annotation::new(
        event.event_id.clone(),
        "✅️".into(),
    )))
    .await?;

    Ok(())
}
