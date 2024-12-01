use std::{cmp::Reverse, collections::HashMap, fmt::Write};

use matrix_sdk::{ruma::events::room::message::OriginalRoomMessageEvent, Room};

use crate::{
    aoc::{day::AocDay, models::PrivateLeaderboardMember},
    matrix::utils::{html_message, RoomExt},
    Context,
};

pub async fn invoke(
    event: &OriginalRoomMessageEvent,
    room: Room,
    context: &Context,
) -> anyhow::Result<()> {
    let aoc_users = context
        .aoc_client
        .get_private_leaderboard(AocDay::most_recent().year)
        .await?
        .0
        .members
        .into_values()
        .map(|u| (u.id, u))
        .collect::<HashMap<_, _>>();

    let mut solutions = String::from(
        r#"
<h3>Advent of Code Solution Repositories</h3>
<table>
<tr> <th>AoC Name</th> <th>Matrix User</th> <th>Repository</th> </tr>
"#,
    );

    let mut rows = Vec::new();

    for user in &context.config.users {
        let Some(repo) = &user.repo else { continue };

        let aoc_user = user.aoc.and_then(|id| aoc_users.get(&id));

        let name = aoc_user.map(|u| u.display_name()).unwrap_or_default();

        let matrix_name = user
            .matrix
            .as_ref()
            .map(|m| m.matrix_to_uri().to_string())
            .unwrap_or_default();

        let repo_title = context
            .config
            .aoc
            .repo_rules
            .match_and_replace(repo)
            .map(|m| m.replacement);

        rows.push((aoc_user, name, matrix_name, repo, repo_title));
    }

    rows.sort_unstable_by(|a, b| {
        fn key<'a>(
            aoc_user: Option<&'a PrivateLeaderboardMember>,
            name: &'a str,
        ) -> impl Ord + use<'a> {
            (Reverse(aoc_user.map(Reverse)), name)
        }
        key(a.0, &a.1).cmp(&key(b.0, &b.1))
    });

    for (_, name, matrix_name, repo, repo_title) in rows {
        let repo_title = repo_title.as_deref().unwrap_or(repo);
        let link_prefix = &context.config.matrix.link_prefix;
        write!(
            &mut solutions,
            r#"
<tr>
    <td>{name}</td>
    <td>{matrix_name}</td>
    <td><a href="{link_prefix}{repo}">{repo_title}</a></td>
</tr>
"#
        )
        .unwrap();
    }

    solutions.push_str("</table>");

    room.reply_to(event, html_message(solutions)).await?;

    Ok(())
}
