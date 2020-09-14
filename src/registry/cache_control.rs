/// Cache control values
///
/// # Examples
///
/// ```rust
/// use async_graphql::*;
///
/// struct QueryRoot;
///
/// #[GQLObject(cache_control(max_age = 60))]
/// impl QueryRoot {
///     #[field(cache_control(max_age = 30))]
///     async fn value1(&self) -> i32 {
///         0
///     }
///
///     #[field(cache_control(private))]
///     async fn value2(&self) -> i32 {
///         0
///     }
/// }
///
/// #[async_std::main]
/// async fn main() {
///     let schema = Schema::new(QueryRoot, EmptyMutation, EmptySubscription);
///     assert_eq!(schema.execute("{ value1 }").await.into_result().unwrap().cache_control, CacheControl { public: true, max_age: 30 });
///     assert_eq!(schema.execute("{ value2 }").await.into_result().unwrap().cache_control, CacheControl { public: false, max_age: 60 });
///     assert_eq!(schema.execute("{ value1 value2 }").await.into_result().unwrap().cache_control, CacheControl { public: false, max_age: 30 });
/// }
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CacheControl {
    /// Scope is public, default is true.
    pub public: bool,

    /// Cache max age, default is 0.
    pub max_age: usize,
}

impl Default for CacheControl {
    fn default() -> Self {
        Self {
            public: true,
            max_age: 0,
        }
    }
}

impl CacheControl {
    /// Get 'Cache-Control' header value.
    #[must_use]
    pub fn value(&self) -> Option<String> {
        if self.max_age > 0 {
            Some(format!(
                "max-age={}{}",
                self.max_age,
                if self.public { "" } else { ", private" }
            ))
        } else {
            None
        }
    }
}

impl CacheControl {
    pub(crate) fn merge(&mut self, other: &CacheControl) {
        self.public = self.public && other.public;
        self.max_age = if self.max_age == 0 {
            other.max_age
        } else if other.max_age == 0 {
            self.max_age
        } else {
            self.max_age.min(other.max_age)
        };
    }
}
