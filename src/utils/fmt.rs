use std::fmt::{Display, Formatter};

pub fn format_rank(rank: usize) -> impl Display {
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
