use chrono::{DateTime, Datelike, TimeZone, Utc};

use crate::utils::datetime::{now_est, EST};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AocDay {
    pub year: i32,
    pub day: u32,
}

impl AocDay {
    pub fn unlock_datetime(self) -> DateTime<Utc> {
        EST.with_ymd_and_hms(self.year, 12, self.day, 0, 0, 0)
            .unwrap()
            .to_utc()
    }

    pub fn url(self) -> String {
        format!("https://adventofcode.com/{}/day/{}", self.year, self.day)
    }

    pub fn current() -> Option<Self> {
        let now = now_est();
        (now.month() == 12 && now.day() <= 25).then_some(Self {
            year: now.year(),
            day: now.day(),
        })
    }

    pub fn next() -> Self {
        let now = now_est();
        let (year, day) = if now.month() != 12 {
            // december this year
            (now.year(), 1)
        } else if now.day() < 25 {
            // tomorrow
            (now.year(), now.day() + 1)
        } else {
            // next year
            (now.year() + 1, 1)
        };
        Self { year, day }
    }

    pub fn most_recent() -> Self {
        let now = now_est();
        let (year, day) = if now.month() != 12 {
            // december last year
            (now.year() - 1, 25)
        } else {
            // this year, today or 25th
            (now.year(), now.day().min(25))
        };
        Self { year, day }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::datetime::set_now;

    #[test]
    fn unlock_datetime() {
        for (year, day, expected) in [
            (2024, 1, "2024-12-01T06:00:00+01:00"),
            (2024, 2, "2024-12-02T06:00:00+01:00"),
            (2024, 16, "2024-12-16T06:00:00+01:00"),
            (2024, 25, "2024-12-25T06:00:00+01:00"),
            (2025, 1, "2025-12-01T06:00:00+01:00"),
        ] {
            let day = AocDay { year, day };
            let expected = expected.parse::<DateTime<Utc>>().unwrap();
            assert_eq!(day.unlock_datetime(), expected);
        }
    }

    #[test]
    fn url() {
        let day = |year, day| AocDay { year, day };
        assert_eq!(day(2023, 1).url(), "https://adventofcode.com/2023/day/1");
        assert_eq!(day(2018, 17).url(), "https://adventofcode.com/2018/day/17");
    }

    #[test]
    fn next() {
        for (now, year, day) in [
            ("2024-07-17T13:37:42+01:00", 2024, 1),
            ("2024-11-17T13:37:42+01:00", 2024, 1),
            ("2024-12-01T05:59:07+01:00", 2024, 1),
            ("2024-12-01T06:00:13+01:00", 2024, 2),
            ("2024-12-15T14:17:00+01:00", 2024, 16),
            ("2024-12-25T05:59:07+01:00", 2024, 25),
            ("2024-12-25T06:00:07+01:00", 2025, 1),
            ("2024-12-31T13:37:42+01:00", 2025, 1),
            ("2025-01-07T13:37:42+01:00", 2025, 1),
        ] {
            let _guard = set_now(now.parse().unwrap());
            let expected = AocDay { year, day };
            assert_eq!(AocDay::next(), expected);
        }
    }

    #[test]
    fn current() {
        for (now, expected) in [
            ("2024-07-17T13:37:42+01:00", None),
            ("2024-11-17T13:37:42+01:00", None),
            ("2024-12-01T05:59:07+01:00", None),
            ("2024-12-01T06:00:13+01:00", Some((2024, 1))),
            ("2024-12-15T14:17:00+01:00", Some((2024, 15))),
            ("2024-12-25T05:59:07+01:00", Some((2024, 24))),
            ("2024-12-25T06:00:07+01:00", Some((2024, 25))),
            ("2024-12-31T13:37:42+01:00", None),
            ("2025-01-07T13:37:42+01:00", None),
        ] {
            let _guard = set_now(now.parse().unwrap());
            let expected = expected.map(|(year, day)| AocDay { year, day });
            assert_eq!(AocDay::current(), expected);
        }
    }

    #[test]
    fn most_recent() {
        for (now, year, day) in [
            ("2024-07-17T13:37:42+01:00", 2023, 25),
            ("2024-11-17T13:37:42+01:00", 2023, 25),
            ("2024-12-01T05:59:07+01:00", 2023, 25),
            ("2024-12-01T06:00:13+01:00", 2024, 1),
            ("2024-12-15T14:17:00+01:00", 2024, 15),
            ("2024-12-25T05:59:07+01:00", 2024, 24),
            ("2024-12-25T06:00:07+01:00", 2024, 25),
            ("2024-12-31T13:37:42+01:00", 2024, 25),
            ("2025-01-07T13:37:42+01:00", 2024, 25),
        ] {
            let _guard = set_now(now.parse().unwrap());
            let expected = AocDay { year, day };
            assert_eq!(AocDay::most_recent(), expected);
        }
    }
}
