use std::fmt::{Display, Formatter};

use chrono::TimeDelta;

pub fn fmt_rank(rank: usize) -> impl Display {
    DisplayWith(move |f| {
        let medal = match rank {
            1 => "🥇 ",
            2 => "🥈 ",
            3 => "🥉 ",
            _ => "",
        };
        let suffix = match (rank / 10 % 10, rank % 10) {
            (1, _) => "th",
            (_, 1) => "st",
            (_, 2) => "nd",
            (_, 3) => "rd",
            _ => "th",
        };
        write!(f, "{medal}{rank}{suffix}")
    })
}

pub fn fmt_timedelta(td: TimeDelta) -> impl Display {
    DisplayWith(move |f| {
        let s = td.num_seconds() % 60;
        let m = td.num_minutes() % 60;
        let h = td.num_hours() % 24;
        let d = td.num_days();
        if td.num_days() >= 1 {
            write!(f, "{d}d {h}h {m}m {s}s")
        } else if td.num_hours() >= 1 {
            write!(f, "{h}h {m}m {s}s")
        } else if td.num_minutes() >= 1 {
            write!(f, "{m}m {s}s")
        } else {
            write!(f, "{s}s")
        }
    })
}

struct DisplayWith<F>(F)
where
    F: Fn(&mut Formatter) -> std::fmt::Result;

impl<F> Display for DisplayWith<F>
where
    F: Fn(&mut Formatter) -> std::fmt::Result,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        (self.0)(f)
    }
}
