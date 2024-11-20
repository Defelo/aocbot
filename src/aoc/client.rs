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

    pub async fn clear_leaderboard_cache(&self) -> anyhow::Result<()> {
        let mut guard = self.leaderboard_cache.write().await;
        guard.clear();
        self.store
            .set::<LeaderboardCache>(LEADERBOARD_CACHE_STORE_KEY, &guard)
            .await?;
        Ok(())
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

        let guard = self.leaderboard_cache.read().await;
        if let Some(cached) = guard.get(&year).filter(|(_, ts)| now < *ts + ttl) {
            trace!(
                year,
                ttl_secs = (cached.1 + ttl - now).num_seconds(),
                "leaderboard cached"
            );
            return Ok(cached.clone());
        }
        drop(guard);

        let mut guard = self.leaderboard_cache.write().await;
        if let Some(cached) = guard.get(&year).filter(|(_, ts)| now < *ts + ttl) {
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

    pub async fn get_daily_private_leaderboard(
        &self,
        year: i32,
        day: u32,
        parts: Parts,
    ) -> anyhow::Result<(PrivateLeaderboard, DateTime<Utc>)> {
        let (mut leaderboard, last_update) = self.get_private_leaderboard(year).await?;

        let mut members_by_p1 = leaderboard
            .members
            .iter()
            .filter_map(|(&id, m)| {
                m.completion_day_level
                    .get(&day)
                    .map(|c| (id, m, c.fst.get_star_ts))
            })
            .collect::<Vec<_>>();
        members_by_p1.sort_unstable_by_key(|&(_, m, ts)| (ts, m));
        let member_ids_and_completion_ts_by_p1 = members_by_p1
            .into_iter()
            .map(|(id, _, c)| (id, c))
            .collect::<Vec<_>>();

        let mut members_by_p2 = leaderboard
            .members
            .iter()
            .filter_map(|(&id, m)| {
                m.completion_day_level
                    .get(&day)
                    .and_then(|c| c.snd.as_ref())
                    .map(|c| (id, m, c.get_star_ts))
            })
            .collect::<Vec<_>>();
        members_by_p2.sort_unstable_by_key(|&(_, m, ts)| (ts, m));
        let member_ids_and_completion_ts_by_p2 = members_by_p2
            .into_iter()
            .map(|(id, _, c)| (id, c))
            .collect::<Vec<_>>();

        for m in leaderboard.members.values_mut() {
            m.global_score = 0;
            m.local_score = 0;
            m.stars = 0;
            m.last_star_ts = Default::default();
        }

        if matches!(parts, Parts::P1 | Parts::Both) {
            for (i, (id, ts)) in member_ids_and_completion_ts_by_p1.into_iter().enumerate() {
                let score = leaderboard.members.len() - i;
                let member = leaderboard.members.get_mut(&id).unwrap();
                member.local_score += score as u32;
                member.stars += 1;
                member.last_star_ts = member.last_star_ts.max(ts);
            }
        }

        if matches!(parts, Parts::P2 | Parts::Both) {
            for (i, (id, ts)) in member_ids_and_completion_ts_by_p2.into_iter().enumerate() {
                let score = leaderboard.members.len() - i;
                let member = leaderboard.members.get_mut(&id).unwrap();
                member.local_score += score as u32;
                member.stars += 1;
                member.last_star_ts = member.last_star_ts.max(ts);
            }
        }

        Ok((leaderboard, last_update))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parts {
    P1,
    P2,
    Both,
}
