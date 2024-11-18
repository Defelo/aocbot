use anyhow::anyhow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub fn serialize<S>(datetime: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    datetime.timestamp().serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let ts = Deserialize::deserialize(deserializer)?;
    DateTime::from_timestamp(ts, 0)
        .ok_or_else(|| serde::de::Error::custom(anyhow!("Invalid timestamp: {ts}")))
}
