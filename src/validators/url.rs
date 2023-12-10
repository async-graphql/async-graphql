use std::str::FromStr;

#[cfg(feature = "http02")]
use http02 as http;
#[cfg(not(feature = "http02"))]
use http1 as http;

use crate::{InputType, InputValueError};

pub fn url<T: AsRef<str> + InputType>(value: &T) -> Result<(), InputValueError<T>> {
    if let Ok(true) = http::uri::Uri::from_str(value.as_ref())
        .map(|uri| uri.scheme().is_some() && uri.authority().is_some())
    {
        Ok(())
    } else {
        Err("invalid url".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url() {
        assert!(url(&"http".to_string()).is_err());
        assert!(url(&"https://google.com".to_string()).is_ok());
        assert!(url(&"http://localhost:80".to_string()).is_ok());
        assert!(url(&"ftp://localhost:80".to_string()).is_ok());
    }
}
