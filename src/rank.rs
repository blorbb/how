use itertools::Itertools as _;
use rust_fuzzy_search::fuzzy_compare;

use crate::db::Entry;

pub fn rank<'a>(query: &str, entries: &'a [Entry]) -> Vec<(usize, f32)> {
    let query = query.to_lowercase();
    let mut matches = entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            // varying weightings for each
            let title_cmp = fuzzy_compare(&query, &entry.title().to_lowercase());
            let desc_cmp = fuzzy_compare(&query, &entry.description().to_lowercase());
            let ans_cmp = fuzzy_compare(&query, &entry.answer().to_lowercase());
            (i, title_cmp * 2.0 + desc_cmp + ans_cmp * 1.5)
        })
        .collect_vec();
    matches.sort_by(|a, b| b.1.total_cmp(&a.1));
    matches
}
