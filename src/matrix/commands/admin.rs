use matrix_sdk::{
    ruma::{events::room::message::OriginalRoomMessageEvent, OwnedUserId},
    Room,
};

use crate::{
    config::Config,
    matrix::utils::{error_message, RoomExt},
};

pub async fn op(
    event: &OriginalRoomMessageEvent,
    room: Room,
    config: &Config,
    mut args: impl Iterator<Item = &str>,
) -> anyhow::Result<()> {
    if !config.matrix.admin_ids.contains(&event.sender) {
        room.reply_to(event, error_message("Permission denied"))
            .await?;
        return Ok(());
    }

    let Some((member, level)) = (|| {
        Some((
            args.next()?.parse::<OwnedUserId>().ok()?,
            args.next()?.parse().ok()?,
        ))
    })() else {
        room.reply_to(event, error_message("Invalid args")).await?;
        return Ok(());
    };

    room.update_power_levels(vec![(&member, level)]).await?;

    Ok(())
}
