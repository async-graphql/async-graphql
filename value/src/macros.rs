/// Construct a `ConstValue`.
#[macro_export]
macro_rules! value {
    ($($json:tt)+) => {
        $crate::value_internal!($($json)+)
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! value_internal {
    // Done with trailing comma.
    (@array [$($elems:expr,)*]) => {
        $crate::value_internal_vec![$($elems,)*]
    };

    // Done without trailing comma.
    (@array [$($elems:expr),*]) => {
        $crate::value_internal_vec![$($elems),*]
    };

    // Next element is `null`.
    (@array [$($elems:expr,)*] null $($rest:tt)*) => {
        $crate::value_internal!(@array [$($elems,)* $crate::value_internal!(null)] $($rest)*)
    };

    // Next element is `true`.
    (@array [$($elems:expr,)*] true $($rest:tt)*) => {
        $crate::value_internal!(@array [$($elems,)* $crate::value_internal!(true)] $($rest)*)
    };

    // Next element is `false`.
    (@array [$($elems:expr,)*] false $($rest:tt)*) => {
        $crate::value_internal!(@array [$($elems,)* $crate::value_internal!(false)] $($rest)*)
    };

    // Next element is an array.
    (@array [$($elems:expr,)*] [$($array:tt)*] $($rest:tt)*) => {
        $crate::value_internal!(@array [$($elems,)* $crate::value_internal!([$($array)*])] $($rest)*)
    };

    // Next element is a map.
    (@array [$($elems:expr,)*] {$($map:tt)*} $($rest:tt)*) => {
        $crate::value_internal!(@array [$($elems,)* $crate::value_internal!({$($map)*})] $($rest)*)
    };

    // Next element is an expression followed by comma.
    (@array [$($elems:expr,)*] $next:expr, $($rest:tt)*) => {
        $crate::value_internal!(@array [$($elems,)* $crate::value_internal!($next),] $($rest)*)
    };

    // Last element is an expression with no trailing comma.
    (@array [$($elems:expr,)*] $last:expr) => {
        $crate::value_internal!(@array [$($elems,)* $crate::value_internal!($last)])
    };

    // Comma after the most recent element.
    (@array [$($elems:expr),*] , $($rest:tt)*) => {
        $crate::value_internal!(@array [$($elems,)*] $($rest)*)
    };

    // Unexpected token after most recent element.
    (@array [$($elems:expr),*] $unexpected:tt $($rest:tt)*) => {
        $crate::value_unexpected!($unexpected)
    };

    // Done.
    (@object $object:ident () () ()) => {};

    // Insert the current entry followed by trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr) , $($rest:tt)*) => {
        let _ = $object.insert($crate::Name::new($($key)+), $value);
        $crate::value_internal!(@object $object () ($($rest)*) ($($rest)*));
    };

    // Current entry followed by unexpected token.
    (@object $object:ident [$($key:tt)+] ($value:expr) $unexpected:tt $($rest:tt)*) => {
        $crate::value_unexpected!($unexpected);
    };

    // Insert the last entry without trailing comma.
    (@object $object:ident [$($key:tt)+] ($value:expr)) => {
        let _ = $object.insert($crate::Name::new($($key)+), $value);
    };

    // Next value is `null`.
    (@object $object:ident ($($key:tt)+) (: null $($rest:tt)*) $copy:tt) => {
        $crate::value_internal!(@object $object [$($key)+] ($crate::value_internal!(null)) $($rest)*);
    };

    // Next value is `true`.
    (@object $object:ident ($($key:tt)+) (: true $($rest:tt)*) $copy:tt) => {
        $crate::value_internal!(@object $object [$($key)+] ($crate::value_internal!(true)) $($rest)*);
    };

    // Next value is `false`.
    (@object $object:ident ($($key:tt)+) (: false $($rest:tt)*) $copy:tt) => {
        $crate::value_internal!(@object $object [$($key)+] ($crate::value_internal!(false)) $($rest)*);
    };

    // Next value is an array.
    (@object $object:ident ($($key:tt)+) (: [$($array:tt)*] $($rest:tt)*) $copy:tt) => {
        $crate::value_internal!(@object $object [$($key)+] ($crate::value_internal!([$($array)*])) $($rest)*);
    };

    // Next value is a map.
    (@object $object:ident ($($key:tt)+) (: {$($map:tt)*} $($rest:tt)*) $copy:tt) => {
        $crate::value_internal!(@object $object [$($key)+] ($crate::value_internal!({$($map)*})) $($rest)*);
    };

    // Next value is an expression followed by comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr , $($rest:tt)*) $copy:tt) => {
        $crate::value_internal!(@object $object [$($key)+] ($crate::value_internal!($value)) , $($rest)*);
    };

    // Last value is an expression with no trailing comma.
    (@object $object:ident ($($key:tt)+) (: $value:expr) $copy:tt) => {
        $crate::value_internal!(@object $object [$($key)+] ($crate::value_internal!($value)));
    };

    // Missing value for last entry. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)+) (:) $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::value_internal!();
    };

    // Missing colon and value for last entry. Trigger a reasonable error
    // message.
    (@object $object:ident ($($key:tt)+) () $copy:tt) => {
        // "unexpected end of macro invocation"
        $crate::value_internal!();
    };

    // Misplaced colon. Trigger a reasonable error message.
    (@object $object:ident () (: $($rest:tt)*) ($colon:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `:`".
        $crate::value_unexpected!($colon);
    };

    // Found a comma inside a key. Trigger a reasonable error message.
    (@object $object:ident ($($key:tt)*) (, $($rest:tt)*) ($comma:tt $($copy:tt)*)) => {
        // Takes no arguments so "no rules expected the token `,`".
        $crate::value_unexpected!($comma);
    };

    // Key is fully parenthesized. This avoids clippy double_parens false
    // positives because the parenthesization may be necessary here.
    (@object $object:ident () (($key:expr) : $($rest:tt)*) $copy:tt) => {
        $crate::value_internal!(@object $object ($key) (: $($rest)*) (: $($rest)*));
    };

    // Refuse to absorb colon token into key expression.
    (@object $object:ident ($($key:tt)*) (: $($unexpected:tt)+) $copy:tt) => {
        $crate::value_expect_expr_comma!($($unexpected)+);
    };

    // Munch a token into the current key.
    (@object $object:ident ($($key:tt)*) ($tt:tt $($rest:tt)*) $copy:tt) => {
        $crate::value_internal!(@object $object ($($key)* $tt) ($($rest)*) ($($rest)*));
    };

    //////////////////////////////////////////////////////////////////////////
    // The main implementation.
    //
    // Must be invoked as: value_internal!($($json)+)
    //////////////////////////////////////////////////////////////////////////

    (null) => {
        $crate::ConstValue::Null
    };

    (true) => {
        $crate::ConstValue::Boolean(true)
    };

    (false) => {
        $crate::ConstValue::Boolean(false)
    };

    ([]) => {
        $crate::ConstValue::List($crate::value_internal_vec![])
    };

    ([ $($tt:tt)+ ]) => {
        $crate::ConstValue::List($crate::value_internal!(@array [] $($tt)+))
    };

    ({}) => {
        $crate::ConstValue::Object(Default::default())
    };

    ({ $($tt:tt)+ }) => {
        $crate::ConstValue::Object({
            let mut object = $crate::indexmap::IndexMap::new();
            $crate::value_internal!(@object object () ($($tt)+) ($($tt)+));
            object
        })
    };

    // Any Serialize type: numbers, strings, struct literals, variables etc.
    // Must be below every other rule.
    ($other:expr) => {
        $crate::to_value(&$other).unwrap()
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! value_internal_vec {
    ($($content:tt)*) => {
        vec![$($content)*]
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! value_unexpected {
    () => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! value_expect_expr_comma {
    ($e:expr , $($tt:tt)*) => {};
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use crate::{ConstValue, Name};

    #[test]
    fn test_macro() {
        assert_eq!(value!(1), ConstValue::Number(1.into()));
        assert_eq!(value!(1 + 2), ConstValue::Number(3.into()));
        assert_eq!(value!("abc"), ConstValue::String("abc".into()));
        assert_eq!(value!(true), ConstValue::Boolean(true));
        assert_eq!(
            value!([1, 2, 3]),
            ConstValue::List((1..=3).map(|n| ConstValue::Number(n.into())).collect())
        );
        assert_eq!(
            value!([1, 2, 3,]),
            ConstValue::List((1..=3).map(|n| ConstValue::Number(n.into())).collect())
        );
        assert_eq!(value!({"a": 10, "b": true}), {
            let mut map = IndexMap::new();
            map.insert(Name::new("a"), ConstValue::Number(10.into()));
            map.insert(Name::new("b"), ConstValue::Boolean(true));
            ConstValue::Object(map)
        });
    }
}
