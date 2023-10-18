# Apollo Federation

Apollo Federation is a GraphQL architecture for combining multiple GraphQL services, or subgraphs, into a single supergraph. You can read more in the [official documentation](https://www.apollographql.com/docs/apollo-server/federation/).

> To see a complete example of federation, check out the [federation example](https://github.com/async-graphql/examples/tree/master/federation). 

## Enabling federation support

`async-graphql` supports all the functionality of Apollo Federation v2. Support will be enabled automatically if any `#[graphql(entity)]` resolvers are found in the schema. To enable it manually, use the `enable_federation` method on the `SchemaBuilder`.

```rust
# extern crate async_graphql;
# use async_graphql::*;
# struct Query;
# #[Object]
# impl Query {
#    async fn hello(&self) -> String { "Hello".to_string() }
# }
fn main() {
  let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
    .enable_federation()
    .finish();
  // ... Start your server of choice
}
```

This will define the [`@link` directive](https://www.apollographql.com/docs/federation/federated-types/federated-directives#link) on your schema to enable Federation v2.

## Entities and `@key`

[Entities](https://www.apollographql.com/docs/federation/entities) are a core feature of federation, they allow multiple subgraphs to contribute fields to the same type. An entity is a GraphQL `type` with at least one [`@key` directive][`@key`]. To create a [`@key`] for a type, create a reference resolver using the `#[graphql(entity)]` attribute. This resolver should be defined on the `Query` struct, but will not appear as a field in the schema.

> Even though a reference resolver looks up an individual entity, it is **crucial that you use a [dataloader](dataloader.md)** in the implementation. The federation router will look up entities in batches, which can quickly lead the N+1 performance issues.

### Example

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(SimpleObject)]
# struct User { id: ID }
struct Query;

#[Object]
impl Query {
    #[graphql(entity)]
    async fn find_user_by_id(&self, id: ID) -> User {
        User { id }
    }

    #[graphql(entity)]
    async fn find_user_by_id_with_username(&self, #[graphql(key)] id: ID, username: String) -> User {
        User { id }
    }

    #[graphql(entity)]
    async fn find_user_by_id_and_username(&self, id: ID, username: String) -> User {
        User { id }
    }
}
```

**Notice the difference between these three lookup functions, which are all looking for the `User` object.**

- `find_user_by_id`: Use `id` to find a `User` object, the key for `User` is `id`.

- `find_user_by_id_with_username`: Use `id` to find an `User` object, the key for `User` is `id`, and the `username` field value of the `User` object is requested (e.g., via `@external` and `@requires`).

- `find_user_by_id_and_username`: Use `id` and `username` to find an `User` object, the keys for `User` are `id` and `username`.

The resulting schema will look like this:

```graphql
type Query {
  # These fields will not be exposed to users, they are only used by the router to resolve entities
  _entities(representations: [_Any!]!): [_Entity]!
  _service: _Service!
}

type User @key(fields: "id") @key(fields: "id username") {
  id: ID!
}
```

### Defining a compound primary key

A single primary key can consist of multiple fields, and even nested fields, you can use `InputObject` to implements a nested primary key.

In the following example, the primary key of the `User` object is `key { a b }`.

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(SimpleObject)]
# struct User { key: Key }
# #[derive(SimpleObject)]
# struct Key { a: i32, b: i32 }
#[derive(InputObject)]
struct NestedKey {
  a: i32,
  b: i32,
}

struct Query;

#[Object]
impl Query {
  #[graphql(entity)]
  async fn find_user_by_key(&self, key: NestedKey) -> User {
    let NestedKey { a, b } = key;
    User { key: Key{a, b} }
  }
}
```

The resulting schema will look like this:

```graphql
type Query {
  # These fields will not be exposed to users, they are only used by the router to resolve entities
  _entities(representations: [_Any!]!): [_Entity]!
  _service: _Service!
}

type User @key(fields: "key { a b }") {
  key: Key!
}

type Key {
  a: Int!
  b: Int!
}
```

### Creating unresolvable entities

There are certain times when you need to reference an entity, but not add any fields to it. This is particularly useful when you want to link data from separate subgraphs together, but neither subgraph has all the data.

If you wanted to implement the [products and reviews subgraphs example](https://www.apollographql.com/docs/federation/entities/#referencing-an-entity-without-contributing-fields) from the Apollo Docs, you would create the following types for the reviews subgraph:

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(SimpleObject)]
struct Review {
    product: Product,
    score: u64,
}

#[derive(SimpleObject)]
#[graphql(unresolvable)]
struct Product {
    id: u64,
}
```

This will add the `@key(fields: "id", resolvable: false)` directive to the `Product` type in the reviews subgraph.

For more complex entity keys, such as ones with nested fields in compound keys, you can override the fields in the directive as so:

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(SimpleObject)]
#[graphql(unresolvable = "id organization { id }")]
struct User {
    id: u64,
    organization: Organization,
}

#[derive(SimpleObject)]
struct Organization {
    id: u64,
}
```

However, it is important to note that no validation will be done to check that these fields exist.

## `@shareable`

Apply the [`@shareable` directive](https://www.apollographql.com/docs/federation/federated-types/federated-directives#shareable) to a type or field to indicate that multiple subgraphs can resolve it.

### `@shareable` fields
```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(SimpleObject)]
#[graphql(complex)]
struct Position {
  #[graphql(shareable)]
  x: u64,
}

#[ComplexObject]
impl Position {
  #[graphql(shareable)]
  async fn y(&self) -> u64 {
    0
  }
}
```

The resulting schema will look like this:

```graphql
type Position {
  x: Int! @shareable
  y: Int! @shareable
}
```


### `@shareable` type

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(SimpleObject)]
#[graphql(shareable)]
struct Position {
  x: u64,
  y: u64,
}
```

The resulting schema will look like this:

```graphql
type Position @shareable {
  x: Int!
  y: Int!
}
```

## `@inaccessible`

The [`@inaccessible` directive](https://www.apollographql.com/docs/federation/federated-types/federated-directives#inaccessible) is used to omit something from the supergraph schema (e.g., if it's not yet added to all subgraphs which share a `@shareable` type).

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(SimpleObject)]
#[graphql(shareable)]
struct Position {
  x: u32,
  y: u32,
  #[graphql(inaccessible)]
  z: u32,
} 
```

Results in:

```graphql
type Position @shareable {
  x: Int!
  y: Int!
  z: Int! @inaccessible
}
```

## `@override`

The [`@override` directive](https://www.apollographql.com/docs/federation/federated-types/federated-directives#override) is used to take ownership of a field from another subgraph. This is useful for migrating a field from one subgraph to another.

For example, if you add a new "Inventory" subgraph which should take over responsibility for the `inStock` field currently provided by the "Products" subgraph, you might have something like this:

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(SimpleObject)]
struct Product {
  id: ID,
  #[graphql(override_from = "Products")]
  in_stock: bool,
}
```

Which results in:

```graphql
type Product @key(fields: "id") {
  id: ID!
  inStock: Boolean! @override(from: "Products")
}
```

## `@external`

The [`@external` directive](https://www.apollographql.com/docs/federation/federated-types/federated-directives#external) is used to indicate that a field is usually provided by another subgraph, but is sometimes required by this subgraph (when combined with `@requires`) or provided by this subgraph (when combined with `@provides`).

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(SimpleObject)]
struct Product {
  id: ID,
  #[graphql(external)]
  name: String,
  in_stock: bool,
}
```

Results in:

```graphql
type Product {
  id: ID!
  name: String! @external
  inStock: Boolean!
}
```

## `@provides`

The [`@provides` directive](https://www.apollographql.com/docs/federation/federated-types/federated-directives#provides) is used to indicate that a field is provided by this subgraph, but only sometimes.

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(SimpleObject)]
struct Product {
    id: ID,
    #[graphql(external)]
    human_name: String,
    in_stock: bool,
}

struct Query;

#[Object]
impl Query {
    /// This operation will provide the `humanName` field on `Product
    #[graphql(provides = "humanName")]
    async fn out_of_stock_products(&self) -> Vec<Product> {
      vec![Product {
        id: "1".into(),
        human_name: "My Product".to_string(),
        in_stock: false,
      }]
    }
    async fn discontinued_products(&self) -> Vec<Product> {
        vec![Product {
            id: "2".into(),
            human_name: String::new(),  // This is ignored by the router
            in_stock: false,
        }]
    }
    #[graphql(entity)]
    async fn find_product_by_id(&self, id: ID) -> Product {
        Product {
            id,
            human_name: String::new(),  // This is ignored by the router
            in_stock: true,
        }
    }
}
```

Note that the `#[graphql(provides)]` attribute takes the field name as it appears in the schema, not the Rust field name.

The resulting schema will look like this:

```graphql
type Product @key(fields: "id") {
    id: ID!
    humanName: String! @external
    inStock: Boolean!
}

type Query {
    outOfStockProducts: [Product!]! @provides(fields: "humanName")
    discontinuedProducts: [Product!]!
}
```

## `@requires`

The [`@requires` directive](https://www.apollographql.com/docs/federation/federated-types/federated-directives#requires) is used to indicate that an `@external` field is required for this subgraph to resolve some other field(s). If our `shippingEstimate` field requires the `size` and `weightInPounts` fields, then we might want a subgraph entity which looks like this:

```graphql
type Product @key(fields: "id") {
  id: ID!
  size: Int! @external
  weightInPounds: Int! @external
  shippingEstimate: String! @requires(fields: "size weightInPounds")
}
```

In order to implement this in Rust, we can use the `#[graphql(requires)]` attribute:

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(SimpleObject)]
#[graphql(complex)]
struct Product {
  id: ID,
  #[graphql(external)]
  size: u32,
  #[graphql(external)]
  weight_in_pounds: u32,
}

#[ComplexObject]
impl Product {
  #[graphql(requires = "size weightInPounds")]
  async fn shipping_estimate(&self) -> String {
    let price = self.size * self.weight_in_pounds;
    format!("${}", price)
  }
}
```

Note that we use the GraphQL field name `weightInPounds`, not the Rust field name `weight_in_pounds` in `requires`. To populate those external fields, we add them as arguments in the entity resolver:

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(SimpleObject)]
# struct Product {
#     id: ID,
#     #[graphql(external)]
#     size: u32,
#     #[graphql(external)]
#     weight_in_pounds: u32,
# }
# struct Query;
#[Object]
impl Query {
  #[graphql(entity)]
  async fn find_product_by_id(
    &self, 
    #[graphql(key)] id: ID, 
    size: Option<u32>, 
    weight_in_pounds: Option<u32>
  ) -> Product {
    Product {
      id,
      size: size.unwrap_or_default(),
      weight_in_pounds: weight_in_pounds.unwrap_or_default(),
    }
  }
}
```

The inputs are `Option<>` even though the fields are required. This is because the external fields are _only_ passed to the subgraph when the field(s) that require them are being selected. If the `shippingEstimate` field is not selected, then the `size` and `weightInPounds` fields will not be passed to the subgraph. **Always use optional types for external fields.**

We have to put _something_ in place for `size` and `weight_in_pounds` as they are still required fields on the type, so we use `unwrap_or_default()` to provide a default value. This looks a little funny, as we're populating the fields with nonsense values, but we have confidence that they will not be needed if they were not provided.  **Make sure to use `@requires` if you are consuming `@external` fields, or your code will be wrong.**

### Nested `@requires`

A case where the `@requires` directive can be confusing is when there are nested entities. For example, if we had an `Order` type which contained a `Product`, then we would need an entity resolver like this:

```rust
# extern crate async_graphql;
# use async_graphql::*;
# #[derive(SimpleObject)]
# pub struct Order { id: ID }
# struct Query;
#[Object]
impl Query {
  #[graphql(entity)]
  async fn find_order_by_id(&self, id: ID) -> Option<Order> {
      Some(Order { id })
  }
}
```

There are no inputs on this entity resolver, so how do we populate the `size` and `weight_in_pounds` fields on `Product` if a user has a query like `order { product { shippingEstimate } }`? The supergraph implementation will solve this for us by calling the `find_product_by_id` separately for any fields which have a `@requires` directive, so the subgraph code does not need to worry about how entities relate.

## `@tag`

The [`@tag` directive](https://www.apollographql.com/docs/federation/federated-types/federated-directives#tag) is used to add metadata to a schema location for features like [contracts](https://www.apollographql.com/docs/studio/contracts/). To add a tag like this:

```graphql
type User @tag(name: "team-accounts") {
  id: String!
  name: String!
}
```

You can write code like this:

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[derive(SimpleObject)]
#[graphql(tag = "team-accounts")]
struct User {
  id: ID,
  name: String,
}
```

## `@composeDirective`

The [`@composeDirective` directive](https://www.apollographql.com/docs/federation/federation-spec/#composedirective) is used to add a custom type system directive to the supergraph schema. Without `@composeDirective`, and [custom type system directives](./custom_directive#type-system-directives) are omitted from the composed supergraph schema. To include a custom type system directive as a composed directive, just add the `composable` attribute to the `#[TypeDirective]` macro:

```rust
# extern crate async_graphql;
# use async_graphql::*;
#[TypeDirective(
    location = "Object",
    composable = "https://custom.spec.dev/extension/v1.0",
)]
fn custom() {}
```

In addition to the [normal type system directive behavior](./custom_directive#type-system-directives), this will add the following bits to the output schema:

```graphql
extend schema @link(
	url: "https://custom.spec.dev/extension/v1.0"
	import: ["@custom"]
)
	@composeDirective(name: "@custom")
```

[`@key`]: https://www.apollographql.com/docs/federation/entities#1-define-a-key
