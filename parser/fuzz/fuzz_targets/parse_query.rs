#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    async_graphql_parser::parse_query(data);
});
