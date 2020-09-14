use crate::{Error, QueryError, Response};
use itertools::Itertools;
use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Serialize, Serializer};

impl Serialize for Response {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match &self.error {
            None => {
                let mut s = serializer
                    .serialize_struct("Response", if self.extensions.is_some() { 2 } else { 1 })?;
                s.serialize_field("data", &self.data)?;
                if let Some(extensions) = &self.extensions {
                    s.serialize_field("extensions", extensions)?;
                }
                s.end()
            }
            Some(err) => {
                let mut s = serializer.serialize_struct("Response", 1)?;
                s.serialize_field("errors", err)?;
                s.end()
            }
        }
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Error::Parse(err) => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                seq.serialize_element(&serde_json::json! ({
                    "message": err.message,
                    "locations": [{"line": err.pos.line, "column": err.pos.column}]
                }))?;
                seq.end()
            }
            Error::Query { pos, path, err } => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                if let QueryError::FieldError {
                    err,
                    extended_error,
                } = err
                {
                    let mut map = serde_json::Map::new();

                    map.insert("message".to_string(), err.to_string().into());
                    map.insert(
                        "locations".to_string(),
                        serde_json::json!([{"line": pos.line, "column": pos.column}]),
                    );

                    if let Some(path) = path {
                        map.insert("path".to_string(), path.clone());
                    }

                    if let Some(obj @ serde_json::Value::Object(_)) = extended_error {
                        map.insert("extensions".to_string(), obj.clone());
                    }

                    seq.serialize_element(&serde_json::Value::Object(map))?;
                } else {
                    seq.serialize_element(&serde_json::json!({
                        "message": err.to_string(),
                        "locations": [{"line": pos.line, "column": pos.column}]
                    }))?;
                }
                seq.end()
            }
            Error::Rule { errors } => {
                let mut seq = serializer.serialize_seq(Some(errors.len()))?;
                for error in errors {
                    seq.serialize_element(&serde_json::json!({
                        "message": error.message,
                        "locations": error.locations.iter().map(|pos| serde_json::json!({"line": pos.line, "column": pos.column})).collect_vec(),
                    }))?;
                }
                seq.end()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pos;
    use serde_json::json;

    #[test]
    fn test_response_data() {
        let resp = Response {
            data: json!({"ok": true}),
            extensions: None,
            cache_control: Default::default(),
            error: None,
        };
        assert_eq!(
            serde_json::to_value(resp).unwrap(),
            json! ({
                "data": {
                    "ok": true,
                }
            })
        );
    }

    #[test]
    fn test_field_error_with_extension() {
        let resp = Response::from(Error::Query {
            pos: Pos {
                line: 10,
                column: 20,
            },
            path: None,
            err: QueryError::FieldError {
                err: "MyErrorMessage".to_owned(),
                extended_error: Some(json!({
                    "code": "MY_TEST_CODE"
                })),
            },
        });

        assert_eq!(
            serde_json::to_value(resp).unwrap(),
            json!({
                "errors": [{
                    "message":"MyErrorMessage",
                    "extensions": {
                        "code": "MY_TEST_CODE"
                    },
                    "locations": [{"line": 10, "column": 20}]
                }]
            })
        );
    }

    #[test]
    fn test_response_error_with_pos() {
        let resp = Response::from(Error::Query {
            pos: Pos {
                line: 10,
                column: 20,
            },
            path: None,
            err: QueryError::NotSupported,
        });
        assert_eq!(
            serde_json::to_value(resp).unwrap(),
            json!({
                "errors": [{
                    "message":"Not supported.",
                    "locations": [
                        {"line": 10, "column": 20}
                    ]
                }]
            })
        );
    }
}
