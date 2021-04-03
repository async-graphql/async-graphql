# SDL Export

You can export your schema in Schema Definition Language (SDL) by using the `Schema::sdl()` method.


```rust
use async_graphql::*;

struct Query;

#[Object]
impl Query {
    async fn add(&self, u: i32, v: i32) -> i32 {
        u + v
    }
}

#[tokio::main]
async fn main() {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();
    
    // Print the schema in SDL format
    println!("{}", &schema.sdl());
}
```
