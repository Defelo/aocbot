use regex::Regex;
use serde::{Deserialize, Deserializer};

pub mod timestamp;
pub mod via_string;

pub fn deserialize_regex<'de, D>(deserializer: D) -> Result<Regex, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Regex::new(&s).map_err(serde::de::Error::custom)
}
