use std::{sync::Arc, time::Duration};

use chrono::{DateTime, TimeZone, Utc};
use matrix_sdk::{Room, RoomState};
use tracing::{error, trace, warn};

use crate::{
    aoc::{
        day::AocDay,
        models::{PrivateLeaderboard, PrivateLeaderboardMember, PrivateLeaderboardMembers},
    },
    matrix::utils::notice,
    utils::{
        datetime::{now, DateTimeExt},
        fmt::fmt_rank,
    },
    Context,
};

pub async fn start(context: Arc<Context>) -> ! {
    let mut year = AocDay::most_recent().year;
    let mut leaderboard = context
        .aoc_client
        .get_private_leaderboard_cached(year)
        .await
        .map(|(lb, _)| lb);

    loop {
        if let Err(err) = trigger(&context, &mut year, &mut leaderboard).await {
            error!("Failed to check for new puzzle solves: {err}");
        }
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

async fn trigger(
    context: &Context,
    year: &mut i32,
    leaderboard: &mut Option<PrivateLeaderboard>,
) -> anyhow::Result<()> {
    let room = &context.room;
    if room.state() != RoomState::Joined {
        warn!("not a member of target room {}", room.room_id());
        room.join().await?;
    }

    let current_year = AocDay::most_recent().year;
    if current_year != *year {
        *leaderboard = None;
        *year = current_year;
    }

    trace!(year, "checking for new puzzle solves");

    let new_leaderboard = context.aoc_client.get_private_leaderboard(*year).await?.0;

    send_notifications(
        room,
        context,
        *year,
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
    year: i32,
    old_leaderboard: &PrivateLeaderboardMembers,
    new_leaderboard: &PrivateLeaderboardMembers,
) -> anyhow::Result<()> {
    let mut notifications = Vec::new();
    for (id, member) in new_leaderboard {
        let Some(old_member) = old_leaderboard.get(id) else {
            continue;
        };

        for (&day, completion) in &member.completion_day_level {
            let old_completion = old_member.completion_day_level.get(&day);

            if old_completion.is_none() {
                let rank = new_leaderboard
                    .values()
                    .filter(|m| {
                        m.completion_day_level
                            .get(&day)
                            .is_some_and(|c| c.fst.get_star_ts <= completion.fst.get_star_ts)
                    })
                    .count();
                notifications.push(Notification {
                    member,
                    part2: false,
                    day: AocDay { year, day },
                    ts: completion.fst.get_star_ts,
                    rank,
                });
            }

            if let Some(part2) = completion
                .snd
                .as_ref()
                .filter(|_| old_completion.is_none_or(|oc| oc.snd.is_none()))
            {
                let rank = new_leaderboard
                    .values()
                    .filter(|m| {
                        m.completion_day_level
                            .get(&day)
                            .and_then(|c| c.snd.as_ref())
                            .is_some_and(|c| c.get_star_ts <= part2.get_star_ts)
                    })
                    .count();
                notifications.push(Notification {
                    member,
                    part2: true,
                    day: AocDay { year, day },
                    ts: part2.get_star_ts,
                    rank,
                });
            }
        }
    }

    let now = now();
    notifications.retain(|n| now <= n.ts + Duration::from_secs(24 * 3600));
    notifications.sort_unstable_by_key(|n| n.ts);

    trace!(?notifications, "sending puzzle solve notifications");
    for notification in notifications {
        room.send(notice(notification.to_string(context))).await?;
    }

    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct Notification<'a> {
    member: &'a PrivateLeaderboardMember,
    part2: bool,
    day: AocDay,
    ts: DateTime<Utc>,
    rank: usize,
}

impl Notification<'_> {
    fn to_string(self, context: &Context) -> String {
        let Self {
            member,
            part2,
            day,
            ts,
            rank,
        } = self;

        let matrix = context
            .users
            .by_aoc
            .get(&member.id)
            .and_then(|m| m.matrix.as_deref());

        let part = if part2 { "two" } else { "one" };

        let url = day.url();
        let AocDay { year, day } = day;
        let ts = context
            .config
            .local_timezone
            .from_utc_datetime(&ts.naive_utc())
            .format_ymd_hms_z();

        let name = member.matrix_mention_or_display_name(matrix);

        let rank = fmt_rank(rank);

        format!(
            "{name} has solved part {part} of [**Advent of Code {year} Day {day}**]({url}) at \
             {ts} ({rank})"
        )
    }
}
