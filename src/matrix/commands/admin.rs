use matrix_sdk::{
    ruma::{events::room::message::OriginalRoomMessageEvent, OwnedUserId},
    Room,
};

use super::{parser::ParsedCommand, send_error};
use crate::{
    config::Config,
    matrix::utils::{error_message, RoomExt},
};

pub async fn op(
    event: &OriginalRoomMessageEvent,
    room: Room,
    config: &Config,
    mut cmd: ParsedCommand<'_>,
) -> anyhow::Result<()> {
    if !config.matrix.admin_ids.contains(&event.sender) {
        room.reply_to(event, error_message("Permission denied"))
            .await?;
        return Ok(());
    }

    let member = match cmd
        .get_from_kwargs_or_args("member")
        .map(|x| x.parse::<OwnedUserId>().ok())
    {
        Some(Some(x)) => x,
        Some(None) => return send_error(&room, event, "Failed to parse argument 'member'").await,
        None => return send_error(&room, event, "Argument 'member' is required").await,
    };

    let level = match cmd.get_from_kwargs_or_args("level").map(|x| x.parse().ok()) {
        Some(Some(x)) => x,
        Some(None) => return send_error(&room, event, "Failed to parse argument 'level'").await,
        None => return send_error(&room, event, "Argument 'level' is required").await,
    };

    room.update_power_levels(vec![(&member, level)]).await?;

    Ok(())
}
