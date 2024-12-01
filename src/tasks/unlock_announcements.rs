use std::sync::Arc;

use matrix_sdk::RoomState;
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
        if let Err(err) = trigger(&context, next).await {
            error!("Failed to send unlock announcement: {err}");
        }
    }
}

async fn trigger(context: &Context, day: AocDay) -> anyhow::Result<()> {
    let room = &context.room;
    if room.state() != RoomState::Joined {
        warn!("not a member of target room {}", room.room_id());
        room.join().await?;
    }

    let url = day.url();
    let AocDay { year, day } = day;
    let link_prefix = &context.config.matrix.link_prefix;
    room.send(message(format!(
        "✨ The puzzles of **Advent of Code {year} Day {day}** can now be solved at \
         [{url}]({link_prefix}{url}) ✨ <!-- 🎉 -->",
    )))
    .await?;
    Ok(())
}
