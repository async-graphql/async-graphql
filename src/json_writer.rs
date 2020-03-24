#![allow(missing_docs)]

use serde_json::ser::CharEscape;

const BB: u8 = b'b'; // \x08
const TT: u8 = b't'; // \x09
const NN: u8 = b'n'; // \x0A
const FF: u8 = b'f'; // \x0C
const RR: u8 = b'r'; // \x0D
const QU: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const UU: u8 = b'u'; // \x00...\x1F except the ones above
const __: u8 = 0;

#[inline]
fn from_escape_table(escape: u8, byte: u8) -> CharEscape {
    match escape {
        self::BB => CharEscape::Backspace,
        self::TT => CharEscape::Tab,
        self::NN => CharEscape::LineFeed,
        self::FF => CharEscape::FormFeed,
        self::RR => CharEscape::CarriageReturn,
        self::QU => CharEscape::Quote,
        self::BS => CharEscape::ReverseSolidus,
        self::UU => CharEscape::AsciiControl(byte),
        _ => unreachable!(),
    }
}

static ESCAPE: [u8; 256] = [
    //   1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    UU, UU, UU, UU, UU, UU, UU, UU, BB, TT, NN, UU, FF, RR, UU, UU, // 0
    UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, UU, // 1
    __, __, QU, __, __, __, __, __, __, __, __, __, __, __, __, __, // 2
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 3
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
    __, __, __, __, __, __, __, __, __, __, __, __, BS, __, __, __, // 5
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 6
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 7
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // A
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // B
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // C
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // D
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // E
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // F
];

#[inline]
fn write_char_escape(data: &mut Vec<u8>, char_escape: CharEscape) {
    use self::CharEscape::*;

    let s = match char_escape {
        Quote => b"\\\"",
        ReverseSolidus => b"\\\\",
        Solidus => b"\\/",
        Backspace => b"\\b",
        FormFeed => b"\\f",
        LineFeed => b"\\n",
        CarriageReturn => b"\\r",
        Tab => b"\\t",
        AsciiControl(byte) => {
            static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
            let bytes = &[
                b'\\',
                b'u',
                b'0',
                b'0',
                HEX_DIGITS[(byte >> 4) as usize],
                HEX_DIGITS[(byte & 0xF) as usize],
            ];
            data.extend_from_slice(bytes);
            return;
        }
    };
    data.extend_from_slice(s);
}

/// JSON Writer
#[derive(Default)]
pub struct JsonWriter {
    data: Vec<u8>,
}

impl JsonWriter {
    #[inline]
    pub fn into_string(self) -> String {
        unsafe { String::from_utf8_unchecked(self.data) }
    }

    #[inline]
    pub fn null(&mut self) {
        self.data.extend_from_slice(b"null");
    }

    #[inline]
    pub fn bool(&mut self, value: bool) {
        if value {
            self.data.extend_from_slice(b"true");
        } else {
            self.data.extend_from_slice(b"false");
        }
    }

    #[inline]
    pub fn int(&mut self, value: i64) {
        let mut buffer = itoa::Buffer::new();
        self.data.extend_from_slice(buffer.format(value).as_bytes());
    }

    #[inline]
    pub fn float(&mut self, value: f64) {
        let mut buffer = ryu::Buffer::new();
        let s = buffer.format_finite(value);
        self.data.extend_from_slice(s.as_bytes());
    }

    #[inline]
    pub fn string(&mut self, value: &str) {
        self.data.push(b'"');
        let bytes = value.as_bytes();

        let mut start = 0;

        for (i, &byte) in bytes.iter().enumerate() {
            let escape = ESCAPE[byte as usize];
            if escape == 0 {
                continue;
            }

            if start < i {
                self.data.extend_from_slice(&bytes[start..i]);
            }

            let char_escape = from_escape_table(escape, byte);
            write_char_escape(&mut self.data, char_escape);

            start = i + 1;
        }

        if start != bytes.len() {
            self.data.extend_from_slice(&bytes[start..]);
        }
        self.data.push(b'"');
    }

    #[inline]
    pub fn begin_array(&mut self) {
        self.data.push(b'[');
    }

    #[inline]
    pub fn end_array(&mut self) {
        if let Some(lc) = self.data.last_mut() {
            if *lc == b',' {
                *lc = b']';
                return;
            }
        }
        self.data.push(b']');
    }

    #[inline]
    pub fn begin_array_value(&mut self) {}

    #[inline]
    pub fn end_array_value(&mut self) {
        self.data.push(b',');
    }

    #[inline]
    pub fn begin_object(&mut self) {
        self.data.push(b'{');
    }

    #[inline]
    pub fn end_object(&mut self) {
        if let Some(lc) = self.data.last_mut() {
            if *lc == b',' {
                *lc = b'}';
                return;
            }
        }
        self.data.push(b'}');
    }

    #[inline]
    pub fn begin_object_key(&mut self) {}

    #[inline]
    pub fn end_object_key(&mut self) {}

    #[inline]
    pub fn begin_object_value(&mut self) {
        self.data.push(b':');
    }

    #[inline]
    pub fn end_object_value(&mut self) {
        self.data.push(b',');
    }
}
