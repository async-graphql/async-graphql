use async_graphql::{InputValueError, Scalar, ScalarType, Value};

/// Email addresss newtype
///
/// Can be used as a scalar to parse a string into a newtype
///
/// ```ignore
/// async fn handler(&self, EmailAddress(email): EmailAddress) {
/// }
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "email-address")))]
pub struct EmailAddress(pub email_address::EmailAddress);

#[Scalar(
    name = "EmailAddress",
    specified_by_url = "https://en.wikipedia.org/wiki/Email_address#Syntax"
)]
impl ScalarType for EmailAddress {
    fn parse(value: Value) -> Result<Self, InputValueError<Self>> {
        if let Value::String(string) = &value {
            Ok(Self(string.parse()?))
        } else {
            Err(InputValueError::expected_type(value))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.0.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn email(email: &str) -> Result<EmailAddress, InputValueError<EmailAddress>> {
        EmailAddress::parse(Value::String(email.into()))
    }

    #[test]
    fn test_email() {
        assert!(email("joe@example.com").is_ok());
        assert!(email("joe.test@example.com").is_ok());
        assert!(email("email@example-one.com").is_ok());
        assert!(email("1234567890@example.com").is_ok());

        assert!(email("plainaddress").is_err());
        assert!(email("@example.com").is_err());
        assert!(email("email.example.com").is_err());
    }
}
