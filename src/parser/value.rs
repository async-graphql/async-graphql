use std::collections::BTreeMap;
use std::fmt;

/// Represents a GraphQL value
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub enum Value {
    Null,
    Variable(String),
    Int(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Enum(String),
    List(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;

        match (self, other) {
            (Variable(a), Variable(b)) => a.eq(b),
            (Int(a), Int(b)) => a.eq(b),
            (Float(a), Float(b)) => a.eq(b),
            (String(a), String(b)) => a.eq(b),
            (Boolean(a), Boolean(b)) => a.eq(b),
            (Null, Null) => true,
            (Enum(a), Enum(b)) => a.eq(b),
            (List(a), List(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for i in 0..a.len() {
                    if !a[i].eq(&b[i]) {
                        return false;
                    }
                }
                true
            }
            (Object(a), Object(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (key, a_value) in a.iter() {
                    if let Some(b_value) = b.get(key) {
                        if !a_value.eq(b_value) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            _ => false,
        }
    }
}

fn write_quoted(s: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "\"")?;
    for c in s.chars() {
        match c {
            '\r' => write!(f, "\r")?,
            '\n' => writeln!(f)?,
            '\t' => write!(f, "\t")?,
            '"' => write!(f, "\"")?,
            '\\' => write!(f, "\\")?,
            '\u{0020}'..='\u{FFFF}' => write!(f, "{}", c)?,
            _ => write!(f, "\\u{:04}", c as u32).unwrap(),
        }
    }
    write!(f, "\"")
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Variable(name) => write!(f, "${}", name),
            Value::Int(num) => write!(f, "{}", *num),
            Value::Float(val) => write!(f, "{}", *val),
            Value::String(ref val) => write_quoted(val, f),
            Value::Boolean(true) => write!(f, "true"),
            Value::Boolean(false) => write!(f, "false"),
            Value::Null => write!(f, "null"),
            Value::Enum(ref name) => write!(f, "{}", name),
            Value::List(ref items) => {
                write!(f, "[")?;
                if !items.is_empty() {
                    write!(f, "{}", items[0])?;
                    for item in &items[1..] {
                        write!(f, ", ")?;
                        write!(f, "{}", item)?;
                    }
                }
                write!(f, "]")
            }
            Value::Object(items) => {
                write!(f, "{{")?;
                let mut first = true;
                for (name, value) in items {
                    if first {
                        first = false;
                    } else {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", name)?;
                    write!(f, ": ")?;
                    write!(f, "{}", value)?;
                }
                write!(f, "}}")
            }
        }
    }
}
