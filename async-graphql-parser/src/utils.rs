use crate::{Error, Pos, Result};
use arrayvec::ArrayVec;
use pest::iterators::Pair;
use pest::RuleType;
use std::borrow::Cow;
use std::iter::Peekable;
use std::str::Chars;

pub struct PositionCalculator<'a> {
    input: Peekable<Chars<'a>>,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> PositionCalculator<'a> {
    pub fn new(input: &'a str) -> PositionCalculator<'a> {
        Self {
            input: input.chars().peekable(),
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
                    if let Some(&'\n') = self.input.peek() {
                        self.input.next();
                        self.line += 1;
                        self.column = 1;
                    } else {
                        self.column += 1;
                    }
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

#[inline]
pub fn to_static_str(s: &str) -> &'static str {
    unsafe { (s as *const str).as_ref().unwrap() }
}

pub fn unquote_string(s: &str, pos: Pos) -> Result<Cow<'static, str>> {
    let s = if s.starts_with(r#"""""#) {
        &s[3..s.len() - 3]
    } else if s.starts_with('"') {
        &s[1..s.len() - 1]
    } else {
        unreachable!()
    };

    if !s.contains('\\') {
        return Ok(Cow::Borrowed(to_static_str(s)));
    }

    let mut chars = s.chars();
    let mut res = String::with_capacity(s.len());
    let mut temp_code_point = ArrayVec::<[u8; 4]>::new();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                match chars.next().expect("slash cant be at the end") {
                    c @ '"' | c @ '\\' | c @ '/' => res.push(c),
                    'b' => res.push('\u{0010}'),
                    'f' => res.push('\u{000C}'),
                    'n' => res.push('\n'),
                    'r' => res.push('\r'),
                    't' => res.push('\t'),
                    'u' => {
                        temp_code_point.clear();
                        for _ in 0..4 {
                            match chars.next() {
                                Some(inner_c) if inner_c.is_digit(16) => {
                                    temp_code_point.push(inner_c as u8)
                                }
                                Some(inner_c) => {
                                    return Err(Error {
                                        pos,
                                        message: format!(
                                            "{} is not a valid unicode code point",
                                            inner_c
                                        ),
                                    });
                                }
                                None => {
                                    return Err(Error {
                                        pos,
                                        message: format!(
                                            "{} must have 4 characters after it",
                                            unsafe {
                                                std::str::from_utf8_unchecked(
                                                    temp_code_point.as_slice(),
                                                )
                                            }
                                        ),
                                    });
                                }
                            }
                        }

                        // convert our hex string into a u32, then convert that into a char
                        match u32::from_str_radix(
                            unsafe { std::str::from_utf8_unchecked(temp_code_point.as_slice()) },
                            16,
                        )
                        .map(std::char::from_u32)
                        {
                            Ok(Some(unicode_char)) => res.push(unicode_char),
                            _ => {
                                return Err(Error {
                                    pos,
                                    message: format!(
                                        "{} is not a valid unicode code point",
                                        unsafe {
                                            std::str::from_utf8_unchecked(
                                                temp_code_point.as_slice(),
                                            )
                                        }
                                    ),
                                });
                            }
                        }
                    }
                    c => {
                        return Err(Error {
                            pos,
                            message: format!("bad escaped char {:?}", c),
                        });
                    }
                }
            }
            c => res.push(c),
        }
    }

    Ok(Cow::Owned(res))
}
