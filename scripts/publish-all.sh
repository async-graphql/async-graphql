#!/bin/sh

cd async-graphql-derive && cargo publish && cd ..
sleep 5
cd async-graphql-actix-web && cargo publish && cd ..
sleep 5
cargo publish
sleep 5
