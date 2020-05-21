# Custom extensions

A GraphQL extension object can receive events in various stages of a query's execution, and you can collect various kinds of data to be returned in the query results.

You can use `async_graphql::Extension` to define an extension object, and your application must call `Schema::extension` when your `Schema` is created.

You can refer to [Apollo Tracing](https://github.com/async-graphql/async-graphql/blob/master/src/extensions/tracing.rs) to implement your own extension types.
