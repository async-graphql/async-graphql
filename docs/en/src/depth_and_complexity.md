# Query complexity and depth

⚠️GraphQL provides a powerful way to query your data, but putting great
power in the hands of your API clients also exposes you to a risk of denial 
of service attacks. You can mitigate that risk with `Async-graphql` by limiting the 
complexity and depth of the queries you allow.

## Expensive Queries

Consider a schema that allows listing blog posts. Each blog post is also related to other posts.

```graphql
type Query {
	posts(count: Int = 10): [Post!]!
}

type Post {
	title: String!
	text: String!
	related(count: Int = 10): [Post!]!
}
```

It's not too hard to craft a query that will cause a very large response:

```graphql
{
    posts(count: 100) {
        related(count: 100) {
            related(count: 100) {
                related(count: 100) {
                    title
                }
            }
        }
    }
}
```

The size of the response increases exponentially with every other level of the `related` field. Fortunately, `Async-graphql` provides 
a way to prevent such queries.

## Limiting Query depth

The depth is the number of nesting levels of the field, and the following is a query with a depth of `3`.

```graphql
{
    a {
        b {
            c
        }
    }
}
```

You can limit the depth when creating `Schema`. If the query exceeds this limit, an error will occur and the 
message `Query is nested too deep` will be returned.

```rust
# extern crate async_graphql;
# use async_graphql::*;
# struct Query;
# #[Object]
# impl Query { async fn version(&self) -> &str { "1.0" } }
let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    .limit_depth(5) // Limit the maximum depth to 5
    .finish();
```

## Limiting Query complexity

The complexity is the number of fields in the query. The default complexity of each field is `1`. Below is a 
query with a complexity of `6`.

```graphql
{
    a b c {
        d {
            e f
        }
    }
}
```

You can limit the complexity when creating the `Schema`. If the query exceeds this limit, an error will occur 
and `Query is too complex` will be returned.

```rust
# extern crate async_graphql;
# use async_graphql::*;
# struct Query;
# #[Object]
# impl Query { async fn version(&self) -> &str { "1.0" } }
let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    .limit_complexity(5) // Limit the maximum complexity to 5
    .finish();
```

## Custom Complexity Calculation

There are two ways to customize the complexity for non-list type and list type fields.

In the following code, the complexity of the `value` field is `5`. The complexity of the `values` field is `count * child_complexity`, 
`child_complexity` is a special variable that represents the complexity of the subquery, and `count` is the parameter of the field,
used to calculate the complexity of the `values` field, and the type of the return value must be `usize`.

```rust
# extern crate async_graphql;
# use async_graphql::*;
struct Query;

#[Object]
impl Query {
    #[graphql(complexity = 5)]
    async fn value(&self) -> i32 {
        todo!()
    }

    #[graphql(complexity = "count * child_complexity")]
    async fn values(&self, count: usize) -> i32 {
        todo!()
    }
}
```

**Note: The complexity calculation is done in the validation phase and not the execution phase,
so you don't have to worry about partial execution of over-limit queries.**
