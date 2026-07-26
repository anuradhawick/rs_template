# lambdamux

[![Publish crates](https://github.com/anuradhawick/rs_template/actions/workflows/publish-crates.yml/badge.svg)](https://github.com/anuradhawick/rs_template/actions/workflows/publish-crates.yml)
[![Crates.io](https://img.shields.io/crates/v/lambdamux.svg)](https://crates.io/crates/lambdamux)
[![Downloads](https://img.shields.io/crates/d/lambdamux.svg)](https://crates.io/crates/lambdamux)
[![Docs.rs](https://docs.rs/lambdamux/badge.svg)](https://docs.rs/lambdamux)
[![License](https://img.shields.io/crates/l/lambdamux.svg)](https://crates.io/crates/lambdamux)

`lambdamux` is an AWS Lambda HTTP router for Rust.

It lets you write route handlers with `#[route(...)]` attributes and dispatch API Gateway requests without managing a global route cache yourself.

This repository also includes a Rust AWS Lambda HTTP API template, managed with `cargo-lambda` and deployed with Terraform.

For a short explanation of how the three crates work together, including the trie router, see [AISUMMARY.md](https://github.com/anuradhawick/rs_template/blob/main/AISUMMARY.md).

## Crates

- `lambdamux` is the user-facing facade crate.
- `lambdamux-core` contains the generic trie-based routing primitives.
- `lambdamux-macro` provides the `#[route(...)]` and `generate_routes!()` macros.

## What it gives you

- route registration with `#[route(path = ..., method = ...)]`
- compile-time route table generation with `generate_routes!()`
- built-in API Gateway v2 dispatch with `handle_apigw_v2!`
- built-in API Gateway v1 dispatch with `handle_apigw_v1!`
- synchronous and asynchronous route handlers in the same route table
- path parameter matching such as `/hello/:id`

## Installation

Add these dependencies to your Lambda crate:

```toml
[dependencies]
aws_lambda_events = { version = "1.2.0", default-features = false, features = ["apigw"] }
lambda_runtime = "1.3.0"
serde_json = "1.0"
tokio = "1.52.3"
lambdamux = "1.0.3"
```

Inside this repository, `rust/test_lambda` uses the local workspace crate:

```toml
lambdamux = { version = "1.0.3", path = "../lambdamux" }
```

## Quick Start

Create a route module such as `hello.rs`:

```rust,ignore
use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use aws_lambda_events::encodings::Body;
use aws_lambda_events::http::{HeaderMap, Result};
use lambda_runtime::LambdaEvent;
use lambdamux::route;
use serde_json::{json, Value};

fn json_response(status_code: i64, value: Value) -> ApiGatewayV2httpResponse {
	let mut headers = HeaderMap::new();
	headers.insert("content-type", "application/json".parse().unwrap());

	let mut response = ApiGatewayV2httpResponse::default();
	response.status_code = status_code;
	response.body = Some(Body::Text(value.to_string()));
	response.headers = headers.clone();
	response.multi_value_headers = headers;
	response
}

#[route(path = "/hello", method = "get")]
pub async fn hello_get(
	_event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse> {
	Ok(json_response(200, json!({ "success": true })))
}

#[route(path = "/hello/:id", method = "get")]
pub fn hello_id_get(
	event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse> {
	let id = event
		.payload
		.path_parameters
		.get("id")
		.cloned()
		.unwrap_or_default();

	Ok(json_response(200, json!({
		"success": true,
		"id": id,
	})))
}

#[route(path = "/hello", method = "post")]
pub async fn hello_post(
	event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse> {
	let body = event.payload.body.unwrap_or("{}".into());
	let body: Value = serde_json::from_str(&body).unwrap_or(json!({}));

	let Some(name) = body.get("name").and_then(|value| value.as_str()) else {
		return Ok(json_response(400, json!({
			"success": true,
			"name": "not found",
		})));
	};

	Ok(json_response(201, json!({
		"success": true,
		"name": name,
	})))
}
```

Then wire the Lambda entry point in `main.rs`:

```rust,ignore
use aws_lambda_events::apigw::{ApiGatewayV2httpRequest, ApiGatewayV2httpResponse};
use aws_lambda_events::encodings::Error;
use lambda_runtime::{service_fn, LambdaEvent};

mod hello;
use hello::*;

async fn handler(
	event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse, Error> {
	lambdamux::handle_apigw_v2!(event)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
	lambda_runtime::run(service_fn(handler)).await
}
```

That is enough to route requests like:

- `GET /hello`
- `GET /hello/123`
- `POST /hello`

## Synchronous and asynchronous routes

Route handlers can be synchronous or asynchronous, and both forms can be mixed in the same Lambda:

```rust,ignore
#[route(path = "/health", method = "get")]
pub fn health(
	event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse> {
	// Return a response directly.
}

#[route(path = "/users/:id", method = "get")]
pub async fn user(
	event: LambdaEvent<ApiGatewayV2httpRequest>,
) -> Result<ApiGatewayV2httpResponse> {
	// Await database, HTTP, or other asynchronous work before returning.
}
```

No extra dispatch setup is needed. `handle_apigw_v1!` and `handle_apigw_v2!` await the selected route internally.

## API Gateway v1

If your Lambda is fronted by API Gateway REST API payloads instead of HTTP API v2 payloads, use `ApiGatewayProxyRequest`, `ApiGatewayProxyResponse`, and the `handle_apigw_v1!` macro instead.

## Prerequisites

Before building or deploying the template, install:

- Rust from [rust-lang.org](https://www.rust-lang.org/tools/install)
- `cargo-lambda` from [cargo-lambda.info](https://www.cargo-lambda.info/guide/getting-started.html)
- Terraform from [hashicorp.com](https://developer.hashicorp.com/terraform/install)

## Building and Testing

Run the workspace tests from the Rust workspace:

```bash
cd rust
cargo test --workspace
```

Build the Lambda function for the `x86_64-unknown-linux-gnu` target:

```bash
cargo lambda build --release --target x86_64-unknown-linux-gnu
```

This produces a binary in `target/lambda/<function-name>`. The Terraform example handles this build automatically.

To run a focused test:

```bash
cargo test --package test_lambda --bin test_lambda -- hello::hello_tests --show-output
cargo test --package test_lambda --bin test_lambda -- hello::hello_tests::hello_get_test --exact --show-output
```

## Deployment Example

This repository includes a Terraform example that shows how to deploy a `lambdamux` Lambda behind API Gateway:

- Terraform example: <https://github.com/anuradhawick/rs_template/tree/main/terraform>

That example currently:

- builds the Rust Lambda from `rust/test_lambda`
- packages it with `cargo lambda build`
- creates an API Gateway HTTP API
- forwards `ANY /{proxy+}` to the Lambda function

Typical usage looks like this:

```bash
cd terraform
terraform init
terraform apply
```

If your crate path or Lambda binary name is different, update the Terraform `source_path` and function settings before applying.

## Adding More Functions and Endpoints

Follow the style in `rust/test_lambda` to add route modules, tests, and handlers.

To add another Lambda crate to the workspace:

```bash
cd rust
cargo new test_lambda_2
```

Then add the dependencies from the installation section, wire a handler with `handle_apigw_v2!`, and add Terraform to route traffic to the new Lambda. The examples in [terraform-aws-apigateway-v2](https://github.com/terraform-aws-modules/terraform-aws-apigateway-v2/tree/master/examples) are a useful reference.

For non-HTTP Lambda events, use `serde_json::Value`, the relevant `aws_lambda_events` event type, or your own `serde` structs.

## Authentication

You can authenticate inside `main.rs` or a route module such as `hello.rs`. The event object includes the complete request context, including API Gateway authorizer context.

## Development

Useful checks before publishing or deploying:

```bash
cd rust
cargo test --workspace
cargo deny check
```

When publishing the crates manually, publish them in dependency order: `lambdamux-core`, `lambdamux-macro`, then `lambdamux`. The GitHub release workflow follows the same order.

## Notes

- The internal route cache is managed by `lambdamux`; users do not need to define a global `Lazy` or `OnceLock`.
- `lambdamux` is the user-facing facade crate. The repository also contains `lambdamux-core` and `lambdamux-macro`, but most users only need `lambdamux`.

## License

Licensed under either GPL-3.0-only or Apache-2.0, at your option.
