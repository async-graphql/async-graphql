# Schema

After defining the basic types, you need to define a schema to combine them. The schema consists of three types: a query object, a mutation object, and a subscription object, where the mutation object and subscription object are optional.

When the schema is created, `Async-graphql` will traverse all object graphs and register all types. This means that if a GraphQL object is defined but never referenced, this object will not be exposed in the schema.

