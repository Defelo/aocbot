use matrix_sdk::{ruma::events::room::message::OriginalRoomMessageEvent, Room};

use crate::{
    aoc::day::AocDay,
    context::Context,
    matrix::utils::{message, RoomExt},
};

pub async fn invoke(
    event: &OriginalRoomMessageEvent,
    room: Room,
    context: &Context,
) -> anyhow::Result<()> {
    let members = context
        .aoc_client
        .get_private_leaderboard(AocDay::most_recent().year)
        .await?
        .0
        .members
        .len();

    let link_prefix = &context.config.matrix.link_prefix;

    let content = format!(
        r#"
### How to Join the Private Leaderboard

1. Log in at [https://adventofcode.com/]({link_prefix}https://adventofcode.com/)
2. Go to [https://adventofcode.com/leaderboard/private]({link_prefix}https://adventofcode.com/leaderboard/private)
3. Enter the code `{}`. Last time I checked, the leaderboard had {} of 200 members.
4. (optional) Add your matrix username and/or solution repository to [`users.toml`]({link_prefix}https://github.com/Defelo/aocbot/blob/develop/users.toml).

Good Luck, Have Fun! 🎄 🎁
"#,
        context.aoc_client.whoami().invite_code,
        members,
    );

    room.reply_to(event, message(content)).await?;

    Ok(())
}
