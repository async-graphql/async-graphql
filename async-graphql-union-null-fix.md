# async-graphql Union Null Support — Fix Specification

**Issue:** #1577  
**Fork:** `~/git/async-graphql` (v8.0.0-rc.5)  
**Branch:** `feature/dynamic-schema-directive`  
**Remote:** `git@github.com:andrewkgabler/async-graphql.git`  
**Upstream:** `https://github.com/async-graphql/async-graphql.git`

---

## Problem

Dynamic unions in async-graphql require `FieldValue::WithType` for all items. When the Apollo Federation entities resolver returns `FieldValue::NULL` (bare null) for not-found entities, the schema throws:

```
"internal: invalid value for union \"_Entity\", expected \"FieldValue::WithType\""
```

This breaks null entity handling per the Apollo Federation spec, which requires returning `null` for entities that don't exist.

**Root cause:** In `src/dynamic/resolve.rs:617-661`, `resolve_value` handles unions only with `FieldValueInner::WithType`. No case exists for null values.

---

## Proposed Fix

### 1. Add `NullWithTy` variant to `FieldValueInner`

**File:** `src/dynamic/field.rs`

Add new variant to the enum:

```rust
pub(crate) enum FieldValueInner<'a> {
    Value(Value),
    BorrowedAny(Cow<'static, str>, &'a (dyn Any + Send + Sync)),
    OwnedAny(Cow<'static, str>, Box<dyn Any + Send + Sync>),
    List(Vec<FieldValue<'a>>),
    WithType {
        value: Box<FieldValue<'a>>,
        ty: Cow<'static, str>,
    },
    NullWithTy {
        /// Type name for validation (union/interface member)
        ty: Cow<'static, str>,
    },
}
```

### 2. Add `null_with_type` constructor

**File:** `src/dynamic/field.rs`

Add method to `impl<'a> FieldValue<'a>`:

```rust
/// Create a null FieldValue with a type name for union/interface resolution.
///
/// Used by Apollo Federation entities resolver to return null for not-found entities.
///
/// # Examples
///
/// ```
/// use async_graphql::dynamic::*;
///
/// // Return null for a not-found entity
/// Ok(Some(FieldValue::null_with_type("Astronaut")))
/// ```
pub fn null_with_type(ty: impl Into<Cow<'static, str>>) -> Self {
    Self(FieldValueInner::NullWithTy { ty: ty.into() })
}
```

### 3. Handle `NullWithTy` in union resolution

**File:** `src/dynamic/resolve.rs`

Modify `resolve_value` function to add cases for `NullWithTy`:

```rust
(Type::Union(union), FieldValueInner::NullWithTy { ty }) => {
    if !union.possible_types.contains(ty.as_ref()) {
        return Err(ctx.set_error_path(
            Error::new(format!(
                "internal: union \"{}\" does not contain object \"{}\"",
                union.name, ty,
            ))
            .into_server_error(ctx.item.pos),
        ));
    }
    Ok(Some(Value::Null))
}
```

### 4. Handle `NullWithTy` in interface resolution

**File:** `src/dynamic/resolve.rs`

Modify `resolve_value` function to add cases for `NullWithTy`:

```rust
(Type::Interface(interface), FieldValueInner::NullWithTy { ty }) => {
    let is_contains_obj = schema
        .0
        .env
        .registry
        .types
        .get(&interface.name)
        .and_then(|meta_type| {
            meta_type
                .possible_types()
                .map(|possible_types| possible_types.contains(ty.as_ref()))
        })
        .unwrap_or_default();
    if !is_contains_obj {
        return Err(ctx.set_error_path(
            Error::new(format!(
                "internal: object \"{}\" does not implement interface \"{}\"",
                ty, interface.name,
            ))
            .into_server_error(ctx.item.pos),
        ));
    }
    Ok(Some(Value::Null))
}
```

---

## Usage in Allograph

**File:** `applications/allograph/src/graphql/resolvers/entities_resolver.rs`

When an entity is not found, return:

```rust
FieldValue::null_with_type("Astronaut")  // type name from entity key
```

Instead of:

```rust
FieldValue::NULL  // causes error
```

---

## Testing

1. Add test for union field returning `FieldValue::null_with_type`
2. Add test for interface field returning `FieldValue::null_with_type`
3. Verify Apollo Federation `_entities` query returns `null` for not-found entities
4. Verify type validation still works (invalid type name should error)

---

## Impact

- **Breaking changes:** None — new variant, no existing behavior changed
- **API changes:** New public method `FieldValue::null_with_type()`
- **Performance:** Negligible — one additional match arm in resolve path
- **Compatibility:** Fixes Apollo Federation null entity handling per spec

---

## Related

- GitHub Issue: #1577
- Allograph test: `null_entries` (test 3) — currently failing due to this issue
- Apollo Federation spec: Entities query must return `null` for not-found entities