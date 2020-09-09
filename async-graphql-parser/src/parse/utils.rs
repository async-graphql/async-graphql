use super::Rule;
use crate::Result;
use pest::iterators::{Pair, Pairs};

pub(super) fn next_if_rule<'a>(pairs: &mut Pairs<'a, Rule>, rule: Rule) -> Option<Pair<'a, Rule>> {
    if pairs.peek().map_or(false, |pair| pair.as_rule() == rule) {
        Some(pairs.next().unwrap())
    } else {
        None
    }
}
pub(super) fn parse_if_rule<'a, T>(
    pairs: &mut Pairs<'a, Rule>,
    rule: Rule,
    f: impl FnOnce(Pair<Rule>) -> Result<T>,
) -> Result<Option<T>> {
    next_if_rule(pairs, rule).map(f).transpose()
}

pub(super) fn exactly_one<T>(iter: impl IntoIterator<Item = T>) -> T {
    let mut iter = iter.into_iter();
    let res = iter.next().unwrap();
    debug_assert!(matches!(iter.next(), None));
    res
}

pub(super) fn block_string_value(raw: &str) -> String {
    // Split the string by either \r\n, \r or \n
    let lines: Vec<_> = raw
        .split("\r\n")
        .flat_map(|s| s.split(['\r', '\n'].as_ref()))
        .collect();

    // Find the common indent
    let common_indent = lines
        .iter()
        .skip(1)
        .copied()
        .filter_map(|line| line.find(|c| c != '\t' && c != ' '))
        .min()
        .unwrap_or(0);

    let line_has_content = |line: &str| line.as_bytes().iter().any(|&c| c != b'\t' && c != b' ');

    let first_contentful_line = lines
        .iter()
        .copied()
        .position(line_has_content)
        .unwrap_or_else(|| lines.len());
    let ending_lines_start = lines
        .iter()
        .copied()
        .rposition(line_has_content)
        .map_or(0, |i| i + 1);

    lines
        .iter()
        .copied()
        .enumerate()
        .take(ending_lines_start)
        .skip(first_contentful_line)
        // Remove the common indent, but not on the first line
        .map(|(i, line)| if i == 0 { line } else { &line[common_indent..] })
        // Put a newline between each line
        .enumerate()
        .flat_map(|(i, line)| {
            if i == 0 { [].as_ref() } else { ['\n'].as_ref() }
                .iter()
                .copied()
                .chain(line.chars())
        })
        .collect()
}

#[test]
fn test_block_string_value() {
    assert_eq!(block_string_value(""), "");
    assert_eq!(block_string_value("\r\n"), "");
    assert_eq!(block_string_value("\r\r\r\r\n\n\r\n\r\r"), "");
    assert_eq!(block_string_value("abc"), "abc");
    assert_eq!(
        block_string_value("line 1\r\n   line 2\n     line 3\r    line 4"),
        "line 1\nline 2\n  line 3\n line 4"
    );
    assert_eq!(
        block_string_value("\r\r  some text\r\n \n \n "),
        "some text"
    );
}

pub(super) fn string_value(s: &str) -> String {
    let mut chars = s.chars();

    std::iter::from_fn(|| {
        Some(match chars.next()? {
            '\\' => match chars.next().expect("backslash at end") {
                c @ '\"' | c @ '\\' | c @ '/' => c,
                'b' => '\x08',
                'f' => '\x0C',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                'u' => std::char::from_u32(
                    (0..4)
                        .map(|_| chars.next().unwrap().to_digit(16).unwrap())
                        .fold(0, |acc, digit| acc * 16 + digit),
                )
                .unwrap(),
                _ => unreachable!(),
            },
            other => other,
        })
    })
    .collect()
}

#[test]
fn test_string_value() {
    assert_eq!(string_value("abc"), "abc");
    assert_eq!(string_value("\\n\\b\\u2a1A"), "\n\x08\u{2A1A}");
    assert_eq!(string_value("\\\"\\\\"), "\"\\");
}
