use std::{cmp::Reverse, fmt::Write};

use chrono::{DateTime, TimeZone, Utc};
use matrix_sdk::{
    ruma::{events::room::message::OriginalRoomMessageEvent, OwnedUserId},
    Room,
};

use crate::{
    aoc::day::AocDay,
    context::Context,
    matrix::{
        commands::{parser::ParsedCommand, send_error},
        utils::{error_message, html_message, RoomExt},
    },
    utils::{
        datetime::DateTimeExt,
        fmt::{fmt_rank, fmt_timedelta},
    },
};

pub async fn invoke(
    event: &OriginalRoomMessageEvent,
    room: Room,
    context: &Context,
    mut cmd: ParsedCommand<'_>,
) -> anyhow::Result<()> {
    let most_recent = AocDay::most_recent();

    let user = cmd.get_from_kwargs_or_args("user");
    let most_recent_year = AocDay::most_recent().year;
    let year = match cmd.get_from_kwargs_or_args("year").map(|y| {
        y.parse()
            .ok()
            .filter(|y| (2015..=most_recent_year).contains(y))
    }) {
        Some(Some(y)) => y,
        Some(None) => return send_error(&room, event, "Failed to parse argument 'year'").await,
        None => most_recent_year,
    };

    let (leaderboard, last_update) = context.aoc_client.get_private_leaderboard(year).await?;

    let last_update = context
        .config
        .local_timezone
        .from_utc_datetime(&last_update.naive_utc())
        .format_ymd_hms_z();

    let Some(user) = (match user {
        Some(name) => leaderboard
            .members
            .values()
            .find(|m| {
                name.parse() == Ok(m.id)
                    || m.name
                        .as_ref()
                        .is_some_and(|n| n.to_lowercase() == name.to_lowercase())
            })
            .or_else(|| {
                context
                    .users
                    .by_matrix
                    .get(
                        name.parse::<OwnedUserId>()
                            .ok()
                            .as_ref()
                            .or_else(|| event.content.mentions.as_ref()?.user_ids.iter().next())?,
                    )
                    .and_then(|u| leaderboard.members.values().find(|m| u.aoc == Some(m.id)))
            }),
        None => context
            .users
            .by_matrix
            .get(&event.sender)
            .and_then(|u| leaderboard.members.values().find(|m| u.aoc == Some(m.id))),
    }) else {
        room.reply_to(event, error_message("User not found"))
            .await?;
        return Ok(());
    };

    let name = user.display_name();

    let aoc_id = user.id;

    let matrix = context
        .users
        .by_aoc
        .get(&user.id)
        .and_then(|u| u.matrix.as_ref())
        .map(|m| m.matrix_to_uri().to_string())
        .unwrap_or_default();

    let repo = context
        .users
        .by_aoc
        .get(&user.id)
        .and_then(|u| u.repo.as_deref())
        .unwrap_or_default();
    let repo_title = context
        .config
        .aoc
        .repo_rules
        .match_and_replace(repo)
        .map(|m| m.replacement);
    let repo_title = repo_title.as_deref().unwrap_or(repo);

    let stars = user.stars;
    let rank = fmt_rank(leaderboard.members.values().filter(|&o| o <= user).count());
    let local_score = user.local_score;
    let global_score = user.global_score;

    let days = if year == most_recent.year {
        most_recent.day
    } else {
        25
    };
    let max_stars = days * 2;
    let progress_percent = stars as f64 / max_stars as f64 * 100.0;

    let link_prefix = &context.config.matrix.link_prefix;
    let mut out = format!(
        r#"
<table>
    <tr>
        <th>AoC ID</th>
        <th>AoC Name</th>
        <th>Matrix User</th>
        <th>Repository</th>
    </tr>

    <tr>
        <td>{aoc_id}</td>
        <td>{name}</td>
        <td>{matrix}</td>
        <td><a href="{link_prefix}{repo}">{repo_title}</a></td>
    </tr>

    <tr>
        <th>Stars</th>
        <th>Rank</th>
        <th>Local Score</th>
        <th>Global Score</th>
    </tr>

    <tr>
        <td>{stars}/{max_stars} ({progress_percent:.0}%)</td>
        <td>{rank}</td>
        <td>{local_score}</td>
        <td>{global_score}</td>
    </tr>
</table>

<table>
    <tr>
        <th>Day</th>
        <th>Part 1</th>
        <th>Part 2</th>
    </tr>
"#
    );

    for d in 1..=days {
        let unlock = AocDay { year, day: d }.unlock_datetime();
        let fmt_dt = |dt: DateTime<Utc>| {
            context
                .config
                .local_timezone
                .from_utc_datetime(&dt.naive_utc())
                .format_ymd_hms()
        };
        let p1 = user.completion_day_level.get(&d).map(|c| c.fst.get_star_ts);
        let p2 = user
            .completion_day_level
            .get(&d)
            .and_then(|c| c.snd.as_ref())
            .map(|c| c.get_star_ts);

        let rank_p1 = leaderboard
            .members
            .values()
            .filter(|m| {
                m.completion_day_level
                    .get(&d)
                    .map(|c| Reverse(c.fst.get_star_ts))
                    >= p1.map(Reverse)
            })
            .count();

        let rank_p2 = leaderboard
            .members
            .values()
            .filter(|m| {
                m.completion_day_level
                    .get(&d)
                    .and_then(|d| d.snd.as_ref())
                    .map(|c| Reverse(c.get_star_ts))
                    >= p2.map(Reverse)
            })
            .count();

        match (p1, p2) {
            (None, _) => write!(&mut out, "<tr><td>{d}</td><td></td><td></td></tr>"),
            (Some(p1), None) => write!(
                &mut out,
                "<tr><td>{d}</td><td>{} (<b>{}</b>, <b>{}</b>)</td><td></td></tr>",
                fmt_dt(p1),
                fmt_timedelta(p1 - unlock),
                fmt_rank(rank_p1),
            ),
            (Some(p1), Some(p2)) => write!(
                &mut out,
                "<tr><td>{d}</td><td>{} (<b>{}</b>, <b>{}</b>)</td><td>{} (+<b>{}</b> &rArr; \
                 <b>{}</b>, <b>{}</b>)</td></tr>",
                fmt_dt(p1),
                fmt_timedelta(p1 - unlock),
                fmt_rank(rank_p1),
                fmt_dt(p2),
                fmt_timedelta(p2 - p1),
                fmt_timedelta(p2 - unlock),
                fmt_rank(rank_p2),
            ),
        }
        .unwrap();
    }

    write!(&mut out, "</table><sup>Last update: {last_update}</sup>",).unwrap();

    room.reply_to(event, html_message(out)).await?;

    Ok(())
}
