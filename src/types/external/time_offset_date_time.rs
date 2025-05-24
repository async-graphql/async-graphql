use time::{OffsetDateTime, UtcOffset, format_description::well_known::Rfc3339};

use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

/// A datetime with timezone offset.
///
/// The input is a string in RFC3339 format, e.g. "2022-01-12T04:00:19.12345Z"
/// or "2022-01-12T04:00:19+03:00". The output is also a string in RFC3339
/// format, but it is always normalized to the UTC (Z) offset, e.g.
/// "2022-01-12T04:00:19.12345Z".
#[Scalar(
    internal,
    name = "DateTime",
    specified_by_url = "https://datatracker.ietf.org/doc/html/rfc3339"
)]
impl ScalarType for OffsetDateTime {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::String(s) => Ok(Self::parse(s, &Rfc3339)?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(
            self.to_offset(UtcOffset::UTC)
                .format(&Rfc3339)
                .unwrap_or_else(|e| panic!("Failed to format `OffsetDateTime`: {}", e)),
        )
    }
}

#[cfg(test)]
mod tests {
    use time::{OffsetDateTime, macros::datetime};

    use crate::{ScalarType, Value};

    #[test]
    fn test_offset_date_time_to_value() {
        let cases = [
            (
                datetime!(2022-01-12 07:30:19.12345 +3:30),
                "2022-01-12T04:00:19.12345Z",
            ),
            (datetime!(2022-01-12 07:30:19-0), "2022-01-12T07:30:19Z"),
        ];
        for (value, expected) in cases {
            let value = value.to_value();

            if let Value::String(s) = value {
                assert_eq!(s, expected);
            } else {
                panic!(
                    "Unexpected Value type when formatting OffsetDateTime: {:?}",
                    value
                );
            }
        }
    }

    #[test]
    fn test_offset_date_time_parse() {
        let cases = [
            (
                "2022-01-12T04:00:19.12345Z",
                datetime!(2022-01-12 07:30:19.12345 +3:30),
            ),
            (
                "2022-01-12T23:22:19.12345-00:00",
                datetime!(2022-01-12 23:22:19.12345-0),
            ),
        ];
        for (value, expected) in cases {
            let value = Value::String(value.to_string());
            let parsed = <OffsetDateTime as ScalarType>::parse(value).unwrap();
            assert_eq!(parsed, expected);
        }
    }
}
