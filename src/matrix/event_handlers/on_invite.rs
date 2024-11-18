use std::sync::Arc;

use matrix_sdk::{
    event_handler::Ctx, ruma::events::room::member::StrippedRoomMemberEvent, Client, Room,
};
use tracing::{error, info};

use crate::context::Context;

pub async fn handle(
    event: StrippedRoomMemberEvent,
    client: Client,
    room: Room,
    context: Ctx<Arc<Context>>,
) -> anyhow::Result<()> {
    if event.state_key != client.user_id().unwrap() {
        return Ok(());
    }

    if room.room_id() != context.config.matrix.room_id
        && !context.config.matrix.admin_ids.contains(&event.sender)
        && !room.is_direct().await?
    {
        info!(room_id=%room.room_id(), sender=%event.sender, "Rejecting invitation to non-dm room");
        room.leave().await?;
        return Ok(());
    }

    info!(room_id=%room.room_id(), sender=%event.sender, "Trying to join room");
    while let Err(err) = room.join().await {
        error!("Failed to join room {}: {:?}", room.room_id(), err);
    }
    info!("Joined room {} successfully", room.room_id());

    Ok(())
}
