use uuid::Uuid;

use crate::{InputType, InputValueError};

pub fn uuid<T: AsRef<str> + InputType>(
    value: &T,
    version_option: Option<usize>,
) -> Result<(), InputValueError<T>> {
    match Uuid::try_parse(value.as_ref()) {
        Ok(uuid) => {
            if let Some(version) = version_option {
                if uuid.get_version_num() != version {
                    return Err(InputValueError::custom("UUID version mismatch"));
                }
            }
            Ok(())
        }
        Err(_) => Err(InputValueError::custom("Invalid UUID")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid() {
        assert!(uuid(&"94c59486-c302-4f43-abd7-a9c980ddab36".to_string(), None).is_ok());
        assert!(
            uuid(&"94c59486-c302-4f43-abd7-a9c980ddab3".to_string(), None).is_err_and(|e| {
                let message = format!("{:?}", e);
                println!("{}", message);
                message.contains("Invalid UUID")
            })
        );
    }

    #[test]
    fn test_uuid_version() {
        assert!(uuid(&"94c59486-c302-4f43-abd7-a9c980ddab36".to_string(), Some(4)).is_ok());
        assert!(
            uuid(&"94c59486-c302-4f43-abd7-a9c980ddab3".to_string(), Some(4)).is_err_and(|e| {
                let message = format!("{:?}", e);
                println!("{}", message);
                message.contains("Invalid UUID")
            })
        );
        assert!(
            uuid(&"94c59486-c302-5f43-abd7-a9c980ddab36".to_string(), Some(4)).is_err_and(|e| {
                let message = format!("{:?}", e);
                println!("{}", message);
                message.contains("UUID version mismatch")
            })
        );
    }
}
