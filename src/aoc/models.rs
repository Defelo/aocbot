use std::{cmp::Reverse, collections::HashMap};

use chrono::{DateTime, Utc};
use matrix_sdk::ruma::UserId;
use serde::{Deserialize, Serialize};

use crate::utils;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AocWhoami {
    pub user_id: u64,
    pub invite_code: String,
}

pub type PrivateLeaderboardMembers = HashMap<String, PrivateLeaderboardMember>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateLeaderboard {
    pub event: String,
    pub owner_id: u64,
    pub members: PrivateLeaderboardMembers,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateLeaderboardMember {
    pub id: u64,
    pub name: Option<String>,
    pub global_score: u32,
    pub local_score: u32,
    pub stars: u32,
    #[serde(with = "utils::serde::timestamp")]
    pub last_star_ts: DateTime<Utc>,
    pub completion_day_level: HashMap<u32, PrivateLeaderboardMemberCompletionDay>,
}

impl PrivateLeaderboardMember {
    pub fn display_name(&self) -> String {
        match self.name.clone() {
            Some(name) => name,
            None => format!("[anonymous user #{}]", self.id),
        }
    }

    pub fn matrix_mention_or_display_name(&self, matrix: Option<&UserId>) -> String {
        match matrix {
            Some(matrix) => format!("{} ({})", matrix.matrix_to_uri(), self.display_name()),
            None => format!("**{}**", self.display_name()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateLeaderboardMemberCompletionDay {
    #[serde(rename = "1")]
    pub fst: PrivateLeaderboardMemberCompletionDayPart,
    #[serde(rename = "2")]
    pub snd: Option<PrivateLeaderboardMemberCompletionDayPart>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateLeaderboardMemberCompletionDayPart {
    #[serde(with = "utils::serde::timestamp")]
    pub get_star_ts: DateTime<Utc>,
    pub star_index: u64,
}

impl PartialOrd for PrivateLeaderboardMember {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrivateLeaderboardMember {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let key = |m: &Self| {
            (
                Reverse(m.local_score),
                Reverse(m.stars),
                m.last_star_ts,
                m.id,
            )
        };
        key(self).cmp(&key(other))
    }
}
