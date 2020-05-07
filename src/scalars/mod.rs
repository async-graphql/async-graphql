mod any;
mod bool;
mod chrono_tz;
mod datetime;
mod floats;
mod id;
mod integers;
mod json;
mod string;
mod url;

#[cfg(feature = "bson")]
mod bson;
#[cfg(feature = "uuid")]
mod uuid;

pub use any::Any;
pub use id::ID;
pub use json::Json;

#[cfg(test)]
mod tests {
    use super::ID;
    use crate::Type;
    use bson::oid::ObjectId;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    #[test]
    fn test_scalar_type() {
        assert_eq!(<bool as Type>::type_name(), "Boolean");
        assert_eq!(<bool as Type>::qualified_type_name(), "Boolean!");

        assert_eq!(<i32 as Type>::type_name(), "Int");
        assert_eq!(<i32 as Type>::qualified_type_name(), "Int!");

        assert_eq!(<f32 as Type>::type_name(), "Float");
        assert_eq!(<f32 as Type>::qualified_type_name(), "Float!");

        assert_eq!(<&str as Type>::type_name(), "String");
        assert_eq!(<&str as Type>::qualified_type_name(), "String!");

        assert_eq!(<String as Type>::type_name(), "String");
        assert_eq!(<String as Type>::qualified_type_name(), "String!");

        assert_eq!(<ID as Type>::type_name(), "ID");
        assert_eq!(<ID as Type>::qualified_type_name(), "ID!");

        assert_eq!(<DateTime::<Utc> as Type>::type_name(), "DateTime");
        assert_eq!(
            <DateTime::<Utc> as Type>::qualified_type_name(),
            "DateTime!"
        );

        #[cfg(feature = "uuid")]
        {
            assert_eq!(<Uuid as Type>::type_name(), "UUID");
            assert_eq!(<Uuid as Type>::qualified_type_name(), "UUID!");
        }

        #[cfg(feature = "bson")]
        {
            assert_eq!(<ObjectId as Type>::type_name(), "ObjectId");
            assert_eq!(<ObjectId as Type>::qualified_type_name(), "ObjectId!");
        }
    }
}
