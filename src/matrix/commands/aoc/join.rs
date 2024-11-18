use matrix_sdk::{ruma::events::room::message::OriginalRoomMessageEvent, Room};

use crate::{
    aoc::{client::AocClient, day::AocDay},
    matrix::utils::{message, RoomExt},
};

pub async fn invoke(
    event: &OriginalRoomMessageEvent,
    room: Room,
    aoc_client: &AocClient,
) -> anyhow::Result<()> {
    let members = aoc_client
        .get_private_leaderboard(AocDay::most_recent().year, false)
        .await?
        .0
        .members
        .len();

    let content = format!(
        r#"
### How to Join the Private Leaderboard

1. Log in at https://adventofcode.com/
2. Go to https://adventofcode.com/leaderboard/private
3. Enter the code `{}`. Last time I checked, the leaderboard had {} of 200 members.
4. (optional) Add your matrix username and/or solution repository to [`users.toml`](https://github.com/Defelo/aocbot/blob/develop/users.toml).

Good Luck, Have Fun! 🎄 🎁
"#,
        aoc_client.whoami().invite_code,
        members,
    );

    room.reply_to(event, message(content)).await?;

    Ok(())
}
