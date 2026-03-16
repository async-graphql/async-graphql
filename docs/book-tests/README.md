# book-tests

This crate tests the Rust code examples embedded in the book's markdown files (`docs/en/` and `docs/zh-CN/`).

## How it works

Each markdown chapter is included as a doc comment via `#[doc = include_str!("...")]` on an empty module in `src/lib.rs`. Running `cargo test --doc -p book-tests` compiles and runs all the embedded code blocks, just like `mdbook test` would — but without the duplicate-rlib problem.

## Why not `mdbook test`?

`mdbook test -L target/debug/deps` fails in this workspace because Cargo resolver v2 produces multiple `.rlib` files for the same crate when different workspace members use different feature sets. This causes `rustc` to error with E0464 ("multiple candidates for rlib dependency"). See [mdBook#394](https://github.com/rust-lang/mdBook/issues/394) and [async-graphql#1794](https://github.com/async-graphql/async-graphql/issues/1794).

`cargo test --doc` doesn't have this problem because it uses `--extern` with specific paths rather than `-L` with a flat directory scan.

## Adding a new chapter

When a new markdown chapter with Rust code blocks is added to the book:

1. Add a `#[doc = include_str!("../../<lang>/src/<chapter>.md")]` line to the appropriate language module in `src/lib.rs`
2. If the chapter uses a crate not already listed in `Cargo.toml`, add it as a dependency

Only files containing `` ```rust `` code blocks need to be included. Chapters that are purely prose (like `introduction.md` or `SUMMARY.md`) can be skipped.
