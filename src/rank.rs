use std::rc::Rc;

use itertools::Itertools as _;
use rust_fuzzy_search::fuzzy_compare;

use crate::db::Entry;

pub fn rank<'a>(query: &str, entries: &'a [Rc<Entry>]) -> Vec<(usize, f32)> {
    let mut matches = entries
        .iter()
        .enumerate()
        .map(|(i, entry)| (i, fuzzy_compare(query, &entry.to_haystack())))
        .collect_vec();
    matches.sort_by(|a, b| b.1.total_cmp(&a.1));
    matches
}
