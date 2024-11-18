use std::fmt::Write;

use chrono::TimeZone;
use matrix_sdk::{
    ruma::{api::client::error::ErrorKind, events::room::message::OriginalRoomMessageEvent},
    Room,
};

use crate::{
    aoc::{day::AocDay, models::PrivateLeaderboardMember},
    context::Context,
    matrix::utils::{error_message, html_message, RoomExt},
    utils::{datetime::DateTimeExt, fmt::format_rank},
};

pub async fn invoke(
    event: &OriginalRoomMessageEvent,
    room: Room,
    context: &Context,
    args: impl Iterator<Item = &str>,
) -> anyhow::Result<()> {
    let most_recent_year = AocDay::most_recent().year;

    let mut year = most_recent_year;
    let mut offset = 0;
    let mut rows = None;
    let mut force_refresh = false;

    for arg in args {
        if let Some(x) = arg.parse().ok().filter(|&x| x >= 2015) {
            year = x;
        } else if let Some(x) = arg.parse().ok().filter(|&x| x <= 200) {
            if rows.is_none() {
                rows = Some(x);
            } else {
                offset = x;
            }
        } else if matches!(arg, "-r" | "--refresh") {
            force_refresh = true;
        } else {
            room.reply_to(event, error_message("Invalid arguments"))
                .await?;
            return Ok(());
        }
    }
    let rows = rows.unwrap_or(context.config.aoc.leaderboard_rows);

    if force_refresh && !context.config.matrix.admin_ids.contains(&event.sender) {
        room.reply_to(event, error_message("Permission denied"))
            .await?;
        return Ok(());
    }

    let (leaderboard, last_update) = match context
        .aoc_client
        .get_private_leaderboard(year, force_refresh)
        .await
    {
        Ok(resp) => resp,
        Err(err) => match err.downcast::<reqwest::Error>() {
            Ok(err) => {
                if let Some(status) = err.status() {
                    room.reply_to(
                        event,
                        error_message(format!(
                            "Failed to fetch private leaderboard for {year} ({status})"
                        )),
                    )
                    .await?;
                    return Ok(());
                } else {
                    return Err(err.into());
                }
            }
            Err(err) => return Err(err),
        },
    };
    let last_update = context
        .config
        .local_timezone
        .from_utc_datetime(&last_update.naive_utc())
        .format_ymd_hms_z();

    let mut members = leaderboard.members.into_values().collect::<Vec<_>>();
    members.sort_unstable();

    let mut leaderboard = format!(
        r#"
<h3>Private Leaderboard (Advent of Code {year})</h3>
<table>
<tr> <th>Rank</th> <th>Local Score</th> <th>Global Score</th> <th>Stars</th> <th>AoC Name</th> <th>Matrix User</th> <th>Repository</th> </tr>
"#
    );

    let mut last_score = u32::MAX;
    let mut rank = 0;
    for (rank, member) in members
        .into_iter()
        .enumerate()
        .map(|(i, member)| {
            if member.local_score != last_score {
                last_score = member.local_score;
                rank = i + 1;
            }
            (rank, member)
        })
        .skip(offset)
        .take(rows)
    {
        let PrivateLeaderboardMember {
            local_score,
            global_score,
            stars,
            ..
        } = member;

        let name = member.display_name();

        let matrix_name = context
            .users
            .by_aoc
            .get(&member.id)
            .and_then(|u| u.matrix.as_ref())
            .map(|m| m.matrix_to_uri().to_string())
            .unwrap_or_default();

        let repo = context
            .users
            .by_aoc
            .get(&member.id)
            .and_then(|u| u.repo.as_deref())
            .unwrap_or_default();
        let repo_title = context
            .config
            .aoc
            .repo_rules
            .match_and_replace(repo)
            .map(|m| m.replacement);
        let repo_title = repo_title.as_deref().unwrap_or(repo);

        let (m, m_) = if rank <= 3 {
            ("<b>", "</b>")
        } else {
            Default::default()
        };

        let rank = format_rank(rank);

        write!(
            &mut leaderboard,
            r#"
<tr>
    <td>{m}{rank}{m_}</td>
    <td>{m}{local_score}{m_}</td>
    <td>{m}{global_score}{m_}</td>
    <td>{m}{stars}{m_}</td>
    <td>{m}{name}{m_}</td>
    <td>{matrix_name}</td>
    <td>{m}<a href="{repo}">{repo_title}</a>{m_}</td>
</tr>
"#
        )
        .unwrap();
    }

    write!(
        &mut leaderboard,
        r#"
</table>
<sup>Last update: {last_update}</sup>
"#
    )
    .unwrap();

    if let Err(err) = room.reply_to(event, html_message(leaderboard)).await {
        if err
            .as_client_api_error()
            .and_then(|err| err.error_kind())
            .is_some_and(|kind| matches!(kind, ErrorKind::TooLarge))
        {
            room.reply_to(
                event,
                error_message(
                    "The requested leaderboard slice would be too large to fit in a matrix \
                     message. Try to reduce the number of rows.",
                ),
            )
            .await?;
            return Ok(());
        } else {
            return Err(err.into());
        }
    }

    Ok(())
}
