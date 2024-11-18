use std::{
    collections::{BTreeMap, HashMap},
    ops::Bound,
    time::Duration,
};

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tracing::trace;

use super::{
    api::AocApiClient,
    models::{AocWhoami, PrivateLeaderboard},
};
use crate::{
    aoc::day::AocDay,
    utils::{datetime::now, store::Store},
};

const LEADERBOARD_CACHE_STORE_KEY: &[u8] = b"aoc_leaderboard";

pub type LeaderboardCache = HashMap<i32, (PrivateLeaderboard, DateTime<Utc>)>;

pub struct AocClient {
    api: AocApiClient,
    whoami: AocWhoami,
    default_cache_ttl: Duration,
    cache_ttl_rules: BTreeMap<i64, Duration>,
    leaderboard_cache: RwLock<LeaderboardCache>,
    store: Store,
}

impl AocClient {
    pub async fn new(
        session: &str,
        default_cache_ttl: Duration,
        cache_ttl_rules: BTreeMap<i64, Duration>,
        store: Store,
    ) -> anyhow::Result<Self> {
        let api = AocApiClient::new(session)?;

        let whoami = api.whoami().await?;

        let leaderboard_cache = store
            .get::<LeaderboardCache>(LEADERBOARD_CACHE_STORE_KEY)
            .await?
            .unwrap_or_default();

        Ok(Self {
            api,
            whoami,
            default_cache_ttl,
            cache_ttl_rules,
            leaderboard_cache: leaderboard_cache.into(),
            store,
        })
    }

    pub fn whoami(&self) -> &AocWhoami {
        &self.whoami
    }

    pub async fn get_private_leaderboard_cached(
        &self,
        year: i32,
    ) -> Option<(PrivateLeaderboard, DateTime<Utc>)> {
        self.leaderboard_cache.read().await.get(&year).cloned()
    }

    pub async fn get_private_leaderboard(
        &self,
        year: i32,
        force_refresh: bool,
    ) -> anyhow::Result<(PrivateLeaderboard, DateTime<Utc>)> {
        let now = now();
        let ttl = match AocDay::current() {
            Some(day) => {
                let minutes_since_unlock = (now - day.unlock_datetime()).num_minutes();
                self.cache_ttl_rules
                    .range((Bound::Excluded(minutes_since_unlock), Bound::Unbounded))
                    .next()
                    .map(|(_, &ttl)| ttl)
                    .unwrap_or(self.default_cache_ttl)
            }
            None => self.default_cache_ttl,
        };

        if !force_refresh {
            let guard = self.leaderboard_cache.read().await;
            if let Some(cached) = guard.get(&year).filter(|(_, ts)| now < *ts + ttl) {
                trace!(
                    year,
                    ttl_secs = (cached.1 + ttl - now).num_seconds(),
                    "leaderboard cached"
                );
                return Ok(cached.clone());
            }
        }

        let mut guard = self.leaderboard_cache.write().await;
        if let Some(cached) = guard
            .get(&year)
            .filter(|(_, ts)| !force_refresh && now < *ts + ttl)
        {
            trace!(
                year,
                ttl_secs = (cached.1 + ttl - now).num_seconds(),
                "leaderboard cached"
            );
            return Ok(cached.clone());
        }

        trace!(year, "fetching leaderboard");
        let leaderboard = self
            .api
            .get_private_leaderboard(year, self.whoami.user_id)
            .await?;

        let entry = (leaderboard, now);
        guard.insert(year, entry.clone());
        self.store
            .set::<LeaderboardCache>(LEADERBOARD_CACHE_STORE_KEY, &guard)
            .await?;
        Ok(entry)
    }
}
