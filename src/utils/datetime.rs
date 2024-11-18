use std::time::Duration;

use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use tracing::{instrument, trace};

pub const EST: FixedOffset = FixedOffset::west_opt(5 * 3600).unwrap();

#[cfg(test)]
static NOW: std::sync::LazyLock<std::sync::Arc<std::sync::RwLock<DateTime<Utc>>>> =
    std::sync::LazyLock::new(Default::default);
#[cfg(test)]
static SET_NOW_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub fn now() -> DateTime<Utc> {
    #[cfg(test)]
    return *NOW.read().unwrap();

    #[cfg_attr(test, expect(unreachable_code))]
    Utc::now()
}

#[cfg(test)]
pub fn set_now(now: DateTime<Utc>) -> std::sync::MutexGuard<'static, ()> {
    let guard = SET_NOW_LOCK.lock().unwrap();
    *NOW.write().unwrap() = now;
    guard
}

pub fn now_est() -> DateTime<FixedOffset> {
    EST.from_utc_datetime(&now().naive_utc())
}

#[instrument(level = "trace")]
pub async fn sleep_until(datetime: DateTime<Utc>) {
    loop {
        let now = now();
        if now >= datetime {
            trace!(?now, "done waiting");
            break;
        }
        let duration = (datetime - now).to_std().unwrap();
        let sleep_duration = duration.min(Duration::from_secs(60));
        trace!(?sleep_duration, ?duration, ?now, "wait for");
        tokio::time::sleep(sleep_duration).await;
    }
}

pub trait DateTimeExt {
    fn format_ymd_hms_z(self) -> impl std::fmt::Display;
    fn format_ymd_hms(self) -> impl std::fmt::Display;
}

impl<Tz> DateTimeExt for DateTime<Tz>
where
    Tz: TimeZone,
    Tz::Offset: std::fmt::Display,
{
    fn format_ymd_hms_z(self) -> impl std::fmt::Display {
        self.format("%Y-%m-%d %H:%M:%S %:z")
    }

    fn format_ymd_hms(self) -> impl std::fmt::Display {
        self.format("%Y-%m-%d %H:%M:%S")
    }
}
