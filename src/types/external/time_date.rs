use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use time::{format_description::FormatItem, macros::format_description, Date};

const DATE_FORMAT: &[FormatItem<'_>] = format_description!("[year]-[month]-[day]");

/// ISO 8601 calendar date without timezone.
/// Format: %Y-%m-%d
///
/// # Examples
///
/// * `1994-11-13`
/// * `2000-02-24`
#[Scalar(internal, name = "Date")]
impl ScalarType for Date {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::String(s) => Ok(Self::parse(s, &DATE_FORMAT)?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(
            self.format(&DATE_FORMAT)
                .unwrap_or_else(|e| panic!("Failed to format `Date`: {}", e)),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{ScalarType, Value};
    use time::{macros::date, Date};

    #[test]
    fn test_date_to_value() {
        let cases = [
            (date!(1994 - 11 - 13), "1994-11-13"),
            (date!(2000 - 01 - 24), "2000-01-24"),
        ];
        for (value, expected) in cases {
            let value = value.to_value();

            if let Value::String(s) = value {
                assert_eq!(s, expected);
            } else {
                panic!(
                    "Unexpected Value type when formatting PrimitiveDateTime: {:?}",
                    value
                );
            }
        }
    }

    #[test]
    fn test_date_parse() {
        let cases = [
            ("1994-11-13", date!(1994 - 11 - 13)),
            ("2000-01-24", date!(2000 - 01 - 24)),
        ];
        for (value, expected) in cases {
            let value = Value::String(value.to_string());
            let parsed = <Date as ScalarType>::parse(value).unwrap();
            assert_eq!(parsed, expected);
        }
    }
}
