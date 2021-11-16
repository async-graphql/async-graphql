# Integrations for async-graphql

This directory provides various integrations for `async-graphql` to various crates in the ecosystem.

## Requirements for an HTTP integration

This is a list of criteria for HTTP integrations with `async-graphql` in order to make sure all
integrations are implemented consistently.

Integrations may provide additional functionality to better integrate with the specific library, but
they must all internally use the below functions.

- Conversion from HTTP library's request to `async_graphql::BatchRequest`:
	1. If the request is a `GET` request:
		1. Return the request's query parameters deserialized as an `async_graphql::Request`.
	1. If the request is a `POST` request:
		1. Get the request's `Content-Type` header.
		1. Call `async_graphql::http::receive_batch_body` on the request's body.
		1. Convert `ParseRequestError::PayloadTooLarge` to a 413 Payload Too Large response.
		1. Convert all other errors to a 400 Bad Request response.
	1. Otherwise return a 405 Method Not Allowed.
- Conversion from HTTP library's request to `async_graphql::Request`:
	1. Call the above function to convert the request to an `async_graphql::BatchRequest`.
	1. Call `BatchRequest::into_single` on the result.
	1. Convert all errors to a 400 Bad Request response.
- Conversion from `async_graphql::BatchResponse` to HTTP library's response:
	1. Create a 200 OK response.
	1. If the GraphQL response is ok, set the response's `Cache-Control` header to the response's
	   cache control value.
	1. Set the response's body to the GraphQL response serialized as JSON, also setting the
	   `Content-Type` header to `application/json`.
- GraphQL over websocket support:
	1. Create an `async_graphql::http:WebSocket` using `async_graphql::http::WebSocket::with_data`.
	1. Support the basics of the websocket protocol:
		- Respond to ping messages with pong messages.
		- Treat continuation messages identically to data messages.
	1. Stream all websocket messages that send data (bytes/text/continuations) to the
	   `async_graphql::http::WebSocket`.
	1. Convert all responses to websocket text responses.

## Integration Status

- **Poem**: Complete integration.
- **Actix-web**: Complete integration.
- **Rocket**: Missing websocket support (blocked on [support in Rocket itself](https://github.com/SergioBenitez/Rocket/issues/90)).
- **Warp**: Complete integration.
- **Axum**: Complete integration.
- 