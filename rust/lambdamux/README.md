# lambdamux

`lambdamux` is an AWS Lambda HTTP router for Rust.

It lets you write route handlers with `#[route(...)]` attributes and dispatch API Gateway requests without managing a global route cache yourself.

## What it gives you

- route registration with `#[route(path = ..., method = ...)]`
- compile-time route table generation with `generate_routes!()`
- built-in API Gateway v2 dispatch with `handle_apigw_v2!`
- built-in API Gateway v1 dispatch with `handle_apigw_v1!`
- path parameter matching such as `/hello/:id`

## Installation

Add these dependencies to your Lambda crate:

```toml
[dependencies]
aws_lambda_events = "1.2.0"
lambda_runtime = "1.3.0"
serde_json = "1.0"
tokio = "1.52.3"
lambdamux = "0.1.0"
```

## Quick Start

Create a route module such as `hello.rs`:

```rust
use aws_lambda_events::apigw::ApiGatewayV2httpRequest;
use aws_lambda_events::http::Result;
use lambda_runtime::LambdaEvent;
use lambdamux::route;
use serde_json::{json, Value};

#[route(path = "/hello", method = "get")]
pub fn hello_get(_event: LambdaEvent<ApiGatewayV2httpRequest>) -> Result<Value> {
	Ok(json!({ "success": true }))
}

#[route(path = "/hello/:id", method = "get")]
pub fn hello_id_get(event: LambdaEvent<ApiGatewayV2httpRequest>) -> Result<Value> {
	let id = event
		.payload
		.path_parameters
		.get("id")
		.cloned()
		.unwrap_or_default();

	Ok(json!({
		"success": true,
		"id": id,
	}))
}

#[route(path = "/hello", method = "post")]
pub fn hello_post(event: LambdaEvent<ApiGatewayV2httpRequest>) -> Result<Value> {
	let body = event.payload.body.unwrap_or("{}".into());
	let body: Value = serde_json::from_str(&body).unwrap_or(json!({}));

	let Some(name) = body.get("name").and_then(|value| value.as_str()) else {
		return Ok(json!({
			"success": true,
			"name": "not found",
		}));
	};

	Ok(json!({
		"success": true,
		"name": name,
	}))
}
```

Then wire the Lambda entry point in `main.rs`:

```rust
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

## API Gateway v1

If your Lambda is fronted by API Gateway REST API payloads instead of HTTP API v2 payloads, use `ApiGatewayProxyRequest`, `ApiGatewayProxyResponse`, and the `handle_apigw_v1!` macro instead.

## Deployment Example

This repository includes a Terraform example that shows how to deploy a `lambdamux` Lambda behind API Gateway:

- Terraform example: https://github.com/anuradhawick/rs_template/tree/main/terraform

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

## Notes

- The internal route cache is managed by `lambdamux`; users do not need to define a global `Lazy` or `OnceLock`.
- `lambdamux` is the user-facing facade crate. The repository also contains `lambdamux-core` and `lambdamux-macro`, but most users only need `lambdamux`.

## License

Licensed under either GPL-3.0-only or Apache-2.0, at your option.
