# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Development Commands

### Building and Testing
- `cargo build` - Build the main library
- `cargo build --workspace` - Build all workspace members
- `cargo build --all-features` - Build with all features enabled
- `cargo test` - Run tests
- `cargo test --workspace` - Run tests for all workspace members
- `cargo clippy` - Run linting checks
- `cargo fmt` - Format code according to rustfmt.toml configuration

### Features and Examples
- `cargo build --features dynamic-schema` - Build with dynamic schema support
- `cd examples && cargo run --bin [example-name]` - Run specific example
- `git submodule update` - Update examples submodule before running examples

### Single Test Execution
- `cargo test [test_name]` - Run a specific test
- `cargo test --test [test_file]` - Run tests from a specific test file

## Architecture Overview

async-graphql is a high-performance GraphQL server library for Rust with a modular, trait-based architecture:

### Core Components

**Schema System**: The library uses a registry-based type system where types recursively register themselves and their dependencies. The `Schema` struct coordinates query execution through three main operation types: Query, Mutation, and Subscription.

**Type System Traits**: 
- `InputType` - Represents GraphQL input values with conversions to/from `async_graphql::Value`
- `OutputType` - Async trait for resolving GraphQL output values with a `resolve` method

**Query Processing Pipeline**:
1. Parse request using `async-graphql-parser` crate
2. Validate document against GraphQL validation rules in `validation/` module  
3. Execute selected operation through resolver logic
4. Scalars/enums serialize themselves, objects/interfaces/unions resolve selection sets

### Workspace Structure

- **Root crate** (`src/`): Main library with core GraphQL implementation
- **derive/** - Procedural macros for deriving GraphQL traits
- **parser/** - GraphQL query parsing and AST
- **value/** - GraphQL value types and serialization
- **integrations/** - Framework integrations (poem, actix-web, axum, warp, tide, rocket)
- **examples/** - Example applications (git submodule)
- **tests/** - Comprehensive test suite

### Key Source Directories

- `src/types/` - Built-in GraphQL scalar and complex types
- `src/validation/` - GraphQL specification validation rules  
- `src/extensions/` - Middleware system (tracing, Apollo extensions, etc.)
- `src/dynamic/` - Dynamic schema construction (requires `dynamic-schema` feature)
- `src/resolver_utils/` - Utilities for implementing custom resolvers
- `src/http/` - HTTP transport layer and GraphiQL/Playground integration

### Configuration

The project uses Rust 2024 edition with minimum supported version 1.86.0. Rustfmt configuration in `.rustfmt.toml` enforces specific code style including crate-level import grouping and comment formatting.

### Integration Pattern

This library provides a framework-agnostic core with separate integration crates for popular web frameworks. Each integration handles HTTP transport, WebSocket subscriptions, and framework-specific middleware integration.