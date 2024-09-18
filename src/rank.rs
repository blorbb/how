use std::cmp::Reverse;

use itertools::Itertools as _;
use nucleo::{Config, Matcher, Utf32Str};

use crate::db::Entry;

pub fn rank<'a>(query: &'a str, entries: &'a [Entry]) -> Vec<(&'a Entry, u16)> {
    let mut query_buf = Vec::new();
    let query_chars = Utf32Str::new(&query, &mut query_buf);

    let mut config = Config::DEFAULT;
    config.ignore_case = true;
    config.normalize = true;
    config.prefer_prefix = true;
    let mut matcher = Matcher::new(config);

    let mut matches = entries
        .iter()
        .map(|entry| {
            (
                entry,
                matcher
                    .fuzzy_match(entry.to_haystack().slice(..), query_chars)
                    .unwrap_or(0),
            )
        })
        .collect_vec();
    matches.sort_by_key(|x| Reverse(x.1));
    matches
}
