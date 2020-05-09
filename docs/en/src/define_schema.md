# Schema

After defining the basic types, you need to define a schema to combine them. The schema consists of three types: a query object, mutation object, and subscription object, where the mutation object and subscription object are optional.

When the schema is created, `Async-Graphql` will traverse all object graphs and register all types. This means that if a GraphQL object is defined but never referenced, then this object will not be exposed in the schema.

