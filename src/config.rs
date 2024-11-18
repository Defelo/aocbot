use std::path::PathBuf;

use chrono::FixedOffset;
use config::{File, FileFormat};
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId};
use regex::Regex;
use serde::{Deserialize, Deserializer};

use crate::utils::{self, regex_set_replacer::RegexSetReplacer};

pub fn load<'a>(config_path: impl Iterator<Item = &'a str>) -> anyhow::Result<Config> {
    config_path
        .fold(
            config::Config::builder().add_source(File::from_str(
                include_str!("../config.toml"),
                FileFormat::Toml,
            )),
            |builder, file| builder.add_source(File::with_name(file.trim())),
        )
        .build()?
        .try_deserialize()
        .map_err(Into::into)
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(deserialize_with = "utils::serde::deserialize_from_string")]
    pub local_timezone: FixedOffset,
    pub matrix: MatrixConfig,
    pub aoc: AocConfig,
    pub garygrady: GarygradyConfig,
    pub users: Vec<User>,
}

#[derive(Debug, Deserialize)]
pub struct MatrixConfig {
    pub homeserver: String,
    pub store_path: PathBuf,
    pub admin_ids: Vec<OwnedUserId>,
    pub room_id: OwnedRoomId,
    pub command_prefix: String,
}

#[derive(Debug, Deserialize)]
pub struct AocConfig {
    pub session_file: PathBuf,
    pub leaderboard_rows: usize,
    pub default_cache_ttl: u64,
    pub cache_ttl_rules: Vec<CacheTtlRule>,
    #[serde(deserialize_with = "deserialize_repo_rules")]
    pub repo_rules: RegexSetReplacer,
}

#[derive(Debug, Deserialize)]
pub struct CacheTtlRule {
    pub minutes_after_unlock: i64,
    pub ttl: u64,
}

#[derive(Debug, Deserialize)]
pub struct GarygradyConfig {
    pub interval: u64,
    pub max_age: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub aoc: Option<u64>,
    pub matrix: Option<OwnedUserId>,
    pub repo: Option<String>,
}

fn deserialize_repo_rules<'de, D>(deserializer: D) -> Result<RegexSetReplacer, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct RepoRule {
        #[serde(deserialize_with = "utils::serde::deserialize_regex")]
        regex: Regex,
        title: Option<String>,
    }

    let rules = Vec::<RepoRule>::deserialize(deserializer)?
        .into_iter()
        .map(|rule| (rule.regex, rule.title.unwrap_or_else(|| "$0".into())))
        .collect();

    Ok(RegexSetReplacer::new(rules))
}
