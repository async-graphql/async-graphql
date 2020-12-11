# async-graphql Architecture

This document describes the internal architecture of `async-graphql`, and can be useful to
people wanting to contribute.

## Schema

When you create a schema, the first thing it does it asks the query, mutation and subscription types
to register themselves in the schema's list of GraphQL types called the **registry**. Those types
will then recursively register all the types that they depend on in the registry, and so on until
every single type that is used has been registered in the registry.

## Query Execution

First of all, `async-graphql` will use the `async-graphql-parser` crate (located in the `parser/`
directory) to parse the request document source. This also performs some necessary validations
such as making sure that operation (i.e. query/mutation/subscription) names are unique and that the
query does not contain an anonymous operation as well as a named one.

It then will validate the document as per the rest of GraphQL's validation rules. The `validation/`
module handles this, and it works by walking through the entire document once and notifying the
validation rules on the way to perform their validations and report errors if necessary. If
`ValidationMode::Fast` is turned on, far far fewer rules are used.

Also at this stage some non-validators use the same architecture, such as the query depth calculator
which keeps track of how deeply nested the query gets as the document is walked through.

At this point all the unnecessary operations (ones not selected by `operationName`) are dropped, and
we will execute just one.

At the core of all the resolver logic there are two traits: `InputType` and `OutputType`
which represent a GraphQL input value and GraphQL output value respectively. `InputType` just
requires conversions to and from `async_graphql::Value`. `OutputType` is an async trait with a
single method, `resolve`, which takes a field (e.g. `user(name: "sunli829") { display_name }`) and
resolves it to a single value.

Scalars and enums are expected to ignore the input and serialize themselves, while objects,
interfaces and unions are expected to read the selection set in the field and resolve and serialize
each one of their fields.

As implementing `OutputType::resolve` manually quickly becomes very tedious helpful utilities
are provided in the `resolver_utils` module and via macros.
