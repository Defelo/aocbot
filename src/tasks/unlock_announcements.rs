use std::sync::Arc;

use matrix_sdk::{Room, RoomState};
use tracing::{error, info, warn};

use crate::{
    aoc::day::AocDay, context::Context, matrix::utils::message, utils::datetime::sleep_until,
};

pub async fn start(context: Arc<Context>) -> ! {
    loop {
        let next = AocDay::next();
        let datetime = next.unlock_datetime();
        info!(?next, ?datetime, "waiting until next unlock");
        sleep_until(datetime).await;
        info!(?next, ?datetime, "new puzzles unlocked");
        if let Err(err) = trigger(&context.room, next).await {
            error!("Failed to send unlock announcement: {err}");
        }
    }
}

async fn trigger(room: &Room, day: AocDay) -> anyhow::Result<()> {
    if room.state() != RoomState::Joined {
        warn!("not a member of target room {}", room.room_id());
        room.join().await?;
    }

    let url = day.url();
    let AocDay { year, day } = day;
    room.send(message(format!(
        "✨ The puzzles of **Advent of Code {year} Day {day}** can now be solved at {url} ✨ <!-- \
         🎉 -->",
    )))
    .await?;
    Ok(())
}
