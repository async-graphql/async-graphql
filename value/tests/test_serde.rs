use std::{collections::BTreeMap, fmt::Debug};

use async_graphql_value::*;
use bytes::Bytes;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

fn test_value<T: Serialize + DeserializeOwned + Clone + PartialEq + Debug>(value: T) {
    assert_eq!(
        from_value::<T>(to_value(value.clone()).unwrap()).unwrap(),
        value
    )
}

#[test]
fn test_serde() {
    test_value(true);
    test_value(100i32);
    test_value(1.123f64);
    test_value(Some(100i32));
    test_value(ConstValue::Null);
    test_value(vec![0i32, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    test_value(b"123456".to_vec());

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
    struct NewType(i32);
    test_value(NewType(100i32));

    #[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash, Copy, Clone, Ord, PartialOrd)]
    enum Enum {
        A,
        B,
    }
    test_value(Enum::A);
    test_value(Enum::B);

    let mut obj = BTreeMap::<Name, ConstValue>::new();
    obj.insert(Name::new("A"), ConstValue::Number(10.into()));
    obj.insert(Name::new("B"), ConstValue::Number(20.into()));
    test_value(obj);

    let mut obj = BTreeMap::<Enum, ConstValue>::new();
    obj.insert(Enum::A, ConstValue::Number(10.into()));
    obj.insert(Enum::B, ConstValue::Number(20.into()));
    test_value(obj);

    #[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
    struct Struct {
        a: i32,
        b: Option<Enum>,
    }
    test_value(Struct {
        a: 100,
        b: Some(Enum::B),
    });
}

#[test]
fn test_binary() {
    assert_eq!(
        to_value(Bytes::from_static(b"123456")).unwrap(),
        ConstValue::Binary(Bytes::from_static(b"123456"))
    );

    assert_eq!(
        from_value::<Bytes>(ConstValue::Binary(Bytes::from_static(b"123456"))).unwrap(),
        Bytes::from_static(b"123456")
    );
}

#[cfg(feature = "raw_value")]
#[test]
fn test_raw_value() {
    use indexmap::IndexMap;
    use serde_json::value::RawValue;

    #[derive(Serialize)]
    struct Struct {
        field: Box<RawValue>,
    }

    let object = Struct {
        field: RawValue::from_string("[0, 1, 2]".into()).unwrap(),
    };

    let value = to_value(&object).unwrap();
    assert_eq!(
        value,
        ConstValue::Object({
            let mut map = IndexMap::default();
            map.insert(
                Name::new("field"),
                ConstValue::Object({
                    let mut map = IndexMap::default();
                    map.insert(
                        Name::new(RAW_VALUE_TOKEN),
                        ConstValue::String("[0, 1, 2]".into()),
                    );
                    map
                }),
            );
            map
        })
    );

    let value = serde_json::to_string(&value).unwrap();
    assert_eq!(value, r#"{"field":[0, 1, 2]}"#);
}

// Regression test for https://github.com/async-graphql/async-graphql/issues/1719.
//
// This crate's `dev-dependencies` enable `serde_json`'s `arbitrary_precision`
// feature (see `Cargo.toml`), which is what actually reproduces the bug: with
// it on, `serde_json` hands every fractional number to the `Deserialize`
// implementation as a single-entry map keyed by
// `$serde_json::private::Number` rather than calling `visit_f64`, since
// serde's data model has no arbitrary-precision-number primitive. Before the
// fix, `ConstValue`/`Value`'s `visit_map` had no case for that token and
// produced an `Object` wrapping the number's text instead of a `Number`,
// which is exactly what made GraphQL variables carrying a fractional number
// fail with "Invalid value for argument ..., expected type Float".
#[test]
fn test_arbitrary_precision_fractional_number() {
    let json = r#"{"latitude": 12.34, "count": 5, "negative": -0.5}"#;

    let value: ConstValue = serde_json::from_str(json).unwrap();
    let ConstValue::Object(map) = &value else {
        panic!("expected an object, got {value:?}");
    };
    assert_eq!(
        map.get("latitude"),
        Some(&ConstValue::Number(Number::from_f64(12.34).unwrap()))
    );
    assert_eq!(map.get("count"), Some(&ConstValue::Number(5.into())));
    assert_eq!(
        map.get("negative"),
        Some(&ConstValue::Number(Number::from_f64(-0.5).unwrap()))
    );

    // `Value` (the non-const variant used while parsing queries/variables)
    // must be fixed identically.
    let value: Value = serde_json::from_str(json).unwrap();
    let Value::Object(map) = &value else {
        panic!("expected an object, got {value:?}");
    };
    assert_eq!(
        map.get("latitude"),
        Some(&Value::Number(Number::from_f64(12.34).unwrap()))
    );

    // A bare top-level fractional number (as would arrive for a single
    // variable value) must also round-trip to a `Number`, not an `Object`.
    let value: ConstValue = serde_json::from_str("42.5").unwrap();
    assert_eq!(value, ConstValue::Number(Number::from_f64(42.5).unwrap()));
}
