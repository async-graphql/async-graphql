#![allow(dead_code, non_camel_case_types, unused_macros)]

use async_graphql::{InputValueResult, ScalarType, Value};

// Std

mod core {}
mod alloc {}
mod std {}

// Prelude

struct Copy;
struct Send;
// TODO: removed this once https://github.com/nvzqz/static-assertions-rs/issues/37 is fixed
// struct Sized;
struct Sync;
struct Unpin;

struct Drop;
struct Fn;
struct FnMut;
struct FnOnce;

fn drop() {}

// TODO: remove this once https://github.com/dtolnay/async-trait/issues/132 is fixed
//struct Box;

struct ToOwned;

struct Clone;

struct PartialEq;
struct PartialOrd;
struct Eq;
struct Ord;

struct AsRef;
struct AsMut;
struct Into;
struct From;

struct Default;

struct Iterator;
struct Extend;
struct IntoIterator;
struct DoubleEndedIterator;
struct ExactSizeIterator;

struct Option;
struct Some;
struct None;

struct Result;
struct Ok;
struct Err;

struct String<T>(::std::marker::PhantomData<T>);
struct ToString;

struct Vec;

// Primitives

struct bool<T>(::std::marker::PhantomData<T>);
struct char<T>(::std::marker::PhantomData<T>);
struct f32<T>(::std::marker::PhantomData<T>);
struct f64<T>(::std::marker::PhantomData<T>);
struct i128<T>(::std::marker::PhantomData<T>);
struct i16<T>(::std::marker::PhantomData<T>);
struct i32<T>(::std::marker::PhantomData<T>);
struct i64<T>(::std::marker::PhantomData<T>);
struct i8<T>(::std::marker::PhantomData<T>);
struct isize<T>(::std::marker::PhantomData<T>);
struct str<T>(::std::marker::PhantomData<T>);
struct u128<T>(::std::marker::PhantomData<T>);
struct u16<T>(::std::marker::PhantomData<T>);
struct u32<T>(::std::marker::PhantomData<T>);
struct u64<T>(::std::marker::PhantomData<T>);
struct u8<T>(::std::marker::PhantomData<T>);
struct usize<T>(::std::marker::PhantomData<T>);

// Macros
// Note: panic! isn't included here because the stdlib's macros like todo!() rely on a working
// panic! macro being in scope.

macro_rules! assert {
    (__unusable_macro__) => {};
}
macro_rules! assert_eq {
    (__unusable_macro__) => {};
}
macro_rules! assert_ne {
    (__unusable_macro__) => {};
}
macro_rules! cfg {
    (__unusable_macro__) => {};
}
macro_rules! Clone {
    (__unusable_macro__) => {};
}
macro_rules! Eq {
    (__unusable_macro__) => {};
}
macro_rules! Ord {
    (__unusable_macro__) => {};
}
macro_rules! PartialEq {
    (__unusable_macro__) => {};
}
macro_rules! PartialOrd {
    (__unusable_macro__) => {};
}
macro_rules! column {
    (__unusable_macro__) => {};
}
macro_rules! compile_error {
    (__unusable_macro__) => {};
}
macro_rules! concat {
    (__unusable_macro__) => {};
}
macro_rules! dbg {
    (__unusable_macro__) => {};
}
macro_rules! debug_assert {
    (__unusable_macro__) => {};
}
macro_rules! debug_assert_eq {
    (__unusable_macro__) => {};
}
macro_rules! debug_assert_ne {
    (__unusable_macro__) => {};
}
macro_rules! Default {
    (__unusable_macro__) => {};
}
macro_rules! env {
    (__unusable_macro__) => {};
}
macro_rules! eprint {
    (__unusable_macro__) => {};
}
macro_rules! eprintln {
    (__unusable_macro__) => {};
}
macro_rules! file {
    (__unusable_macro__) => {};
}
macro_rules! Debug {
    (__unusable_macro__) => {};
}
macro_rules! format {
    (__unusable_macro__) => {};
}
macro_rules! format_args {
    (__unusable_macro__) => {};
}
macro_rules! Hash {
    (__unusable_macro__) => {};
}
macro_rules! include {
    (__unusable_macro__) => {};
}
macro_rules! include_bytes {
    (__unusable_macro__) => {};
}
macro_rules! include_str {
    (__unusable_macro__) => {};
}
macro_rules! is_x86_feature_detected {
    (__unusable_macro__) => {};
}
macro_rules! line {
    (__unusable_macro__) => {};
}
macro_rules! Copy {
    (__unusable_macro__) => {};
}
macro_rules! matches {
    (__unusable_macro__) => {};
}
macro_rules! module_path {
    (__unusable_macro__) => {};
}
macro_rules! option_env {
    (__unusable_macro__) => {};
}
macro_rules! print {
    (__unusable_macro__) => {};
}
macro_rules! println {
    (__unusable_macro__) => {};
}
macro_rules! stringify {
    (__unusable_macro__) => {};
}
macro_rules! thread_local {
    (__unusable_macro__) => {};
}
macro_rules! todo {
    (__unusable_macro__) => {};
}
macro_rules! r#try {
    (__unusable_macro__) => {};
}
macro_rules! unimplemented {
    (__unusable_macro__) => {};
}
macro_rules! unreachable {
    (__unusable_macro__) => {};
}
macro_rules! vec {
    (__unusable_macro__) => {};
}
macro_rules! write {
    (__unusable_macro__) => {};
}
macro_rules! writeln {
    (__unusable_macro__) => {};
}

// Tests

struct MyObject;
#[async_graphql::Object]
impl MyObject {
    #[graphql(deprecation = "abc")]
    async fn value(&self) -> &::std::primitive::i32 {
        &5
    }
    async fn other_value(&self) -> &::std::primitive::i16 {
        &5
    }
    /// Add one to the number.
    async fn add_one(
        &self,
        #[graphql(default = 0)] v: ::std::primitive::i32,
    ) -> ::std::primitive::i32 {
        v + 1
    }
}

#[derive(async_graphql::SimpleObject)]
struct MySimpleObject {
    /// Value.
    value: ::std::primitive::i32,
    other_value: ::std::primitive::i16,
    #[graphql(deprecation = "bar")]
    bar: ::std::string::String,
    #[graphql(skip)]
    skipped: ::std::any::TypeId,
}

struct MySubscription;
#[async_graphql::Subscription]
impl MySubscription {
    #[graphql(deprecation = "abc")]
    async fn values(&self) -> impl ::futures_util::stream::Stream<Item = ::core::primitive::i32> {
        ::futures_util::stream::iter(5..7)
    }
    /// Count up from the value.
    async fn count_up_from(
        &self,
        #[graphql(default = 0)] v: ::std::primitive::i32,
    ) -> impl ::futures_util::stream::Stream<Item = ::core::primitive::i32> {
        ::futures_util::stream::iter(v..v + 20)
    }
}

struct MyScalar;
#[async_graphql::Scalar]
impl ScalarType for MyScalar {
    fn parse(_value: Value) -> InputValueResult<Self> {
        ::std::result::Result::Ok(Self)
    }
    fn to_value(&self) -> Value {
        Value::String("Hello world!".to_owned())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MyScalar2(::std::primitive::i32);
::async_graphql::scalar!(MyScalar2);

#[derive(Clone, Copy, PartialEq, Eq, async_graphql::Enum)]
enum MyEnum {
    /// Foo.
    Foo,
    Bar,
}

#[derive(async_graphql::InputObject)]
struct MyInputObject {
    /// Foo.
    foo: ::std::primitive::i32,
    #[graphql(default)]
    bar: ::std::string::String,
}

#[derive(async_graphql::Interface)]
#[graphql(
    field(name = "value", type = "&::std::primitive::i32"),
    field(name = "other_value", type = "&::std::primitive::i16")
)]
enum MyInterface {
    First(MyObject),
    Second(MySimpleObject),
}

#[derive(async_graphql::Union)]
enum MyUnion {
    First(MyObject),
    Second(MySimpleObject),
}

#[derive(async_graphql::MergedObject)]
struct MyMergedObject(MyObject, MySimpleObject);

#[derive(async_graphql::MergedSubscription)]
struct MyMergedSubscription(MySubscription);
