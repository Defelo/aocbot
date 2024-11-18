use regex::{Regex, RegexSet};

#[derive(Debug)]
pub struct RegexSetReplacer {
    regex_set: RegexSet,
    rules: Vec<(Regex, String)>,
}

impl RegexSetReplacer {
    pub fn new(rules: Vec<(Regex, String)>) -> Self {
        let regex_set = RegexSet::new(rules.iter().map(|(r, _)| r.as_str())).unwrap();
        Self { regex_set, rules }
    }

    pub fn match_and_replace(&self, haystack: &str) -> Option<RegexSetReplacerMatch> {
        self.regex_set.matches(haystack).iter().next().map(|idx| {
            let (regex, replacement) = &self.rules[idx];
            let replacement = regex.replace(haystack, replacement).into_owned();
            RegexSetReplacerMatch { idx, replacement }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexSetReplacerMatch {
    pub idx: usize,
    pub replacement: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regex_set_replacer() {
        let replacer = RegexSetReplacer::new(vec![
            (Regex::new(r"foo(\d+)").unwrap(), "${1}FOO".into()),
            (Regex::new(r"bar(\d+)").unwrap(), "${1}BAR".into()),
        ]);

        let RegexSetReplacerMatch { idx, replacement } =
            replacer.match_and_replace("foo42").unwrap();
        assert_eq!(idx, 0);
        assert_eq!(replacement, "42FOO");

        let RegexSetReplacerMatch { idx, replacement } =
            replacer.match_and_replace("bar1337").unwrap();
        assert_eq!(idx, 1);
        assert_eq!(replacement, "1337BAR");

        assert_eq!(replacer.match_and_replace("test"), None);
    }
}
