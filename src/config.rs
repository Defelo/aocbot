use std::path::PathBuf;

use chrono::FixedOffset;
use config::{File, FileFormat};
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId};
use regex::Regex;
use serde::{Deserialize, Deserializer};

use crate::{
    aoc::models::AocId,
    utils::{self, regex_set_replacer::RegexSetReplacer},
};

pub fn load<'a>(config_path: impl Iterator<Item = &'a str>) -> anyhow::Result<Config> {
    load_with_defaults(std::iter::empty(), config_path)
}

fn load_with_defaults<'a>(
    defaults: impl Iterator<Item = &'a str>,
    config_path: impl Iterator<Item = &'a str>,
) -> anyhow::Result<Config> {
    let mut builder = config::Config::builder();

    for content in defaults.chain([include_str!("../config.toml")]) {
        let source = File::from_str(content, FileFormat::Toml);
        builder = builder.add_source(source);
    }

    for path in config_path {
        builder = builder.add_source(File::with_name(path.trim()));
    }

    builder.build()?.try_deserialize().map_err(Into::into)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(with = "utils::serde::via_string")]
    pub local_timezone: FixedOffset,
    pub matrix: MatrixConfig,
    pub aoc: AocConfig,
    pub garygrady: GarygradyConfig,
    pub users: Vec<User>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatrixConfig {
    pub homeserver: String,
    pub store_path: PathBuf,
    pub admin_ids: Vec<OwnedUserId>,
    pub room_id: OwnedRoomId,
    pub command_prefix: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AocConfig {
    pub session_file: PathBuf,
    pub leaderboard_rows: usize,
    pub default_cache_ttl: u64,
    pub cache_ttl_rules: Vec<CacheTtlRule>,
    #[serde(deserialize_with = "deserialize_repo_rules")]
    pub repo_rules: RegexSetReplacer,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CacheTtlRule {
    pub minutes_after_unlock: i64,
    pub ttl: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GarygradyConfig {
    pub interval: u64,
    pub max_age: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct User {
    pub aoc: Option<AocId>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load() {
        load_with_defaults(
            [
                "matrix.homeserver = \"https://matrix.example.com\"",
                "matrix.store_path = \".store\"",
                "matrix.admin_ids = []",
                "matrix.room_id = \"!xoXcjSEJPUfQmzETtS:matrix.example.com\"",
                "aoc.session_file = \".session\"",
            ]
            .into_iter(),
            [concat!(env!("CARGO_MANIFEST_DIR"), "/users.toml")].into_iter(),
        )
        .unwrap();
    }
}
