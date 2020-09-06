use crate::Pos;
use pest::iterators::Pair;
use pest::RuleType;
use std::str::Chars;

pub struct PositionCalculator<'a> {
    input: Chars<'a>,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> PositionCalculator<'a> {
    pub fn new(input: &'a str) -> PositionCalculator<'a> {
        Self {
            input: input.chars(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn step<R: RuleType>(&mut self, pair: &Pair<R>) -> Pos {
        let pos = pair.as_span().start();
        debug_assert!(pos >= self.pos);
        for _ in 0..pos - self.pos {
            match self.input.next() {
                Some('\r') => {
                    self.column = 1;
                }
                Some('\n') => {
                    self.line += 1;
                    self.column = 1;
                }
                Some(_) => {
                    self.column += 1;
                }
                None => break,
            }
        }
        self.pos = pos;
        Pos {
            line: self.line,
            column: self.column,
        }
    }
}

// See https://spec.graphql.org/June2018/#BlockStringValue()
pub(crate) fn block_string_value(raw: &str) -> String {
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
            if i == 0 {
                [].as_ref()
            } else {
                ['\n'].as_ref()
            }
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
    dbg!();
    assert_eq!(
        block_string_value("\r\r  some text\r\n \n \n "),
        "some text"
    );
}

pub(crate) fn string_value(s: &str) -> String {
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
