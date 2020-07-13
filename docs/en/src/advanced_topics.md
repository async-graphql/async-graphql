# Advanced topics
## Batch request support
`async-graphql` provides support for request batching as implemented by Apollo and Relay: several requests may be batched in an array and sent as a single HTTP request. The server will then reply with an array of responses in order.

One limitation of batch request implementation is lack of support for `@defer` and `@stream` directives - all parts of the queries will be processed and returned as a single response array. Therefore you may choose whether you want to continue support streaming and keep only single request support, or switch to batch request support, and lose streaming response support depending on your application's needs.  
