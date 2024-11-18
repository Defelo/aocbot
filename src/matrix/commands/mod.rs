use std::sync::Arc;

use matrix_sdk::{ruma::events::room::message::OriginalRoomMessageEvent, Room};

use crate::{
    aoc::day::AocDay,
    config::Config,
    matrix::utils::{error_message, message, RoomExt},
    Context,
};

pub mod admin;
pub mod aoc;

pub async fn handle(
    event: &OriginalRoomMessageEvent,
    room: Room,
    context: Arc<Context>,
    cmd: &str,
    args: impl Iterator<Item = &str>,
) -> anyhow::Result<()> {
    match cmd {
        // Advent of Code
        "join" => aoc::join::invoke(event, room, &context.aoc_client).await,
        "leaderboard" | "lb" => aoc::leaderboard::invoke(event, room, &context, args).await,
        "user" => aoc::user::invoke(event, room, &context).await,
        "solutions" | "repos" => aoc::solutions::invoke(event, room, &context).await,

        // General
        "ping" => ping(event, room).await,
        "help" => help(event, room, &context.config).await,

        // Administration
        "op" => admin::op(event, room, &context.config, args).await,

        _ => unknown_command(event, room).await,
    }
}

pub async fn help(
    event: &OriginalRoomMessageEvent,
    room: Room,
    config: &Config,
) -> anyhow::Result<()> {
    let prefix = &config.matrix.command_prefix;

    let default_year = AocDay::most_recent().year;
    let default_rows = config.aoc.leaderboard_rows;
    let content = format!(
        r#"
### AoC-Bot Commands

#### Advent of Code
- `{prefix}join` - Request instructions to join the private leaderboard
- `{prefix}leaderboard [--refresh] [year={default_year}] [rows={default_rows}] [offset=0]` - Show the given slice of the private leaderboard
- `{prefix}user [user] [year={default_year}]` - Show statistics of the given user
- `{prefix}solutions` - Show the list of solution repositories

#### General
- `{prefix}ping` - Check bot health
- `{prefix}help` - Show this help message
"#
    );

    room.reply_to(event, message(content)).await?;
    Ok(())
}

async fn unknown_command(event: &OriginalRoomMessageEvent, room: Room) -> anyhow::Result<()> {
    room.reply_to(
        event,
        error_message("Unknown command. Send `!help` for a list of available commands."),
    )
    .await?;
    Ok(())
}

pub async fn ping(event: &OriginalRoomMessageEvent, room: Room) -> anyhow::Result<()> {
    room.reply_to(event, message("Pong!")).await?;
    Ok(())
}
