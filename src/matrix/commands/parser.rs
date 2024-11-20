use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommand<'a> {
    pub command: &'a str,
    pub args: VecDeque<&'a str>,
    pub kwargs: HashMap<Cow<'a, str>, &'a str>,
}

impl<'a> ParsedCommand<'a> {
    pub fn get_from_kwargs_or_args(&mut self, key: &str) -> Option<&'a str> {
        self.kwargs
            .get(key)
            .copied()
            .or_else(|| self.args.pop_front())
    }
}

pub fn parse(cmd: &str) -> ParsedCommand {
    let mut iter = cmd
        .chars()
        .scan(0, |s, c| {
            let old_s = *s;
            *s += c.len_utf8();
            Some((old_s, c))
        })
        .peekable();

    macro_rules! skip_until {
        ($cond:expr) => {
            while iter.next_if(|&(_, c)| !($cond)(c)).is_some() {}
        };
    }
    macro_rules! position {
        () => {
            iter.peek().map(|&(i, _)| i).unwrap_or(cmd.len())
        };
    }

    skip_until!(|c: char| !c.is_whitespace());
    let start = position!();
    skip_until!(|c: char| c.is_whitespace());
    let command = &cmd[start..position!()];

    let mut args = VecDeque::new();
    let mut kwargs = HashMap::new();

    while let Some(&(start, c)) = {
        skip_until!(|c: char| !c.is_whitespace());
        iter.peek()
    } {
        if c == '"' {
            iter.next();
            let start = position!();
            skip_until!(|c| c == '"');
            args.push_back(&cmd[start..position!()]);
            iter.next();
            continue;
        }

        skip_until!(|c: char| c.is_whitespace() || c == '=');

        if iter.peek().is_some_and(|&(_, c)| c == '=') {
            let key = &cmd[start..position!()];
            let key = if key.chars().all(|c| c.is_lowercase()) {
                Cow::Borrowed(key)
            } else {
                Cow::Owned(key.to_lowercase())
            };
            iter.next();

            if iter.peek().is_some_and(|&(_, c)| c == '"') {
                iter.next();
                let start = position!();
                skip_until!(|c| c == '"');
                kwargs.insert(key, &cmd[start..position!()]);
                iter.next();
            } else {
                let start = position!();
                skip_until!(|c: char| c.is_whitespace());
                kwargs.insert(key, &cmd[start..position!()]);
            }
        } else {
            args.push_back(&cmd[start..position!()]);
        }
    }

    ParsedCommand {
        command,
        args,
        kwargs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test {
        ($inp:expr, $command:expr, $args:expr, $kwargs:expr) => {{
            let input = $inp;
            let expected = ParsedCommand {
                command: $command,
                args: $args.into(),
                kwargs: $kwargs
                    .into_iter()
                    .map(|(k, v)| (Cow::Borrowed(k), v))
                    .collect(),
            };
            assert_eq!(parse(input), expected);
        }};
    }

    #[test]
    fn no_args() {
        test!("test", "test", [], []);
        test!(" test", "test", [], []);
        test!("test ", "test", [], []);
    }

    #[test]
    fn args() {
        test!("test foo  bar", "test", ["foo", "bar"], []);
        test!("test foo  bar ", "test", ["foo", "bar"], []);
    }

    #[test]
    fn kwargs() {
        let kwargs = [("foo", "bar"), ("x", "123")];
        test!("test foo=bar X=123", "test", [], kwargs);
        test!("test Foo=bar x=123 ", "test", [], kwargs);
    }

    #[test]
    fn quotes() {
        test!(
            r#"test foo "bar  baz" test="a b" 42"#,
            "test",
            ["foo", "bar  baz", "42"],
            [("test", "a b")]
        );
        test!(r#"test "x=y""#, "test", ["x=y"], []);
    }
}
