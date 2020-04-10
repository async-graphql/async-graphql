#!/bin/bash

function cleanup {
    kill "$ACCOUNTS_PID"
    kill "$PRODUCTS_PID"
    kill "$REVIEWS_PID"
}
trap cleanup EXIT

cargo build --example federation-accounts
cargo build --example federation-products
cargo build --example federation-reviews

cargo run --example federation-accounts &
ACCOUNTS_PID=$!

cargo run --example federation-products &
PRODUCTS_PID=$!

cargo run --example federation-reviews &
REVIEWS_PID=$!

sleep 3

node federation/index.js
