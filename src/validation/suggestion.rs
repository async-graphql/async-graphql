use std::collections::HashMap;

use itertools::Itertools;

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let mut column = (0..=s1.len()).collect_vec();
    for (x, rx) in s2.bytes().enumerate() {
        column[0] = x + 1;
        let mut lastdiag = x;
        for (y, ry) in s1.bytes().enumerate() {
            let olddiag = column[y + 1];
            if rx != ry {
                lastdiag += 1;
            }
            column[y + 1] = (column[y + 1] + 1).min((column[y] + 1).min(lastdiag));
            lastdiag = olddiag;
        }
    }
    column[s1.len()]
}

pub fn make_suggestion<'a, I>(prefix: &str, options: I, input: &str) -> Option<String>
where
    I: Iterator<Item = &'a str>,
{
    let mut selected = Vec::new();
    let mut distances = HashMap::new();

    for opt in options {
        let distance = levenshtein_distance(input, opt);
        let threshold = (input.len() / 2).max((opt.len() / 2).max(1));
        if distance < threshold {
            selected.push(opt);
            distances.insert(opt, distance);
        }
    }

    if selected.is_empty() {
        return None;
    }
    selected.sort_by(|a, b| distances[a].cmp(&distances[b]));

    Some(format!(
        "{} {}?",
        prefix,
        selected
            .into_iter()
            .map(|s| format!("\"{}\"", s))
            .join(", ")
    ))
}
