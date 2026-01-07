use jiff::{
    Span, Timestamp, Zoned,
    civil::{Date, Time},
};

use crate::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

/// The `printf`-style format string for serializing/deserializing [`Date`].
const DATE_FORMAT: &'static str = "%Y-%m-%d";

/// The `printf`-style format string for serializing/deserializing [`Time`].
const TIME_FORMAT: &'static str = "%H:%M:%S%.f";

#[Scalar(internal, name = "Date")]
impl ScalarType for Date {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(Date::strptime(DATE_FORMAT, s)?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.strftime(DATE_FORMAT).to_string())
    }
}

#[Scalar(internal, name = "Time")]
impl ScalarType for Time {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(Time::strptime(TIME_FORMAT, s)?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.strftime(TIME_FORMAT).to_string())
    }
}

#[Scalar(
    internal,
    name = "DateTime",
    specified_by_url = "https://datatracker.ietf.org/doc/html/rfc3339"
)]
impl ScalarType for Timestamp {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(s.parse()?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

#[Scalar(
    internal,
    name = "ZonedDateTime",
    specified_by_url = "https://datatracker.ietf.org/doc/html/rfc8536"
)]
impl ScalarType for Zoned {
    fn parse(value: Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(s.parse()?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

#[Scalar(
    internal,
    name = "Duration",
    specified_by_url = "https://en.wikipedia.org/wiki/ISO_8601#Durations"
)]
impl ScalarType for Span {
    fn parse(value: Value) -> InputValueResult<Self> {
        match &value {
            Value::String(s) => Ok(s.parse()?),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::{ScalarType, Value};
    use jiff::{
        Span, Timestamp, ToSpan, Zoned,
        civil::{Date, Time},
    };

    #[test]
    fn test_span_to_value() {
        let cases = [
            (40.days(), "P40D"),
            (1.year().days(1), "P1Y1D"),
            (3.days().hours(4).minutes(59), "P3DT4H59M"),
            (2.hours().minutes(30), "PT2H30M"),
            (1.month(), "P1M"),
            (1.week(), "P1W"),
            (1.week().days(4), "P1W4D"),
            (1.minute(), "PT1M"),
            (2.milliseconds().microseconds(100), "PT0.0021S"),
            (0.seconds(), "PT0S"),
            (
                1.year()
                    .months(1)
                    .days(1)
                    .hours(1)
                    .minutes(1)
                    .seconds(1)
                    .milliseconds(100),
                "P1Y1M1DT1H1M1.1S",
            ),
        ];

        for (value, expected) in cases {
            let value = value.to_value();

            if let Value::String(s) = value {
                assert_eq!(s, expected);
            } else {
                panic!("Unexpected Value type when formatting Span: {:?}", value);
            }
        }
    }

    #[test]
    fn test_span_parse() {
        let cases = [
            ("P40D", 40.days()),
            ("P1y1d", 1.year().days(1)),
            ("P3dT4h59m", 3.days().hours(4).minutes(59)),
            ("PT2H30M", 2.hours().minutes(30)),
            ("P1m", 1.month()),
            ("P1w", 1.week()),
            ("P1w4d", 1.week().days(4)),
            ("PT1m", 1.minute()),
            ("PT0.0021s", 2.milliseconds().microseconds(100)),
            ("PT0s", 0.seconds()),
            (
                "P1y1m1dT1h1m1.1s",
                1.year()
                    .months(1)
                    .days(1)
                    .hours(1)
                    .minutes(1)
                    .seconds(1)
                    .milliseconds(100),
            ),
        ];

        for (value, expected) in cases {
            let value = Value::String(value.to_string());
            let parsed = <Span as ScalarType>::parse(value).unwrap();
            assert_eq!(parsed.fieldwise(), expected.fieldwise());
        }
    }

    #[test]
    fn test_zoned_to_value() {
        let cases = [
            (
                "2022-01-12T04:00:19.12345+00:00[UTC]"
                    .parse::<Zoned>()
                    .unwrap(),
                "2022-01-12T04:00:19.12345+00:00[UTC]",
            ),
            (
                "2022-01-12T04:00:19.12345-05:00[America/New_York]"
                    .parse::<Zoned>()
                    .unwrap(),
                "2022-01-12T04:00:19.12345-05:00[America/New_York]",
            ),
        ];

        for (value, expected) in cases {
            let value = value.to_value();

            if let Value::String(s) = value {
                assert_eq!(s, expected);
            } else {
                panic!("Unexpected Value type when formatting Zoned: {:?}", value);
            }
        }
    }

    #[test]
    fn test_zoned_parse() {
        let cases = [
            (
                "2022-01-12T04:00:19.12345+00:00[UTC]",
                "2022-01-12T04:00:19.12345+00:00[UTC]"
                    .parse::<Zoned>()
                    .unwrap(),
            ),
            (
                "2022-01-12T04:00:19.12345-05:00[America/New_York]",
                "2022-01-12T04:00:19.12345-05:00[America/New_York]"
                    .parse::<Zoned>()
                    .unwrap(),
            ),
        ];

        for (value, expected) in cases {
            let value = Value::String(value.to_string());
            let parsed = <Zoned as ScalarType>::parse(value).unwrap();
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn test_timestamp_to_value() {
        let cases = [
            (
                "2022-01-12T04:00:19.12345Z".parse::<Timestamp>().unwrap(),
                "2022-01-12T04:00:19.12345Z",
            ),
            (
                "2022-01-12T07:30:19Z".parse::<Timestamp>().unwrap(),
                "2022-01-12T07:30:19Z",
            ),
        ];
        for (value, expected) in cases {
            let value = value.to_value();

            if let Value::String(s) = value {
                assert_eq!(s, expected);
            } else {
                panic!(
                    "Unexpected Value type when formatting Timestamp: {:?}",
                    value
                );
            }
        }
    }

    #[test]
    fn test_timestamp_parse() {
        let cases = [
            (
                "2022-01-12T04:00:19.12345Z",
                "2022-01-12T04:00:19.12345Z".parse::<Timestamp>().unwrap(),
            ),
            (
                "2022-01-12T07:30:19Z",
                "2022-01-12T07:30:19Z".parse::<Timestamp>().unwrap(),
            ),
        ];
        for (value, expected) in cases {
            let value = Value::String(value.to_string());
            let parsed = <Timestamp as ScalarType>::parse(value).unwrap();
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn test_date_to_value() {
        let cases = [
            ("2022-01-12".parse::<Date>().unwrap(), "2022-01-12"),
            ("2023-12-31".parse::<Date>().unwrap(), "2023-12-31"),
        ];
        for (value, expected) in cases {
            let value = value.to_value();

            if let Value::String(s) = value {
                assert_eq!(s, expected);
            } else {
                panic!("Unexpected Value type when formatting Date: {:?}", value);
            }
        }
    }

    #[test]
    fn test_time_to_value() {
        let cases = [
            ("04:00:19.12345".parse::<Time>().unwrap(), "04:00:19.12345"),
            ("07:30:19".parse::<Time>().unwrap(), "07:30:19"),
        ];
        for (value, expected) in cases {
            let value = value.to_value();

            if let Value::String(s) = value {
                assert_eq!(s, expected);
            } else {
                panic!("Unexpected Value type when formatting Time: {:?}", value);
            }
        }
    }

    #[test]
    fn test_time_parse() {
        let cases = [
            ("04:00:19.12345", "04:00:19.12345".parse::<Time>().unwrap()),
            ("07:30:19", "07:30:19".parse::<Time>().unwrap()),
        ];
        for (value, expected) in cases {
            let value = Value::String(value.to_string());
            let parsed = <Time as ScalarType>::parse(value).unwrap();
            assert_eq!(parsed, expected);
        }
    }
}
