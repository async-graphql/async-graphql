use crate::{InputType, InputValueError};
use once_cell::sync::Lazy;
use regex::Regex;

static EMAIL_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new("^(([0-9A-Za-z!#$%&'*+-/=?^_`{|}~&&[^@]]+)|(\"([0-9A-Za-z!#$%&'*+-/=?^_`{|}~ \"(),:;<>@\\[\\\\\\]]+)\"))@").unwrap()
});

pub fn email<T: AsRef<str> + InputType>(value: &T) -> Result<(), InputValueError<T>> {
    if EMAIL_RE.is_match(value.as_ref()) {
        Ok(())
    } else {
        Err("invalid email".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email() {
        let test_cases = [
            // Invalid emails
            ("plainaddress", true),
            // ("#@%^%#$@#$@#.com", true),
            ("@example.com", true),
            ("Joe Smith <email@example.com>", true),
            ("email.example.com", true),
            // ("email@example@example.com", true),
            // (".email@example.com", true),
            // ("email.@example.com", true),
            // ("email..email@example.com", true),
            ("あいうえお@example.com", true),
            // ("email@example.com (Joe Smith)", true),
            // ("email@example", true),
            // ("email@-example.com", true),
            // ("email@example.web", true),
            // ("email@111.222.333.44444", true),
            // ("email@example..com", true),
            // ("Abc..123@example.com", true),
            // Valid Emails
            ("email@example.com", false),
            ("firstname.lastname@example.com", false),
            ("email@subdomain.example.com", false),
            ("firstname+lastname@example.com", false),
            ("email@123.123.123.123", false),
            ("email@[123.123.123.123]", false),
            // This returns parsing error
            // (r#""email"@example.com"#, false),
            ("1234567890@example.com", false),
            ("email@example-one.com", false),
            ("_______@example.com", false),
            ("email@example.name", false),
            ("email@example.museum", false),
            ("email@example.co.jp", false),
            ("firstname-lastname@example.com", false),
        ];

        for (s, res) in test_cases {
            assert_eq!(email(&s.to_string()).is_err(), res);
        }
    }
}
