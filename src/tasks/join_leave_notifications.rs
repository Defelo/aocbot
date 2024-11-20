use std::{sync::Arc, time::Duration};

use matrix_sdk::{Room, RoomState};
use tracing::{error, info, trace, warn};

use crate::{
    aoc::{
        day::AocDay,
        models::{PrivateLeaderboard, PrivateLeaderboardMembers},
    },
    matrix::utils::notice,
    Context,
};

pub async fn start(context: Arc<Context>) -> ! {
    let year = AocDay::most_recent().year;
    let mut leaderboard = context
        .aoc_client
        .get_private_leaderboard_cached(year)
        .await
        .map(|(lb, _)| lb);

    loop {
        if let Err(err) = trigger(&context, &mut leaderboard).await {
            error!("Failed to check for member join/leave events: {err}");
        }
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

async fn trigger(
    context: &Context,
    leaderboard: &mut Option<PrivateLeaderboard>,
) -> anyhow::Result<()> {
    let room = &context.room;
    if room.state() != RoomState::Joined {
        warn!("not a member of target room {}", room.room_id());
        room.join().await?;
    }

    let year = AocDay::most_recent().year;

    trace!("checking for member leave/join events");

    let new_leaderboard = context.aoc_client.get_private_leaderboard(year).await?.0;

    send_notifications(
        room,
        context,
        leaderboard
            .as_ref()
            .map(|l| &l.members)
            .unwrap_or(&Default::default()),
        &new_leaderboard.members,
    )
    .await?;

    *leaderboard = Some(new_leaderboard);

    Ok(())
}

async fn send_notifications(
    room: &Room,
    context: &Context,
    old_leaderboard: &PrivateLeaderboardMembers,
    new_leaderboard: &PrivateLeaderboardMembers,
) -> anyhow::Result<()> {
    if old_leaderboard.is_empty() {
        info!(
            members = new_leaderboard.len(),
            "ignoring first leaderboard membership update"
        );
        return Ok(());
    }

    let mut notifications = Vec::new();

    for (id, member) in new_leaderboard {
        if !old_leaderboard.contains_key(id) {
            notifications.push((member, true));
        }
    }

    for (id, member) in old_leaderboard {
        if !new_leaderboard.contains_key(id) {
            notifications.push((member, false));
        }
    }

    trace!(
        ?notifications,
        "sending leaderboard join/leave notifications"
    );

    for (member, joined) in notifications {
        let matrix = context
            .users
            .by_aoc
            .get(&member.id)
            .and_then(|m| m.matrix.as_deref());

        let name = member.matrix_mention_or_display_name(matrix);
        let action = if joined { "joined" } else { "left" };

        room.send(notice(format!(
            "{name} has {action} the private leaderboard"
        )))
        .await?;
    }

    Ok(())
}
