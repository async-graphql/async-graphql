mod bool;
mod floats;
mod id;
mod integers;
mod string;

#[cfg(feature = "chrono")]
mod datetime;
#[cfg(feature = "uuid")]
mod uuid;

pub use id::ID;

#[cfg(test)]
mod tests {
    use super::ID;
    use crate::GQLType;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    #[test]
    fn test_scalar_type() {
        assert_eq!(<bool as GQLType>::type_name(), "Boolean");
        assert_eq!(<bool as GQLType>::qualified_type_name(), "Boolean!");

        assert_eq!(<i32 as GQLType>::type_name(), "Int");
        assert_eq!(<i32 as GQLType>::qualified_type_name(), "Int!");

        assert_eq!(<f32 as GQLType>::type_name(), "Float");
        assert_eq!(<f32 as GQLType>::qualified_type_name(), "Float!");

        assert_eq!(<&str as GQLType>::type_name(), "String");
        assert_eq!(<&str as GQLType>::qualified_type_name(), "String!");

        assert_eq!(<String as GQLType>::type_name(), "String");
        assert_eq!(<String as GQLType>::qualified_type_name(), "String!");

        assert_eq!(<ID as GQLType>::type_name(), "ID");
        assert_eq!(<ID as GQLType>::qualified_type_name(), "ID!");

        #[cfg(feature = "chrono")]
        {
            assert_eq!(<DateTime::<Utc> as GQLType>::type_name(), "DateTime");
            assert_eq!(
                <DateTime::<Utc> as GQLType>::qualified_type_name(),
                "DateTime!"
            );
        }

        #[cfg(feature = "uuid")]
        {
            assert_eq!(<Uuid as GQLType>::type_name(), "UUID");
            assert_eq!(<Uuid as GQLType>::qualified_type_name(), "UUID!");
        }
    }
}
