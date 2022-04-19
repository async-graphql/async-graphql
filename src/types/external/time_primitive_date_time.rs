use time::{format_description::FormatItem, macros::format_description, PrimitiveDateTime};

use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

const PRIMITIVE_DATE_TIME_FORMAT: &[FormatItem<'_>] =
    format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond]");

/// A local datetime without timezone offset.
///
/// The input/output is a string in ISO 8601 format without timezone, including
/// subseconds. E.g. "2022-01-12T07:30:19.12345".
#[Scalar(internal, name = "LocalDateTime")]
impl ScalarType for PrimitiveDateTime {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::String(s) => Ok(Self::parse(s, &PRIMITIVE_DATE_TIME_FORMAT)?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(
            self.format(&PRIMITIVE_DATE_TIME_FORMAT)
                .unwrap_or_else(|e| panic!("Failed to format `PrimitiveDateTime`: {}", e)),
        )
    }
}

#[cfg(test)]
mod tests {
    use time::{macros::datetime, PrimitiveDateTime};

    use crate::{ScalarType, Value};

    #[test]
    fn test_primitive_date_time_to_value() {
        let cases = [
            (
                datetime!(2022-01-12 07:30:19.12345),
                "2022-01-12T07:30:19.12345",
            ),
            (datetime!(2022-01-12 07:30:19), "2022-01-12T07:30:19.0"),
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
    fn test_primitive_date_time_parse() {
        let cases = [
            (
                "2022-01-12T07:30:19.12345",
                datetime!(2022-01-12 07:30:19.12345),
            ),
            ("2022-01-12T07:30:19.0", datetime!(2022-01-12 07:30:19)),
        ];
        for (value, expected) in cases {
            let value = Value::String(value.to_string());
            let parsed = <PrimitiveDateTime as ScalarType>::parse(value).unwrap();
            assert_eq!(parsed, expected);
        }
    }
}
